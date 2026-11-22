//! EXP-01-B3 witness search + proof generation + dual verifier differential.
//!
//! Built as an extra bin inside the FIXED upstream `zkml` crate (no upstream
//! source files are modified; this file is copied into zkml/src/bin/ inside the
//! ephemeral CI checkout only, then compiled with the fixed Cargo.lock).
//!
//! Search uses DeepProve's own quantized path:
//!   load_from_model -> into_provable_llm -> Driver::run (single forward over P||z).
//! Proof uses the B2-confirmed Path M: Driver::run(P||z) -> Driver::prove.
//! Original verifier = fixed-version LLMVerifierContext::verify.
//! Patched verifier = copy of the same logic with ONLY the EOS index base
//! changed from skip-relative to absolute (see patched_verify below).

use std::{
    collections::HashSet,
    path::PathBuf,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail, ensure};
use ark_bn254::Bn254;
use dp_crypto::arkyper::HyperKZG;
use dp_crypto::arkyper::transcript::blake3::Blake3Transcript;
use itertools::Itertools;
use rand::{Rng, SeedableRng, rngs::StdRng};
use sha2::{Digest, Sha256};
use tenstore::GenStore;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use zkml::{
    IO, Proof, ProverContext,
    graph::Node,
    iop::verifier::verify as verify_proof,
    layers::Layer,
    model::{
        Trace,
        llm::{Driver, LLMVerifierContext, WithMaxContext},
    },
    number::Number,
    parser::{
        llm::{HFTokenizer, LLMTokenizer, Token, models::gpt2::GPT2},
        safe::RawSafeTensors,
    },
    quantization::{Dequantize, ModelMetadata, ToElement},
};

type F = ark_bn254::Fr;
type Pcs = HyperKZG<Bn254>;
type QDriver = Driver<zkml::Element>;

const EOS_TOKEN: usize = 50256;
const MAX_PROMPTS: usize = 48;
const MAX_CANDIDATES: usize = 64;
const MAX_EVALS: usize = 3072;
const MAX_PROOFS: usize = 3;
const MAX_CONTEXT: usize = 32;
const PROMPT_MIN: usize = 2;
const PROMPT_MAX: usize = 20;
const SEARCH_BUDGET_SECS: u64 = 2700; // 45 minutes

#[derive(Debug)]
struct Args {
    model_dir: PathBuf,
    prompt_seeds: PathBuf,
    out_dir: PathBuf,
}

fn parse_args() -> Result<Args> {
    let mut it = std::env::args().skip(1);
    let mut model_dir = None;
    let mut prompt_seeds = None;
    let mut out_dir = None;
    while let Some(a) = it.next() {
        match a.as_str() {
            "--model-dir" => model_dir = Some(PathBuf::from(it.next().context("--model-dir value")?)),
            "--prompt-seeds" => prompt_seeds = Some(PathBuf::from(it.next().context("--prompt-seeds value")?)),
            "--out-dir" => out_dir = Some(PathBuf::from(it.next().context("--out-dir value")?)),
            other => bail!("unknown arg: {other}"),
        }
    }
    Ok(Args {
        model_dir: model_dir.context("--model-dir required")?,
        prompt_seeds: prompt_seeds.context("--prompt-seeds required")?,
        out_dir: out_dir.context("--out-dir required")?,
    })
}

fn sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

fn read_seeds(path: &std::path::Path) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect())
}

/// Single quantized forward over `tokens`, returning (argmax ids, trace).
fn forward(
    driver: &QDriver,
    tokens: &[Token],
) -> Result<(Vec<usize>, Trace<zkml::Element>)> {
    let mut store = GenStore::default();
    let tensor = driver.tokens_to_tensor(tokens)?;
    let trace: Trace<zkml::Element> = driver.run(vec![tensor], &mut store)?;
    let out = trace
        .outputs()
        .last()
        .context("trace has no output")?
        .tensor()?;
    let argmax = out.data().iter().map(|t| Number::to_usize(t)).collect();
    Ok((argmax, trace))
}

/// Extract quantized logits of the LAST real position from the final Logits
/// layer, dequantize, and return top-k token ids. Mirrors extract_logits.rs.
fn logits_topk(
    driver: &QDriver,
    trace: &Trace<zkml::Element>,
    meta: &ModelMetadata,
    seq_len: usize,
    vocab_size: usize,
    k: usize,
) -> Result<Vec<usize>> {
    for (node_id, node) in driver.model.graph().nodes() {
        let Node::Inner(layer) = node else { continue };
        if !matches!(layer, Layer::Logits(_)) {
            continue;
        }
        let Some(step) = trace.get_step(node_id) else { continue };
        let inputs = step.inputs();
        if inputs.is_empty() {
            continue;
        }
        let tens = inputs[0].tensor()?;
        let sc = meta.layer_input_scaling_factor(*node_id);
        let Some(scaling) = sc.first() else { continue };
        let deq = tens.data().dequantize(scaling);
        let cols = if tens.shape().rank() >= 2 {
            tens.shape().dim(tens.shape().rank() - 1)
        } else {
            deq.len()
        };
        let row_start = seq_len.saturating_sub(1) * cols;
        let take = cols.min(vocab_size);
        if row_start + take > deq.len() {
            continue;
        }
        let row = &deq[row_start..row_start + take];
        let mut pairs: Vec<(usize, f32)> = row.iter().copied().enumerate().collect();
        pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        return Ok(pairs.into_iter().take(k).map(|(i, _)| i).collect());
    }
    bail!("no Logits step found in trace")
}

/// Build candidate z set: quantized-logits top-k for position n-1 (i.e. a-like
/// alternatives) + deterministic seeded token ids + EOS. Deduped, capped.
fn build_candidates(
    seed: u64,
    vocab_size: usize,
    topk: &[usize],
) -> Vec<usize> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    let mut push = |t: usize, out: &mut Vec<usize>| {
        if t < vocab_size && seen.insert(t) && out.len() < MAX_CANDIDATES {
            out.push(t);
        }
    };
    for &t in topk {
        push(t, &mut out);
    }
    while out.len() < MAX_CANDIDATES - 1 {
        let t = rng.gen_range(0..vocab_size);
        push(t, &mut out);
    }
    push(EOS_TOKEN, &mut out);
    out
}

/// Copy of fixed-version LLMVerifierContext::verify (zkml/src/model/llm.rs:804-877)
/// with ONE change: EOS index base is absolute instead of skip-relative.
fn patched_verify(
    ctx: &LLMVerifierContext<F, Pcs>,
    proof: Proof<F, Pcs>,
    user_input: Vec<Token>,
    inference_output: IO<F>,
) -> Result<()> {
    ensure!(
        inference_output.output.len() == 1,
        "Expected 1 output tensor, found {}",
        inference_output.output.len()
    );
    let output = inference_output.output[0].clone();
    let padded_max_len = output.shape().numel();
    let max_window = ctx.max_context.unwrap_or(ctx.md.context_length());
    ensure!(
        padded_max_len <= max_window.next_power_of_two(),
        "output length is greater than the padded maximum context length"
    );

    let eos_token = ctx.md.config.eos_token;
    let eos_found = output
        .data()
        .iter()
        .take(max_window)
        .skip(user_input.len() - 1)
        .find_position(|token| token.to_element() as usize == usize::from(eos_token));

    // ---- THE ONLY PATCH ----------------------------------------------------
    // Original : unpadded_output_len = pos.0            (relative to skip(n-1))
    // Patched  : EOS absolute index = (n-1) + pos.0 ; length = absolute + 1
    let unpadded_output_len = if let Some(pos) = &eos_found {
        (user_input.len() - 1) + pos.0 + 1
    } else {
        max_window
    };
    // ------------------------------------------------------------------------

    let prover_input = inference_output.input[0].clone();
    let prover_output = inference_output.output[0].clone();
    verify_proof::<_, Blake3Transcript, _>(&ctx.verifier_ctx, proof, inference_output)?;

    let seq_len = user_input.len();
    ensure!(
        prover_input.data()[..seq_len]
            .iter()
            .zip(user_input[..seq_len].iter())
            .all(|(a, b)| a.to_element().to_usize() == usize::from(*b)),
        "user input not the same"
    );

    for i in seq_len - 1..unpadded_output_len - 1 {
        ensure!(
            prover_input.data()[i + 1] == prover_output.data()[i],
            "next input token is not the one generated by this row pos i {}: {:?} != {:?}",
            i,
            prover_input.data(),
            prover_output.data()
        );
    }
    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct Candidate {
    prompt_index: usize,
    prompt_tokens: Vec<usize>,
    prompt_text: String,
    n: usize,
    a: usize,
    z: usize,
    y_at_n_minus_1: usize,
    y_at_n: usize,
}

fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).ok();

    let args = parse_args()?;
    std::fs::create_dir_all(&args.out_dir)?;

    let model_file = args.model_dir.join("model.safetensors");
    let tokenizer_file = args.model_dir.join("tokenizer.json");
    let config_file = args.model_dir.join("config.json");
    for p in [&model_file, &tokenizer_file, &config_file] {
        ensure!(p.exists(), "missing fixed-revision file: {}", p.display());
    }
    let safe = RawSafeTensors::new(&model_file, &tokenizer_file, &config_file);
    let tokenizer = HFTokenizer::from_tokenizer_json_path(&tokenizer_file)?;

    info!("Loading quantized GPT-2 (max_context={})", MAX_CONTEXT);
    let raw: Driver<f32> = Driver::load_from_model(GPT2::new(), &safe, Some(MAX_CONTEXT))?;
    let (mut driver, meta): (QDriver, ModelMetadata) = raw.into_provable_llm(None)?;
    driver.with_max_context(MAX_CONTEXT);
    let vocab_size = driver.vocab_size();
    info!("vocab_size={}", vocab_size);

    let seeds: Vec<String> = read_seeds(&args.prompt_seeds)?.into_iter().take(MAX_PROMPTS).collect();
    info!("prompt seeds: {}", seeds.len());

    let start = Instant::now();
    let mut evals = 0usize;
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut found_keys = HashSet::new();

    'seed_loop: for (pi, text) in seeds.iter().enumerate() {
        let p_tokens: Vec<Token> = tokenizer.tokenize(text);
        let n = p_tokens.len();
        if n < PROMPT_MIN || n > PROMPT_MAX || n + 1 >= MAX_CONTEXT {
            warn!("seed {} token len {} out of range, skip", pi, n);
            continue;
        }

        // a = Y[n-1] from a single quantized forward over P; keep trace for top-k
        let (y_p, trace_p) = forward(&driver, &p_tokens)?;
        evals += 1;
        ensure!(y_p.len() >= n, "short output for prompt forward");
        let a = y_p[n - 1];

        let topk = logits_topk(&driver, &trace_p, &meta, n, vocab_size, 16)
            .unwrap_or_else(|e| {
                warn!("top-k extraction failed ({}); using seeded tokens only", e);
                Vec::new()
            });
        let toks = build_candidates(pi as u64 + 20261122, vocab_size, &topk);
        info!("prompt {} n={} a={} candidates={}", pi, n, a, toks.len());

        for z in toks {
            if z == a {
                continue; // witness requires z != a
            }
            if evals >= MAX_EVALS || start.elapsed() >= Duration::from_secs(SEARCH_BUDGET_SECS) {
                info!("search budget reached: evals={} elapsed={:?}", evals, start.elapsed());
                break 'seed_loop;
            }
            let mut x = p_tokens.clone();
            x.push(Token::from(z as u64));
            let (y, _) = forward(&driver, &x)?;
            evals += 1;
            if y.len() < n + 1 {
                continue;
            }
            let y_nm1 = y[n - 1];
            let y_n = y[n];
            if y_nm1 == a && y_n == EOS_TOKEN {
                let key = (pi, z);
                if found_keys.insert(key) {
                    info!("CANDIDATE prompt={} z={} a={} y[n]={} (evals={})", pi, z, a, y_n, evals);
                    candidates.push(Candidate {
                        prompt_index: pi,
                        prompt_tokens: p_tokens.iter().map(|t| u32::from(t) as usize).collect(),
                        prompt_text: text.clone(),
                        n,
                        a,
                        z,
                        y_at_n_minus_1: y_nm1,
                        y_at_n: y_n,
                    });
                }
            }
        }
    }

    std::fs::write(
        args.out_dir.join("candidates.json"),
        serde_json::to_string_pretty(&candidates)?,
    )?;
    let mut csv = String::from("prompt_index,n,a,z,y_at_n_minus_1,y_at_n,prompt_text\n");
    for c in &candidates {
        csv.push_str(&format!(
            "{},{},{},{},{},{},\"{}\"\n",
            c.prompt_index, c.n, c.a, c.z, c.y_at_n_minus_1, c.y_at_n,
            c.prompt_text.replace('"', "\"\"")
        ));
    }
    std::fs::write(args.out_dir.join("candidates.csv"), &csv)?;
    info!("search done: evals={} elapsed={:?} candidates={}", evals, start.elapsed(), candidates.len());

    if candidates.is_empty() {
        println!("B3_STATUS=B3_NO_WITNESS_WITHIN_BUDGET");
        println!("evals={} elapsed_secs={}", evals, start.elapsed().as_secs());
        return Ok(());
    }

    let (prover_ctx, verifier_ctx) = driver.context::<F, Pcs>()?;
    let verifier_ctx = verifier_ctx.with_max_context(MAX_CONTEXT);

    let mut meta_json = serde_json::Map::new();
    meta_json.insert("model".into(), serde_json::json!("openai-community/gpt2"));
    meta_json.insert("max_context".into(), serde_json::json!(MAX_CONTEXT));
    meta_json.insert("vocab_size".into(), serde_json::json!(vocab_size));
    meta_json.insert("eos_token".into(), serde_json::json!(EOS_TOKEN));
    meta_json.insert("search_evals".into(), serde_json::json!(evals));
    meta_json.insert("search_elapsed_secs".into(), serde_json::json!(start.elapsed().as_secs()));
    meta_json.insert("candidate_count".into(), serde_json::json!(candidates.len()));

    let mut reproduced = 0usize;
    for (idx, c) in candidates.iter().take(MAX_PROOFS).enumerate() {
        println!("--- proof attempt {} ---", idx);
        let p_tokens: Vec<Token> = c.prompt_tokens.iter().map(|&t| Token::from(t as u64)).collect();
        let mut x = p_tokens.clone();
        x.push(Token::from(c.z as u64));

        // Path M: run(P||z) -> prove
        let trace: Trace<zkml::Element> = {
            let mut store = GenStore::default();
            let tensor = driver.tokens_to_tensor(&x)?;
            driver.run(vec![tensor], &mut store)?
        };
        let (proof, io) = driver.prove(&prover_ctx, trace)?;

        let proof_bytes = rmp_serde::to_vec(&proof)?;
        let io_bytes = rmp_serde::to_vec(&io)?;
        let proof_hash = sha256(&proof_bytes);
        let io_hash = sha256(&io_bytes);
        info!("candidate {} proof_sha256={} io_sha256={}", idx, proof_hash, io_hash);

        std::fs::write(args.out_dir.join(format!("proof-{}.bin", idx)), &proof_bytes)?;
        std::fs::write(args.out_dir.join(format!("io-{}.bin", idx)), &io_bytes)?;

        let orig = verifier_ctx.verify(proof.clone(), p_tokens.clone(), io.clone());
        let patched = patched_verify(&verifier_ctx, proof, p_tokens.clone(), io);

        let orig_ok = orig.is_ok();
        let patched_ok = patched.is_ok();
        let orig_msg = format!("{:?}", orig.as_ref().err());
        let patched_msg = format!("{:?}", patched.as_ref().err());
        println!(
            "candidate {}: original={} patched={}",
            idx,
            if orig_ok { "ACCEPT" } else { "REJECT" },
            if patched_ok { "ACCEPT" } else { "REJECT" }
        );
        println!("  orig_err={}", orig_msg);
        println!("  patched_err={}", patched_msg);

        if orig_ok && !patched_ok {
            reproduced += 1;
        }

        meta_json.insert(format!("candidate_{}_proof_sha256", idx), serde_json::json!(proof_hash));
        meta_json.insert(format!("candidate_{}_io_sha256", idx), serde_json::json!(io_hash));
        meta_json.insert(format!("candidate_{}_n", idx), serde_json::json!(c.n));
        meta_json.insert(format!("candidate_{}_a", idx), serde_json::json!(c.a));
        meta_json.insert(format!("candidate_{}_z", idx), serde_json::json!(c.z));
        meta_json.insert(format!("candidate_{}_original", idx), serde_json::json!(if orig_ok { "ACCEPT" } else { "REJECT" }));
        meta_json.insert(format!("candidate_{}_original_error", idx), serde_json::json!(orig_msg));
        meta_json.insert(format!("candidate_{}_patched", idx), serde_json::json!(if patched_ok { "ACCEPT" } else { "REJECT" }));
        meta_json.insert(format!("candidate_{}_patched_error", idx), serde_json::json!(patched_msg));
    }

    std::fs::write(
        args.out_dir.join("proof-metadata.json"),
        serde_json::to_string_pretty(&serde_json::Value::Object(meta_json))?,
    )?;

    // Determine final status. Distinguish the differential-required check:
    // patched rejection must specifically be the sequentiality error.
    let sequentiality_rejection = {
        let mut ok = false;
        for (k, v) in &meta_json {
            if k.ends_with("_patched_error") {
                if let serde_json::Value::String(s) = v {
                    if s.contains("next input token is not the one generated") {
                        ok = true;
                    }
                }
            }
        }
        ok
    };

    if reproduced > 0 && sequentiality_rejection {
        println!("B3_STATUS=B3_REPRODUCED");
    } else if reproduced > 0 {
        // original accepted + patched rejected, but not via sequentiality
        println!("B3_STATUS=B3_PATCHED_REJECTED_OTHER");
    } else {
        println!("B3_STATUS=B3_NO_DIFFERENTIAL");
    }
    Ok(())
}

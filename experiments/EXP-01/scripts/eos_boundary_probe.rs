// EXP-01-B1: EOS boundary probe using the REAL pinned upstream driver path.
//
// This is a temporary integration test copied into the isolated upstream
// worktree at zkml/tests/eos_boundary_probe.rs for execution, then removed.
// It does NOT modify upstream verifier logic, model weights, or EOS token ID.
// It calls the same run_elements -> prove -> LLMVerifierContext::verify chain
// as bench-llm, but with a controllable, legitimate text prompt (via the
// real GPT2 tokenizer) instead of bench's random token prompt.
//
// Usage (from upstream root):
//   cargo test --release -p zkml --test eos_boundary_probe -- --nocapture
//
// Environment variables:
//   EOS_PROBE_CASE=first|second|none   which boundary to attempt
//   EOS_PROBE_PROMPT=<text>            prompt text to tokenize (default below)
//   EOS_PROBE_CTX=<usize>              max_context passed to driver/verifier

use std::env;

use ark_bn254::Bn254;
use dp_crypto::arkyper::HyperKZG;
use tenstore::GenStore;
use zkml::model::llm::{Driver, LLMVerifierContext, WithMaxContext};
use zkml::parser::{
    file_cache::from_cache,
    gguf::RawGGUF,
    llm::{Token, models::gpt2::GPT2, tokenizer::TokenizerLoader},
    safe::RawSafeTensors,
};

type F = ark_bn254::Fr;
type Pcs = HyperKZG<Bn254>;

// The upstream unit tests use this cached Q8 GGUF for GPT-2 inference.
// For --hf path, see RawSafeTensors::from_hugging_face_cached below.
const GPT2_Q8_0: &str = "gpt2.Q8_0.gguf";

fn probe(case: &str, prompt: &str, ctx: usize) -> anyhow::Result<()> {
    // Real model, real tokenizer, from the pinned upstream cache/download path.
    let raw = RawSafeTensors::from_hugging_face_cached("openai-community/gpt2")?;
    let tokenizer = GPT2::new().load_tokenizer(&raw)?;
    let (driver, _md) = Driver::load_from_model(GPT2::new(), &raw, Some(ctx))?
        .into_provable_llm(None)?;
    let mut driver = driver;
    driver.with_max_context(ctx);

    let eos_token = driver.md.config.eos_token;
    let user_tokens: Vec<Token> = tokenizer.tokenize(prompt);
    println!("CASE={case} PROMPT={prompt:?}");
    println!("EOS_TOKEN_ID={eos_token} USER_TOKENS={user_tokens:?} USER_LEN={}", user_tokens.len());
    println!("MAX_CONTEXT={ctx}");

    let input_tensor = driver.tokens_to_tensor(&user_tokens)?;
    let mut store = GenStore::default();
    let trace = driver.run_elements(input_tensor, &mut store)?;

    // Inspect the honest trace's full input/output tensors exactly as the
    // verifier will see them inside IO.
    let in_data = trace.inputs()[0].tensor()?.data().to_vec();
    let out_data = trace.outputs()[0].tensor()?.data().to_vec();
    let in_usize: Vec<usize> = in_data.iter().map(|x| x.to_usize()).collect();
    let out_usize: Vec<usize> = out_data.iter().map(|x| x.to_usize()).collect();
    println!("TRACE_INPUT_LEN={} DATA={in_usize:?}", in_usize.len());
    println!("TRACE_OUTPUT_LEN={} DATA={out_usize:?}", out_usize.len());
    let eos_positions: Vec<usize> = out_usize
        .iter()
        .enumerate()
        .filter_map(|(i, v)| if *v == usize::from(eos_token) { Some(i) } else { None })
        .collect();
    println!("OUTPUT_EOS_POSITIONS={eos_positions:?}");

    // Compute the verifier's find_position index semantics on the honest trace.
    let seq_len = user_tokens.len();
    let take = ctx.min(out_usize.len());
    let rel = out_usize
        .iter()
        .take(take)
        .skip(seq_len.saturating_sub(1))
        .position(|v| *v == usize::from(eos_token));
    println!("VERIFIER_SEQ_LEN={seq_len} TAKE={take} SKIP={} REL_POS={rel:?}",
        seq_len.saturating_sub(1));

    // Generate a real proof and IO, then verify with the unmodified verifier.
    let (prover_ctx, verifier_ctx) = driver.context::<F, Pcs>()?;
    let verifier_ctx = verifier_ctx.with_max_context(ctx);
    let (proof, io) = driver.prove(&prover_ctx, trace)?;
    println!("PROOF_IO_GENERATED input_shapes={:?} output_shapes={:?}",
        io.input.iter().map(|t| t.shape().clone()).collect::<Vec<_>>(),
        io.output.iter().map(|t| t.shape().clone()).collect::<Vec<_>>());
    match verifier_ctx.verify(proof, user_tokens, io) {
        Ok(()) => println!("VERIFY_RESULT=ACCEPT"),
        Err(e) => println!("VERIFY_RESULT=REJECT ERROR={e:?}"),
    }
    println!("CASE_DONE={case}");
    Ok(())
}

#[test]
fn eos_boundary_probe() -> anyhow::Result<()> {
    let case = env::var("EOS_PROBE_CASE").unwrap_or_else(|_| "none".into());
    let prompt = env::var("EOS_PROBE_PROMPT")
        .unwrap_or_else(|_| "The sky is".into());
    let ctx: usize = env::var("EOS_PROBE_CTX")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);
    probe(&case, &prompt, ctx)
}

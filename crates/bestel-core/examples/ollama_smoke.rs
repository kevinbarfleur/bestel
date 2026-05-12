//! End-to-end smoke test for the Ollama provider.
//!
//! Run with: `cargo run --release -p bestel-core --example ollama_smoke -- <tag>`
//! e.g.      `cargo run --release -p bestel-core --example ollama_smoke -- qwen3:8b`
//!
//! What it does:
//! 1. Hits `/api/tags` and prints every dynamic ModelProfile we'd build.
//! 2. Spawns the OllamaClient with the tag passed on argv.
//! 3. Sends a fixed PoE1 prompt that should trigger at least one wiki tool.
//! 4. Streams every LlmDelta to stdout AND a log file under `target/`.
//! 5. Prints aggregate stats (text bytes, tool calls, reasoning blocks, time).

use std::io::Write;
use std::time::Instant;

use bestel_core::llm::models::list_profiles_with_local;
use bestel_core::llm::ollama::OllamaClient;
use bestel_core::llm::recorder::{save_run, Recorder};
use bestel_core::llm::tools::BuildContext;
use bestel_core::llm::{ChatMessage, LlmDelta, Role, ToolStatus};
use tokio::sync::mpsc;

const TEST_PROMPT: &str = "Explain the keystone Resolute Technique in PoE1: what it gives, what it costs, and one situation where you definitely DON'T want to take it. Cite the wiki URL.";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tag = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "qwen3:8b".to_string());

    println!("=== Local Ollama profiles (dynamic) ===");
    let profiles = list_profiles_with_local().await;
    for p in &profiles {
        if !p.id.starts_with("ollama:") {
            continue;
        }
        println!(
            "  {:<28}  speed={:?}  model_id={}  desc='{}'",
            p.id, p.speed, p.model_id, p.display_name
        );
    }
    let count_ollama = profiles.iter().filter(|p| p.id.starts_with("ollama:")).count();
    println!("  → {count_ollama} Ollama profile(s) shown to picker");
    println!();

    println!("=== Chat run · model={tag} ===");
    println!("Prompt: {TEST_PROMPT}");
    println!();

    let client = OllamaClient::with("http://localhost:11434".to_string(), tag.clone());
    let history = vec![ChatMessage {
        role: Role::User,
        content: TEST_PROMPT.to_string(),
        attachments: Vec::new(),
    }];
    let ctx = BuildContext::new();
    let (tx, mut rx) = mpsc::unbounded_channel::<LlmDelta>();

    let log_path = std::env::current_dir()
        .unwrap_or_default()
        .join("target")
        .join(format!("ollama_smoke_{}.log", tag.replace(':', "_")));
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let mut log = std::fs::File::create(&log_path)?;
    writeln!(log, "model={tag}")?;
    writeln!(log, "prompt={TEST_PROMPT}")?;
    writeln!(log, "----")?;

    let mut recorder = Recorder::new(
        "smoke",
        "ollama",
        tag.clone(),
        format!("Ollama · {tag}"),
        TEST_PROMPT.to_string(),
        Vec::new(),
    );

    let start = Instant::now();
    let provider_handle = tokio::spawn(async move { client.run(history, ctx, tx).await });

    let mut text_bytes = 0usize;
    let mut reasoning_bytes = 0usize;
    let mut tool_calls = 0usize;
    let mut tool_done = 0usize;
    let mut tool_failed = 0usize;
    let mut tool_names: Vec<String> = Vec::new();
    let mut error: Option<String> = None;

    while let Some(d) = rx.recv().await {
        recorder.apply(&d);
        match &d {
            LlmDelta::TextDelta(t) => {
                text_bytes += t.len();
                print!("{t}");
                std::io::stdout().flush().ok();
                writeln!(log, "TEXT {t:?}").ok();
            }
            LlmDelta::ReasoningBegin => {
                println!("\n[reasoning >>>]");
                writeln!(log, "REASONING_BEGIN").ok();
            }
            LlmDelta::ReasoningDelta(t) => {
                reasoning_bytes += t.len();
                print!("\x1b[2m{t}\x1b[0m");
                std::io::stdout().flush().ok();
                writeln!(log, "REASONING {t:?}").ok();
            }
            LlmDelta::ReasoningEnd => {
                println!("\n[<<< reasoning]");
                writeln!(log, "REASONING_END").ok();
            }
            LlmDelta::ToolBegin { id, name, detail } => {
                tool_calls += 1;
                tool_names.push(name.clone());
                println!("\n[tool→ {name} (id={id}) detail={detail:?}]");
                writeln!(log, "TOOL_BEGIN {name} id={id} detail={detail:?}").ok();
            }
            LlmDelta::ToolDetailUpdate { id, summary_input } => {
                writeln!(log, "TOOL_DETAIL id={id} summary_input={summary_input:?}").ok();
            }
            LlmDelta::ToolOutput { id, chunk } => {
                writeln!(log, "TOOL_OUTPUT id={id} chunk={chunk:?}").ok();
            }
            LlmDelta::ToolEnd { id, status, summary } => {
                match status {
                    ToolStatus::Done => tool_done += 1,
                    ToolStatus::Failed => tool_failed += 1,
                    _ => {}
                }
                println!("\n[tool← id={id} status={status:?} summary={summary:?}]");
                writeln!(log, "TOOL_END id={id} status={status:?} summary={summary:?}").ok();
            }
            LlmDelta::Usage(u) => {
                writeln!(
                    log,
                    "USAGE in={} cache_read={} cache_write={} out={} cost={:?}",
                    u.input_tokens, u.cached_input_tokens, u.cache_creation_tokens, u.output_tokens, u.cost_usd
                )
                .ok();
            }
            LlmDelta::MessageEnd => {
                writeln!(log, "MESSAGE_END").ok();
            }
            LlmDelta::Verifier(v) => {
                writeln!(
                    log,
                    "VERIFIER status={:?} findings={}",
                    v.status,
                    v.findings.len()
                )
                .ok();
            }
            LlmDelta::SheetDraftUpdate { section_id, .. } => {
                writeln!(log, "SHEET_DRAFT section_id={section_id}").ok();
            }
            LlmDelta::SheetAskUser { question_id, .. } => {
                writeln!(log, "SHEET_ASK question_id={question_id}").ok();
            }
            LlmDelta::SheetInterviewOpen { .. } => {
                writeln!(log, "SHEET_INTERVIEW_OPEN").ok();
            }
            LlmDelta::SheetFinalized { sheet_id, name } => {
                writeln!(log, "SHEET_FINALIZED sheet_id={sheet_id} name={name}").ok();
            }
            LlmDelta::SheetLoaded { sheet_id, name, stale, .. } => {
                writeln!(log, "SHEET_LOADED sheet_id={sheet_id} name={name} stale={stale}").ok();
            }
            LlmDelta::ModeAssigned { mode } => {
                writeln!(log, "MODE_ASSIGNED {mode}").ok();
            }
            LlmDelta::Error(msg) => {
                error = Some(msg.clone());
                eprintln!("\n[ERROR] {msg}");
                writeln!(log, "ERROR {msg}").ok();
            }
        }
    }

    let final_text = provider_handle.await??;
    let elapsed = start.elapsed();

    let run = recorder.finish(final_text.clone());
    let saved_path = match save_run(&run) {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("warning: could not persist debug run: {e}");
            None
        }
    };

    println!();
    println!();
    println!("=== Stats ===");
    println!("  elapsed         : {:.1}s", elapsed.as_secs_f64());
    println!("  text bytes      : {text_bytes}");
    println!("  reasoning bytes : {reasoning_bytes}");
    println!("  tool calls      : {tool_calls} (done={tool_done} failed={tool_failed})");
    println!("  tools invoked   : {tool_names:?}");
    println!("  final text len  : {}", final_text.len());
    if let Some(e) = &error {
        println!("  error           : {e}");
    }
    println!("  log             : {}", log_path.display());
    if let Some(p) = saved_path.as_ref() {
        println!("  debug record    : {}", p.display());
    }

    writeln!(log, "----")?;
    writeln!(log, "elapsed_s={:.3}", elapsed.as_secs_f64())?;
    writeln!(log, "text_bytes={text_bytes}")?;
    writeln!(log, "reasoning_bytes={reasoning_bytes}")?;
    writeln!(log, "tool_calls={tool_calls}")?;
    writeln!(log, "tool_done={tool_done}")?;
    writeln!(log, "tool_failed={tool_failed}")?;
    writeln!(log, "tools={tool_names:?}")?;
    writeln!(log, "final_text_len={}", final_text.len())?;

    Ok(())
}

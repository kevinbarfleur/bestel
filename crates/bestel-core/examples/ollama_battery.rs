//! Run a curated battery of PoE-flavoured prompts against every locally
//! installed Ollama model. Each prompt → its own persisted debug run, so
//! the result is browsable from the in-app `/debug` view (Ctrl+Shift+D).
//!
//! Usage: `cargo run --release -p bestel-core --example ollama_battery`
//!
//! The prompt mix is intentionally diverse: precise mechanic queries,
//! vague open-ended questions, build diagnoses, comparative questions, an
//! off-topic refusal probe. The point is to see how each model behaves
//! across the realistic spectrum of what a player would actually ask.

use std::time::Instant;

use bestel_core::llm::ollama::{probe_models_detailed, OllamaClient};
use bestel_core::llm::recorder::{save_run, Recorder};
use bestel_core::llm::tools::BuildContext;
use bestel_core::llm::{ChatMessage, LlmDelta, Role};
use tokio::sync::mpsc;

/// (label, prompt) — label is just for console logging; the prompt text is
/// what the model sees and what shows up in the debug viewer.
const BATTERY: &[(&str, &str)] = &[
    (
        "precise_mechanic",
        "How does Forbidden Rite work in PoE1? I want the actual damage formula — base chaos damage, the percent-of-life self-cost, and how scaling sources interact with the chaos hit.",
    ),
    (
        "precise_diag",
        "I'm playing a Pathfinder Toxic Rain in PoE1, level 92, around 5k life and 75/75/75 res. My single-target against pinnacle bosses feels weak — the build clears maps fine but takes forever on Maven. What's the standard fix and what should I look at first?",
    ),
    (
        "vague_defense",
        "my character keeps dying in red maps and i don't know why, help",
    ),
    (
        "vague_spend",
        "Got like 30 divines this league, what should I do with them?",
    ),
    (
        "comparison",
        "What are the three biggest mechanical differences between PoE1 and PoE2 that someone migrating should actually internalize?",
    ),
    (
        "off_topic_refusal",
        "What's the best aRPG that isn't Path of Exile in 2026?",
    ),
];

const DEFAULT_TARGETS: &[&str] = &["qwen3:30b", "qwen3:8b", "qwen2.5:7b"];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let installed = probe_models_detailed(None).await.unwrap_or_default();
    let installed_names: Vec<String> = installed.iter().map(|i| i.name.clone()).collect();

    println!("=== Local Ollama models ===");
    for n in &installed_names {
        println!("  · {n}");
    }
    println!();

    let cli_targets: Vec<String> = std::env::args().skip(1).collect();
    let target_refs: Vec<&str> = if cli_targets.is_empty() {
        DEFAULT_TARGETS.to_vec()
    } else {
        cli_targets.iter().map(|s| s.as_str()).collect()
    };

    let to_run: Vec<String> = target_refs
        .iter()
        .filter(|t| installed_names.iter().any(|n| n == **t))
        .map(|s| s.to_string())
        .collect();

    if to_run.is_empty() {
        eprintln!("No target Ollama models installed. Available: {installed_names:?}");
        return Ok(());
    }

    println!("Running battery against: {to_run:?}");
    println!("Battery: {} prompt(s) per model", BATTERY.len());
    println!();

    for tag in &to_run {
        for (label, prompt) in BATTERY {
            println!("┌── [{tag}] {label}");
            println!("│   {}", trim_for_log(prompt, 100));
            let start = Instant::now();
            match run_one(tag, label, prompt).await {
                Ok(stats) => {
                    println!(
                        "└── done in {:.1}s · {} text bytes · {} tool calls",
                        start.elapsed().as_secs_f64(),
                        stats.text_bytes,
                        stats.tool_calls,
                    );
                }
                Err(e) => {
                    println!("└── ERROR: {e}");
                }
            }
            println!();
        }
    }

    println!("Battery complete. Open the in-app Debug view (Ctrl+Shift+D) to browse.");
    Ok(())
}

struct RunStats {
    text_bytes: usize,
    tool_calls: usize,
}

async fn run_one(tag: &str, label: &str, prompt: &str) -> anyhow::Result<RunStats> {
    let client = OllamaClient::with("http://localhost:11434".to_string(), tag.to_string());
    let history = vec![ChatMessage {
        role: Role::User,
        content: prompt.to_string(),
        attachments: Vec::new(),
    }];
    let ctx = BuildContext::new();
    let (tx, mut rx) = mpsc::unbounded_channel::<LlmDelta>();

    let mut recorder = Recorder::new(
        format!("battery:{label}"),
        "ollama",
        tag.to_string(),
        format!("Ollama · {tag}"),
        prompt.to_string(),
        Vec::new(),
    );

    let provider_handle = tokio::spawn(async move { client.run(history, ctx, tx).await });

    let mut text_bytes = 0usize;
    let mut tool_calls = 0usize;

    while let Some(d) = rx.recv().await {
        recorder.apply(&d);
        match &d {
            LlmDelta::TextDelta(t) => text_bytes += t.len(),
            LlmDelta::ToolBegin { .. } => tool_calls += 1,
            _ => {}
        }
    }

    let final_text = provider_handle.await??;
    let run = recorder.finish(final_text);
    save_run(&run)?;

    Ok(RunStats {
        text_bytes,
        tool_calls,
    })
}

fn trim_for_log(s: &str, max: usize) -> String {
    let normalised: String = s.chars().map(|c| if c == '\n' { ' ' } else { c }).collect();
    if normalised.chars().count() <= max {
        normalised
    } else {
        let mut out: String = normalised.chars().take(max - 1).collect();
        out.push('…');
        out
    }
}

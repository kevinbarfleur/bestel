//! Run the real-user prompt battery against any model profile (defaults to
//! DeepSeek V3.2). Each prompt → its own persisted debug run, so the result
//! is browsable from the in-app `/debug` view (Ctrl+Shift+D).
//!
//! Single source of truth for prompts: `docs/test_prompts/real_user_prompts.toml`
//!
//! Usage:
//!   cargo run --release -p bestel-core --example real_battery
//!   cargo run --release -p bestel-core --example real_battery -- \
//!       --model deepseek-v3-2 \
//!       --cat vague,mechanics,off_topic_refusal
//!
//! Flags:
//!   --model <id>          one of the curated profile ids (default: deepseek-v3-2)
//!   --cat   <c1,c2,...>   only run prompts in these categories
//!   --id    <id1,id2,...> only run these specific prompt ids
//!   --limit <N>           cap to first N matching prompts (after filtering)
//!
//! The battery loads the active PoB watcher when `needs_build = true` is set
//! on a prompt; if no build is detected it skips that prompt with a warning.

use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bestel_core::llm::detect::build_provider_for_profile;
use bestel_core::llm::models::find_profile;
use bestel_core::llm::recorder::{save_run, Recorder};
use bestel_core::llm::tools::BuildContext;
use bestel_core::llm::{ChatMessage, LlmDelta, Provider, Role};
use bestel_core::pob::watcher::PobWatcher;
use serde::Deserialize;
use tokio::sync::mpsc;

#[derive(Debug, Deserialize)]
struct PromptFile {
    prompt: Vec<Prompt>,
}

#[derive(Debug, Clone, Deserialize)]
struct Prompt {
    id: String,
    category: String,
    text: String,
    #[allow(dead_code)]
    intent: String,
    #[allow(dead_code)]
    expected: String,
    #[serde(default)]
    needs_build: bool,
    #[serde(default)]
    #[allow(dead_code)]
    source: Option<String>,
}

struct Args {
    model_id: String,
    categories: Option<Vec<String>>,
    ids: Option<Vec<String>>,
    limit: Option<usize>,
    /// Seconds to sleep between prompts. Anthropic Tier 1 caps Haiku at
    /// 50K input tokens / minute and our system+tools+context payload runs
    /// 5-10K tokens per request, so back-to-back firing trips a 429 cascade
    /// after ~8-12 prompts. 8s pacing keeps us well under the bucket.
    delay_secs: u64,
}

fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut model_id = "deepseek-v3-2".to_string();
    let mut categories: Option<Vec<String>> = None;
    let mut ids: Option<Vec<String>> = None;
    let mut limit: Option<usize> = None;
    let mut delay_secs: u64 = 8;

    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--model" => {
                i += 1;
                if let Some(v) = raw.get(i) {
                    model_id = v.clone();
                }
            }
            "--cat" => {
                i += 1;
                if let Some(v) = raw.get(i) {
                    categories = Some(v.split(',').map(|s| s.trim().to_string()).collect());
                }
            }
            "--id" => {
                i += 1;
                if let Some(v) = raw.get(i) {
                    ids = Some(v.split(',').map(|s| s.trim().to_string()).collect());
                }
            }
            "--limit" => {
                i += 1;
                if let Some(v) = raw.get(i) {
                    limit = v.parse().ok();
                }
            }
            "--delay" => {
                i += 1;
                if let Some(v) = raw.get(i) {
                    if let Ok(n) = v.parse() {
                        delay_secs = n;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    Args {
        model_id,
        categories,
        ids,
        limit,
        delay_secs,
    }
}

fn prompts_path() -> PathBuf {
    // Resolve relative to this crate's manifest dir so the example works no
    // matter where `cargo run` is invoked from. Crate is at
    // `crates/bestel-core/`, repo root is two levels up, prompts at
    // `docs/test_prompts/`.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .join("..")
        .join("..")
        .join("docs")
        .join("test_prompts")
        .join("real_user_prompts.toml")
}

fn load_prompts() -> Result<Vec<Prompt>> {
    let path = prompts_path();
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    let parsed: PromptFile = toml::from_str(&raw).context("parse real_user_prompts.toml")?;
    Ok(parsed.prompt)
}

fn filter_prompts(all: Vec<Prompt>, args: &Args) -> Vec<Prompt> {
    let mut out: Vec<Prompt> = all
        .into_iter()
        .filter(|p| {
            args.categories
                .as_ref()
                .map(|cats| cats.iter().any(|c| c == &p.category))
                .unwrap_or(true)
        })
        .filter(|p| {
            args.ids
                .as_ref()
                .map(|ids| ids.iter().any(|i| i == &p.id))
                .unwrap_or(true)
        })
        .collect();
    if let Some(n) = args.limit {
        out.truncate(n);
    }
    out
}

#[tokio::main]
async fn main() -> Result<()> {
    // Match the Tauri binary's boot order: any keys saved via the in-app
    // ApiKeyField land in ~/.bestel/runtime/keys.json — load them into the
    // process env before we instantiate any client.
    bestel_core::llm::keys::apply_to_env();

    let args = parse_args();

    let profile = find_profile(&args.model_id)
        .ok_or_else(|| anyhow!("unknown model id '{}'", args.model_id))?;

    let provider_label = match profile.provider {
        bestel_core::llm::models::ProviderKind::Anthropic => {
            if profile.api_key_env.as_deref() == Some("DEEPSEEK_API_KEY") {
                "deepseek".to_string()
            } else {
                "anthropic".to_string()
            }
        }
        bestel_core::llm::models::ProviderKind::Ollama => "ollama".to_string(),
    };

    let all_prompts = load_prompts()?;
    let prompts = filter_prompts(all_prompts, &args);

    if prompts.is_empty() {
        eprintln!("No prompts match the filters. Aborting.");
        return Ok(());
    }

    println!("=== Real-user battery ===");
    println!("Model:     {} ({})", profile.display_name, profile.id);
    println!("Provider:  {provider_label}");
    println!("Prompts:   {} (after filters)", prompts.len());
    println!("Pacing:    {}s between prompts", args.delay_secs);
    if let Some(cats) = &args.categories {
        println!("Categories: {}", cats.join(", "));
    }
    println!();

    // Load active PoB once, in case any prompt needs it. PobWatcher::start
    // reads the user's PoB documents folder synchronously.
    let watcher = match PobWatcher::start() {
        Ok(w) => Some(w),
        Err(e) => {
            eprintln!(
                "warn: PoB watcher failed to start ({e}); needs_build prompts will be skipped"
            );
            None
        }
    };

    let mut total_text = 0usize;
    let mut total_tools = 0usize;
    let mut total_errors = 0usize;
    let battery_start = Instant::now();

    for (i, prompt) in prompts.iter().enumerate() {
        if i > 0 && args.delay_secs > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(args.delay_secs)).await;
        }

        let n = i + 1;
        let total = prompts.len();
        println!("┌── [{n}/{total}] {} · {}", prompt.category, prompt.id);
        println!("│   {}", trim_for_log(&prompt.text, 110));

        let ctx = BuildContext::new();
        if prompt.needs_build {
            let build = watcher.as_ref().and_then(|w| w.initial_build());
            match build {
                Some(b) => {
                    println!("│   ◆ attached PoB: {}", b.summary_line());
                    ctx.set(b);
                }
                None => {
                    println!("└── SKIP — needs_build=true but no PoB detected");
                    println!();
                    continue;
                }
            }
        }

        let start = Instant::now();
        match run_one(&profile, &provider_label, prompt, ctx).await {
            Ok(stats) => {
                total_text += stats.text_bytes;
                total_tools += stats.tool_calls;
                println!(
                    "└── done in {:.1}s · {} text bytes · {} tool calls",
                    start.elapsed().as_secs_f64(),
                    stats.text_bytes,
                    stats.tool_calls,
                );
            }
            Err(e) => {
                total_errors += 1;
                println!("└── ERROR: {e}");
            }
        }
        println!();
    }

    println!(
        "Battery complete in {:.1}s · {} prompts · {} tool calls · {} text bytes · {} errors",
        battery_start.elapsed().as_secs_f64(),
        prompts.len(),
        total_tools,
        total_text,
        total_errors,
    );
    println!("Open the in-app Debug view (Ctrl+Shift+D) and filter by source `battery:{}` to browse.", args.model_id);
    Ok(())
}

struct RunStats {
    text_bytes: usize,
    tool_calls: usize,
}

async fn run_one(
    profile: &bestel_core::llm::models::ModelProfile,
    provider_label: &str,
    prompt: &Prompt,
    ctx: BuildContext,
) -> Result<RunStats> {
    let provider = build_provider_for_profile(profile)
        .await
        .with_context(|| format!("build provider for {}", profile.id))?;

    let history = vec![ChatMessage {
        role: Role::User,
        content: prompt.text.clone(),
        attachments: Vec::new(),
    }];

    let (tx, mut rx) = mpsc::unbounded_channel::<LlmDelta>();

    let mut recorder = Recorder::new(
        format!("battery:{}", profile.id),
        provider_label,
        profile.model_id.clone(),
        profile.display_name.clone(),
        prompt.text.clone(),
        Vec::new(),
    );

    let provider_handle = tokio::spawn(async move { run_provider(provider, history, ctx, tx).await });

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

async fn run_provider(
    provider: Provider,
    history: Vec<ChatMessage>,
    ctx: BuildContext,
    tx: mpsc::UnboundedSender<LlmDelta>,
) -> Result<String> {
    match provider {
        Provider::Anthropic(c) => c.run(history, ctx, tx).await,
        Provider::Ollama(c) => c.run(history, ctx, tx).await,
    }
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

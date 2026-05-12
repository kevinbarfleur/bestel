#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod dto;
mod events;
mod prompts_watcher;
mod state;
mod titlebar;

use std::sync::Arc;

use anyhow::Result;
use tauri::{Emitter, Manager};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use bestel_core::llm::detect::detect_provider;
use bestel_core::pob::watcher::PobWatcher;

use crate::dto::{summary_dto_from_build, BuildEvent, DetectionDto, PobBuildDto, ProviderStatusEvent};
use crate::events::{POB_BUILD, POB_CLEARED, PROVIDER_STATUS};
use crate::state::AppState;

fn main() {
    init_tracing();

    // Load user-saved API keys from disk into the process env BEFORE any
    // subcommand runs (mcp-serve included), so AnthropicClient::from_env()
    // and friends see the keys without per-launch user setup.
    bestel_core::llm::keys::apply_to_env();

    // Seed ~/.bestel/prompts/ with the bundled defaults on first launch.
    // Idempotent — never overwrites user edits. Failures are logged but
    // not fatal: the runtime loader falls back to BUNDLED_* constants.
    if let Err(e) = bestel_core::prompts::seed_defaults_if_missing() {
        eprintln!("bestel: failed to seed prompts directory: {e}");
    }

    let mut args = std::env::args().skip(1);
    if let Some(cmd) = args.next() {
        match cmd.as_str() {
            "mcp-serve" => {
                run_mcp_serve();
                return;
            }
            "run-battery" => {
                let rest: Vec<String> = args.collect();
                run_battery(rest);
                return;
            }
            "eval-judge" => {
                let rest: Vec<String> = args.collect();
                run_eval_judge(rest);
                return;
            }
            "kb-eval" => {
                let rest: Vec<String> = args.collect();
                run_kb_eval(rest);
                return;
            }
            "db" => {
                let rest: Vec<String> = args.collect();
                run_db(rest);
                return;
            }
            "--version" | "-V" => {
                println!("bestel {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            other => {
                eprintln!("bestel: unknown subcommand '{other}'");
                print_help();
                std::process::exit(2);
            }
        }
    }

    // Must run before tauri::Builder constructs the WebView — WebView2
    // snapshots env vars at construction time, so a later mutation has no
    // effect on the embedded browser instance.
    maybe_enable_cdp_debug();

    run_tauri();
}

/// Tracing pipeline. Stderr layer is always present (visible when launched
/// from a terminal). When `BESTEL_FILE_LOG=1`, an additional rolling-daily
/// file layer writes to `~/.bestel/runtime/logs/bestel.log` — this is the
/// only way to capture Rust-side logs in release builds, since the GUI
/// subsystem detaches stdio. The `bestel-driver` debug harness reads this
/// file to combine backend tracing with WebView console events.
///
/// The `_FILE_GUARD` is leaked into a static once-cell-like slot via
/// `Box::leak` to keep the non-blocking writer alive for the program's
/// lifetime. Without it, the worker thread shuts down at function exit
/// and log writes get dropped silently.
fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    let stderr_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stderr);

    let file_layer = if std::env::var("BESTEL_FILE_LOG").is_ok() {
        if let Some(home) = dirs::home_dir() {
            let log_dir = home.join(".bestel").join("runtime").join("logs");
            if std::fs::create_dir_all(&log_dir).is_ok() {
                let appender = tracing_appender::rolling::daily(&log_dir, "bestel.log");
                let (non_blocking, guard) = tracing_appender::non_blocking(appender);
                // Leak the guard so the worker thread stays alive for the
                // process lifetime. We never need to drop it explicitly.
                Box::leak(Box::new(guard));
                Some(
                    tracing_subscriber::fmt::layer()
                        .with_writer(non_blocking)
                        .with_ansi(false),
                )
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(stderr_layer)
        .with(file_layer)
        .try_init();
}

/// Reads `BESTEL_DEBUG_PORT` and, if set to a valid u16, forwards it to
/// WebView2's `--remote-debugging-port` flag via the
/// `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` env var. Must run BEFORE
/// `tauri::Builder` constructs the WebView, otherwise WebView2 has already
/// snapshotted env vars and the flag is ignored.
///
/// Opt-in by design: production users never set this, no CDP listener
/// runs, no debug bridge attack surface. The driver's helper script
/// (`tools/launch-debuggable.ps1`) sets it explicitly.
fn maybe_enable_cdp_debug() {
    if let Ok(port_str) = std::env::var("BESTEL_DEBUG_PORT") {
        match port_str.parse::<u16>() {
            Ok(port) => {
                let webview_arg = format!("--remote-debugging-port={port}");
                std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", &webview_arg);
                tracing::warn!(
                    target: "bestel.debug",
                    port,
                    "CDP debug bridge enabled — DO NOT USE IN PRODUCTION"
                );
            }
            Err(_) => {
                tracing::warn!(
                    target: "bestel.debug",
                    raw = %port_str,
                    "BESTEL_DEBUG_PORT is set but not a valid u16 — ignoring"
                );
            }
        }
    }
}

fn print_help() {
    println!("Bestel — chronicler of Wraeclast.\n");
    println!("USAGE:");
    println!("  bestel                                  launch the desktop app");
    println!("  bestel mcp-serve                        run as MCP server (stdio JSON-RPC)");
    println!("  bestel run-battery <scenarios-dir>      headless test runner (live providers)");
    println!("    [--model <id>] [--out <dir>]");
    println!("    [--filter-name <substring>]");
    println!("    [--pob-fixture <name|path>]");
    println!("    [--strict]                              exit non-zero on lint FAIL findings");
    println!("  bestel eval-judge --runs-dir <dir> --eval-set <path>");
    println!("    [--judge-model <id>]                    default: claude-sonnet-4-6");
    println!("    [--out <baseline.json>]                 default: <runs-dir>/baseline.json");
    println!("    [--filter-id <substring>]               grade only matching scenario ids");
    println!("  bestel kb-eval --queries <path>");
    println!("    [--top-k <int>]                         default: 5");
    println!("    [--filter-id <substring>]               run only matching ids");
    println!("    [--out <baseline.json>]                 write per-query results");
    println!("  bestel db <subcommand>                  SQLite mirror inspection");
    println!("    db migrate                              run pending migrations");
    println!("    db stats                                row counts + schema version");
    println!("    db import [--backup]                    import ~/.bestel/runtime/debug-chats/*.json");
    println!("    db query --sql \"<SELECT ...>\"           raw SQL passthrough");
    println!("    db bench [--n <count>]                  seed N runs and time identity-fail query");
    println!("  bestel --version / --help");
}

fn run_battery(args: Vec<String>) {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("bestel: failed to start tokio runtime: {e}");
            std::process::exit(1);
        }
    };
    let exit_code = rt.block_on(async { run_battery_inner(args).await });
    std::process::exit(exit_code);
}

async fn run_battery_inner(raw_args: Vec<String>) -> i32 {
    use std::path::PathBuf;
    use std::time::Instant;

    use bestel_core::llm::detect::build_provider_for_profile;
    use bestel_core::llm::models::find_profile;
    use bestel_core::llm::recorder::{mirror_lint_findings, save_run, Recorder};
    use bestel_core::llm::tools::BuildContext;
    use bestel_core::llm::{ChatMessage, LlmDelta, Provider, Role};
    use bestel_core::pob::parser::parse_file as parse_pob_file;
    use bestel_core::test_runner::{lint_run, load_scenarios, FindingSeverity, Scenario};
    use tokio::sync::mpsc;

    bestel_core::llm::keys::apply_to_env();
    match bestel_core::llm::kb::bootstrap_global().await {
        Ok(true) => eprintln!("bestel: KB engine bootstrapped (background ingest spawned)"),
        Ok(false) => eprintln!("bestel: KB engine disabled (no embedding backend or corpus)"),
        Err(e) => eprintln!("bestel: KB bootstrap error (continuing without KB): {e}"),
    }

    let mut scenarios_dir: Option<PathBuf> = None;
    let mut model_id = "deepseek-v4-flash".to_string();
    let mut out_dir: Option<PathBuf> = None;
    let mut filter_name: Option<String> = None;
    let mut pob_fixture: Option<String> = None;
    let mut strict_lint = false;

    let mut i = 0;
    while i < raw_args.len() {
        match raw_args[i].as_str() {
            "--model" => {
                i += 1;
                if let Some(v) = raw_args.get(i) {
                    model_id = v.clone();
                }
            }
            "--out" => {
                i += 1;
                if let Some(v) = raw_args.get(i) {
                    out_dir = Some(PathBuf::from(v));
                }
            }
            "--filter-name" => {
                i += 1;
                if let Some(v) = raw_args.get(i) {
                    filter_name = Some(v.clone());
                }
            }
            "--pob-fixture" => {
                i += 1;
                if let Some(v) = raw_args.get(i) {
                    pob_fixture = Some(v.clone());
                }
            }
            "--strict" => {
                strict_lint = true;
            }
            other if !other.starts_with("--") && scenarios_dir.is_none() => {
                scenarios_dir = Some(PathBuf::from(other));
            }
            other => {
                eprintln!("bestel run-battery: unknown arg '{other}'");
                return 2;
            }
        }
        i += 1;
    }

    let scenarios_dir = match scenarios_dir {
        Some(d) => d,
        None => {
            eprintln!("bestel run-battery: scenarios dir is required (e.g. tests/scenarios)");
            return 2;
        }
    };

    let scenarios: Vec<Scenario> = match load_scenarios(&scenarios_dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("bestel run-battery: load scenarios failed: {e}");
            return 1;
        }
    };

    let scenarios: Vec<Scenario> = match filter_name {
        Some(needle) => scenarios
            .into_iter()
            .filter(|s| s.name.to_lowercase().contains(&needle.to_lowercase()))
            .collect(),
        None => scenarios,
    };

    if scenarios.is_empty() {
        eprintln!("bestel run-battery: no scenarios match the filter");
        return 1;
    }

    let profile = match find_profile(&model_id) {
        Some(p) => p,
        None => {
            eprintln!("bestel run-battery: unknown model id '{model_id}'");
            return 1;
        }
    };

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

    println!("=== bestel run-battery ===");
    println!(
        "Scenarios: {} (from {})",
        scenarios.len(),
        scenarios_dir.display()
    );
    println!("Model:     {} ({})", profile.display_name, profile.id);
    println!("Provider:  {provider_label}");
    println!(
        "Lint mode: {}",
        if strict_lint {
            "strict (exit non-zero on FAIL)"
        } else {
            "warn-only"
        }
    );
    if let Some(fix) = &pob_fixture {
        println!("Fixture:   {fix}");
    }
    if let Some(dir) = &out_dir {
        println!("Out dir:   {}", dir.display());
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("create out dir: {e}");
            return 1;
        }
    }
    println!();

    let battery_start = Instant::now();
    let mut total_errors = 0usize;
    let mut total_text = 0usize;
    let mut total_tools = 0usize;
    let mut total_lint_warn = 0usize;
    let mut total_lint_fail = 0usize;
    let mut scenarios_with_lint_fail = 0usize;

    for (idx, scenario) in scenarios.iter().enumerate() {
        let n = idx + 1;
        let total = scenarios.len();
        println!(
            "┌── [{n}/{total}] {} ({}, fixture={})",
            scenario.name,
            scenario.provider.as_cli_flag(),
            scenario.build_fixture.as_deref().unwrap_or("none"),
        );

        let ctx = BuildContext::new();
        let fixture_to_load = pob_fixture
            .clone()
            .or_else(|| scenario.build_fixture.clone());
        if let Some(fix) = fixture_to_load {
            let candidate = if fix.contains('/') || fix.contains('\\') {
                PathBuf::from(&fix)
            } else {
                workspace_root_for_battery().join(format!("tests/fixtures/pob/{fix}.xml"))
            };
            match parse_pob_file(&candidate) {
                Ok(b) => {
                    println!("│   ◆ loaded PoB fixture {}", candidate.display());
                    ctx.set(b);
                }
                Err(e) => {
                    println!("└── SKIP — fixture load failed: {e}");
                    total_errors += 1;
                    continue;
                }
            }
        }

        let provider = match build_provider_for_profile(&profile).await {
            Ok(p) => p,
            Err(e) => {
                println!("└── ERROR: build provider: {e}");
                total_errors += 1;
                continue;
            }
        };

        // Single Recorder per scenario, even when the scenario has
        // multiple turns. user_text is the first turn (legacy field);
        // additional turns are concatenated for visibility in the
        // persisted log via a separator. Each turn's deltas flow into
        // the same recorder so the final transcript is a coherent
        // multi-turn conversation.
        let effective_turns = scenario.effective_turns();
        let combined_user_text = effective_turns
            .iter()
            .enumerate()
            .map(|(i, t)| {
                if i == 0 {
                    t.user.clone()
                } else {
                    format!("\n\n[turn {}]\n{}", i + 1, t.user)
                }
            })
            .collect::<String>();

        let mut recorder = Recorder::new(
            format!("battery:{}", profile.id),
            provider_label.clone(),
            profile.model_id.clone(),
            profile.display_name.clone(),
            combined_user_text,
            Vec::new(),
        );

        let mut history: Vec<ChatMessage> = Vec::new();
        let mut text_bytes = 0usize;
        let mut tool_calls = 0usize;
        let mut last_assistant = String::new();
        let mut turn_error: Option<String> = None;

        let scenario_start = Instant::now();
        for (turn_idx, turn) in effective_turns.iter().enumerate() {
            history.push(ChatMessage {
                role: Role::User,
                content: turn.user.clone(),
                attachments: Vec::new(),
            });
            if turn_idx > 0 {
                println!("│   turn {} → {}", turn_idx + 1, truncate_for_log(&turn.user, 80));
            }

            let (tx, mut rx) = mpsc::unbounded_channel::<LlmDelta>();
            let history_for_turn = history.clone();
            let ctx_for_turn = ctx.clone();

            let provider_fut = async {
                match &provider {
                    Provider::Anthropic(c) => {
                        c.run(history_for_turn, ctx_for_turn, tx).await
                    }
                    Provider::Ollama(c) => {
                        c.run(history_for_turn, ctx_for_turn, tx).await
                    }
                }
            };

            let drain_fut = async {
                while let Some(d) = rx.recv().await {
                    recorder.apply(&d);
                    match &d {
                        LlmDelta::TextDelta(t) => text_bytes += t.len(),
                        LlmDelta::ToolBegin { .. } => tool_calls += 1,
                        _ => {}
                    }
                }
            };

            let (run_result, ()) = tokio::join!(provider_fut, drain_fut);

            match run_result {
                Ok(final_text) => {
                    last_assistant = final_text.clone();
                    history.push(ChatMessage {
                        role: Role::Assistant,
                        content: final_text,
                        attachments: Vec::new(),
                    });
                }
                Err(e) => {
                    turn_error = Some(format!("turn {}: {e}", turn_idx + 1));
                    break;
                }
            }

            if turn_idx + 1 < effective_turns.len() && turn.pause_secs > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(turn.pause_secs)).await;
            }
        }
        let elapsed = scenario_start.elapsed();

        match turn_error {
            None => {
                total_text += text_bytes;
                total_tools += tool_calls;
                println!(
                    "└── done in {:.1}s · {} turn(s) · {} text bytes · {} tool calls",
                    elapsed.as_secs_f64(),
                    effective_turns.len(),
                    text_bytes,
                    tool_calls,
                );
                let run = recorder.finish(last_assistant);
                let lint_report = lint_run(&run);
                if !lint_report.findings.is_empty() {
                    let warn_n = lint_report.warn_count();
                    let fail_n = lint_report.fail_count();
                    total_lint_warn += warn_n;
                    total_lint_fail += fail_n;
                    if fail_n > 0 {
                        scenarios_with_lint_fail += 1;
                    }
                    println!(
                        "    lint: {} fail · {} warn",
                        fail_n, warn_n,
                    );
                    for f in &lint_report.findings {
                        let tag = match f.severity {
                            FindingSeverity::Fail => "FAIL",
                            FindingSeverity::Warn => "WARN",
                        };
                        println!(
                            "      [{tag}] {} — {}",
                            f.id, truncate_for_log(&f.message, 100),
                        );
                    }
                }
                if let Err(e) = save_run(&run) {
                    eprintln!("    save_run failed: {e}");
                }
                let mirror_rows: Vec<bestel_core::persistence::LintFindingRow> = lint_report
                    .findings
                    .iter()
                    .map(|f| bestel_core::persistence::LintFindingRow {
                        run_id: run.id.clone(),
                        rule: f.id.clone(),
                        severity: match f.severity {
                            FindingSeverity::Fail => "fail".to_string(),
                            FindingSeverity::Warn => "warn".to_string(),
                        },
                        message: f.message.clone(),
                    })
                    .collect();
                mirror_lint_findings(&run.id, mirror_rows);
                if let Some(dir) = &out_dir {
                    let path = dir.join(format!("{}.json", scenario.name));
                    match serde_json::to_vec_pretty(&run) {
                        Ok(b) => {
                            if let Err(e) = std::fs::write(&path, b) {
                                eprintln!("    write {} failed: {e}", path.display());
                            }
                        }
                        Err(e) => eprintln!("    serialize run: {e}"),
                    }
                    let lint_path = dir.join(format!("{}.lint.json", scenario.name));
                    if let Ok(b) = serde_json::to_vec_pretty(&lint_report) {
                        let _ = std::fs::write(&lint_path, b);
                    }
                }
            }
            Some(msg) => {
                total_errors += 1;
                println!("└── ERROR: {msg}");
                let run = recorder.finish(last_assistant);
                let _ = save_run(&run);
                if let Some(dir) = &out_dir {
                    let path = dir.join(format!("{}.json", scenario.name));
                    if let Ok(b) = serde_json::to_vec_pretty(&run) {
                        let _ = std::fs::write(&path, b);
                    }
                }
            }
        }
        println!();
    }

    println!(
        "Battery complete in {:.1}s · {} scenarios · {} tool calls · {} text bytes · {} errors",
        battery_start.elapsed().as_secs_f64(),
        scenarios.len(),
        total_tools,
        total_text,
        total_errors,
    );
    println!(
        "Lint summary: {} FAIL · {} WARN · {}/{} scenarios with at least one FAIL ({} mode)",
        total_lint_fail,
        total_lint_warn,
        scenarios_with_lint_fail,
        scenarios.len(),
        if strict_lint { "strict" } else { "warn-only" },
    );

    if total_errors > 0 {
        return 1;
    }
    if strict_lint && total_lint_fail > 0 {
        return 1;
    }
    0
}

fn run_eval_judge(args: Vec<String>) {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("bestel: failed to start tokio runtime: {e}");
            std::process::exit(1);
        }
    };
    let exit_code = rt.block_on(async { run_eval_judge_inner(args).await });
    std::process::exit(exit_code);
}

async fn run_eval_judge_inner(raw_args: Vec<String>) -> i32 {
    use std::path::PathBuf;
    use std::time::Instant;

    use bestel_core::eval::judge::{
        judge_one, load_eval_set, load_persisted_runs, summarise, Baseline, JudgeRecord,
        SkippedRun,
    };

    bestel_core::llm::keys::apply_to_env();

    let mut runs_dir: Option<PathBuf> = None;
    let mut eval_set_path: Option<PathBuf> = None;
    let mut out_path: Option<PathBuf> = None;
    let mut judge_model = "claude-sonnet-4-6".to_string();
    let mut filter_id: Option<String> = None;

    let mut i = 0;
    while i < raw_args.len() {
        match raw_args[i].as_str() {
            "--runs-dir" => {
                i += 1;
                if let Some(v) = raw_args.get(i) {
                    runs_dir = Some(PathBuf::from(v));
                }
            }
            "--eval-set" => {
                i += 1;
                if let Some(v) = raw_args.get(i) {
                    eval_set_path = Some(PathBuf::from(v));
                }
            }
            "--out" => {
                i += 1;
                if let Some(v) = raw_args.get(i) {
                    out_path = Some(PathBuf::from(v));
                }
            }
            "--judge-model" => {
                i += 1;
                if let Some(v) = raw_args.get(i) {
                    judge_model = v.clone();
                }
            }
            "--filter-id" => {
                i += 1;
                if let Some(v) = raw_args.get(i) {
                    filter_id = Some(v.clone());
                }
            }
            other => {
                eprintln!("bestel eval-judge: unknown arg '{other}'");
                return 2;
            }
        }
        i += 1;
    }

    let Some(runs_dir) = runs_dir else {
        eprintln!("bestel eval-judge: --runs-dir is required");
        return 2;
    };
    let Some(eval_set_path) = eval_set_path else {
        eprintln!("bestel eval-judge: --eval-set is required");
        return 2;
    };
    let out_path = out_path.unwrap_or_else(|| runs_dir.join("baseline.json"));

    let api_key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!("bestel eval-judge: ANTHROPIC_API_KEY not set");
            return 1;
        }
    };

    let eval_map = match load_eval_set(&eval_set_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("bestel eval-judge: load eval_set failed: {e}");
            return 1;
        }
    };

    let runs = match load_persisted_runs(&runs_dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("bestel eval-judge: load runs failed: {e}");
            return 1;
        }
    };

    let runs_filtered: Vec<_> = match &filter_id {
        Some(needle) => runs
            .into_iter()
            .filter(|(stem, _, _)| stem.to_lowercase().contains(&needle.to_lowercase()))
            .collect(),
        None => runs,
    };

    if runs_filtered.is_empty() {
        eprintln!("bestel eval-judge: no PersistedRun *.json files matched in {}", runs_dir.display());
        return 1;
    }

    println!("=== bestel eval-judge ===");
    println!("Runs dir:    {} ({} runs)", runs_dir.display(), runs_filtered.len());
    println!("Eval set:    {} ({} entries)", eval_set_path.display(), eval_map.len());
    println!("Judge model: {judge_model}");
    println!("Out:         {}", out_path.display());
    println!();

    let http = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("bestel eval-judge: build http client: {e}");
            return 1;
        }
    };

    let started = Instant::now();
    let mut records: Vec<JudgeRecord> = Vec::new();
    let mut skipped: Vec<SkippedRun> = Vec::new();
    let mut judge_errors = 0usize;

    for (idx, (stem, path, run)) in runs_filtered.iter().enumerate() {
        let n = idx + 1;
        let total = runs_filtered.len();
        let entry = match eval_map.get(stem) {
            Some(e) => e,
            None => {
                println!("[{n}/{total}] {stem} — SKIP (no eval_set entry)");
                skipped.push(SkippedRun {
                    run_file: path.display().to_string(),
                    reason: "no eval_set entry for id".into(),
                });
                continue;
            }
        };

        print!("[{n}/{total}] {stem} ({}) … ", entry.category);
        let _ = std::io::Write::flush(&mut std::io::stdout());
        let t = Instant::now();
        match judge_one(&http, &api_key, &judge_model, entry, run).await {
            Ok(r) => {
                println!("score {} (in {:.1}s)", r.score, t.elapsed().as_secs_f64());
                let sidecar = path.with_extension("judge.json");
                if let Ok(b) = serde_json::to_vec_pretty(&r) {
                    let _ = std::fs::write(&sidecar, b);
                }
                records.push(r);
            }
            Err(e) => {
                judge_errors += 1;
                println!("ERROR: {e}");
                skipped.push(SkippedRun {
                    run_file: path.display().to_string(),
                    reason: format!("{e}"),
                });
            }
        }
    }

    let summary = summarise(&records, judge_errors);
    let baseline = Baseline {
        eval_set_path: eval_set_path.display().to_string(),
        runs_dir: runs_dir.display().to_string(),
        judge_model: judge_model.clone(),
        judged_at: chrono::Utc::now().to_rfc3339(),
        summary: summary.clone(),
        records,
        skipped,
    };

    match serde_json::to_vec_pretty(&baseline) {
        Ok(b) => {
            if let Some(parent) = out_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(&out_path, b) {
                eprintln!("bestel eval-judge: write baseline {} failed: {e}", out_path.display());
                return 1;
            }
        }
        Err(e) => {
            eprintln!("bestel eval-judge: serialize baseline: {e}");
            return 1;
        }
    }

    println!();
    println!(
        "Done in {:.1}s · {} graded · {} skipped · {} judge errors",
        started.elapsed().as_secs_f64(),
        baseline.records.len(),
        baseline.skipped.len(),
        judge_errors,
    );
    println!("Avg score: {:.1}/100", summary.avg_score);
    for (cat, stats) in &summary.by_category {
        println!("  {cat:10} n={} avg={:.1}", stats.n, stats.avg_score);
    }
    println!("Baseline: {}", out_path.display());

    if judge_errors > 0 {
        return 1;
    }
    0
}

fn truncate_for_log(s: &str, max: usize) -> String {
    let normalised: String = s.chars().map(|c| if c == '\n' { ' ' } else { c }).collect();
    if normalised.chars().count() <= max {
        normalised
    } else {
        let mut out: String = normalised.chars().take(max - 1).collect();
        out.push('…');
        out
    }
}

fn workspace_root_for_battery() -> std::path::PathBuf {
    use std::path::PathBuf;
    let mut cur = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for _ in 0..6 {
        if cur.join("Cargo.toml").exists() && cur.join("crates").is_dir() {
            return cur;
        }
        if !cur.pop() {
            break;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn run_kb_eval(args: Vec<String>) {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("bestel: failed to start tokio runtime: {e}");
            std::process::exit(1);
        }
    };
    let exit_code = rt.block_on(async { run_kb_eval_inner(args).await });
    std::process::exit(exit_code);
}

async fn run_kb_eval_inner(raw_args: Vec<String>) -> i32 {
    use bestel_rag::{detect_default, IngestConfig, KbEngine, LanceIndex};
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Instant;

    bestel_core::llm::keys::apply_to_env();

    let mut queries_path: Option<PathBuf> = None;
    let mut top_k: usize = 5;
    let mut filter_id: Option<String> = None;
    let mut out_path: Option<PathBuf> = None;

    let mut i = 0;
    while i < raw_args.len() {
        match raw_args[i].as_str() {
            "--queries" => {
                i += 1;
                if let Some(v) = raw_args.get(i) {
                    queries_path = Some(PathBuf::from(v));
                }
            }
            "--top-k" => {
                i += 1;
                if let Some(v) = raw_args.get(i) {
                    if let Ok(n) = v.parse::<usize>() {
                        top_k = n.clamp(1, 20);
                    }
                }
            }
            "--filter-id" => {
                i += 1;
                if let Some(v) = raw_args.get(i) {
                    filter_id = Some(v.clone());
                }
            }
            "--out" => {
                i += 1;
                if let Some(v) = raw_args.get(i) {
                    out_path = Some(PathBuf::from(v));
                }
            }
            other => {
                eprintln!("bestel kb-eval: unknown arg '{other}'");
                return 2;
            }
        }
        i += 1;
    }

    let queries_path = match queries_path {
        Some(p) => p,
        None => {
            eprintln!("bestel kb-eval: --queries is required (e.g. tests/eval/kb_queries.toml)");
            return 2;
        }
    };

    #[derive(Debug, Deserialize)]
    struct QueryFile {
        #[serde(default)]
        query: Vec<QueryEntry>,
    }
    #[derive(Debug, Deserialize, Serialize)]
    struct QueryEntry {
        id: String,
        prompt: String,
        /// Any one of these paths counts as a hit (suffix match). Lets a
        /// query name multiple legitimate target files when knowledge
        /// crosses references.
        expected_rel_paths: Vec<String>,
        #[serde(default)]
        notes: Option<String>,
    }
    let toml_text = match std::fs::read_to_string(&queries_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("bestel kb-eval: read {queries_path:?} failed: {e}");
            return 1;
        }
    };
    let parsed: QueryFile = match toml::from_str(&toml_text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("bestel kb-eval: parse {queries_path:?} failed: {e}");
            return 1;
        }
    };
    let mut queries: Vec<QueryEntry> = parsed.query;
    if let Some(needle) = &filter_id {
        let needle_lower = needle.to_lowercase();
        queries.retain(|q| q.id.to_lowercase().contains(&needle_lower));
    }
    if queries.is_empty() {
        eprintln!("bestel kb-eval: no queries to run after filter");
        return 1;
    }

    println!("=== bestel kb-eval ===");
    println!("queries: {}", queries.len());
    println!("top-k:   {}", top_k);

    // Synchronous bootstrap so the index is ready when search() runs.
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            eprintln!("bestel kb-eval: home dir not resolvable");
            return 1;
        }
    };
    let bestel_dir = home.join(".bestel");
    let corpus_root = bestel_dir.join("prompts").join("references");
    if !corpus_root.exists() {
        eprintln!(
            "bestel kb-eval: corpus root missing at {corpus_root:?} — launch the desktop app once to seed prompts"
        );
        return 1;
    }
    let index_dir = bestel_dir.join("index").join("kb.lance");
    let http = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("bestel kb-eval: http client build failed: {e}");
            return 1;
        }
    };
    let embedder = match detect_default(http).await {
        Ok(e) => e,
        Err(err) => {
            eprintln!("bestel kb-eval: no embedding backend: {err}");
            return 1;
        }
    };
    let label = embedder.label();
    let dim = bestel_rag::EmbeddingProvider::dim(&embedder);
    let index = match LanceIndex::open_or_create(&index_dir, dim).await {
        Ok(i) => i,
        Err(err) => {
            eprintln!("bestel kb-eval: open index failed: {err}");
            return 1;
        }
    };
    let engine = Arc::new(KbEngine::new(index, embedder));
    println!("embedder: {}", label);
    println!("index:    {}", index_dir.display());

    let ingest_started = Instant::now();
    let cfg = IngestConfig::default();
    match engine.ingest(&corpus_root, &cfg).await {
        Ok(stats) => println!(
            "ingest:   {} files seen, {} indexed, {} skipped, {} chunks ({} ms)",
            stats.files_seen,
            stats.files_indexed,
            stats.files_skipped_unchanged,
            stats.chunks_emitted,
            stats.elapsed_ms
        ),
        Err(err) => {
            eprintln!("bestel kb-eval: ingest failed: {err}");
            return 1;
        }
    }
    let _ = ingest_started;
    if let Err(err) = engine.ensure_fts_index().await {
        eprintln!("bestel kb-eval: ensure_fts_index failed: {err}");
        return 1;
    }

    #[derive(Debug, Serialize)]
    struct QueryResult {
        id: String,
        prompt: String,
        expected_rel_paths: Vec<String>,
        passed: bool,
        rank: Option<usize>,
        matched_path: Option<String>,
        top_hits: Vec<TopHit>,
        elapsed_ms: u64,
    }
    #[derive(Debug, Serialize)]
    struct TopHit {
        rel_path: String,
        heading_path: Vec<String>,
        score: f32,
    }

    let mut passed_count = 0usize;
    let mut total_elapsed_ms = 0u64;
    let mut results: Vec<QueryResult> = Vec::with_capacity(queries.len());
    println!();
    for (idx, q) in queries.iter().enumerate() {
        let started = Instant::now();
        let hits = match engine.search(&q.prompt, top_k, None, &[]).await {
            Ok(h) => h,
            Err(err) => {
                eprintln!("[{}/{}] {} — search error: {err}", idx + 1, queries.len(), q.id);
                continue;
            }
        };
        let elapsed_ms = started.elapsed().as_millis() as u64;
        total_elapsed_ms += elapsed_ms;
        let mut rank: Option<usize> = None;
        let mut matched_path: Option<String> = None;
        'outer: for (i, h) in hits.iter().enumerate() {
            for expected in &q.expected_rel_paths {
                if h.chunk.doc_path.ends_with(expected) {
                    rank = Some(i + 1);
                    matched_path = Some(h.chunk.doc_path.clone());
                    break 'outer;
                }
            }
        }
        let passed = rank.is_some();
        if passed {
            passed_count += 1;
        }
        let top_hits: Vec<TopHit> = hits
            .iter()
            .take(top_k)
            .map(|h| TopHit {
                rel_path: h.chunk.doc_path.clone(),
                heading_path: h.chunk.heading_path.clone(),
                score: h.score,
            })
            .collect();
        let marker = if passed { "PASS" } else { "FAIL" };
        let rank_str = rank
            .map(|r| format!("rank={}", r))
            .unwrap_or_else(|| "rank=miss".into());
        println!(
            "[{:>2}/{}] {} {} {} ({}ms)  expected={:?}",
            idx + 1,
            queries.len(),
            marker,
            q.id,
            rank_str,
            elapsed_ms,
            q.expected_rel_paths
        );
        if !passed {
            for (i, h) in top_hits.iter().enumerate().take(3) {
                println!("        #{} {} {:?}", i + 1, h.rel_path, h.heading_path);
            }
        }
        results.push(QueryResult {
            id: q.id.clone(),
            prompt: q.prompt.clone(),
            expected_rel_paths: q.expected_rel_paths.clone(),
            passed,
            rank,
            matched_path,
            top_hits,
            elapsed_ms,
        });
    }
    let total = queries.len();
    let pct = if total == 0 {
        0.0
    } else {
        100.0 * (passed_count as f64) / (total as f64)
    };
    let avg_ms = if total == 0 { 0 } else { total_elapsed_ms / (total as u64) };
    println!();
    println!("=== summary ===");
    println!(
        "passed: {}/{} ({:.1}%)  avg {} ms/query",
        passed_count, total, pct, avg_ms
    );
    let exit_threshold = 0.90_f64;
    let exit_code = if (pct / 100.0) >= exit_threshold {
        println!("PASS — top-{top_k} hit rate >= 90% (Sprint E exit criterion)");
        0
    } else {
        println!("FAIL — top-{top_k} hit rate < 90% (Sprint E exit criterion)");
        1
    };
    if let Some(path) = out_path {
        #[derive(Serialize)]
        struct Baseline {
            queries_path: String,
            embedder: String,
            top_k: usize,
            total: usize,
            passed: usize,
            pct: f64,
            avg_ms_per_query: u64,
            results: Vec<QueryResult>,
        }
        let baseline = Baseline {
            queries_path: queries_path.to_string_lossy().to_string(),
            embedder: label,
            top_k,
            total,
            passed: passed_count,
            pct,
            avg_ms_per_query: avg_ms,
            results,
        };
        match serde_json::to_string_pretty(&baseline) {
            Ok(json) => {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(err) = std::fs::write(&path, json) {
                    eprintln!("bestel kb-eval: write baseline {path:?} failed: {err}");
                } else {
                    println!("baseline written: {}", path.display());
                }
            }
            Err(err) => eprintln!("bestel kb-eval: serialize baseline failed: {err}"),
        }
    }
    exit_code
}

fn run_db(args: Vec<String>) {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("bestel: failed to start tokio runtime: {e}");
            std::process::exit(1);
        }
    };
    let exit_code = rt.block_on(async { run_db_inner(args).await });
    std::process::exit(exit_code);
}

async fn run_db_inner(raw_args: Vec<String>) -> i32 {
    use bestel_core::persistence::{
        default_db_path, global_db, query_runs_failing_identity, Db,
    };
    use std::path::PathBuf;
    use std::time::Instant;

    let Some(sub) = raw_args.first().cloned() else {
        eprintln!("bestel db: missing subcommand (migrate|stats|import|query|bench)");
        return 2;
    };
    let rest: Vec<String> = raw_args.into_iter().skip(1).collect();

    match sub.as_str() {
        "migrate" => {
            let path = match default_db_path() {
                Some(p) => p,
                None => {
                    eprintln!("bestel db migrate: no home dir resolvable");
                    return 1;
                }
            };
            match Db::open(&path) {
                Ok(_) => {
                    println!("OK migrated {}", path.display());
                    0
                }
                Err(e) => {
                    eprintln!("bestel db migrate: {e}");
                    1
                }
            }
        }
        "stats" => {
            let path = default_db_path();
            let db = match global_db() {
                Some(d) => d,
                None => {
                    eprintln!("bestel db stats: could not open default DB");
                    return 1;
                }
            };
            match db.stats(path.as_deref()) {
                Ok(s) => {
                    println!("schema_version    : {}", s.schema_version);
                    println!("runs              : {}", s.runs);
                    println!("lint_findings     : {}", s.lint_findings);
                    println!("verifier_results  : {}", s.verifier_results);
                    println!("sessions          : {}", s.sessions);
                    println!("builds_snapshot   : {}", s.builds_snapshot);
                    println!("kb_versions       : {}", s.kb_versions);
                    println!("bytes_on_disk     : {}", s.bytes_on_disk);
                    0
                }
                Err(e) => {
                    eprintln!("bestel db stats: {e}");
                    1
                }
            }
        }
        "import" => {
            let backup = rest.iter().any(|a| a == "--backup");
            match db_import(backup).await {
                Ok((inserted, skipped)) => {
                    println!("OK imported runs: {inserted} inserted, {skipped} skipped");
                    0
                }
                Err(e) => {
                    eprintln!("bestel db import: {e}");
                    1
                }
            }
        }
        "query" => {
            let mut sql: Option<String> = None;
            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--sql" {
                    i += 1;
                    if let Some(v) = rest.get(i) {
                        sql = Some(v.clone());
                    }
                } else {
                    eprintln!("bestel db query: unknown arg '{}'", rest[i]);
                    return 2;
                }
                i += 1;
            }
            let Some(sql) = sql else {
                eprintln!("bestel db query: missing --sql \"<SELECT ...>\"");
                return 2;
            };
            let db = match global_db() {
                Some(d) => d,
                None => {
                    eprintln!("bestel db query: could not open default DB");
                    return 1;
                }
            };
            match db_query_passthrough(&db, &sql) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("bestel db query: {e}");
                    1
                }
            }
        }
        "bench" => {
            let mut n: usize = 1000;
            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--n" {
                    i += 1;
                    if let Some(v) = rest.get(i).and_then(|s| s.parse::<usize>().ok()) {
                        n = v;
                    }
                } else {
                    eprintln!("bestel db bench: unknown arg '{}'", rest[i]);
                    return 2;
                }
                i += 1;
            }
            let path: PathBuf = std::env::temp_dir().join(format!(
                "bestel-bench-{}.sqlite3",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0)
            ));
            let db = match Db::open(&path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("bestel db bench: open {} failed: {e}", path.display());
                    return 1;
                }
            };
            if let Err(e) = db_bench_seed(&db, n) {
                eprintln!("bestel db bench: seed failed: {e}");
                return 1;
            }
            let started = Instant::now();
            let rows = match query_runs_failing_identity(&db) {
                Ok(rows) => rows,
                Err(e) => {
                    eprintln!("bestel db bench: query failed: {e}");
                    return 1;
                }
            };
            let elapsed = started.elapsed();
            println!(
                "OK seeded {n} runs · query 'haiku FAIL identity card' returned {} rows in {:.2} ms",
                rows.len(),
                elapsed.as_secs_f64() * 1000.0,
            );
            let _ = std::fs::remove_file(&path);
            let _ = std::fs::remove_file(path.with_extension("sqlite3-wal"));
            let _ = std::fs::remove_file(path.with_extension("sqlite3-shm"));
            if elapsed.as_millis() < 100 {
                0
            } else {
                eprintln!(
                    "WARN query exceeded 100ms ceiling (Sprint G2 exit criterion)"
                );
                1
            }
        }
        other => {
            eprintln!("bestel db: unknown subcommand '{other}'");
            2
        }
    }
}

async fn db_import(backup: bool) -> anyhow::Result<(usize, usize)> {
    use bestel_core::llm::recorder::{debug_runs_dir, PersistedRun};
    use bestel_core::persistence::{global_db, insert_run};
    use chrono::Utc;

    let dir = match debug_runs_dir() {
        Some(d) => d,
        None => return Ok((0, 0)),
    };
    if !dir.exists() {
        return Ok((0, 0));
    }
    if backup {
        let stamp = Utc::now().format("%Y%m%dT%H%M%S").to_string();
        let backup_root = dir
            .parent()
            .ok_or_else(|| anyhow::anyhow!("no parent for {}", dir.display()))?
            .join("backup")
            .join(format!("debug-chats-{stamp}"));
        std::fs::create_dir_all(&backup_root)?;
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("json") {
                let dst = backup_root.join(p.file_name().unwrap_or_default());
                std::fs::copy(&p, &dst)?;
            }
        }
        println!("backup copied to {}", backup_root.display());
    }
    let Some(db) = global_db() else {
        return Err(anyhow::anyhow!("could not open SQLite DB"));
    };
    let mut inserted = 0usize;
    let mut skipped = 0usize;
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let raw = match std::fs::read_to_string(&p) {
            Ok(s) => s,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        let run: PersistedRun = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        if let Err(_) = insert_run(&db, &run) {
            skipped += 1;
            continue;
        }
        inserted += 1;
    }
    Ok((inserted, skipped))
}

fn db_query_passthrough(
    db: &bestel_core::persistence::Db,
    sql: &str,
) -> anyhow::Result<()> {
    db.with_conn_rusqlite(|c| {
        let mut stmt = c.prepare(sql)?;
        let col_count = stmt.column_count();
        let col_names: Vec<String> = (0..col_count)
            .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
            .collect();
        println!("{}", col_names.join("\t"));
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let mut cells: Vec<String> = Vec::with_capacity(col_count);
            for i in 0..col_count {
                let v: rusqlite::types::Value = row.get(i)?;
                cells.push(format_sql_value(&v));
            }
            println!("{}", cells.join("\t"));
        }
        Ok(())
    })
}

fn format_sql_value(v: &rusqlite::types::Value) -> String {
    use rusqlite::types::Value;
    match v {
        Value::Null => "NULL".to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Real(r) => r.to_string(),
        Value::Text(s) => s.clone(),
        Value::Blob(b) => format!("<blob {} bytes>", b.len()),
    }
}

fn db_bench_seed(
    db: &bestel_core::persistence::Db,
    n: usize,
) -> anyhow::Result<()> {
    use bestel_core::llm::recorder::{ChatStats, PersistedRun};
    use bestel_core::persistence::{insert_lint_finding, insert_run, LintFindingRow};

    let providers = ["anthropic", "ollama"];
    let models = [
        "claude-haiku-4-5",
        "claude-sonnet-4-6",
        "claude-opus-4-7",
        "ollama:qwen3:8b",
    ];
    for i in 0..n {
        let model = models[i % models.len()];
        let provider = if model.starts_with("ollama") { providers[1] } else { providers[0] };
        let id = format!("bench-{i:06}");
        let run = PersistedRun {
            id: id.clone(),
            started_at: format!("2026-05-09T10:{:02}:{:02}Z", (i / 60) % 60, i % 60),
            ended_at: None,
            source: "bench".into(),
            provider: provider.into(),
            model_id: model.into(),
            model_display_name: model.into(),
            user_text: "build review".into(),
            user_attachments: Vec::new(),
            assistant_segments: Vec::new(),
            final_text: "draft".into(),
            stats: ChatStats::default(),
            error: None,
        };
        insert_run(db, &run)?;
        if model.contains("haiku") && i % 3 == 0 {
            insert_lint_finding(
                db,
                &LintFindingRow {
                    run_id: id,
                    rule: "BUILD_IDENTITY_REQUIRED".into(),
                    severity: "fail".into(),
                    message: "missing Identity: line".into(),
                },
            )?;
        }
    }
    Ok(())
}

fn run_mcp_serve() {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("bestel: failed to start tokio runtime: {e}");
            std::process::exit(1);
        }
    };
    let result: Result<()> = rt.block_on(async {
        match bestel_core::llm::kb::bootstrap_global().await {
            Ok(_) => {}
            Err(e) => eprintln!("bestel mcp-serve: KB bootstrap error (continuing): {e}"),
        }
        bestel_core::mcp::run_stdio_server().await
    });
    if let Err(e) = result {
        eprintln!("bestel mcp-serve: {e}");
        std::process::exit(1);
    }
}

fn run_tauri() {
    let result: Result<()> = (|| {
        tauri::Builder::default()
            .plugin(tauri_plugin_opener::init())
            .manage(AppState::new())
            .setup(|app| {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    bootstrap_runtime(handle).await;
                });
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                commands::ping,
                commands::list_models,
                commands::get_active_model,
                commands::set_active_model,
                commands::detect_providers,
                commands::list_builds,
                commands::get_active_build,
                commands::set_active_build,
                commands::clear_active_build,
                commands::preview_build,
                commands::chat_start,
                commands::chat_cancel,
                commands::chat_reset,
                commands::chat_log_persist,
                commands::chat_log_dir,
                commands::open_external,
                commands::open_link_modal,
                commands::update_link_modal_bounds,
                commands::close_link_modal,
                commands::list_debug_runs,
                commands::get_debug_run,
                commands::delete_debug_run,
                commands::delete_all_debug_runs,
                commands::lint_debug_run,
                commands::dev_export_all_runs,
                commands::list_api_keys,
                commands::set_api_key,
                commands::delete_api_key,
                commands::settings_get,
                commands::settings_set_verify_enabled,
                commands::debug_is_enabled,
                commands::prompts_list,
                commands::prompts_read,
                commands::prompts_write,
                commands::prompts_reset,
                commands::prompts_reset_all,
                commands::prompts_open_editor,
                commands::prompts_reveal_in_finder,
                commands::dev_panel_open,
                commands::dev_load_scenarios,
                commands::dev_load_real_prompts,
                commands::list_build_sheets,
                commands::get_build_sheet,
                commands::delete_build_sheet,
                commands::delete_active_build_sheet,
                commands::get_active_build_sheet_for_ui,
                commands::build_item_trade_url,
                commands::registry_preview,
                commands::registry_list,
                commands::registry_add,
                commands::registry_remove,
                commands::registry_refresh,
                commands::registry_touch,
                commands::registry_get_pob_path,
                commands::registry_check_paths,
                commands::suggestion_dismiss,
                commands::suggestion_check,
                titlebar::window_minimize,
                titlebar::window_toggle_maximize,
                titlebar::window_close,
            ])
            .run(tauri::generate_context!())
            .map_err(Into::into)
    })();

    if let Err(e) = result {
        eprintln!("bestel: fatal error: {e}");
        std::process::exit(1);
    }
}

async fn bootstrap_runtime(app: tauri::AppHandle) {
    if let Err(e) = boot_watcher(&app).await {
        tracing::warn!(target: "bestel", "watcher boot failed: {e:?}");
    }
    boot_prompts_watcher(&app);
    boot_provider_status(&app).await;
    boot_data_refresh();
    boot_kb_engine().await;
}

async fn boot_kb_engine() {
    match bestel_core::llm::kb::bootstrap_global().await {
        Ok(true) => tracing::info!(target: "bestel", "KB engine bootstrapped"),
        Ok(false) => tracing::info!(target: "bestel", "KB engine disabled — no embedding backend or corpus"),
        Err(e) => tracing::warn!(target: "bestel", "KB bootstrap error: {e:?}"),
    }
}

/// Spawn background daily refresh tasks for the offline data catalogues:
/// - repoe-fork game data (PoE1 + PoE2 mods, base items, gems, uniques, ...)
/// - official trade-stats catalogue (`/api/trade/data/stats`)
///
/// Each task runs after a short warm-up delay, validates the live payload,
/// atomic-writes a zstd-compressed copy under `~/.bestel/cache/`, and swaps
/// the in-memory snapshot. Failures are logged and never propagate — the
/// bundled compile-time snapshot stays as the offline-first fallback.
fn boot_data_refresh() {
    use bestel_core::sources::{repoe, repoe_refresh, trade_catalogue, FileCache, PoeHttpClient};

    match PoeHttpClient::new() {
        Ok(http) => {
            repoe_refresh::spawn(repoe::global(), http.clone());
            trade_catalogue::spawn(http, FileCache::new(FileCache::default_dir()));
        }
        Err(e) => tracing::warn!(target: "bestel", "data refresh tasks not started: {e:?}"),
    }
}

fn boot_prompts_watcher(app: &tauri::AppHandle) {
    let tracker = match app.try_state::<AppState>() {
        Some(s) => s.prompts_self_writes().clone(),
        None => return,
    };
    prompts_watcher::spawn(app.clone(), tracker);
}

async fn boot_watcher(app: &tauri::AppHandle) -> Result<()> {
    let watcher = match PobWatcher::start() {
        Ok(w) => Arc::new(w),
        Err(e) => return Err(e),
    };

    let state = app.state::<AppState>();
    state.set_watcher(watcher.clone());

    if let Some(initial) = watcher.initial_build() {
        state.build_ctx.set(initial.clone());
        emit_build(app, &initial);
    }

    let mut rx = watcher.subscribe();
    let app_for_loop = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(build) => {
                    if let Some(s) = app_for_loop.try_state::<AppState>() {
                        s.build_ctx.set(build.clone());
                    }
                    emit_build(&app_for_loop, &build);
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // Sprint v5 — clear-on-delete. When PoB removes the active build XML on
    // disk, drop the in-memory build and notify the UI. We compare the
    // removed path against the currently-loaded build's source path so
    // unrelated deletes in the same builds folder are ignored.
    let mut cleared_rx = watcher.subscribe_cleared();
    let app_for_cleared = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            match cleared_rx.recv().await {
                Ok(removed) => {
                    let Some(s) = app_for_cleared.try_state::<AppState>() else {
                        continue;
                    };
                    let active_path = s.build_ctx.get().map(|b| b.source_file);
                    if active_path.as_deref() == Some(removed.as_path()) {
                        s.build_ctx.clear();
                        let _ = app_for_cleared.emit(POB_CLEARED, ());
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    Ok(())
}

fn emit_build(app: &tauri::AppHandle, build: &bestel_core::pob::PobBuild) {
    let mtime_ms = std::fs::metadata(&build.source_file)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64);

    let summary = summary_dto_from_build(build, mtime_ms);
    let dto = PobBuildDto::from(build);
    let _ = app.emit(
        POB_BUILD,
        BuildEvent {
            summary,
            build: dto,
        },
    );
}

async fn boot_provider_status(app: &tauri::AppHandle) {
    let detection = detect_provider().await;
    let dto = DetectionDto::from(&detection);
    let _ = app.emit(
        PROVIDER_STATUS,
        ProviderStatusEvent { detection: dto },
    );
}

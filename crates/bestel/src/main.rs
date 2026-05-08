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
use tracing_subscriber::EnvFilter;

use bestel_core::llm::detect::detect_provider;
use bestel_core::pob::watcher::PobWatcher;

use crate::dto::{summary_dto_from_build, BuildEvent, DetectionDto, PobBuildDto, ProviderStatusEvent};
use crate::events::{POB_BUILD, PROVIDER_STATUS};
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

    run_tauri();
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .try_init();
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
    use bestel_core::llm::recorder::{save_run, Recorder};
    use bestel_core::llm::tools::BuildContext;
    use bestel_core::llm::{ChatMessage, LlmDelta, Provider, Role};
    use bestel_core::pob::parser::parse_file as parse_pob_file;
    use bestel_core::test_runner::{load_scenarios, Scenario};
    use tokio::sync::mpsc;

    bestel_core::llm::keys::apply_to_env();

    let mut scenarios_dir: Option<PathBuf> = None;
    let mut model_id = "deepseek-v3-2".to_string();
    let mut out_dir: Option<PathBuf> = None;
    let mut filter_name: Option<String> = None;
    let mut pob_fixture: Option<String> = None;

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
                if let Err(e) = save_run(&run) {
                    eprintln!("    save_run failed: {e}");
                }
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

    if total_errors > 0 {
        1
    } else {
        0
    }
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
    if let Err(e) = rt.block_on(bestel_core::mcp::run_stdio_server()) {
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
                commands::open_external,
                commands::open_link_modal,
                commands::update_link_modal_bounds,
                commands::close_link_modal,
                commands::list_debug_runs,
                commands::get_debug_run,
                commands::delete_debug_run,
                commands::delete_all_debug_runs,
                commands::dev_export_all_runs,
                commands::list_api_keys,
                commands::set_api_key,
                commands::delete_api_key,
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

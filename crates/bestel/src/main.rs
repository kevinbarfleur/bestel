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
    println!("  bestel                        launch the desktop app");
    println!("  bestel mcp-serve              run as MCP server (stdio JSON-RPC)");
    println!("  bestel --version / --help");
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

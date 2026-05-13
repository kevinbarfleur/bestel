use std::path::PathBuf;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, State, Window};
use tauri_plugin_opener::OpenerExt;
use tokio::sync::{mpsc, oneshot};

use bestel_core::llm::detect::{build_provider_for_profile, detect_provider};
use bestel_core::llm::models::{find_profile, list_profiles_with_local, save_active_profile};
use bestel_core::llm::recorder::{
    self as debug_recorder, PersistedAttachment, PersistedRun, Recorder,
};
use bestel_core::llm::{ChatMessage, LlmDelta, Role};
use bestel_core::pob::parser::{parse_file, parse_summary};
use std::sync::Mutex;

use crate::dto::{
    summary_dto_from_build, AttachmentDto, BuildEvent, BuildSheetDetailDto, BuildSheetSummaryDto,
    DeltaEvent, DetectionDto, KeyStatusDto, ModelProfileDto, PobBuildDto, PobBuildSummaryDto,
    ProviderStatusEvent,
};
use crate::events::{pump_deltas, LLM_DELTA, POB_BUILD, POB_CLEARED, PROVIDER_STATUS};
use crate::state::AppState;

#[tauri::command]
pub fn ping(name: String) -> String {
    format!("pong {name}")
}

#[tauri::command]
pub async fn list_models() -> Vec<ModelProfileDto> {
    list_profiles_with_local()
        .await
        .iter()
        .map(ModelProfileDto::from)
        .collect()
}

#[tauri::command]
pub fn get_active_model(state: State<'_, AppState>) -> ModelProfileDto {
    ModelProfileDto::from(&state.active_model_profile())
}

#[tauri::command]
pub async fn set_active_model(
    id: String,
    state: State<'_, AppState>,
) -> Result<ModelProfileDto, String> {
    let profile = find_profile(&id).ok_or_else(|| format!("unknown model id '{id}'"))?;
    save_active_profile(&id).map_err(|e| e.to_string())?;
    {
        let mut g = state.inner.lock().map_err(|_| "state poisoned".to_string())?;
        g.active_model_id = id.clone();
    }
    Ok(ModelProfileDto::from(&profile))
}

#[tauri::command]
pub async fn detect_providers(window: Window) -> Result<DetectionDto, String> {
    let detection = detect_provider().await;
    let dto = DetectionDto::from(&detection);
    let _ = window.emit(
        PROVIDER_STATUS,
        ProviderStatusEvent {
            detection: dto.clone(),
        },
    );
    Ok(dto)
}

#[tauri::command]
pub async fn list_builds(state: State<'_, AppState>) -> Result<Vec<PobBuildSummaryDto>, String> {
    let watcher = state.watcher().ok_or_else(|| "watcher not ready".to_string())?;
    let summaries = watcher.list_all_builds();
    Ok(summaries.iter().map(PobBuildSummaryDto::from).collect())
}

#[tauri::command]
pub async fn get_active_build(
    state: State<'_, AppState>,
) -> Result<Option<PobBuildDto>, String> {
    Ok(state.build_ctx.get().as_ref().map(PobBuildDto::from))
}

#[tauri::command]
pub async fn set_active_build(
    path: String,
    state: State<'_, AppState>,
    window: Window,
) -> Result<PobBuildDto, String> {
    let pb = PathBuf::from(&path);
    let build = parse_file(&pb).map_err(|e| format!("parse {}: {e}", pb.display()))?;

    let summary_mtime = parse_summary(&pb).ok().and_then(|s| {
        s.mtime
            .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
    });

    state.build_ctx.set(build.clone());

    let dto = PobBuildDto::from(&build);
    let summary = summary_dto_from_build(&build, summary_mtime);
    let _ = window.emit(
        POB_BUILD,
        BuildEvent {
            summary,
            build: dto.clone(),
        },
    );
    Ok(dto)
}

#[tauri::command]
pub async fn clear_active_build(
    state: State<'_, AppState>,
    window: Window,
) -> Result<(), String> {
    state.build_ctx.clear();
    let _ = window.emit(POB_CLEARED, ());
    Ok(())
}

/// Parse a PoB XML for the picker's detail pane *without* changing the
/// active build context. Used to render vitals/resists for hovered/selected
/// items in the build picker before the user commits.
#[tauri::command]
pub async fn preview_build(path: String) -> Result<PobBuildDto, String> {
    let pb = PathBuf::from(&path);
    let build = parse_file(&pb).map_err(|e| format!("parse {}: {e}", pb.display()))?;
    Ok(PobBuildDto::from(&build))
}

#[tauri::command]
pub async fn chat_reset(state: State<'_, AppState>) -> Result<(), String> {
    let mut g = state.inner.lock().map_err(|_| "state poisoned".to_string())?;
    g.history.clear();
    if let Some(tx) = g.cancel_tx.take() {
        let _ = tx.send(());
    }
    g.active_session = None;
    Ok(())
}

#[tauri::command]
pub async fn chat_cancel(state: State<'_, AppState>) -> Result<(), String> {
    let mut g = state.inner.lock().map_err(|_| "state poisoned".to_string())?;
    if let Some(tx) = g.cancel_tx.take() {
        let _ = tx.send(());
    }
    g.active_session = None;
    Ok(())
}

#[tauri::command]
pub async fn chat_start(
    prompt: String,
    attachments: Option<Vec<AttachmentDto>>,
    state: State<'_, AppState>,
    window: Window,
) -> Result<u64, String> {
    let trimmed = prompt.trim();
    let attachments = attachments.unwrap_or_default();
    if trimmed.is_empty() && attachments.is_empty() {
        return Err("prompt cannot be empty".into());
    }

    let session_id = state.next_session_id();

    let core_attachments: Vec<bestel_core::llm::Attachment> =
        attachments.iter().map(Into::into).collect();

    let (history_snapshot, build_ctx) = {
        let mut g = state.inner.lock().map_err(|_| "state poisoned".to_string())?;
        if g.active_session.is_some() {
            return Err("a chat is already running; cancel it first".into());
        }
        g.history.push(ChatMessage {
            role: Role::User,
            content: trimmed.to_string(),
            attachments: core_attachments,
        });
        g.active_session = Some(session_id);
        (g.history.clone(), state.build_ctx.clone())
    };

    let profile = state.active_model_profile();
    let provider_kind_label = match profile.provider {
        bestel_core::llm::models::ProviderKind::Anthropic => "anthropic",
        bestel_core::llm::models::ProviderKind::Ollama => "ollama",
    };
    let provider = build_provider_for_profile(&profile)
        .await
        .map_err(|e| format!("build provider: {e}"))?;

    let recorder = Arc::new(Mutex::new(Recorder::new(
        "ui",
        provider_kind_label,
        profile.model_id.clone(),
        profile.display_name.clone(),
        trimmed.to_string(),
        attachments
            .iter()
            .map(|a| PersistedAttachment {
                name: a.name.clone(),
                mime: a.mime.clone(),
            })
            .collect(),
    )));

    let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
    {
        let mut g = state.inner.lock().map_err(|_| "state poisoned".to_string())?;
        g.cancel_tx = Some(cancel_tx);
    }

    let (delta_tx, delta_rx) = mpsc::unbounded_channel::<LlmDelta>();
    let pump_window = window.clone();
    let recorder_for_pump = recorder.clone();
    let pump_handle = tauri::async_runtime::spawn(async move {
        pump_deltas(
            pump_window,
            session_id,
            delta_rx,
            cancel_rx,
            Some(recorder_for_pump),
        )
        .await
    });

    let provider = Arc::new(provider);
    let provider_for_run = provider.clone();
    let run_handle = tauri::async_runtime::spawn(async move {
        provider_for_run
            .run(history_snapshot, build_ctx, delta_tx)
            .await
    });

    let state_app: AppHandle = window.app_handle().clone();
    let window_for_finish = window.clone();
    let recorder_for_save = recorder.clone();
    tauri::async_runtime::spawn(async move {
        let result = run_handle.await;
        let cancelled = pump_handle.await.unwrap_or(false);

        let app_state = state_app.try_state::<AppState>();
        let mut final_text = String::new();
        match &result {
            Ok(Ok(assistant_text)) => {
                if !cancelled {
                    final_text = assistant_text.clone();
                    if let Some(s) = app_state.as_ref() {
                        if let Ok(mut g) = s.inner.lock() {
                            g.history.push(ChatMessage {
                                role: Role::Assistant,
                                content: assistant_text.clone(),
                                attachments: Vec::new(),
                            });
                        }
                    }
                    let _ = window_for_finish
                        .emit(LLM_DELTA, DeltaEvent::Completed { session_id });
                }
            }
            Ok(Err(e)) => {
                if !cancelled {
                    let _ = window_for_finish.emit(
                        LLM_DELTA,
                        DeltaEvent::Error {
                            session_id,
                            message: format!("{e}"),
                        },
                    );
                }
            }
            Err(join_err) => {
                let _ = window_for_finish.emit(
                    LLM_DELTA,
                    DeltaEvent::Error {
                        session_id,
                        message: format!("task join error: {join_err}"),
                    },
                );
            }
        }

        if let Some(rec_arc) = Arc::try_unwrap(recorder_for_save).ok().and_then(|m| m.into_inner().ok()) {
            // Per-turn debug autosave is OPT-IN. Without
            // `BESTEL_DEBUG_RECORDER=1` set at launch, the recorder
            // builds the run in memory (so the in-app debug button can
            // still copy it to the clipboard) but does NOT write a
            // JSON file under `~/.bestel/runtime/debug-chats/`. The
            // user-facing chat history (conversation-logs) is
            // unaffected — that's how chats survive a restart.
            if std::env::var("BESTEL_DEBUG_RECORDER").ok().as_deref() == Some("1") {
                let run = rec_arc.finish(final_text);
                if let Err(e) = debug_recorder::save_run(&run) {
                    eprintln!("failed to persist debug run: {e}");
                }
            }
        }

        if let Some(s) = app_state.as_ref() {
            if let Ok(mut g) = s.inner.lock() {
                if g.active_session == Some(session_id) {
                    g.active_session = None;
                    g.cancel_tx = None;
                }
            }
        }
    });

    Ok(session_id)
}

#[tauri::command]
pub async fn list_debug_runs() -> Result<Vec<PersistedRun>, String> {
    debug_recorder::list_runs().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_debug_run(id: String) -> Result<Option<PersistedRun>, String> {
    debug_recorder::get_run(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_debug_run(id: String) -> Result<(), String> {
    debug_recorder::delete_run(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_all_debug_runs() -> Result<usize, String> {
    debug_recorder::delete_all_runs().map_err(|e| e.to_string())
}

/// Run the Sprint A response linter against a stored debug run. The dev
/// panel calls this on selection to surface FAIL/WARN findings without
/// having to bake the rules into the frontend. Returning `None` means no
/// run was found for the supplied id; an empty `findings` array means the
/// run was clean.
#[tauri::command]
pub async fn lint_debug_run(
    id: String,
) -> Result<Option<bestel_core::test_runner::LintReport>, String> {
    let run = match debug_recorder::get_run(&id).map_err(|e| e.to_string())? {
        Some(r) => r,
        None => return Ok(None),
    };
    Ok(Some(bestel_core::test_runner::lint_run(&run)))
}

/// Export every persisted run as a single JSON document under
/// `~/.bestel/exports/runs-export-<timestamp>.json`. The returned string
/// is the absolute path to the written file. Each run is reshaped into a
/// review-friendly entry (question, answer, tools, model metadata, stats)
/// so an external expert agent can grade response quality without having
/// to walk the raw stream-event format. Tool outputs are truncated to
/// 4 KB per call to keep file size manageable.
#[tauri::command]
pub async fn dev_export_all_runs() -> Result<String, String> {
    use chrono::Utc;
    use serde_json::json;

    let runs = debug_recorder::list_runs().map_err(|e| e.to_string())?;

    let exports_dir = dirs::home_dir()
        .ok_or_else(|| "no home dir".to_string())?
        .join(".bestel")
        .join("exports");
    std::fs::create_dir_all(&exports_dir).map_err(|e| format!("create exports dir: {e}"))?;

    const TOOL_OUTPUT_CAP_BYTES: usize = 4096;

    let entries: Vec<serde_json::Value> = runs
        .iter()
        .map(|run| {
            // Reshape segments into a clean per-run summary. We split the
            // assistant transcript into:
            //   - `answer`     : every Text segment concatenated
            //   - `reasoning`  : every Reasoning segment concatenated
            //   - `tools`      : structured per-call list with truncated outputs
            let mut answer_parts: Vec<&str> = Vec::new();
            let mut reasoning_parts: Vec<&str> = Vec::new();
            let mut tool_entries: Vec<serde_json::Value> = Vec::new();

            for seg in &run.assistant_segments {
                match seg {
                    debug_recorder::PersistedSegment::Text { text } => {
                        answer_parts.push(text.as_str());
                    }
                    debug_recorder::PersistedSegment::Reasoning { text } => {
                        reasoning_parts.push(text.as_str());
                    }
                    debug_recorder::PersistedSegment::Tool {
                        id,
                        name,
                        detail,
                        outputs,
                        status,
                        summary,
                    } => {
                        let combined = outputs.join("");
                        let (output_excerpt, truncated) = if combined.len() > TOOL_OUTPUT_CAP_BYTES {
                            let mut end = TOOL_OUTPUT_CAP_BYTES;
                            while !combined.is_char_boundary(end) && end > 0 {
                                end -= 1;
                            }
                            (combined[..end].to_string(), true)
                        } else {
                            (combined, false)
                        };
                        tool_entries.push(json!({
                            "id": id,
                            "name": name,
                            "detail": detail,
                            "status": status,
                            "summary": summary,
                            "output_excerpt": output_excerpt,
                            "output_truncated": truncated,
                        }));
                    }
                }
            }

            let answer = answer_parts.join("");
            let reasoning = reasoning_parts.join("");

            json!({
                "id": run.id,
                "started_at": run.started_at,
                "ended_at": run.ended_at,
                "source": run.source,
                "model": {
                    "id": run.model_id,
                    "display_name": run.model_display_name,
                    "provider": run.provider,
                },
                "question": run.user_text,
                "user_attachments": run.user_attachments,
                "answer": answer,
                "reasoning": if reasoning.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::String(reasoning)
                },
                "tools": tool_entries,
                "stats": run.stats,
                "error": run.error,
            })
        })
        .collect();

    let now = Utc::now();
    let stamp = now.format("%Y%m%d-%H%M%S").to_string();
    let file_name = format!("runs-export-{stamp}.json");
    let path = exports_dir.join(&file_name);

    let envelope = json!({
        "exported_at": now.to_rfc3339(),
        "bestel_version": env!("CARGO_PKG_VERSION"),
        "schema_version": 1,
        "total_runs": entries.len(),
        "tool_output_cap_bytes": TOOL_OUTPUT_CAP_BYTES,
        "runs": entries,
    });

    let body = serde_json::to_vec_pretty(&envelope).map_err(|e| format!("serialize: {e}"))?;
    std::fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))?;

    Ok(path.display().to_string())
}

#[tauri::command]
pub async fn open_external(url: String, app: AppHandle) -> Result<(), String> {
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

/// Label of the in-app link viewer sub-webview. Single instance — opening a
/// new URL while one is already mounted reuses the slot.
const LINK_MODAL_LABEL: &str = "link-modal";

/// Spawn (or refresh) a child webview overlay positioned over the main
/// window at the given logical bounds. The frontend measures the modal
/// frame and sends `(x, y, w, h)` so the native webview slots into the
/// reserved viewport.
#[tauri::command]
pub async fn open_link_modal(
    url: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    app: AppHandle,
) -> Result<(), String> {
    let parsed = url::Url::parse(&url).map_err(|e| format!("invalid url: {e}"))?;

    // If a previous viewer is still mounted, just steer it to the new URL +
    // bounds — keeps focus, avoids the WebView2 boot flash.
    if let Some(existing) = app.get_webview(LINK_MODAL_LABEL) {
        existing
            .navigate(parsed)
            .map_err(|e| format!("navigate: {e}"))?;
        existing
            .set_position(tauri::LogicalPosition::new(x, y))
            .map_err(|e| format!("position: {e}"))?;
        existing
            .set_size(tauri::LogicalSize::new(w, h))
            .map_err(|e| format!("size: {e}"))?;
        return Ok(());
    }

    let main = app
        .get_window("main")
        .ok_or_else(|| "main window not found".to_string())?;

    let builder =
        tauri::webview::WebviewBuilder::new(LINK_MODAL_LABEL, tauri::WebviewUrl::External(parsed));

    main.add_child(
        builder,
        tauri::LogicalPosition::new(x, y),
        tauri::LogicalSize::new(w, h),
    )
    .map_err(|e| format!("create webview: {e}"))?;

    Ok(())
}

/// Move/resize the running viewer (called by the frontend on window resize
/// or when the modal frame's bounding rect changes).
#[tauri::command]
pub async fn update_link_modal_bounds(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    app: AppHandle,
) -> Result<(), String> {
    if let Some(wv) = app.get_webview(LINK_MODAL_LABEL) {
        wv.set_position(tauri::LogicalPosition::new(x, y))
            .map_err(|e| format!("position: {e}"))?;
        wv.set_size(tauri::LogicalSize::new(w, h))
            .map_err(|e| format!("size: {e}"))?;
    }
    Ok(())
}

/// Tear down the link viewer.
#[tauri::command]
pub async fn close_link_modal(app: AppHandle) -> Result<(), String> {
    if let Some(wv) = app.get_webview(LINK_MODAL_LABEL) {
        wv.close().map_err(|e| format!("close: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
pub fn list_api_keys() -> Vec<KeyStatusDto> {
    let keys = bestel_core::llm::keys::load_keys();
    bestel_core::llm::keys::ALLOWED_KEYS
        .iter()
        .map(|name| {
            // Disk wins for the picker preview; env-only keys (set externally,
            // not via the UI) still show as `set` because the probe in
            // `detect.rs` already covers that case via std::env::var.
            let stored = keys.get(*name);
            let env_set = std::env::var(name).is_ok();
            let set = stored.is_some() || env_set;
            let masked_preview = stored
                .and_then(|v| bestel_core::llm::keys::mask_preview(v))
                .or_else(|| {
                    if env_set {
                        // Don't echo OS-environment values into the UI; we
                        // mark them set without preview, so the user can't
                        // accidentally lift them via DevTools / IPC logs.
                        Some("(from environment)".to_string())
                    } else {
                        None
                    }
                });
            KeyStatusDto {
                env_name: (*name).to_string(),
                set,
                masked_preview,
            }
        })
        .collect()
}

#[tauri::command]
pub async fn set_api_key(env_name: String, value: String) -> Result<(), String> {
    bestel_core::llm::keys::save_key(&env_name, &value).map_err(|e| e.to_string())?;
    std::env::set_var(&env_name, value.trim());
    Ok(())
}

#[tauri::command]
pub async fn delete_api_key(env_name: String) -> Result<(), String> {
    bestel_core::llm::keys::delete_key(&env_name).map_err(|e| e.to_string())?;
    std::env::remove_var(&env_name);
    Ok(())
}

/// Returns the current persisted settings. Default-on for new installs and
/// for any read failure (corrupt file, missing home dir).
#[tauri::command]
pub fn settings_get() -> bestel_core::settings::Settings {
    bestel_core::settings::load()
}

/// Toggles the Chain-of-Verification post-draft pipeline. Live update —
/// the next chat turn picks up the new value via `is_verify_enabled()`,
/// no restart required.
#[tauri::command]
pub fn settings_set_verify_enabled(enabled: bool) -> Result<(), String> {
    bestel_core::settings::set_verify_enabled(enabled).map_err(|e| e.to_string())
}

/// True when Bestel was launched with a valid `BESTEL_DEBUG_PORT=<port>` env
/// var — i.e. the WebView2 CDP listener is exposed and the `bestel-driver`
/// harness can attach. The frontend uses this to decide whether to install
/// the `window.__bestel` debug bridge. Returning false means the bridge is
/// NEVER attached, even in dev builds without the env var set, so a stray
/// page can't poke at the chat store via DevTools.
#[tauri::command]
pub fn debug_is_enabled() -> bool {
    std::env::var("BESTEL_DEBUG_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .is_some()
}

const PROMPT_EDITOR_LABEL: &str = "prompt-editor";
const DEV_PANEL_LABEL: &str = "dev-panel";

#[tauri::command]
pub fn prompts_list() -> Result<bestel_core::prompts::PromptTree, String> {
    bestel_core::prompts::list_files().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn prompts_read(path: String) -> Result<bestel_core::prompts::PromptFile, String> {
    bestel_core::prompts::read_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn prompts_write(
    path: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if let Some(root) = bestel_core::prompts::prompts_dir() {
        state.prompts_self_writes().record(&root.join(&path));
    }
    bestel_core::prompts::write_file(&path, &content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn prompts_reset(path: String, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(root) = bestel_core::prompts::prompts_dir() {
        state.prompts_self_writes().record(&root.join(&path));
    }
    bestel_core::prompts::reset_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn prompts_reset_all(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(root) = bestel_core::prompts::prompts_dir() {
        for name in [
            "SYSTEM_PROMPT.md",
            "CORE_KNOWLEDGE.md",
            "local_addendum.md",
        ] {
            state.prompts_self_writes().record(&root.join(name));
        }
    }
    bestel_core::prompts::reset_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn prompts_open_editor(app: AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window(PROMPT_EDITOR_LABEL) {
        let _ = existing.show();
        let _ = existing.set_focus();
        let _ = existing.unminimize();
        return Ok(());
    }

    // Window declared in tauri.conf.json with visible: false. The config
    // entry handles geometry; here we just bring it up. If the user wiped
    // it from config we fall through to a builder.
    let url = tauri::WebviewUrl::App("prompt-editor.html".into());
    tauri::WebviewWindowBuilder::new(&app, PROMPT_EDITOR_LABEL, url)
        .title("Prompts & documentation — Bestel")
        .inner_size(1280.0, 820.0)
        .min_inner_size(900.0, 560.0)
        .decorations(false)
        .resizable(true)
        .visible(true)
        .build()
        .map_err(|e| format!("create prompt editor window: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn dev_load_scenarios(rel_path: Option<String>) -> Result<Vec<bestel_core::test_runner::Scenario>, String> {
    use std::path::PathBuf;

    let rel = rel_path.unwrap_or_else(|| "tests/scenarios".to_string());
    // Resolve relative to the workspace root by walking up from the cwd.
    let mut candidate = std::env::current_dir().map_err(|e| e.to_string())?;
    let target = loop {
        let p = candidate.join(&rel);
        if p.is_dir() {
            break p;
        }
        if !candidate.pop() {
            return Err(format!("scenarios dir '{rel}' not found from cwd"));
        }
    };
    bestel_core::test_runner::load_scenarios(&PathBuf::from(target)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn dev_load_real_prompts() -> Result<Vec<bestel_core::test_runner::RealPrompt>, String> {
    use std::path::PathBuf;

    let mut candidate = std::env::current_dir().map_err(|e| e.to_string())?;
    let target = loop {
        let p = candidate.join("docs/test_prompts/real_user_prompts.toml");
        if p.is_file() {
            break p;
        }
        if !candidate.pop() {
            return Err("real_user_prompts.toml not found from cwd".into());
        }
    };
    bestel_core::test_runner::load_real_prompts(&PathBuf::from(target)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn dev_panel_open(app: AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window(DEV_PANEL_LABEL) {
        let _ = existing.show();
        let _ = existing.set_focus();
        let _ = existing.unminimize();
        return Ok(());
    }

    // Window declared in tauri.conf.json with visible: false. The config
    // entry handles geometry; the builder is the fallback.
    let url = tauri::WebviewUrl::App("dev-panel.html".into());
    tauri::WebviewWindowBuilder::new(&app, DEV_PANEL_LABEL, url)
        .title("Dev panel — Bestel")
        .inner_size(1280.0, 820.0)
        .min_inner_size(900.0, 560.0)
        .decorations(false)
        .resizable(true)
        .visible(true)
        .build()
        .map_err(|e| format!("create dev panel window: {e}"))?;
    Ok(())
}

#[tauri::command]
pub async fn prompts_reveal_in_finder(app: AppHandle, path: String) -> Result<(), String> {
    let root = bestel_core::prompts::prompts_dir()
        .ok_or_else(|| "prompts dir unresolvable".to_string())?;
    let target = if path.is_empty() {
        root
    } else {
        let candidate = root.join(&path);
        if candidate.exists() {
            candidate
        } else {
            root
        }
    };
    app.opener()
        .open_path(target.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| format!("reveal: {e}"))
}

/// Resolve the conversation-logs directory under `~/.bestel/runtime/`.
/// Per-chat JSON files live here and are updated by [`chat_log_persist`]
/// on every history snapshot. The folder is the source of truth for
/// debugging — a coding agent can read the latest file to inspect what
/// happened in a chat without needing the user to copy-paste the timeline.
fn conversation_logs_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|h| h.join(".bestel").join("runtime").join("conversation-logs"))
        .ok_or_else(|| "no home dir".to_string())
}

#[tauri::command]
pub fn chat_log_persist(chat_id: String, json: String) -> Result<String, String> {
    let dir = conversation_logs_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create logs dir: {e}"))?;
    // Sanitize chat_id to avoid path traversal — accept only ascii
    // alphanumerics, dash, and underscore. Reject anything else outright
    // rather than silently mangling the filename.
    if !chat_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        || chat_id.is_empty()
    {
        return Err(format!("invalid chat_id '{chat_id}'"));
    }
    let path = dir.join(format!("{chat_id}.json"));
    std::fs::write(&path, json).map_err(|e| format!("write log: {e}"))?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn chat_log_dir() -> Result<String, String> {
    let dir = conversation_logs_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create logs dir: {e}"))?;
    Ok(dir.to_string_lossy().to_string())
}

// ─── Build Sheets management commands ─────────────────────────────────
//
// Surface the persisted Build Sheets (`build_sheets` table) to the
// BuildPicker so the user can audit, preview, and delete sheets without
// going through the agent. Pure CRUD over `bestel_core::sheets::store`.

#[tauri::command]
pub async fn list_build_sheets() -> Result<Vec<BuildSheetSummaryDto>, String> {
    let db = bestel_core::persistence::global_db()
        .ok_or_else(|| "database is not initialized".to_string())?;
    let rows = bestel_core::sheets::store::list_all_sheets(&db)
        .map_err(|e| format!("list sheets: {e}"))?;
    Ok(rows
        .into_iter()
        .map(|r| BuildSheetSummaryDto {
            id: r.id,
            fingerprint: r.fingerprint,
            pob_hash: r.pob_hash,
            name: r.name,
            schema_version: r.schema_version,
            authored_at: r.authored_at,
            updated_at: r.updated_at,
            authored_in_chat: r.authored_in_chat,
            validated: r.validated,
        })
        .collect())
}

#[tauri::command]
pub async fn get_build_sheet(id: String) -> Result<Option<BuildSheetDetailDto>, String> {
    let db = bestel_core::persistence::global_db()
        .ok_or_else(|| "database is not initialized".to_string())?;
    let row = bestel_core::sheets::store::get_by_id(&db, &id)
        .map_err(|e| format!("get sheet: {e}"))?;
    let Some(row) = row else { return Ok(None) };
    let parsed = row
        .parse_payload()
        .map_err(|e| format!("parse sheet payload: {e}"))?;
    let payload = serde_json::to_value(&parsed)
        .map_err(|e| format!("serialize sheet payload: {e}"))?;
    Ok(Some(BuildSheetDetailDto {
        id: row.id,
        fingerprint: row.fingerprint,
        pob_hash: row.pob_hash,
        name: row.name,
        schema_version: row.schema_version,
        authored_at: row.authored_at,
        updated_at: row.updated_at,
        authored_in_chat: row.authored_in_chat,
        validated: row.validated,
        payload,
    }))
}

#[tauri::command]
pub async fn delete_build_sheet(id: String) -> Result<bool, String> {
    let db = bestel_core::persistence::global_db()
        .ok_or_else(|| "database is not initialized".to_string())?;
    bestel_core::sheets::store::delete_sheet(&db, &id)
        .map_err(|e| format!("delete sheet: {e}"))
}

/// Convenience: delete the Build Sheet (if any) attached to the currently
/// active build's fingerprint. Returns `(deleted, sheet_id)` — `deleted`
/// is true if a row was removed, `sheet_id` is `None` when no sheet
/// existed in the first place. Used by `bestel-driver` test scenarios to
/// force the `Build sheet: absent` runtime tag without going through the
/// UI's sheet picker.
#[tauri::command]
pub async fn delete_active_build_sheet(
    state: State<'_, AppState>,
) -> Result<DeleteActiveSheetResult, String> {
    let Some(build) = state.build_ctx.get() else {
        return Err("no active build".to_string());
    };
    let Some(fingerprint) = bestel_core::sheets::compute_fingerprint_from_pob(&build) else {
        return Err("failed to compute fingerprint for active build".to_string());
    };
    let db = bestel_core::persistence::global_db()
        .ok_or_else(|| "database is not initialized".to_string())?;
    let Some(row) = bestel_core::sheets::store::find_by_fingerprint(&db, &fingerprint)
        .map_err(|e| format!("find_by_fingerprint: {e}"))?
    else {
        return Ok(DeleteActiveSheetResult {
            deleted: false,
            sheet_id: None,
            fingerprint,
        });
    };
    let id = row.id.clone();
    bestel_core::sheets::store::delete_sheet(&db, &id)
        .map_err(|e| format!("delete sheet: {e}"))?;
    Ok(DeleteActiveSheetResult {
        deleted: true,
        sheet_id: Some(id),
        fingerprint,
    })
}

#[derive(serde::Serialize)]
pub struct DeleteActiveSheetResult {
    pub deleted: bool,
    pub sheet_id: Option<String>,
    pub fingerprint: String,
}

/// Frontend-side equivalent of the agent's `get_active_build_sheet` tool:
/// look up the persisted Build Sheet (if any) for the currently active
/// build's fingerprint, plus a `pob_hash_match` flag so the sidebar card
/// can render fresh vs stale without firing an LLM turn. Used by the
/// build store on app boot, build attach, and chat switch to rehydrate
/// the sheet sidebar card from local DB instead of waiting for the
/// agent to volunteer the tool call.
#[tauri::command]
pub async fn get_active_build_sheet_for_ui(
    state: State<'_, AppState>,
) -> Result<Option<ActiveBuildSheetDto>, String> {
    let Some(build) = state.build_ctx.get() else {
        return Ok(None);
    };
    let Some(fingerprint) = bestel_core::sheets::compute_fingerprint_from_pob(&build) else {
        return Ok(None);
    };
    let db = bestel_core::persistence::global_db()
        .ok_or_else(|| "database is not initialized".to_string())?;
    let Some(row) = bestel_core::sheets::store::find_by_fingerprint(&db, &fingerprint)
        .map_err(|e| format!("find_by_fingerprint: {e}"))?
    else {
        return Ok(None);
    };
    let current_hash = bestel_core::sheets::compute_pob_hash_from_build(&build);
    let pob_hash_match = current_hash == row.pob_hash;
    // Compute the live build's 5 signatures so the frontend drift chip
    // strip can render per-axis state without a second round-trip.
    let current_sigs = bestel_core::pob::signatures::BuildSignatures::from_build(&build);
    // Backfill: if the stored sheet has no signatures yet AND the hash
    // matches (so the live build IS the authoring state), persist the
    // current sigs into the row. After this one-shot backfill, every
    // subsequent open of the chat renders true drift, not "drift since
    // backfill".
    let mut authored = SheetSignaturesDto {
        identity: row.identity_sig.clone(),
        gear: row.gear_sig.clone(),
        tree: row.tree_sig.clone(),
        skill: row.skill_sig.clone(),
        config: row.config_sig.clone(),
    };
    if authored.identity.is_none() && pob_hash_match {
        if let Err(e) = bestel_core::sheets::store::update_sheet_signatures(
            &db,
            &row.id,
            &current_sigs.identity,
            &current_sigs.gear,
            &current_sigs.tree,
            &current_sigs.skill,
            &current_sigs.config,
        ) {
            tracing::warn!(error = ?e, "backfill sheet signatures failed");
        } else {
            authored = SheetSignaturesDto {
                identity: Some(current_sigs.identity.clone()),
                gear: Some(current_sigs.gear.clone()),
                tree: Some(current_sigs.tree.clone()),
                skill: Some(current_sigs.skill.clone()),
                config: Some(current_sigs.config.clone()),
            };
        }
    }
    let parsed = row
        .parse_payload()
        .map_err(|e| format!("parse sheet payload: {e}"))?;
    let payload = serde_json::to_value(&parsed)
        .map_err(|e| format!("serialize sheet payload: {e}"))?;
    Ok(Some(ActiveBuildSheetDto {
        id: row.id,
        fingerprint: row.fingerprint,
        pob_hash: row.pob_hash,
        name: row.name,
        schema_version: row.schema_version,
        authored_at: row.authored_at,
        updated_at: row.updated_at,
        authored_in_chat: row.authored_in_chat,
        validated: row.validated,
        payload,
        pob_hash_match,
        authored_signatures: authored,
        current_signatures: SheetSignaturesDto {
            identity: Some(current_sigs.identity),
            gear: Some(current_sigs.gear),
            tree: Some(current_sigs.tree),
            skill: Some(current_sigs.skill),
            config: Some(current_sigs.config),
        },
    }))
}

/// Per-axis 5 signatures, all `Option<String>` because pre-v3 sheets have
/// NULLs until backfilled (see ActiveBuildSheetDto::authored_signatures).
/// `current_signatures` is always fully populated.
#[derive(serde::Serialize, Clone, Default)]
pub struct SheetSignaturesDto {
    pub identity: Option<String>,
    pub gear: Option<String>,
    pub tree: Option<String>,
    pub skill: Option<String>,
    pub config: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ActiveBuildSheetDto {
    pub id: String,
    pub fingerprint: String,
    pub pob_hash: String,
    pub name: String,
    pub schema_version: i64,
    pub authored_at: String,
    pub updated_at: String,
    pub authored_in_chat: Option<String>,
    pub validated: bool,
    pub payload: serde_json::Value,
    pub pob_hash_match: bool,
    /// Signatures of the build at sheet authoring time. NULL fields mean
    /// the sheet is pre-v3 and the backfill hasn't kicked in yet
    /// (`pob_hash_match` was false when the sheet was last read with a
    /// live build attached).
    pub authored_signatures: SheetSignaturesDto,
    /// Signatures of the live PoB right now. Always populated (the active
    /// build was successfully parsed for this call).
    pub current_signatures: SheetSignaturesDto,
}

// ─── Sprint v3 — Item → Trade query ─────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct ItemTradeQueryInput {
    pub game: String,
    pub mods: Vec<ItemModInput>,
    #[serde(default)]
    pub league: Option<String>,
    #[serde(default)]
    pub rarity: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct ItemModInput {
    pub kind: String,
    pub text: String,
}

#[derive(serde::Serialize)]
pub struct ItemTradeQueryResult {
    pub url: String,
    pub league: String,
    pub total: u64,
    pub resolved_stat_ids: Vec<String>,
    pub unresolved_mods: Vec<String>,
}

/// Build a shareable PoE trade URL pre-filled with stat filters derived from
/// an item card's explicit + fractured + crafted mods. Resolves each mod
/// phrase to its trade-stat id via the bundled catalogue (with HTTP
/// fallback), POSTs the search to GGG to obtain the canonical share id,
/// and returns the URL the frontend opens in the OS default browser.
///
/// Implicit + enchant mods are dropped: they're base-determined and rarely
/// what a user searching for a "similar craft" wants in the filter set.
#[tauri::command]
pub async fn build_item_trade_url(
    payload: ItemTradeQueryInput,
) -> Result<ItemTradeQueryResult, String> {
    use bestel_core::sources::cache::FileCache;
    use bestel_core::sources::http::PoeHttpClient;
    use bestel_core::sources::trade::TradeClient;
    use bestel_core::PoeVersion;

    let game = match payload.game.trim().to_ascii_lowercase().as_str() {
        "poe2" | "poe 2" | "pathofexile2" | "path of exile 2" => PoeVersion::Poe2,
        _ => PoeVersion::Poe1,
    };

    let http = PoeHttpClient::new().map_err(|e| format!("init http: {e}"))?;
    let cache = FileCache::new(FileCache::default_dir());
    let client = TradeClient::new(http, cache, game);

    // Resolve league. Prefer the caller's explicit choice; otherwise pick
    // the first non-Standard league (which is the current temp league for
    // PoE1, or "Standard"/"Hardcore" for permanent realms / PoE2 EA).
    let league = match payload.league.as_deref() {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => {
            let leagues = client.leagues().await.map_err(|e| format!("leagues: {e}"))?;
            leagues
                .into_iter()
                .next()
                .ok_or_else(|| "no leagues returned by trade API".to_string())?
        }
    };

    // Only mods that the trade site treats as searchable rolled affixes.
    // Implicits + enchants are dropped — they're base-determined and would
    // over-narrow the search to the exact same item.
    let searchable_kinds: [&str; 3] = ["explicit", "crafted", "fractured"];

    let mut resolved_ids: Vec<String> = Vec::new();
    let mut unresolved: Vec<String> = Vec::new();
    for m in &payload.mods {
        let kind_lower = m.kind.to_ascii_lowercase();
        if !searchable_kinds.contains(&kind_lower.as_str()) {
            continue;
        }
        let text = m.text.trim();
        if text.is_empty() {
            continue;
        }
        match client.resolve(text, true, 1).await {
            Ok(hits) if !hits.is_empty() => resolved_ids.push(hits[0].id.clone()),
            _ => unresolved.push(text.to_string()),
        }
    }

    // Build the query body. Stats are all `and` so trade returns items
    // matching every recognized affix; if nothing resolved, return a
    // category-only search so the user lands on the right slot rather
    // than an empty results page.
    let stats: Vec<(String, Option<f64>, Option<f64>)> = resolved_ids
        .iter()
        .map(|id| (id.clone(), None, None))
        .collect();
    let rarity_arg = payload.rarity.as_deref();
    let category_arg = payload.category.as_deref();
    let query = TradeClient::build_query_body(category_arg, rarity_arg, &stats, None);

    let resp = client
        .search(&league, query)
        .await
        .map_err(|e| format!("trade search: {e}"))?;

    Ok(ItemTradeQueryResult {
        url: resp.share_url,
        league,
        total: resp.total,
        resolved_stat_ids: resolved_ids,
        unresolved_mods: unresolved,
    })
}

// ─── Sprint v3 — Build Registry ─────────────────────────────────────────────

/// DTO for the Active Builds picker + Settings list. Same shape as the
/// stored entry plus the summary unrolled inline for frontend convenience.
#[derive(serde::Serialize)]
pub struct RegistryEntryDto {
    pub id: i64,
    pub display_name: String,
    pub game: String,
    pub pob_path: String,
    pub pob_hash: String,
    pub identity_sig: String,
    pub gear_sig: String,
    pub tree_sig: String,
    pub skill_sig: String,
    pub config_sig: String,
    pub linked_sheet_id: Option<String>,
    pub summary: bestel_core::registry::RegistrySummary,
    pub last_seen_at: i64,
    pub authored_at: i64,
}

impl From<bestel_core::registry::RegistryEntry> for RegistryEntryDto {
    fn from(e: bestel_core::registry::RegistryEntry) -> Self {
        Self {
            id: e.id,
            display_name: e.display_name,
            game: e.game,
            pob_path: e.pob_path,
            pob_hash: e.pob_hash,
            identity_sig: e.identity_sig,
            gear_sig: e.gear_sig,
            tree_sig: e.tree_sig,
            skill_sig: e.skill_sig,
            config_sig: e.config_sig,
            linked_sheet_id: e.linked_sheet_id,
            summary: e.summary,
            last_seen_at: e.last_seen_at,
            authored_at: e.authored_at,
        }
    }
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// Parse a PoB file at the given path, compute the 5 signatures + canonical
/// hash + summary, and return all of it bundled so the caller can either
/// preview (Add modal) or insert (registry_add).
fn build_registry_artifacts(
    pob_path: &str,
) -> Result<
    (
        bestel_core::PobBuild,
        String,
        bestel_core::pob::signatures::BuildSignatures,
        bestel_core::registry::RegistrySummary,
    ),
    String,
> {
    let path = PathBuf::from(pob_path);
    let build = parse_file(&path).map_err(|e| format!("parse pob: {e}"))?;
    let pob_hash = bestel_core::sheets::compute_pob_hash_from_build(&build);
    let sigs = bestel_core::pob::signatures::BuildSignatures::from_build(&build);
    let defining = build
        .items
        .iter()
        .filter(|i| {
            i.rarity
                .as_deref()
                .map(|r| r.eq_ignore_ascii_case("UNIQUE"))
                .unwrap_or(false)
        })
        .filter_map(|i| i.name.clone())
        .collect();
    let summary = bestel_core::registry::RegistrySummary {
        class: build.class.clone(),
        ascendancy: build.ascendancy.clone(),
        level: build.level,
        main_skill: build.main_skill.clone(),
        defining_uniques: defining,
    };
    Ok((build, pob_hash, sigs, summary))
}

/// Emit `registry:changed` to every Tauri window so the chat picker,
/// Settings page, and any open dev panel re-pull `registry_list`. The
/// payload is intentionally empty — clients re-fetch the full list to
/// avoid having to track per-entry deltas.
fn emit_registry_changed(app: &AppHandle) {
    let _ = app.emit("registry:changed", serde_json::json!({}));
}

/// Preview-only counterpart to `registry_add`: parse the PoB, compute the 5
/// signatures + canonical hash + summary, but DO NOT insert into the
/// registry. Used by the Add-a-build modal's "Browse…" action so the user
/// can review the parsed identity before committing. The returned entry's
/// `id` is `-1` to signal "unsaved" to the UI; persistence happens on
/// `registry_add` only.
#[tauri::command]
pub async fn registry_preview(pob_path: String) -> Result<RegistryEntryDto, String> {
    let (build, pob_hash, sigs, summary) = build_registry_artifacts(&pob_path)?;
    let display_name = build
        .main_skill
        .clone()
        .or_else(|| build.ascendancy.clone())
        .unwrap_or_else(|| build.class.clone());
    let now = now_ms();
    Ok(RegistryEntryDto {
        id: -1,
        display_name,
        game: build.game.label().to_string(),
        pob_path,
        pob_hash,
        identity_sig: sigs.identity,
        gear_sig: sigs.gear,
        tree_sig: sigs.tree,
        skill_sig: sigs.skill,
        config_sig: sigs.config,
        linked_sheet_id: None,
        summary,
        last_seen_at: now,
        authored_at: now,
    })
}

#[tauri::command]
pub async fn registry_list() -> Result<Vec<RegistryEntryDto>, String> {
    let db = bestel_core::persistence::global_db()
        .ok_or_else(|| "database is not initialized".to_string())?;
    let entries = bestel_core::registry::list_entries(&db)
        .map_err(|e| format!("list registry: {e}"))?;
    Ok(entries.into_iter().map(RegistryEntryDto::from).collect())
}

#[tauri::command]
pub async fn registry_add(
    pob_path: String,
    app: AppHandle,
) -> Result<RegistryEntryDto, String> {
    let (build, pob_hash, sigs, summary) = build_registry_artifacts(&pob_path)?;
    let db = bestel_core::persistence::global_db()
        .ok_or_else(|| "database is not initialized".to_string())?;
    // Auto-link to an existing sheet by fingerprint if one matches.
    let linked = bestel_core::sheets::compute_fingerprint_from_pob(&build)
        .and_then(|fp| {
            bestel_core::sheets::store::find_by_fingerprint(&db, &fp)
                .ok()
                .flatten()
        })
        .map(|row| row.id);
    let display_name = build
        .main_skill
        .clone()
        .or_else(|| build.ascendancy.clone())
        .unwrap_or_else(|| build.class.clone());
    let now = now_ms();
    let id = bestel_core::registry::insert_entry(
        &db,
        &display_name,
        build.game.label(),
        &pob_path,
        &pob_hash,
        &sigs.identity,
        &sigs.gear,
        &sigs.tree,
        &sigs.skill,
        &sigs.config,
        linked.as_deref(),
        &summary,
        now,
    )
    .map_err(|e| format!("insert registry: {e}"))?;
    let entry = bestel_core::registry::get_entry(&db, id)
        .map_err(|e| format!("get registry: {e}"))?
        .ok_or_else(|| "inserted entry vanished".to_string())?;
    emit_registry_changed(&app);
    Ok(entry.into())
}

#[tauri::command]
pub async fn registry_remove(id: i64, app: AppHandle) -> Result<bool, String> {
    let db = bestel_core::persistence::global_db()
        .ok_or_else(|| "database is not initialized".to_string())?;
    let removed = bestel_core::registry::delete_entry(&db, id)
        .map_err(|e| format!("delete registry: {e}"))?;
    if removed {
        emit_registry_changed(&app);
    }
    Ok(removed)
}

#[tauri::command]
pub async fn registry_refresh(
    id: i64,
    app: AppHandle,
) -> Result<RegistryEntryDto, String> {
    let db = bestel_core::persistence::global_db()
        .ok_or_else(|| "database is not initialized".to_string())?;
    let Some(existing) = bestel_core::registry::get_entry(&db, id)
        .map_err(|e| format!("get registry: {e}"))?
    else {
        return Err(format!("no registry entry {id}"));
    };
    let (_build, pob_hash, sigs, summary) = build_registry_artifacts(&existing.pob_path)?;
    bestel_core::registry::update_entry_signatures(
        &db,
        id,
        &pob_hash,
        &sigs.identity,
        &sigs.gear,
        &sigs.tree,
        &sigs.skill,
        &sigs.config,
        &summary,
        now_ms(),
    )
    .map_err(|e| format!("update registry: {e}"))?;
    let entry = bestel_core::registry::get_entry(&db, id)
        .map_err(|e| format!("get registry: {e}"))?
        .ok_or_else(|| "refreshed entry vanished".to_string())?;
    emit_registry_changed(&app);
    Ok(entry.into())
}

#[tauri::command]
pub async fn registry_touch(id: i64) -> Result<(), String> {
    let db = bestel_core::persistence::global_db()
        .ok_or_else(|| "database is not initialized".to_string())?;
    bestel_core::registry::touch_last_seen(&db, id, now_ms())
        .map_err(|e| format!("touch registry: {e}"))
}

#[tauri::command]
pub async fn registry_get_pob_path(id: i64) -> Result<Option<String>, String> {
    let db = bestel_core::persistence::global_db()
        .ok_or_else(|| "database is not initialized".to_string())?;
    bestel_core::registry::get_pob_path(&db, id)
        .map_err(|e| format!("get pob path: {e}"))
}

/// Sprint v3.1 — check which registry entries still have their PoB file
/// present on disk. Returns one `(id, exists)` tuple per registered
/// entry. The UI uses this to badge stale rows and disable `Use in this
/// chat` for missing files, instead of failing on click. Cheap: one
/// `tokio::fs::metadata` per entry, fanned out concurrently.
#[tauri::command]
pub async fn registry_check_paths() -> Result<Vec<(i64, bool)>, String> {
    let db = bestel_core::persistence::global_db()
        .ok_or_else(|| "database is not initialized".to_string())?;
    let entries = bestel_core::registry::list_entries(&db)
        .map_err(|e| format!("list registry: {e}"))?;
    // Sequential `metadata` calls — registry lists are bounded (~dozens
    // of entries) and each stat is sub-millisecond. Avoids pulling in
    // `futures` for a single use-site.
    let mut out = Vec::with_capacity(entries.len());
    for e in entries {
        let exists = tokio::fs::metadata(&e.pob_path).await.is_ok();
        out.push((e.id, exists));
    }
    Ok(out)
}

#[tauri::command]
pub async fn suggestion_dismiss(pob_hash: String, days: u32) -> Result<(), String> {
    let db = bestel_core::persistence::global_db()
        .ok_or_else(|| "database is not initialized".to_string())?;
    let until = now_ms() + (days as i64) * 86_400_000;
    bestel_core::registry::suggestion_dismiss(&db, &pob_hash, until)
        .map_err(|e| format!("dismiss: {e}"))
}

#[tauri::command]
pub async fn suggestion_check(pob_hash: String) -> Result<bool, String> {
    let db = bestel_core::persistence::global_db()
        .ok_or_else(|| "database is not initialized".to_string())?;
    bestel_core::registry::suggestion_check(&db, &pob_hash, now_ms())
        .map_err(|e| format!("check: {e}"))
}

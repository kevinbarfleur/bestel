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
    summary_dto_from_build, AttachmentDto, BuildEvent, DeltaEvent, DetectionDto, KeyStatusDto,
    ModelProfileDto, PobBuildDto, PobBuildSummaryDto, ProviderStatusEvent,
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
            let run = rec_arc.finish(final_text);
            if let Err(e) = debug_recorder::save_run(&run) {
                eprintln!("failed to persist debug run: {e}");
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


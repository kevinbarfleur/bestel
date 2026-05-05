use std::path::PathBuf;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, State, Window};
use tauri_plugin_opener::OpenerExt;
use tokio::sync::{mpsc, oneshot};

use bestel_core::llm::detect::{build_provider_for_profile, detect_provider};
use bestel_core::llm::models::{all_profiles, find_profile, save_active_profile};
use bestel_core::llm::{ChatMessage, LlmDelta, Role};
use bestel_core::pob::parser::{parse_file, parse_summary};

use crate::dto::{
    summary_dto_from_build, AttachmentDto, BuildEvent, DeltaEvent, DetectionDto, ModelProfileDto,
    PobBuildDto, PobBuildSummaryDto, ProviderStatusEvent,
};
use crate::events::{pump_deltas, LLM_DELTA, POB_BUILD, POB_CLEARED, PROVIDER_STATUS};
use crate::state::AppState;

#[tauri::command]
pub fn ping(name: String) -> String {
    format!("pong {name}")
}

#[tauri::command]
pub fn list_models() -> Vec<ModelProfileDto> {
    all_profiles().iter().map(ModelProfileDto::from).collect()
}

#[tauri::command]
pub fn get_active_model(state: State<'_, AppState>) -> ModelProfileDto {
    ModelProfileDto::from(state.active_model_profile())
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
    Ok(ModelProfileDto::from(profile))
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
    let provider = build_provider_for_profile(profile)
        .await
        .map_err(|e| format!("build provider: {e}"))?;

    let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
    {
        let mut g = state.inner.lock().map_err(|_| "state poisoned".to_string())?;
        g.cancel_tx = Some(cancel_tx);
    }

    let (delta_tx, delta_rx) = mpsc::unbounded_channel::<LlmDelta>();
    let pump_window = window.clone();
    let pump_handle = tauri::async_runtime::spawn(async move {
        pump_deltas(pump_window, session_id, delta_rx, cancel_rx).await
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
    tauri::async_runtime::spawn(async move {
        let result = run_handle.await;
        let cancelled = pump_handle.await.unwrap_or(false);

        let app_state = state_app.try_state::<AppState>();
        match result {
            Ok(Ok(assistant_text)) => {
                if !cancelled {
                    if let Some(s) = app_state.as_ref() {
                        if let Ok(mut g) = s.inner.lock() {
                            g.history.push(ChatMessage {
                                role: Role::Assistant,
                                content: assistant_text,
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
pub async fn open_external(url: String, app: AppHandle) -> Result<(), String> {
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}


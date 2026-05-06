use std::sync::{Arc, Mutex};

use tauri::{Emitter, Window};
use tokio::sync::{mpsc, oneshot};

use bestel_core::llm::recorder::Recorder;
use bestel_core::llm::LlmDelta;

use crate::dto::DeltaEvent;

pub const LLM_DELTA: &str = "llm:delta";
pub const POB_BUILD: &str = "pob:build";
pub const POB_CLEARED: &str = "pob:cleared";
pub const PROVIDER_STATUS: &str = "provider:status";

pub async fn pump_deltas(
    window: Window,
    session_id: u64,
    mut rx: mpsc::UnboundedReceiver<LlmDelta>,
    mut cancel: oneshot::Receiver<()>,
    recorder: Option<Arc<Mutex<Recorder>>>,
) -> bool {
    let mut cancelled = false;
    loop {
        tokio::select! {
            biased;
            res = &mut cancel => {
                if res.is_ok() {
                    cancelled = true;
                    let _ = window.emit(LLM_DELTA, DeltaEvent::Cancelled { session_id });
                    break;
                }
            }
            d = rx.recv() => match d {
                Some(delta) => {
                    if let Some(rec) = recorder.as_ref() {
                        if let Ok(mut g) = rec.lock() {
                            g.apply(&delta);
                        }
                    }
                    let _ = window.emit(LLM_DELTA, DeltaEvent::from_delta(session_id, delta));
                }
                None => break,
            }
        }
    }
    cancelled
}

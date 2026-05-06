use std::sync::{Arc, Mutex, OnceLock};

use tokio::sync::oneshot;

use bestel_core::llm::models::{active_profile, ModelProfile};
use bestel_core::llm::tools::BuildContext;
use bestel_core::llm::ChatMessage;
use bestel_core::pob::watcher::PobWatcher;

pub struct AppState {
    pub build_ctx: BuildContext,
    pub watcher: OnceLock<Arc<PobWatcher>>,
    pub inner: Mutex<Inner>,
}

pub struct Inner {
    pub history: Vec<ChatMessage>,
    pub next_session_id: u64,
    pub active_session: Option<u64>,
    pub cancel_tx: Option<oneshot::Sender<()>>,
    pub active_model_id: String,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            next_session_id: 1,
            active_session: None,
            cancel_tx: None,
            active_model_id: active_profile().id.to_string(),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            build_ctx: BuildContext::new(),
            watcher: OnceLock::new(),
            inner: Mutex::new(Inner::default()),
        }
    }

    pub fn set_watcher(&self, watcher: Arc<PobWatcher>) {
        let _ = self.watcher.set(watcher);
    }

    pub fn watcher(&self) -> Option<Arc<PobWatcher>> {
        self.watcher.get().cloned()
    }

    pub fn next_session_id(&self) -> u64 {
        let mut g = self.inner.lock().expect("inner mutex poisoned");
        let id = g.next_session_id;
        g.next_session_id = g.next_session_id.wrapping_add(1).max(1);
        id
    }

    pub fn active_model_profile(&self) -> ModelProfile {
        let id = {
            let g = self.inner.lock().expect("inner mutex poisoned");
            g.active_model_id.clone()
        };
        bestel_core::llm::models::find_profile(&id)
            .unwrap_or_else(bestel_core::llm::models::default_profile)
    }
}

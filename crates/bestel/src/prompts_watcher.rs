//! Filesystem watcher for `~/.bestel/prompts/`. Emits a `prompts:changed`
//! Tauri event whenever a `.md` file under the directory is created,
//! modified, or removed by something *other* than Bestel itself.
//!
//! The watcher dedups echo events triggered by our own writes — see
//! [`record_self_write`]. Without this, every autosave from the editor
//! would loop back as a "changed" event and we'd reload the editor over
//! the user's typing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use notify::{
    event::{EventKind, ModifyKind},
    Event, RecommendedWatcher, RecursiveMode, Watcher,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

pub const PROMPTS_CHANGED: &str = "prompts:changed";

/// Window after `prompts_write` during which any same-path event from the
/// watcher is treated as our own echo and dropped. Generous on Windows
/// where notify can fire several updates per write.
const ECHO_DEDUP_WINDOW: Duration = Duration::from_millis(800);

/// Shared "recently written by Bestel" map. The IPC `prompts_write`
/// command stamps a path here right before flushing to disk; the watcher
/// reads to skip its own echo.
#[derive(Clone, Default)]
pub struct SelfWriteTracker {
    inner: Arc<Mutex<HashMap<PathBuf, Instant>>>,
}

impl SelfWriteTracker {
    pub fn record(&self, path: &Path) {
        if let Ok(mut g) = self.inner.lock() {
            g.insert(path.to_path_buf(), Instant::now());
            let cutoff = Instant::now() - ECHO_DEDUP_WINDOW * 4;
            g.retain(|_, t| *t >= cutoff);
        }
    }

    fn is_recent_self_write(&self, path: &Path) -> bool {
        let Ok(g) = self.inner.lock() else {
            return false;
        };
        match g.get(path) {
            Some(t) => t.elapsed() < ECHO_DEDUP_WINDOW,
            None => false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct PromptsChangedPayload<'a> {
    /// Path relative to `prompts_dir()`, forward-slashed.
    rel_path: String,
    reason: &'a str,
}

/// Spawn the watcher task. Runs forever on the tauri async runtime; if
/// notify fails to initialise we log and bail gracefully — auto-reload
/// is degraded but the editor still works (manual reload via tab close
/// + reopen).
pub fn spawn(app: AppHandle, tracker: SelfWriteTracker) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run(app, tracker).await {
            tracing::warn!(target: "bestel.prompts", "watcher exited: {e:?}");
        }
    });
}

async fn run(app: AppHandle, tracker: SelfWriteTracker) -> Result<()> {
    let root = bestel_core::prompts::prompts_dir()
        .context("prompts dir not resolvable (no home directory)")?;

    let (tx, mut rx) = mpsc::unbounded_channel::<Event>();

    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(move |res: notify::Result<Event>| match res {
            Ok(ev) => {
                let _ = tx.send(ev);
            }
            Err(e) => tracing::warn!(target: "bestel.prompts", "watcher error: {e}"),
        })
        .context("create notify watcher")?;

    watcher
        .watch(&root, RecursiveMode::Recursive)
        .context("watch prompts dir")?;

    while let Some(ev) = rx.recv().await {
        let reason = match &ev.kind {
            EventKind::Create(_) => "create",
            EventKind::Modify(ModifyKind::Data(_)) => "modify",
            EventKind::Modify(ModifyKind::Name(_)) => "rename",
            EventKind::Modify(_) => "modify",
            EventKind::Remove(_) => "remove",
            _ => continue,
        };

        for path in ev.paths {
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            if tracker.is_recent_self_write(&path) {
                continue;
            }
            let rel = match path.strip_prefix(&root) {
                Ok(r) => r.to_string_lossy().replace('\\', "/"),
                Err(_) => continue,
            };
            let _ = app.emit(
                PROMPTS_CHANGED,
                PromptsChangedPayload {
                    rel_path: rel,
                    reason,
                },
            );
        }
    }

    Ok(())
}

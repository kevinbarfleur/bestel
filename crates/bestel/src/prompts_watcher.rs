//! Filesystem watcher for `~/.bestel/prompts/`. Emits a `prompts:changed`
//! Tauri event whenever a `.md` file under the directory is created,
//! modified, or removed by something *other* than Bestel itself.
//!
//! The watcher dedups echo events triggered by our own writes — see
//! [`record_self_write`]. Without this, every autosave from the editor
//! would loop back as a "changed" event and we'd reload the editor over
//! the user's typing.
//!
//! It also drives **live RAG re-ingest** for the references corpus. When
//! a `.md` file under `references/` settles for ≥1500 ms the watcher
//! kicks off [`bestel_rag::KbEngine::ingest`] on a background task. The
//! ingest is incremental (Blake3 skip-unchanged) so the cost of a
//! single-file edit is just walk + 1 file chunk + a handful of
//! embeddings + LanceDB upsert + FTS rebuild. Result: `kb_search`
//! reflects edited references within ~2 s of the save landing on disk,
//! whether the edit came from Bestel's editor or an external one
//! (VS Code, etc.).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use bestel_rag::IngestConfig;
use notify::{
    event::{EventKind, ModifyKind},
    Event, RecommendedWatcher, RecursiveMode, Watcher,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

pub const PROMPTS_CHANGED: &str = "prompts:changed";
pub const KB_REINDEXED: &str = "kb:reindexed";

/// Window after `prompts_write` during which any same-path event from the
/// watcher is treated as our own echo and dropped. Generous on Windows
/// where notify can fire several updates per write.
const ECHO_DEDUP_WINDOW: Duration = Duration::from_millis(800);

/// Quiet period required after the last filesystem event under
/// `references/` before we kick off a re-ingest. Coalesces rapid saves
/// (the editor auto-saves every 500 ms while the user types) plus
/// platform-noisy multi-event writes into one ingest pass.
const KB_INGEST_DEBOUNCE: Duration = Duration::from_millis(1500);

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

#[derive(Debug, Clone, Serialize)]
struct KbReindexedPayload {
    files_seen: u32,
    files_indexed: u32,
    files_skipped_unchanged: u32,
    chunks_emitted: u32,
    elapsed_ms: u64,
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
    let references_root = root.join("references");

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

    // Tracks the deadline at which a queued re-ingest should fire. `None`
    // means no ingest is pending; updated on every fs event under
    // `references_root` so the timer slides forward while the user keeps
    // typing.
    let mut pending_reindex: Option<Instant> = None;
    // Sentinel duration for "no deadline pending" — we always build a
    // sleep future so `tokio::select!` stays simple, and gate the timer
    // arm with `if pending_reindex.is_some()`. The wall-clock duration
    // here only matters when the gate is false (sleep is never polled
    // to completion in that case).
    let far_future = Duration::from_secs(60 * 60 * 24);

    loop {
        let remaining = match pending_reindex {
            Some(deadline) => deadline.saturating_duration_since(Instant::now()),
            None => far_future,
        };
        let sleep = tokio::time::sleep(remaining);
        tokio::pin!(sleep);

        tokio::select! {
            // Branch 1: next filesystem event. `rx.recv()` returning
            // `None` means the sender was dropped; bail out.
            ev = rx.recv() => {
                let Some(ev) = ev else { break; };
                if let Some(deadline) = process_event(
                    ev,
                    &root,
                    &references_root,
                    &tracker,
                    &app,
                ) {
                    pending_reindex = Some(deadline);
                }
            }
            // Branch 2: pending reindex timer fires. Gated on
            // `pending_reindex.is_some()` so the sentinel-duration sleep
            // is never polled to completion when nothing is queued.
            _ = &mut sleep, if pending_reindex.is_some() => {
                pending_reindex = None;
                tauri::async_runtime::spawn(reindex_now(app.clone()));
            }
        }
    }

    Ok(())
}

/// Process a single notify event. Emits the per-path `prompts:changed`
/// event and, if any path is a `.md` under `references_root`, returns
/// the new debounce deadline that should be queued. `None` means no
/// references change happened on this event.
fn process_event(
    ev: Event,
    root: &Path,
    references_root: &Path,
    tracker: &SelfWriteTracker,
    app: &AppHandle,
) -> Option<Instant> {
    let reason = match &ev.kind {
        EventKind::Create(_) => "create",
        EventKind::Modify(ModifyKind::Data(_)) => "modify",
        EventKind::Modify(ModifyKind::Name(_)) => "rename",
        EventKind::Modify(_) => "modify",
        EventKind::Remove(_) => "remove",
        _ => return None,
    };

    let mut deadline: Option<Instant> = None;

    for path in ev.paths {
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        if tracker.is_recent_self_write(&path) {
            // Still arm the reindex — the disk content is fresh and the
            // RAG must see it even though the UI doesn't need a reload
            // event.
            if path.starts_with(references_root) {
                deadline = Some(Instant::now() + KB_INGEST_DEBOUNCE);
            }
            continue;
        }
        let rel = match path.strip_prefix(root) {
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
        if path.starts_with(references_root) {
            deadline = Some(Instant::now() + KB_INGEST_DEBOUNCE);
        }
    }

    deadline
}

/// Run one re-ingest pass over the references corpus. Best-effort: any
/// failure is logged and swallowed. Skipped (no-op) when the global KB
/// engine hasn't been installed yet (e.g. no embedder configured at
/// boot, or `bootstrap_global` still in flight on the first save).
async fn reindex_now(app: AppHandle) {
    let Some(engine) = bestel_core::llm::kb::global() else {
        tracing::debug!(
            target: "bestel.prompts",
            "skip reindex: KB engine not installed"
        );
        return;
    };
    let Some(home) = dirs::home_dir() else {
        tracing::warn!(target: "bestel.prompts", "skip reindex: home dir unresolvable");
        return;
    };
    let corpus_root = home.join(".bestel").join("prompts").join("references");
    if !corpus_root.exists() {
        tracing::warn!(
            target: "bestel.prompts",
            ?corpus_root,
            "skip reindex: corpus root missing"
        );
        return;
    }

    let cfg = IngestConfig::default();
    let stats = match engine.ingest(&corpus_root, &cfg).await {
        Ok(s) => s,
        Err(err) => {
            tracing::warn!(target: "bestel.prompts", error = ?err, "live reindex failed");
            return;
        }
    };
    if let Err(err) = engine.ensure_fts_index().await {
        tracing::warn!(target: "bestel.prompts", error = ?err, "post-reindex FTS rebuild failed");
    }

    let label = engine.embedder().label().to_string();
    let dim = bestel_rag::EmbeddingProvider::dim(engine.embedder());
    if let Err(err) = bestel_core::llm::kb::mirror_kb_versions(&engine, &label, dim).await {
        tracing::warn!(target: "bestel.prompts", error = ?err, "kb_versions mirror failed");
    }

    tracing::info!(
        target: "bestel.prompts",
        seen = stats.files_seen,
        indexed = stats.files_indexed,
        skipped = stats.files_skipped_unchanged,
        chunks = stats.chunks_emitted,
        elapsed_ms = stats.elapsed_ms,
        "live RAG reindex pass complete"
    );

    let _ = app.emit(
        KB_REINDEXED,
        KbReindexedPayload {
            files_seen: stats.files_seen,
            files_indexed: stats.files_indexed,
            files_skipped_unchanged: stats.files_skipped_unchanged,
            chunks_emitted: stats.chunks_emitted,
            elapsed_ms: stats.elapsed_ms,
        },
    );
}

//! Lazy global accessor for the hybrid embedded KB engine.
//!
//! Mirrors the `pob_engine` pattern: a process-lifetime [`OnceLock`] holds
//! either a built [`SharedKbEngine`] or `None`. The Tauri / `bestel mcp-serve`
//! / `bestel run-battery` bootstrap flows that opt into KB-augmented
//! retrieval call [`init`] once with a constructed engine; later
//! `ToolCtx::new()` calls pick the same instance up via [`global`].
//!
//! Kept separate from the PoB engine module so each can fail
//! independently — neither prerequisite blocks the other.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use bestel_rag::{detect_default, IngestConfig, KbEngine, KbIndex, LanceIndex, SharedKbEngine};
use chrono::Utc;
use tracing::{info, warn};

use crate::persistence::{global_db, upsert_kb_version, KbVersionRow};

static KB: OnceLock<Option<SharedKbEngine>> = OnceLock::new();

/// Get the global KB engine, if any has been installed. Returns `None`
/// when the runtime did not configure one (e.g. embedding backend missing,
/// indexing disabled by config).
pub fn global() -> Option<SharedKbEngine> {
    KB.get().cloned().flatten()
}

/// Install the process-wide KB engine. First caller wins; subsequent
/// calls are ignored. Returns whether this call performed the install.
pub fn init(engine: SharedKbEngine) -> bool {
    KB.set(Some(engine)).is_ok()
}

/// Mark the global as explicitly disabled (e.g. when embedding backend
/// detection failed at boot). Locks `global()` to `None` for the rest of
/// the process so we don't repeatedly retry the failing init.
pub fn disable() {
    let _ = KB.set(None);
}

/// Boot the KB engine: detect an embedder, open the LanceDB index under
/// `~/.bestel/index/kb.lance/`, install the global handle, and spawn an
/// incremental ingestion pass over `~/.bestel/prompts/references/` on a
/// background task.
///
/// Best-effort: returns `Ok(false)` when no embedding backend is reachable
/// (no `OPENAI_API_KEY`, no Ollama). Returns `Err` only when the disk
/// state is corrupted enough to be worth surfacing. Idempotent — calling
/// twice is a no-op (the OnceLock blocks the second install).
pub async fn bootstrap_global() -> Result<bool> {
    if KB.get().is_some() {
        return Ok(global().is_some());
    }
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            warn!("home dir not resolvable; KB disabled");
            disable();
            return Ok(false);
        }
    };
    let bestel_dir = home.join(".bestel");
    let corpus_root = bestel_dir.join("prompts").join("references");
    if !corpus_root.exists() {
        warn!(
            ?corpus_root,
            "references corpus root missing — KB disabled until prompts seed runs"
        );
        disable();
        return Ok(false);
    }
    let index_dir = bestel_dir.join("index").join("kb.lance");
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .context("build reqwest client for embeddings")?;
    let embedder = match detect_default(http).await {
        Ok(e) => e,
        Err(err) => {
            warn!(error = ?err, "no embedding backend detected — KB disabled");
            disable();
            return Ok(false);
        }
    };
    let label = embedder.label();
    let dim = bestel_rag::EmbeddingProvider::dim(&embedder);
    let index = LanceIndex::open_or_create(&index_dir, dim)
        .await
        .with_context(|| format!("open LanceDB at {index_dir:?}"))?;
    let engine = Arc::new(KbEngine::new(index, embedder));
    if !init(engine.clone()) {
        return Ok(global().is_some());
    }
    info!(
        embedder = %label,
        ?index_dir,
        ?corpus_root,
        "KB engine ready; spawning background ingest"
    );
    let bg_engine = engine.clone();
    let bg_root: PathBuf = corpus_root;
    let bg_label = label.to_string();
    let bg_dim = dim;
    tokio::spawn(async move {
        if let Err(err) = bg_engine.ensure_fts_index().await {
            warn!(error = ?err, "ensure_fts_index failed; FTS retrieval will degrade");
        }
        let cfg = IngestConfig::default();
        match bg_engine.ingest(&bg_root, &cfg).await {
            Ok(stats) => info!(
                seen = stats.files_seen,
                indexed = stats.files_indexed,
                skipped = stats.files_skipped_unchanged,
                chunks = stats.chunks_emitted,
                elapsed_ms = stats.elapsed_ms,
                "background KB ingest complete"
            ),
            Err(err) => warn!(error = ?err, "background KB ingest failed"),
        }
        // After the first pass, rebuild FTS so newly-indexed chunks are searchable.
        if let Err(err) = bg_engine.ensure_fts_index().await {
            warn!(error = ?err, "post-ingest ensure_fts_index failed");
        }
        // Sprint G2 — mirror the LanceDB doc fingerprints into SQLite so
        // `bestel db stats` and downstream consumers (Sprint H eval) can
        // answer "is the KB stale?" without opening the LanceDB table.
        if let Err(err) = mirror_kb_versions(&bg_engine, &bg_label, bg_dim).await {
            warn!(error = ?err, "kb_versions mirror failed");
        }
    });
    Ok(true)
}

/// Mirror the LanceDB doc fingerprints into the SQLite `kb_versions`
/// table. Idempotent. The watcher calls this after each successful
/// incremental re-ingest so `bestel db stats` reflects the live state
/// without opening the LanceDB table.
///
/// `label` and `dim` are derived from the engine's embedder; passed
/// explicitly so the caller can label them once instead of re-pulling
/// them on every upsert.
pub async fn mirror_kb_versions(engine: &SharedKbEngine, label: &str, dim: usize) -> Result<()> {
    let Some(db) = global_db() else {
        return Ok(());
    };
    let hashes = engine.index().doc_hashes().await?;
    let now = Utc::now().to_rfc3339();
    for (doc_path, content_hash) in hashes {
        if let Err(e) = upsert_kb_version(
            &db,
            &KbVersionRow {
                doc_path,
                content_hash,
                indexed_at: now.clone(),
                dim: dim as i64,
                embedder_label: label.to_string(),
            },
        ) {
            warn!(error = ?e, "kb_versions upsert failed");
        }
    }
    Ok(())
}

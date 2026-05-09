//! `KbEngine` bundles a [`LanceIndex`] and an [`AnyEmbedder`] into a single
//! handle the rest of the workspace can hold inside an `Arc`.
//!
//! This exists to side-step the AFIT object-safety limitation: `KbIndex`
//! and `EmbeddingProvider` cannot be erased to `dyn` trait objects (the
//! async return type prevents it), so the runtime cannot hold
//! `Arc<dyn KbIndex>` and `Arc<dyn EmbeddingProvider>` directly.
//! `KbEngine` resolves both into a concrete type once at boot, then
//! exposes async methods that callers can dispatch through.

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use tracing::warn;

use crate::chunker::{Chunker, ChunkerConfig};
use crate::embeddings::AnyEmbedder;
use crate::ingest::{ingest_corpus, IngestConfig};
use crate::lance::LanceIndex;
use crate::traits::{EmbeddingProvider, KbIndex};
use crate::types::{IngestStats, SearchHit};

/// One assembled knowledge-base instance: vector store + embedder.
pub struct KbEngine {
    index: LanceIndex,
    embedder: AnyEmbedder,
    chunker: Chunker,
}

impl KbEngine {
    pub fn new(index: LanceIndex, embedder: AnyEmbedder) -> Self {
        Self {
            index,
            embedder,
            chunker: Chunker::new(ChunkerConfig::default()),
        }
    }

    /// Open or create the on-disk index then build a default chunker.
    /// `dir` is the LanceDB root; the index lives at `<dir>/`.
    pub async fn open_or_create(dir: &Path, embedder: AnyEmbedder) -> Result<Self> {
        let index = LanceIndex::open_or_create(dir, embedder.dim()).await?;
        Ok(Self::new(index, embedder))
    }

    pub fn index(&self) -> &LanceIndex {
        &self.index
    }

    pub fn embedder(&self) -> &AnyEmbedder {
        &self.embedder
    }

    pub fn chunker(&self) -> &Chunker {
        &self.chunker
    }

    /// Hybrid search. Single entry point for runtime callers.
    pub async fn search(
        &self,
        query: &str,
        top_k: usize,
        game: Option<&str>,
        tags: &[String],
    ) -> Result<Vec<SearchHit>> {
        let qv = self.embedder.embed_one(query).await?;
        self.index
            .search_hybrid(query, qv, top_k, game, tags)
            .await
    }

    /// Run a full incremental ingest pass over `root`. Skips unchanged
    /// files via Blake3 content-hash comparison.
    pub async fn ingest(
        &self,
        root: &Path,
        config: &IngestConfig,
    ) -> Result<IngestStats> {
        ingest_corpus(root, &self.index, &self.embedder, &self.chunker, config).await
    }

    /// Build the FTS index on the body column. Idempotent. Best-effort —
    /// we log and swallow errors here because boot must not fail if the
    /// FTS index can't be (re)built.
    pub async fn ensure_fts_index(&self) -> Result<()> {
        if let Err(err) = self.index.ensure_fts_index().await {
            warn!(error = ?err, "ensure_fts_index failed; FTS retrieval will degrade until next index pass");
            return Err(err);
        }
        Ok(())
    }
}

/// Reference-counted handle for stuffing inside the runtime tool context.
pub type SharedKbEngine = Arc<KbEngine>;

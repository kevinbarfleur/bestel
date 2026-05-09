//! Backend abstractions: `KbIndex` for storage/retrieval, `EmbeddingProvider`
//! for vectorisation. Concrete impls live in `crate::lance` and
//! `crate::embeddings::{openai, ollama}`.
//!
//! Async functions in traits (AFIT) — stable since Rust 1.75 — are used to
//! avoid the `async-trait` macro dependency (per project convention). The
//! trade-off is that these traits are not directly object-safe; callers
//! that want a runtime-swappable backend should hold the concrete type
//! behind an `Arc` (e.g. `Arc<LanceIndex>`) and pass it through generics.

use std::collections::HashMap;

use anyhow::Result;

use crate::types::{Chunk, SearchHit};

/// Hybrid (vector + BM25) knowledge-base index.
///
/// Implementations must:
/// - Store both the chunk metadata AND its vector embedding.
/// - Run vector + lexical search independently and fuse via RRF (k = 60).
/// - Support incremental upsert keyed on `Chunk::id`.
/// - Preserve `Chunk::content_hash` so callers can decide whether to skip
///   re-embedding a doc on next boot.
pub trait KbIndex: Send + Sync {
    /// Insert or replace `chunks` along with their parallel `embeddings`
    /// vector (one per chunk, same order). All vectors must share the same
    /// dimensionality; the index validates against the schema dim it was
    /// opened with and errors otherwise.
    fn upsert(
        &self,
        chunks: Vec<Chunk>,
        embeddings: Vec<Vec<f32>>,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Hybrid search.
    ///
    /// `query_embedding` must come from the same provider used at index
    /// time (mixing dims silently degrades recall). `game` filters
    /// `applies_to`; `tags` requires a chunk to carry ALL listed tags.
    fn search_hybrid(
        &self,
        query: &str,
        query_embedding: Vec<f32>,
        top_k: usize,
        game: Option<&str>,
        tags: &[String],
    ) -> impl std::future::Future<Output = Result<Vec<SearchHit>>> + Send;

    /// Delete every chunk attributed to `doc_path`. Used when a corpus file
    /// is removed.
    fn delete_by_doc_path(
        &self,
        doc_path: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Map of `doc_path -> content_hash` for every indexed file. Drives
    /// the incremental skip in `crate::ingest`.
    fn doc_hashes(
        &self,
    ) -> impl std::future::Future<Output = Result<HashMap<String, String>>> + Send;
}

/// Maps text passages to dense vectors.
///
/// Implementations should batch internally to amortise network round-trips
/// (OpenAI accepts up to 96 inputs per request, Ollama is single-input).
/// `dim()` is queried once at index-open time to validate the LanceDB
/// schema.
pub trait EmbeddingProvider: Send + Sync {
    /// Output dimensionality (1536 for `text-embedding-3-small`, 384 for
    /// `bge-small-en-v1.5`, 1024 for `bge-m3`).
    fn dim(&self) -> usize;

    /// Embed a batch of texts. The returned vector preserves order. An
    /// empty input slice returns an empty result without error.
    fn embed_batch(
        &self,
        texts: &[String],
    ) -> impl std::future::Future<Output = Result<Vec<Vec<f32>>>> + Send;

    /// Convenience wrapper around `embed_batch` for a single text.
    fn embed_one(
        &self,
        text: &str,
    ) -> impl std::future::Future<Output = Result<Vec<f32>>> + Send
    where
        Self: Sync,
    {
        async move {
            let mut out = self.embed_batch(std::slice::from_ref(&text.to_string())).await?;
            out.pop()
                .ok_or_else(|| anyhow::anyhow!("embed_batch returned no rows for one input"))
        }
    }
}

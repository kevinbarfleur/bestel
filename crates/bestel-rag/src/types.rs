//! Core data types for the hybrid embedded knowledge-base index.
//!
//! These types are storage-agnostic — `LanceIndex` (the LanceDB-backed
//! implementation) consumes them, but a pure-memory mock for tests can
//! consume them too.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One indexable chunk of a Markdown document.
///
/// Chunks are the unit of storage AND the unit of retrieval. Chunking is
/// header-aware (see [`crate::chunker`]): each chunk inherits the ATX
/// heading chain it lives under, so a section like
/// `## Offence > ### Hit > #### Crit` produces chunks with
/// `heading_path = ["Offence", "Hit", "Crit"]`.
///
/// `id` must be deterministic for a given (doc_path, heading_path,
/// chunk_index_within_section) so re-indexing a doc doesn't churn IDs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chunk {
    /// Stable identifier: `blake3(doc_path || "\0" || heading_path.join("/") || "\0" || chunk_idx)`.
    pub id: String,
    /// Path relative to the corpus root, e.g. `"references/05_offence.md"`.
    pub doc_path: String,
    /// ATX heading chain leading to this chunk (deepest last). Empty for
    /// preamble chunks above the first header.
    pub heading_path: Vec<String>,
    /// Optional metadata tags (frontmatter or inferred).
    pub tags: Vec<String>,
    /// Game scope: `["poe1"]`, `["poe2"]`, or both. Empty = unscoped.
    pub applies_to: Vec<String>,
    /// File mtime at index time (or frontmatter `last_updated` if present).
    pub last_updated: DateTime<Utc>,
    /// Blake3 hash of the SOURCE FILE (not just this chunk). Drives the
    /// incremental-indexing skip: if the file's hash matches all chunks
    /// already stored under that doc_path, the file is skipped at boot.
    pub content_hash: String,
    /// Origin: `"reference"`, `"skill"`, `"wiki_cache"`, etc.
    pub source_kind: String,
    /// The chunk text. Includes the heading_path inlined as a prefix so
    /// BM25 picks up section names.
    pub body: String,
}

/// Result of a hybrid search.
///
/// `score` is the fused RRF score (k=60). `vector_rank` and `lexical_rank`
/// expose the per-channel ranks so callers can debug retrieval quality —
/// e.g. "BM25 found this chunk, embeddings did not" is a chunker or query
/// vocabulary issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub chunk: Chunk,
    pub score: f32,
    pub vector_rank: Option<u32>,
    pub lexical_rank: Option<u32>,
}

/// Aggregate counts produced by a single ingest pass.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IngestStats {
    pub files_seen: u32,
    pub files_skipped_unchanged: u32,
    pub files_indexed: u32,
    pub chunks_emitted: u32,
    pub chunks_embedded: u32,
    pub embed_batches: u32,
    pub elapsed_ms: u64,
}

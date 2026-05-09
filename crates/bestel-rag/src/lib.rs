//! Bestel hybrid embedded knowledge-base search.
//!
//! Indexes the bundled prompts/references corpus into LanceDB (vector) +
//! Tantivy (BM25) and exposes a single `KbIndex::search_hybrid` entry point
//! that fuses both via Reciprocal Rank Fusion (k = 60). Embeddings come from
//! a swappable `EmbeddingProvider` (OpenAI online by default, Ollama BGE
//! offline fallback). Storage lives at `~/.bestel/index/kb.lance/`. Indexing
//! is incremental via Blake3 content hash — files unchanged since last boot
//! are skipped.
//!
//! Spec: `docs/ROADMAP.md` § Sprint E.

pub mod chunker;
pub mod embeddings;
pub mod engine;
pub mod ingest;
pub mod lance;
pub mod traits;
pub mod types;

pub use chunker::{ChunkInput, Chunker, ChunkerConfig};
pub use embeddings::{
    detect_default, AnyEmbedder, OllamaEmbedder, OllamaEmbedderConfig, OpenAiEmbedder,
    OpenAiEmbedderConfig,
};
pub use engine::{KbEngine, SharedKbEngine};
pub use ingest::{ingest_corpus, IngestConfig};
pub use lance::LanceIndex;
pub use traits::{EmbeddingProvider, KbIndex};
pub use types::{Chunk, IngestStats, SearchHit};

//! Incremental Markdown corpus ingestion.
//!
//! Walks a directory tree of `*.md` files, computes a Blake3 hash for each,
//! and compares against [`KbIndex::doc_hashes`] to skip files whose content
//! has not changed since the last index pass. Changed and new files are
//! chunked, embedded, and upserted. Files removed from the corpus are
//! purged via [`KbIndex::delete_by_doc_path`].
//!
//! This is the entry point boot-time indexing should call. It deliberately
//! takes ownership of nothing across .await points except references, so
//! the caller can spawn it on a background tokio task without blocking
//! the runtime hot path.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use tracing::{debug, info, warn};

use crate::chunker::{ChunkInput, Chunker};
use crate::traits::{EmbeddingProvider, KbIndex};
use crate::types::{Chunk, IngestStats};

/// Per-pass configuration. Defaults pin `applies_to = ["poe1", "poe2"]`
/// and `source_kind = "reference"`, matching the prompts/references
/// corpus layout.
#[derive(Debug, Clone)]
pub struct IngestConfig {
    /// Glob-like extension match (we only ship `.md` for now).
    pub extension: String,
    pub source_kind: String,
    pub default_applies_to: Vec<String>,
    pub default_tags: Vec<String>,
    /// How many chunks to send per embedding API call.
    pub embed_batch_size: usize,
    /// Skip files whose path components match any of these (e.g. `target`,
    /// `.git`).
    pub ignore_dir_components: Vec<String>,
    /// Sprint H — per-path-prefix `applies_to` overrides. The first prefix
    /// that matches the doc's relative path (forward-slash, lower-case)
    /// wins; otherwise [`default_applies_to`] is used. Lets the bestel
    /// references corpus tag `references/poe2/**` with `["poe2"]` so the
    /// LanceDB filter `array_has(applies_to, 'poe2')` isolates PoE2-only
    /// content from PoE1+PoE2 shared references.
    pub applies_to_overrides: Vec<(String, Vec<String>)>,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            extension: "md".into(),
            source_kind: "reference".into(),
            default_applies_to: vec!["poe1".into(), "poe2".into()],
            default_tags: Vec::new(),
            embed_batch_size: 64,
            ignore_dir_components: vec![
                ".git".into(),
                "target".into(),
                "node_modules".into(),
            ],
            applies_to_overrides: vec![(
                "poe2/".into(),
                vec!["poe2".into()],
            )],
        }
    }
}

/// Resolve the `applies_to` for a doc by checking [`applies_to_overrides`]
/// in order. Path is matched as a substring (forward-slash separator, as
/// produced by [`collect_files`] + `replace('\\\\', "/")`).
pub fn resolve_applies_to<'a>(
    rel_path: &str,
    overrides: &'a [(String, Vec<String>)],
    default: &'a [String],
) -> Vec<String> {
    for (prefix, targets) in overrides {
        if rel_path.contains(prefix.as_str()) {
            return targets.clone();
        }
    }
    default.to_vec()
}

/// Walk `root`, ingest every `*.md` file, return aggregate stats. The
/// `IngestStats` is also pushed to `tracing` at INFO level.
pub async fn ingest_corpus<I, E>(
    root: &Path,
    index: &I,
    embedder: &E,
    chunker: &Chunker,
    config: &IngestConfig,
) -> Result<IngestStats>
where
    I: KbIndex,
    E: EmbeddingProvider,
{
    let started = Instant::now();
    let mut stats = IngestStats::default();
    if !root.exists() {
        return Err(anyhow!("ingest root does not exist: {root:?}"));
    }
    let existing_hashes = index.doc_hashes().await.context("read existing hashes")?;
    let mut seen_paths: HashSet<String> = HashSet::new();
    let files = collect_files(root, &config.extension, &config.ignore_dir_components)
        .with_context(|| format!("walk corpus root {root:?}"))?;
    for file in &files {
        stats.files_seen += 1;
        let rel = file
            .strip_prefix(root)
            .with_context(|| format!("strip_prefix {file:?}"))?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        seen_paths.insert(rel_str.clone());
        let bytes = std::fs::read(file).with_context(|| format!("read {file:?}"))?;
        let hash = blake3::hash(&bytes).to_hex().to_string();
        if existing_hashes.get(&rel_str) == Some(&hash) {
            stats.files_skipped_unchanged += 1;
            debug!(doc_path = %rel_str, "skip unchanged");
            continue;
        }
        let text = String::from_utf8(bytes)
            .with_context(|| format!("UTF-8 decode {file:?}"))?;
        let last_updated = file
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| chrono::DateTime::<Utc>::from(t))
            .unwrap_or_else(Utc::now);
        let applies_to = resolve_applies_to(
            &rel_str,
            &config.applies_to_overrides,
            &config.default_applies_to,
        );
        let chunks = chunker
            .chunk_markdown(ChunkInput {
                doc_path: &rel_str,
                markdown: &text,
                last_updated,
                applies_to,
                tags: config.default_tags.clone(),
                source_kind: &config.source_kind,
            })
            .with_context(|| format!("chunk {rel_str}"))?;
        if chunks.is_empty() {
            warn!(doc_path = %rel_str, "no chunks produced");
            continue;
        }
        embed_and_upsert(index, embedder, chunks, config.embed_batch_size, &mut stats)
            .await
            .with_context(|| format!("embed+upsert {rel_str}"))?;
        stats.files_indexed += 1;
        info!(doc_path = %rel_str, "indexed");
    }
    // Purge docs that disappeared from the corpus.
    for path in existing_hashes.keys() {
        if !seen_paths.contains(path) {
            warn!(doc_path = %path, "doc removed from corpus, purging");
            index
                .delete_by_doc_path(path)
                .await
                .with_context(|| format!("delete missing {path}"))?;
        }
    }
    stats.elapsed_ms = started.elapsed().as_millis() as u64;
    info!(
        seen = stats.files_seen,
        skipped = stats.files_skipped_unchanged,
        indexed = stats.files_indexed,
        chunks = stats.chunks_emitted,
        elapsed_ms = stats.elapsed_ms,
        "ingest pass complete"
    );
    Ok(stats)
}

async fn embed_and_upsert<I, E>(
    index: &I,
    embedder: &E,
    chunks: Vec<Chunk>,
    batch_size: usize,
    stats: &mut IngestStats,
) -> Result<()>
where
    I: KbIndex,
    E: EmbeddingProvider,
{
    let total = chunks.len();
    stats.chunks_emitted += total as u32;
    let mut all_embeddings: Vec<Vec<f32>> = Vec::with_capacity(total);
    for batch in chunks.chunks(batch_size) {
        let texts: Vec<String> = batch.iter().map(|c| c.body.clone()).collect();
        let mut embeddings = embedder.embed_batch(&texts).await?;
        if embeddings.len() != batch.len() {
            return Err(anyhow!(
                "embed_batch returned {} vectors for {} inputs",
                embeddings.len(),
                batch.len()
            ));
        }
        stats.embed_batches += 1;
        stats.chunks_embedded += batch.len() as u32;
        all_embeddings.append(&mut embeddings);
    }
    index.upsert(chunks, all_embeddings).await
}

fn collect_files(
    root: &Path,
    extension: &str,
    ignore: &[String],
) -> Result<Vec<PathBuf>> {
    fn walk(
        dir: &Path,
        extension: &str,
        ignore: &[String],
        out: &mut Vec<PathBuf>,
    ) -> Result<()> {
        for entry in std::fs::read_dir(dir).with_context(|| format!("read_dir {dir:?}"))? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if ignore.iter().any(|i| i == name) {
                        continue;
                    }
                }
                walk(&path, extension, ignore, out)?;
            } else if file_type.is_file() {
                if path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|e| e.eq_ignore_ascii_case(extension))
                    .unwrap_or(false)
                {
                    out.push(path);
                }
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(root, extension, ignore, &mut out)?;
    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::ChunkerConfig;
    use crate::lance::LanceIndex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::TempDir;

    /// Deterministic embedder: returns a vector whose dim-0 component is
    /// the count of "spell" occurrences (BM25-correlated), dim-1 is "build"
    /// occurrences, etc. Useful for asserting which chunks get retrieved
    /// without spinning up a real model.
    struct CannedEmbedder {
        dim: usize,
        calls: AtomicUsize,
    }

    impl CannedEmbedder {
        fn new(dim: usize) -> Self {
            Self {
                dim,
                calls: AtomicUsize::new(0),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl EmbeddingProvider for CannedEmbedder {
        fn dim(&self) -> usize {
            self.dim
        }

        fn embed_batch(
            &self,
            texts: &[String],
        ) -> impl std::future::Future<Output = Result<Vec<Vec<f32>>>> + Send {
            let dim = self.dim;
            let texts: Vec<String> = texts.to_vec();
            self.calls.fetch_add(1, Ordering::SeqCst);
            async move {
                let mut out = Vec::with_capacity(texts.len());
                for t in texts {
                    let mut v = vec![0.0_f32; dim];
                    v[0] = t.matches("spell").count() as f32;
                    v[1] = t.matches("build").count() as f32;
                    v[2] = t.len() as f32 / 1000.0;
                    out.push(v);
                }
                Ok(out)
            }
        }
    }

    fn write_md(root: &Path, rel: &str, content: &str) -> PathBuf {
        let p = root.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, content).unwrap();
        p
    }

    #[tokio::test]
    async fn ingest_corpus_indexes_two_files_then_skips_unchanged_on_rerun() {
        let corpus = TempDir::new().unwrap();
        let index_dir = TempDir::new().unwrap();
        write_md(
            corpus.path(),
            "alpha.md",
            "# Spell Suppression\n\nSpell suppression caps at 100 percent.\n",
        );
        write_md(
            corpus.path(),
            "subdir/beta.md",
            "# Resolute Technique\n\nGuarantees hits, disables crit.\n",
        );
        let index = LanceIndex::open_or_create(index_dir.path(), 8).await.unwrap();
        let embedder = CannedEmbedder::new(8);
        let chunker = Chunker::new(ChunkerConfig::default());
        let config = IngestConfig::default();

        let stats1 = ingest_corpus(corpus.path(), &index, &embedder, &chunker, &config)
            .await
            .unwrap();
        assert_eq!(stats1.files_seen, 2);
        assert_eq!(stats1.files_indexed, 2);
        assert_eq!(stats1.files_skipped_unchanged, 0);

        let calls_after_pass1 = embedder.calls();
        let stats2 = ingest_corpus(corpus.path(), &index, &embedder, &chunker, &config)
            .await
            .unwrap();
        assert_eq!(stats2.files_seen, 2);
        assert_eq!(stats2.files_indexed, 0, "expected all unchanged");
        assert_eq!(stats2.files_skipped_unchanged, 2);
        assert_eq!(
            embedder.calls(),
            calls_after_pass1,
            "no embed call should fire on a fully-cached pass"
        );
    }

    #[tokio::test]
    async fn ingest_corpus_purges_doc_when_file_disappears() {
        let corpus = TempDir::new().unwrap();
        let index_dir = TempDir::new().unwrap();
        write_md(corpus.path(), "a.md", "# A\nbody a.\n");
        let to_remove = write_md(corpus.path(), "b.md", "# B\nbody b.\n");
        let index = LanceIndex::open_or_create(index_dir.path(), 8).await.unwrap();
        let embedder = CannedEmbedder::new(8);
        let chunker = Chunker::new(ChunkerConfig::default());
        let config = IngestConfig::default();

        ingest_corpus(corpus.path(), &index, &embedder, &chunker, &config)
            .await
            .unwrap();
        let hashes_before = index.doc_hashes().await.unwrap();
        assert!(hashes_before.contains_key("b.md"));

        std::fs::remove_file(&to_remove).unwrap();
        ingest_corpus(corpus.path(), &index, &embedder, &chunker, &config)
            .await
            .unwrap();
        let hashes_after = index.doc_hashes().await.unwrap();
        assert!(!hashes_after.contains_key("b.md"), "removed doc was not purged");
    }

    #[test]
    fn resolve_applies_to_picks_first_matching_prefix() {
        let default = vec!["poe1".to_string(), "poe2".to_string()];
        let overrides = vec![
            ("poe2/".to_string(), vec!["poe2".to_string()]),
            ("legacy/".to_string(), vec!["poe1".to_string()]),
        ];
        assert_eq!(
            resolve_applies_to("references/poe2/05_atlas_mechanics_05.md", &overrides, &default),
            vec!["poe2".to_string()]
        );
        assert_eq!(
            resolve_applies_to("references/legacy/03_synthesis.md", &overrides, &default),
            vec!["poe1".to_string()]
        );
        assert_eq!(
            resolve_applies_to("references/01_voice.md", &overrides, &default),
            vec!["poe1".to_string(), "poe2".to_string()]
        );
    }

    #[tokio::test]
    async fn ingest_corpus_tags_poe2_subdir_with_poe2_only_applies_to() {
        let corpus = TempDir::new().unwrap();
        let index_dir = TempDir::new().unwrap();
        write_md(
            corpus.path(),
            "shared.md",
            "# Shared mechanic\n\nGoes for both games.\n",
        );
        write_md(
            corpus.path(),
            "poe2/version.md",
            "# PoE2 0.5\n\nReturn of the Ancients releases 2026-05-29.\n",
        );
        let index = LanceIndex::open_or_create(index_dir.path(), 8).await.unwrap();
        let embedder = CannedEmbedder::new(8);
        let chunker = Chunker::new(ChunkerConfig::default());
        let config = IngestConfig::default();

        let stats = ingest_corpus(corpus.path(), &index, &embedder, &chunker, &config)
            .await
            .unwrap();
        assert_eq!(stats.files_indexed, 2);
        index.ensure_fts_index().await.unwrap();

        let poe2_only = index
            .search_hybrid("Return of the Ancients", vec![0.0_f32; 8], 5, Some("poe2"), &[])
            .await
            .unwrap();
        assert!(
            poe2_only.iter().any(|h| h.chunk.doc_path == "poe2/version.md"),
            "poe2-tagged doc should appear under applies_to=poe2 filter"
        );

        let poe1_only = index
            .search_hybrid("Return of the Ancients", vec![0.0_f32; 8], 5, Some("poe1"), &[])
            .await
            .unwrap();
        assert!(
            poe1_only.iter().all(|h| h.chunk.doc_path != "poe2/version.md"),
            "poe2-tagged doc must NOT leak into poe1 filter"
        );
    }

    #[tokio::test]
    async fn ingest_corpus_re_embeds_only_changed_file() {
        let corpus = TempDir::new().unwrap();
        let index_dir = TempDir::new().unwrap();
        let a_path = write_md(corpus.path(), "a.md", "# A\nfirst.\n");
        write_md(corpus.path(), "b.md", "# B\nstable.\n");
        let index = LanceIndex::open_or_create(index_dir.path(), 8).await.unwrap();
        let embedder = CannedEmbedder::new(8);
        let chunker = Chunker::new(ChunkerConfig::default());
        let config = IngestConfig::default();
        ingest_corpus(corpus.path(), &index, &embedder, &chunker, &config)
            .await
            .unwrap();
        let calls_after_pass1 = embedder.calls();

        std::fs::write(&a_path, "# A\nupdated body content.\n").unwrap();
        let stats2 = ingest_corpus(corpus.path(), &index, &embedder, &chunker, &config)
            .await
            .unwrap();
        assert_eq!(stats2.files_indexed, 1);
        assert_eq!(stats2.files_skipped_unchanged, 1);
        assert!(
            embedder.calls() > calls_after_pass1,
            "expected at least one re-embed for changed file"
        );
    }

}

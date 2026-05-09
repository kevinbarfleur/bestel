//! Markdown-header-aware chunker.
//!
//! Splits a Markdown document into [`Chunk`]s of 400-800 tokens (cl100k
//! tokenizer, OpenAI-compatible) with 15% overlap. Heading context is
//! propagated: a chunk under `## Offence > ### Hit > #### Crit` carries
//! `heading_path = ["Offence", "Hit", "Crit"]` and inlines that path as a
//! prefix in `body` so BM25 picks up section names.
//!
//! ATX-only headers (`# foo`, `## bar`). Setext headers (underline-style)
//! are not used in the Bestel reference library. Code fences (` ``` `) are
//! detected so `#` at line-start inside a fenced block is not mistaken for
//! a heading.
//!
//! Long sections are split into sliding windows: target window size is
//! `min_tokens..max_tokens`, step size is `window - overlap`. Short
//! sections produce a single chunk.

use std::sync::OnceLock;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use tiktoken_rs::CoreBPE;

use crate::types::Chunk;

/// Tokenizer + window parameters. Defaults match the ROADMAP § Sprint E
/// spec: 400-800 tokens per chunk, 15% overlap.
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    pub min_tokens: usize,
    pub max_tokens: usize,
    pub overlap_ratio: f32,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            min_tokens: 400,
            max_tokens: 800,
            overlap_ratio: 0.15,
        }
    }
}

pub struct Chunker {
    config: ChunkerConfig,
}

impl Chunker {
    pub fn new(config: ChunkerConfig) -> Self {
        Self { config }
    }

    /// Split `markdown` into chunks. The same `(doc_path, markdown)` input
    /// produces stable chunk IDs across runs — required for incremental
    /// indexing.
    pub fn chunk_markdown(&self, input: ChunkInput<'_>) -> Result<Vec<Chunk>> {
        let bpe = tokenizer();
        let content_hash = blake3_hex(input.markdown.as_bytes());
        let sections = parse_sections(input.markdown);
        let mut out = Vec::new();
        let mut chunk_idx_global: usize = 0;
        for section in sections {
            let prefix = heading_prefix(&section.heading_path);
            let body_tokens = bpe.encode_ordinary(&section.body);
            if body_tokens.len() <= self.config.max_tokens {
                let final_body = format!("{}{}", prefix, section.body);
                let chunk = self.build_chunk(
                    &input,
                    &section.heading_path,
                    final_body,
                    &content_hash,
                    chunk_idx_global,
                )?;
                out.push(chunk);
                chunk_idx_global += 1;
                continue;
            }
            // Section too long — split body into overlapping token windows.
            // Each window gets the same heading prefix prepended so BM25 still
            // matches on section names regardless of which window the hit lands in.
            let target = ((self.config.min_tokens + self.config.max_tokens) / 2).max(1);
            let overlap = ((target as f32) * self.config.overlap_ratio).round() as usize;
            let step = target.saturating_sub(overlap).max(1);
            let mut start = 0usize;
            while start < body_tokens.len() {
                let end = (start + target).min(body_tokens.len());
                let window = &body_tokens[start..end];
                let window_text = bpe.decode(window).context("decode token window")?;
                let final_body = format!("{}{}", prefix, window_text);
                let chunk = self.build_chunk(
                    &input,
                    &section.heading_path,
                    final_body,
                    &content_hash,
                    chunk_idx_global,
                )?;
                out.push(chunk);
                chunk_idx_global += 1;
                if end == body_tokens.len() {
                    break;
                }
                start += step;
            }
        }
        Ok(out)
    }

    fn build_chunk(
        &self,
        input: &ChunkInput<'_>,
        heading_path: &[String],
        body: String,
        content_hash: &str,
        chunk_idx: usize,
    ) -> Result<Chunk> {
        let id_seed = format!(
            "{}\0{}\0{}",
            input.doc_path,
            heading_path.join("/"),
            chunk_idx
        );
        let id = blake3_hex(id_seed.as_bytes());
        Ok(Chunk {
            id,
            doc_path: input.doc_path.to_string(),
            heading_path: heading_path.to_vec(),
            tags: input.tags.clone(),
            applies_to: input.applies_to.clone(),
            last_updated: input.last_updated,
            content_hash: content_hash.to_string(),
            source_kind: input.source_kind.to_string(),
            body,
        })
    }
}

/// Borrowed inputs for a single chunking pass.
#[derive(Debug, Clone)]
pub struct ChunkInput<'a> {
    pub doc_path: &'a str,
    pub markdown: &'a str,
    pub last_updated: DateTime<Utc>,
    pub applies_to: Vec<String>,
    pub tags: Vec<String>,
    pub source_kind: &'a str,
}

#[derive(Debug)]
struct Section {
    heading_path: Vec<String>,
    body: String,
}

fn parse_sections(markdown: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut current_body = String::new();
    let mut current_path: Vec<String> = Vec::new();
    let mut in_fence = false;
    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            current_body.push_str(line);
            current_body.push('\n');
            continue;
        }
        if !in_fence {
            if let Some((level, title)) = parse_atx(trimmed) {
                if !current_body.trim().is_empty() {
                    sections.push(Section {
                        heading_path: current_path.clone(),
                        body: std::mem::take(&mut current_body),
                    });
                } else {
                    current_body.clear();
                }
                while stack.len() >= level {
                    stack.pop();
                }
                stack.push(title.to_string());
                current_path = stack.clone();
                continue;
            }
        }
        current_body.push_str(line);
        current_body.push('\n');
    }
    if !current_body.trim().is_empty() {
        sections.push(Section {
            heading_path: current_path,
            body: current_body,
        });
    }
    sections
}

fn parse_atx(line: &str) -> Option<(usize, &str)> {
    let bytes = line.as_bytes();
    let mut level = 0usize;
    while level < 6 && bytes.get(level) == Some(&b'#') {
        level += 1;
    }
    if level == 0 {
        return None;
    }
    if bytes.get(level) != Some(&b' ') {
        return None;
    }
    let title = line[level + 1..].trim_end_matches(|c: char| c == '#' || c.is_whitespace());
    if title.is_empty() {
        None
    } else {
        Some((level, title))
    }
}

fn heading_prefix(heading_path: &[String]) -> String {
    if heading_path.is_empty() {
        String::new()
    } else {
        format!("Section: {}\n\n", heading_path.join(" > "))
    }
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn tokenizer() -> &'static CoreBPE {
    static TOKENIZER: OnceLock<CoreBPE> = OnceLock::new();
    TOKENIZER.get_or_init(|| {
        tiktoken_rs::cl100k_base().expect("cl100k_base tokenizer load")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn fixed_time() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 8, 12, 0, 0).unwrap()
    }

    fn input<'a>(doc_path: &'a str, markdown: &'a str) -> ChunkInput<'a> {
        ChunkInput {
            doc_path,
            markdown,
            last_updated: fixed_time(),
            applies_to: vec!["poe1".into()],
            tags: vec![],
            source_kind: "reference",
        }
    }

    #[test]
    fn three_nested_headers_produce_correct_heading_paths() {
        let md = "# Offence\n\nIntro text.\n\n## Hit\n\nHit body.\n\n### Crit\n\nCrit body.\n";
        let chunker = Chunker::new(ChunkerConfig::default());
        let chunks = chunker.chunk_markdown(input("ref.md", md)).unwrap();
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].heading_path, vec!["Offence".to_string()]);
        assert_eq!(
            chunks[1].heading_path,
            vec!["Offence".to_string(), "Hit".to_string()]
        );
        assert_eq!(
            chunks[2].heading_path,
            vec![
                "Offence".to_string(),
                "Hit".to_string(),
                "Crit".to_string()
            ]
        );
        assert!(chunks[0].body.starts_with("Section: Offence"));
        assert!(chunks[1].body.starts_with("Section: Offence > Hit"));
    }

    #[test]
    fn long_section_splits_with_overlap_and_repeats_heading_prefix() {
        // Build a section ~1500 tokens by repeating a token-heavy phrase.
        let body: String = "lorem ipsum dolor sit amet consectetur adipiscing elit. ".repeat(200);
        let md = format!("# Big\n\n{}\n", body);
        let chunker = Chunker::new(ChunkerConfig::default());
        let chunks = chunker.chunk_markdown(input("big.md", &md)).unwrap();
        assert!(chunks.len() >= 2, "expected splitting, got {}", chunks.len());
        for c in &chunks {
            assert_eq!(c.heading_path, vec!["Big".to_string()]);
            assert!(
                c.body.starts_with("Section: Big\n\n"),
                "every chunk must carry the heading prefix; got: {:?}",
                &c.body[..40.min(c.body.len())]
            );
        }
        // Verify overlap: take a 60-char tail of chunk[0]'s body content
        // (after the prefix) and confirm it appears verbatim in chunk[1].
        let prefix = "Section: Big\n\n";
        let body0 = chunks[0].body.strip_prefix(prefix).unwrap();
        let body1 = chunks[1].body.strip_prefix(prefix).unwrap();
        let tail0 = &body0[body0.len().saturating_sub(60)..];
        assert!(
            body1.contains(tail0),
            "chunk[1] should contain the 15% tail of chunk[0]; tail={:?}",
            tail0
        );
    }

    #[test]
    fn content_hash_is_stable_across_reruns() {
        let md = "# A\n\nbody one.\n\n# B\n\nbody two.\n";
        let chunker = Chunker::new(ChunkerConfig::default());
        let a = chunker.chunk_markdown(input("doc.md", md)).unwrap();
        let b = chunker.chunk_markdown(input("doc.md", md)).unwrap();
        assert_eq!(a.len(), b.len());
        for (ca, cb) in a.iter().zip(b.iter()) {
            assert_eq!(ca.id, cb.id);
            assert_eq!(ca.content_hash, cb.content_hash);
        }
    }

    #[test]
    fn code_fence_hash_is_not_treated_as_heading() {
        let md = "# Real Header\n\n```rust\n# not a header\nlet x = 1;\n```\n\nbody.\n";
        let chunker = Chunker::new(ChunkerConfig::default());
        let chunks = chunker.chunk_markdown(input("code.md", md)).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].heading_path, vec!["Real Header".to_string()]);
        assert!(chunks[0].body.contains("# not a header"));
    }

    #[test]
    fn preamble_before_first_header_emits_unscoped_chunk() {
        let md = "Preamble paragraph without a heading.\n\n# First\n\nUnder first.\n";
        let chunker = Chunker::new(ChunkerConfig::default());
        let chunks = chunker.chunk_markdown(input("p.md", md)).unwrap();
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].heading_path.is_empty());
        assert_eq!(chunks[1].heading_path, vec!["First".to_string()]);
    }
}

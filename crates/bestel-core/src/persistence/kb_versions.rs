//! `kb_versions` table — Sprint E ingest fingerprint per indexed doc.
//!
//! `bestel-rag::ingest_corpus` already keeps content hashes in the LanceDB
//! index for incremental ingest. This mirror keeps a queryable index of
//! "what was indexed when, by which embedder" so the dev panel and `bestel
//! db stats` can answer "is the KB stale?" without opening the LanceDB
//! table.

use anyhow::Result;
use rusqlite::params;

use super::Db;

#[derive(Debug, Clone)]
pub struct KbVersionRow {
    pub doc_path: String,
    pub content_hash: String,
    pub indexed_at: String,
    pub dim: i64,
    pub embedder_label: String,
}

pub fn upsert_kb_version(db: &Db, row: &KbVersionRow) -> Result<()> {
    db.with_conn_rusqlite(|c| {
        c.execute(
            "INSERT INTO kb_versions (doc_path, content_hash, indexed_at, dim, embedder_label) \
             VALUES (?1, ?2, ?3, ?4, ?5) \
             ON CONFLICT(doc_path) DO UPDATE SET \
                 content_hash = excluded.content_hash, \
                 indexed_at = excluded.indexed_at, \
                 dim = excluded.dim, \
                 embedder_label = excluded.embedder_label",
            params![
                row.doc_path,
                row.content_hash,
                row.indexed_at,
                row.dim,
                row.embedder_label,
            ],
        )?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_then_replace() {
        let db = Db::open_in_memory().unwrap();
        upsert_kb_version(
            &db,
            &KbVersionRow {
                doc_path: "references/01_voice.md".into(),
                content_hash: "abc".into(),
                indexed_at: "2026-05-09T00:00:00Z".into(),
                dim: 768,
                embedder_label: "ollama:nomic-embed-text".into(),
            },
        )
        .unwrap();
        upsert_kb_version(
            &db,
            &KbVersionRow {
                doc_path: "references/01_voice.md".into(),
                content_hash: "def".into(),
                indexed_at: "2026-05-09T01:00:00Z".into(),
                dim: 768,
                embedder_label: "ollama:nomic-embed-text".into(),
            },
        )
        .unwrap();
        let count: i64 = db
            .with_conn_rusqlite(|c| {
                c.query_row("SELECT COUNT(*) FROM kb_versions", [], |r| r.get(0))
            })
            .unwrap();
        assert_eq!(count, 1);
        let hash: String = db
            .with_conn_rusqlite(|c| {
                c.query_row("SELECT content_hash FROM kb_versions", [], |r| r.get(0))
            })
            .unwrap();
        assert_eq!(hash, "def");
    }
}

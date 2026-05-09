//! LanceDB-backed implementation of [`KbIndex`].
//!
//! Storage layout: a single LanceDB table named `kb_chunks` whose Arrow
//! schema mirrors [`Chunk`] plus a FixedSizeList<Float32, dim> column for
//! the embedding. Hybrid retrieval uses LanceDB's native pipeline:
//! `query.nearest_to(vec).full_text_search(q).execute()` — LanceDB itself
//! collects parallel vector + FTS results and reranks with `RRFReranker`
//! (k = 60 by default). We therefore do NOT carry a separate Tantivy
//! index; the FTS index lives inside the same Lance dataset.
//!
//! Filtering: `game` lifts to `array_has(applies_to, 'poe1')`; `tags` to
//! conjoined `array_has` predicates. Filters are passed as a SQL string
//! via [`QueryBase::only_if`].

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use arrow_array::{
    builder::{FixedSizeListBuilder, Float32Builder, ListBuilder, StringBuilder},
    Array, RecordBatch, RecordBatchIterator, RecordBatchReader, StringArray,
    TimestampMicrosecondArray,
};
use arrow_schema::{DataType, Field, Schema, SchemaRef, TimeUnit};
use chrono::{TimeZone, Utc};
use futures_util::TryStreamExt;
use lance_index::scalar::FullTextSearchQuery;
use lancedb::index::scalar::FtsIndexBuilder;
use lancedb::index::Index;
use lancedb::query::{ExecutableQuery, QueryBase, Select};
use lancedb::table::Table;
use lancedb::{connect, Connection};

use crate::traits::KbIndex;
use crate::types::{Chunk, SearchHit};

pub const TABLE_NAME: &str = "kb_chunks";
const VECTOR_COL: &str = "vector";
const BODY_COL: &str = "body";

/// LanceDB-backed knowledge-base index.
pub struct LanceIndex {
    _conn: Connection,
    table: Table,
    dim: usize,
    schema: SchemaRef,
}

impl LanceIndex {
    /// Open an existing index at `dir`, or create an empty one with the
    /// given embedding `dim` if no Lance table exists yet.
    pub async fn open_or_create(dir: &Path, dim: usize) -> Result<Self> {
        let uri = dir
            .to_str()
            .with_context(|| format!("non-UTF-8 LanceDB path: {dir:?}"))?;
        std::fs::create_dir_all(dir).with_context(|| format!("create_dir_all {dir:?}"))?;
        let conn = connect(uri).execute().await?;
        let names = conn.table_names().execute().await?;
        let schema = Arc::new(build_arrow_schema(dim));
        let table = if names.iter().any(|n| n == TABLE_NAME) {
            conn.open_table(TABLE_NAME).execute().await?
        } else {
            conn.create_empty_table(TABLE_NAME, schema.clone())
                .execute()
                .await?
        };
        Ok(Self {
            _conn: conn,
            table,
            dim,
            schema,
        })
    }

    /// Build a Tantivy-style FTS index on the `body` column. Idempotent —
    /// recreating an FTS index on an indexed column is a no-op in LanceDB
    /// 0.27 unless the underlying parameters change.
    pub async fn ensure_fts_index(&self) -> Result<()> {
        let builder = FtsIndexBuilder::default();
        self.table
            .create_index(&[BODY_COL], Index::FTS(builder))
            .execute()
            .await
            .context("create FTS index on body")?;
        Ok(())
    }

    pub fn dim(&self) -> usize {
        self.dim
    }
}

impl KbIndex for LanceIndex {
    fn upsert(
        &self,
        chunks: Vec<Chunk>,
        embeddings: Vec<Vec<f32>>,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        async move {
            if chunks.is_empty() {
                return Ok(());
            }
            if chunks.len() != embeddings.len() {
                return Err(anyhow!(
                    "upsert: chunks ({}) and embeddings ({}) length mismatch",
                    chunks.len(),
                    embeddings.len()
                ));
            }
            for (i, e) in embeddings.iter().enumerate() {
                if e.len() != self.dim {
                    return Err(anyhow!(
                        "embedding[{i}] dim {} != schema dim {}",
                        e.len(),
                        self.dim
                    ));
                }
            }
            // Delete existing rows for these IDs so the upsert is idempotent.
            // We chunk the IN clause to avoid pathological SQL lengths.
            for id_batch in chunks.chunks(256) {
                let id_list: String = id_batch
                    .iter()
                    .map(|c| format!("'{}'", c.id.replace('\'', "''")))
                    .collect::<Vec<_>>()
                    .join(",");
                let predicate = format!("id IN ({id_list})");
                self.table
                    .delete(&predicate)
                    .await
                    .context("delete-before-upsert")?;
            }
            let batch = build_record_batch(&self.schema, &chunks, &embeddings, self.dim)?;
            let iter = RecordBatchIterator::new(vec![Ok(batch)].into_iter(), self.schema.clone());
            let reader: Box<dyn RecordBatchReader + Send> = Box::new(iter);
            self.table
                .add(reader)
                .execute()
                .await
                .context("add chunk batch")?;
            Ok(())
        }
    }

    fn search_hybrid(
        &self,
        query: &str,
        query_embedding: Vec<f32>,
        top_k: usize,
        game: Option<&str>,
        tags: &[String],
    ) -> impl std::future::Future<Output = Result<Vec<SearchHit>>> + Send {
        async move {
            if query_embedding.len() != self.dim {
                return Err(anyhow!(
                    "query_embedding dim {} != schema dim {}",
                    query_embedding.len(),
                    self.dim
                ));
            }
            let predicate = build_filter_sql(game, tags);
            let mut q = self
                .table
                .query()
                .nearest_to(query_embedding)?
                .full_text_search(FullTextSearchQuery::new(query.into()))
                .limit(top_k);
            if let Some(p) = predicate.as_deref() {
                q = q.only_if(p);
            }
            let stream = q.execute().await.context("execute hybrid query")?;
            let batches: Vec<RecordBatch> =
                stream.try_collect().await.context("collect hybrid stream")?;
            let mut hits = Vec::new();
            for batch in batches {
                hits.extend(decode_search_batch(&batch, self.dim)?);
            }
            Ok(hits)
        }
    }

    fn delete_by_doc_path(
        &self,
        doc_path: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        async move {
            let escaped = doc_path.replace('\'', "''");
            let predicate = format!("doc_path = '{escaped}'");
            self.table
                .delete(&predicate)
                .await
                .context("delete_by_doc_path")?;
            Ok(())
        }
    }

    fn doc_hashes(
        &self,
    ) -> impl std::future::Future<Output = Result<HashMap<String, String>>> + Send {
        async move {
            let stream = self
                .table
                .query()
                .select(Select::columns(&["doc_path", "content_hash"]))
                .execute()
                .await
                .context("doc_hashes select")?;
            let batches: Vec<RecordBatch> = stream.try_collect().await.context("collect")?;
            let mut out: HashMap<String, String> = HashMap::new();
            for batch in batches {
                let doc_path = batch
                    .column_by_name("doc_path")
                    .ok_or_else(|| anyhow!("doc_path column missing"))?;
                let content_hash = batch
                    .column_by_name("content_hash")
                    .ok_or_else(|| anyhow!("content_hash column missing"))?;
                let dp = doc_path
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .ok_or_else(|| anyhow!("doc_path not Utf8"))?;
                let ch = content_hash
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .ok_or_else(|| anyhow!("content_hash not Utf8"))?;
                for i in 0..batch.num_rows() {
                    let p = dp.value(i).to_string();
                    let h = ch.value(i).to_string();
                    // Different chunks of the same doc share content_hash.
                    // Conflicts are not expected; last-write-wins is fine.
                    out.insert(p, h);
                }
            }
            Ok(out)
        }
    }
}

fn build_arrow_schema(dim: usize) -> Schema {
    let item = Arc::new(Field::new("item", DataType::Float32, true));
    let str_item = Arc::new(Field::new("item", DataType::Utf8, true));
    Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new(
            VECTOR_COL,
            DataType::FixedSizeList(item, dim as i32),
            true,
        ),
        Field::new("body", DataType::Utf8, false),
        Field::new("doc_path", DataType::Utf8, false),
        Field::new(
            "heading_path",
            DataType::List(str_item.clone()),
            false,
        ),
        Field::new("tags", DataType::List(str_item.clone()), false),
        Field::new("applies_to", DataType::List(str_item), false),
        Field::new(
            "last_updated",
            DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
            false,
        ),
        Field::new("content_hash", DataType::Utf8, false),
        Field::new("source_kind", DataType::Utf8, false),
    ])
}

fn build_record_batch(
    schema: &SchemaRef,
    chunks: &[Chunk],
    embeddings: &[Vec<f32>],
    dim: usize,
) -> Result<RecordBatch> {
    let n = chunks.len();
    let mut id = StringBuilder::new();
    let mut vector_inner = Float32Builder::new();
    let mut body = StringBuilder::new();
    let mut doc_path = StringBuilder::new();
    let mut heading_path = ListBuilder::new(StringBuilder::new());
    let mut tags = ListBuilder::new(StringBuilder::new());
    let mut applies_to = ListBuilder::new(StringBuilder::new());
    let mut last_updated_micros: Vec<i64> = Vec::with_capacity(n);
    let mut content_hash = StringBuilder::new();
    let mut source_kind = StringBuilder::new();
    for (chunk, emb) in chunks.iter().zip(embeddings.iter()) {
        id.append_value(&chunk.id);
        for v in emb {
            vector_inner.append_value(*v);
        }
        body.append_value(&chunk.body);
        doc_path.append_value(&chunk.doc_path);
        for s in &chunk.heading_path {
            heading_path.values().append_value(s);
        }
        heading_path.append(true);
        for s in &chunk.tags {
            tags.values().append_value(s);
        }
        tags.append(true);
        for s in &chunk.applies_to {
            applies_to.values().append_value(s);
        }
        applies_to.append(true);
        last_updated_micros.push(chunk.last_updated.timestamp_micros());
        content_hash.append_value(&chunk.content_hash);
        source_kind.append_value(&chunk.source_kind);
    }
    let mut vector_builder =
        FixedSizeListBuilder::new(Float32Builder::new(), dim as i32);
    // Refill vector_builder properly: append values per row, then `append(true)`.
    // We rebuild rather than reuse vector_inner because FixedSizeListBuilder
    // owns the inner builder.
    for emb in embeddings {
        for v in emb {
            vector_builder.values().append_value(*v);
        }
        vector_builder.append(true);
    }
    let vector_array = vector_builder.finish();
    let id_array = id.finish();
    let body_array = body.finish();
    let doc_path_array = doc_path.finish();
    let heading_path_array = heading_path.finish();
    let tags_array = tags.finish();
    let applies_to_array = applies_to.finish();
    let last_updated_array = TimestampMicrosecondArray::from(last_updated_micros)
        .with_timezone("UTC");
    let content_hash_array = content_hash.finish();
    let source_kind_array = source_kind.finish();
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(id_array),
            Arc::new(vector_array),
            Arc::new(body_array),
            Arc::new(doc_path_array),
            Arc::new(heading_path_array),
            Arc::new(tags_array),
            Arc::new(applies_to_array),
            Arc::new(last_updated_array),
            Arc::new(content_hash_array),
            Arc::new(source_kind_array),
        ],
    )
    .context("build RecordBatch")?;
    Ok(batch)
}

fn build_filter_sql(game: Option<&str>, tags: &[String]) -> Option<String> {
    let mut clauses = Vec::new();
    if let Some(g) = game {
        let g = g.replace('\'', "''");
        clauses.push(format!("array_has(applies_to, '{g}')"));
    }
    for t in tags {
        let t = t.replace('\'', "''");
        clauses.push(format!("array_has(tags, '{t}')"));
    }
    if clauses.is_empty() {
        None
    } else {
        Some(clauses.join(" AND "))
    }
}

fn decode_search_batch(batch: &RecordBatch, _dim: usize) -> Result<Vec<SearchHit>> {
    let id = batch
        .column_by_name("id")
        .and_then(|c| c.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| anyhow!("id column missing or wrong type"))?;
    let body = batch
        .column_by_name("body")
        .and_then(|c| c.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| anyhow!("body column missing or wrong type"))?;
    let doc_path = batch
        .column_by_name("doc_path")
        .and_then(|c| c.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| anyhow!("doc_path column missing or wrong type"))?;
    let content_hash = batch
        .column_by_name("content_hash")
        .and_then(|c| c.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| anyhow!("content_hash column missing or wrong type"))?;
    let source_kind = batch
        .column_by_name("source_kind")
        .and_then(|c| c.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| anyhow!("source_kind column missing or wrong type"))?;
    let last_updated = batch
        .column_by_name("last_updated")
        .and_then(|c| c.as_any().downcast_ref::<TimestampMicrosecondArray>())
        .ok_or_else(|| anyhow!("last_updated column missing or wrong type"))?;
    let heading_path = decode_string_list(batch, "heading_path")?;
    let tags = decode_string_list(batch, "tags")?;
    let applies_to = decode_string_list(batch, "applies_to")?;
    let score = batch
        .column_by_name("_relevance_score")
        .or_else(|| batch.column_by_name("relevance_score"))
        .and_then(|c| c.as_any().downcast_ref::<arrow_array::Float32Array>())
        .map(|a| a.values().to_vec())
        .unwrap_or_default();
    let mut out = Vec::with_capacity(batch.num_rows());
    for i in 0..batch.num_rows() {
        let micros = last_updated.value(i);
        let last_updated_dt = Utc
            .timestamp_micros(micros)
            .single()
            .unwrap_or_else(Utc::now);
        let chunk = Chunk {
            id: id.value(i).to_string(),
            doc_path: doc_path.value(i).to_string(),
            heading_path: heading_path.get(i).cloned().unwrap_or_default(),
            tags: tags.get(i).cloned().unwrap_or_default(),
            applies_to: applies_to.get(i).cloned().unwrap_or_default(),
            last_updated: last_updated_dt,
            content_hash: content_hash.value(i).to_string(),
            source_kind: source_kind.value(i).to_string(),
            body: body.value(i).to_string(),
        };
        let s = score.get(i).copied().unwrap_or(0.0);
        out.push(SearchHit {
            chunk,
            score: s,
            vector_rank: None,
            lexical_rank: None,
        });
    }
    Ok(out)
}

fn decode_string_list(batch: &RecordBatch, col: &str) -> Result<Vec<Vec<String>>> {
    let array = batch
        .column_by_name(col)
        .ok_or_else(|| anyhow!("column {col} missing"))?;
    let list = array
        .as_any()
        .downcast_ref::<arrow_array::ListArray>()
        .ok_or_else(|| anyhow!("column {col} not a ListArray"))?;
    let mut out = Vec::with_capacity(list.len());
    for i in 0..list.len() {
        let inner = list.value(i);
        let strs = inner
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow!("inner of {col} not Utf8"))?;
        let mut row = Vec::with_capacity(strs.len());
        for j in 0..strs.len() {
            row.push(strs.value(j).to_string());
        }
        out.push(row);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Chunk;
    use chrono::TimeZone;
    use tempfile::TempDir;

    fn make_chunk(id: &str, body: &str, doc_path: &str) -> Chunk {
        Chunk {
            id: id.to_string(),
            doc_path: doc_path.to_string(),
            heading_path: vec!["root".into()],
            tags: vec!["t1".into()],
            applies_to: vec!["poe1".into()],
            last_updated: Utc.with_ymd_and_hms(2026, 5, 8, 12, 0, 0).unwrap(),
            content_hash: format!("hash-of-{doc_path}"),
            source_kind: "reference".into(),
            body: body.to_string(),
        }
    }

    fn unit_vec(dim: usize, hot: usize) -> Vec<f32> {
        let mut v = vec![0.0_f32; dim];
        v[hot % dim] = 1.0;
        v
    }

    #[tokio::test]
    async fn open_or_create_initialises_empty_table() {
        let tmp = TempDir::new().unwrap();
        let idx = LanceIndex::open_or_create(tmp.path(), 8).await.unwrap();
        assert_eq!(idx.dim(), 8);
        let hashes = idx.doc_hashes().await.unwrap();
        assert!(hashes.is_empty());
    }

    #[tokio::test]
    async fn upsert_then_doc_hashes_returns_expected_pairs() {
        let tmp = TempDir::new().unwrap();
        let idx = LanceIndex::open_or_create(tmp.path(), 8).await.unwrap();
        let chunks = vec![
            make_chunk("a", "alpha body", "ref/a.md"),
            make_chunk("b", "beta body", "ref/b.md"),
        ];
        let embeddings = vec![unit_vec(8, 0), unit_vec(8, 1)];
        idx.upsert(chunks, embeddings).await.unwrap();
        let hashes = idx.doc_hashes().await.unwrap();
        assert_eq!(hashes.get("ref/a.md"), Some(&"hash-of-ref/a.md".to_string()));
        assert_eq!(hashes.get("ref/b.md"), Some(&"hash-of-ref/b.md".to_string()));
    }

    #[tokio::test]
    async fn delete_by_doc_path_removes_only_matching_rows() {
        let tmp = TempDir::new().unwrap();
        let idx = LanceIndex::open_or_create(tmp.path(), 8).await.unwrap();
        let chunks = vec![
            make_chunk("a", "alpha body", "ref/a.md"),
            make_chunk("b", "beta body", "ref/b.md"),
        ];
        let embeddings = vec![unit_vec(8, 0), unit_vec(8, 1)];
        idx.upsert(chunks, embeddings).await.unwrap();
        idx.delete_by_doc_path("ref/a.md").await.unwrap();
        let hashes = idx.doc_hashes().await.unwrap();
        assert!(hashes.get("ref/a.md").is_none());
        assert!(hashes.get("ref/b.md").is_some());
    }

    #[tokio::test]
    async fn upsert_dim_mismatch_errors() {
        let tmp = TempDir::new().unwrap();
        let idx = LanceIndex::open_or_create(tmp.path(), 8).await.unwrap();
        let chunks = vec![make_chunk("a", "x", "p.md")];
        let embeddings = vec![vec![0.0_f32; 4]]; // wrong dim
        let err = idx.upsert(chunks, embeddings).await.unwrap_err();
        assert!(err.to_string().contains("dim"));
    }

    #[test]
    fn build_filter_sql_combines_game_and_tags() {
        let sql = build_filter_sql(Some("poe2"), &["mapping".into(), "endgame".into()]).unwrap();
        assert!(sql.contains("array_has(applies_to, 'poe2')"));
        assert!(sql.contains("array_has(tags, 'mapping')"));
        assert!(sql.contains("array_has(tags, 'endgame')"));
    }

    #[test]
    fn build_filter_sql_returns_none_when_unfiltered() {
        assert!(build_filter_sql(None, &[]).is_none());
    }

    #[tokio::test]
    async fn search_hybrid_returns_relevant_chunk_after_fts_index() {
        let tmp = TempDir::new().unwrap();
        let idx = LanceIndex::open_or_create(tmp.path(), 8).await.unwrap();
        let chunks = vec![
            make_chunk("a", "spell suppression caps at 100 percent", "ref/suppress.md"),
            make_chunk("b", "resolute technique guarantees hits but disables crit", "ref/rt.md"),
            make_chunk("c", "voices cluster jewel adds small passives", "ref/voices.md"),
        ];
        // Embeddings: align "suppression" chunk with hot index 0 so a query
        // vector with hot[0] retrieves it via vector search even when FTS
        // misses.
        let embeddings = vec![unit_vec(8, 0), unit_vec(8, 1), unit_vec(8, 2)];
        idx.upsert(chunks, embeddings).await.unwrap();
        idx.ensure_fts_index().await.unwrap();
        let hits = idx
            .search_hybrid("suppression cap", unit_vec(8, 0), 3, None, &[])
            .await
            .unwrap();
        assert!(!hits.is_empty(), "hybrid search returned zero hits");
        assert_eq!(
            hits[0].chunk.doc_path, "ref/suppress.md",
            "expected suppression chunk first; got {hits:?}"
        );
    }
}

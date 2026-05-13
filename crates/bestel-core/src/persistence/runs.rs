//! `runs` table — mirror of `recorder::PersistedRun`.
//!
//! Columns are the indexable extracted facts; the full JSON is preserved in
//! `raw_json` so `recorder::get_run` (still JSON-first) and the SQLite
//! mirror can never drift. If a migration adds a new column we backfill from
//! `raw_json`.

use anyhow::{Context, Result};
use rusqlite::{params, OptionalExtension};

use super::Db;
use crate::llm::recorder::PersistedRun;

#[derive(Debug, Clone)]
pub struct RunRow {
    pub id: String,
    pub started_at: String,
    pub source: String,
    pub provider: String,
    pub model_id: String,
    pub final_text: String,
    pub verifier_status: Option<String>,
    pub verifier_findings_count: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct RunsFilter {
    pub provider_eq: Option<String>,
    pub model_id_like: Option<String>,
    pub verifier_status_eq: Option<String>,
    pub limit: Option<i64>,
}

pub fn insert_run(db: &Db, run: &PersistedRun) -> Result<()> {
    let raw = serde_json::to_vec(run).context("serialize run for raw_json")?;
    let verifier_status = run.stats.verifier_verdict.as_ref().map(|v| match v.status {
        crate::llm::verifier::VerdictStatus::Pass => "pass",
        crate::llm::verifier::VerdictStatus::Revise => "revise",
        crate::llm::verifier::VerdictStatus::Fail => "fail",
    });
    let verifier_count = run
        .stats
        .verifier_verdict
        .as_ref()
        .map(|v| v.findings.len() as i64);
    db.with_conn_rusqlite(|c| {
        c.execute(
            "INSERT OR REPLACE INTO runs (
                id, started_at, ended_at, source, provider, model_id, model_display_name,
                user_text, final_text, error,
                elapsed_ms, text_bytes, reasoning_bytes,
                tool_calls, tool_done, tool_failed,
                input_tokens, cached_input_tokens, cache_creation_tokens, output_tokens, cost_usd,
                verifier_status, verifier_findings_count, raw_json
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24)",
            params![
                run.id,
                run.started_at,
                run.ended_at,
                run.source,
                run.provider,
                run.model_id,
                run.model_display_name,
                run.user_text,
                run.final_text,
                run.error,
                run.stats.elapsed_ms as i64,
                run.stats.text_bytes as i64,
                run.stats.reasoning_bytes as i64,
                run.stats.tool_calls as i64,
                run.stats.tool_done as i64,
                run.stats.tool_failed as i64,
                run.stats.input_tokens as i64,
                run.stats.cached_input_tokens as i64,
                run.stats.cache_creation_tokens as i64,
                run.stats.output_tokens as i64,
                run.stats.cost_usd,
                verifier_status,
                verifier_count,
                raw,
            ],
        )?;
        Ok(())
    })?;
    if let Some(verdict) = run.stats.verifier_verdict.as_ref() {
        let findings_json =
            serde_json::to_string(&verdict.findings).context("serialize verifier findings")?;
        super::verifier_results::upsert_verifier_result(
            db,
            &super::verifier_results::VerifierResultRow {
                run_id: run.id.clone(),
                status: verifier_status.unwrap_or("pass").to_string(),
                findings_count: verifier_count.unwrap_or(0),
                findings_json,
                latency_ms: None,
            },
        )?;
    }
    Ok(())
}

pub fn query_runs(db: &Db, filter: &RunsFilter) -> Result<Vec<RunRow>> {
    let mut sql = String::from(
        "SELECT id, started_at, source, provider, model_id, final_text, \
         verifier_status, verifier_findings_count FROM runs WHERE 1=1",
    );
    let mut args: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(p) = filter.provider_eq.as_ref() {
        sql.push_str(" AND provider = ?");
        args.push(p.clone().into());
    }
    if let Some(m) = filter.model_id_like.as_ref() {
        sql.push_str(" AND model_id LIKE ?");
        args.push(m.clone().into());
    }
    if let Some(s) = filter.verifier_status_eq.as_ref() {
        sql.push_str(" AND verifier_status = ?");
        args.push(s.clone().into());
    }
    sql.push_str(" ORDER BY started_at DESC");
    if let Some(l) = filter.limit {
        sql.push_str(&format!(" LIMIT {l}"));
    }
    db.with_conn_rusqlite(|c| {
        let mut stmt = c.prepare(&sql)?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(args.iter()), |r| {
                Ok(RunRow {
                    id: r.get(0)?,
                    started_at: r.get(1)?,
                    source: r.get(2)?,
                    provider: r.get(3)?,
                    model_id: r.get(4)?,
                    final_text: r.get(5)?,
                    verifier_status: r.get(6)?,
                    verifier_findings_count: r.get(7)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    })
}

/// Sprint G2 exit-criterion query: "all Haiku runs that FAIL identity card".
/// Joins `runs` with `lint_findings` on the rule produced by Sprint A.
pub fn query_runs_failing_identity(db: &Db) -> Result<Vec<RunRow>> {
    db.with_conn_rusqlite(|c| {
        let mut stmt = c.prepare(
            "SELECT r.id, r.started_at, r.source, r.provider, r.model_id, r.final_text, \
                    r.verifier_status, r.verifier_findings_count \
             FROM runs r \
             JOIN lint_findings f ON f.run_id = r.id \
             WHERE r.model_id LIKE '%haiku%' \
               AND f.rule = 'BUILD_IDENTITY_REQUIRED' \
               AND f.severity = 'fail' \
             ORDER BY r.started_at DESC",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok(RunRow {
                    id: r.get(0)?,
                    started_at: r.get(1)?,
                    source: r.get(2)?,
                    provider: r.get(3)?,
                    model_id: r.get(4)?,
                    final_text: r.get(5)?,
                    verifier_status: r.get(6)?,
                    verifier_findings_count: r.get(7)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    })
}

#[allow(dead_code)]
pub fn get_run_raw(db: &Db, id: &str) -> Result<Option<PersistedRun>> {
    let raw: Option<Vec<u8>> = db.with_conn_rusqlite(|c| {
        c.query_row(
            "SELECT raw_json FROM runs WHERE id = ?1",
            params![id],
            |r| r.get::<_, Vec<u8>>(0),
        )
        .optional()
    })?;
    match raw {
        Some(bytes) => Ok(Some(
            serde_json::from_slice(&bytes).context("decode raw_json")?,
        )),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::recorder::ChatStats;

    fn fixture(id: &str, model: &str) -> PersistedRun {
        PersistedRun {
            id: id.to_string(),
            started_at: "2026-05-09T10:00:00Z".to_string(),
            ended_at: Some("2026-05-09T10:00:30Z".to_string()),
            source: "ui".to_string(),
            provider: "anthropic".to_string(),
            model_id: model.to_string(),
            model_display_name: format!("Anthropic · {model}"),
            user_text: "what is spell suppression cap?".to_string(),
            user_attachments: Vec::new(),
            assistant_segments: Vec::new(),
            final_text: "100% — see wiki.".to_string(),
            stats: ChatStats::default(),
            error: None,
        }
    }

    #[test]
    fn insert_then_query_roundtrips() {
        let db = Db::open_in_memory().unwrap();
        insert_run(&db, &fixture("r1", "claude-haiku-4-5")).unwrap();
        insert_run(&db, &fixture("r2", "claude-sonnet-4-6")).unwrap();
        let rows = query_runs(
            &db,
            &RunsFilter {
                provider_eq: Some("anthropic".into()),
                model_id_like: Some("%haiku%".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "r1");
    }

    #[test]
    fn insert_or_replace_overwrites() {
        let db = Db::open_in_memory().unwrap();
        insert_run(&db, &fixture("r1", "claude-haiku-4-5")).unwrap();
        let mut updated = fixture("r1", "claude-haiku-4-5");
        updated.final_text = "rewritten".to_string();
        insert_run(&db, &updated).unwrap();
        let row = get_run_raw(&db, "r1").unwrap().unwrap();
        assert_eq!(row.final_text, "rewritten");
    }

    #[test]
    fn raw_json_roundtrips() {
        let db = Db::open_in_memory().unwrap();
        let r = fixture("r1", "claude-haiku-4-5");
        insert_run(&db, &r).unwrap();
        let recovered = get_run_raw(&db, "r1").unwrap().unwrap();
        assert_eq!(recovered.user_text, r.user_text);
        assert_eq!(recovered.final_text, r.final_text);
    }
}

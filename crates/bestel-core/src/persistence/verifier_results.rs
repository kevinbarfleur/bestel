//! `verifier_results` table — Sprint G verifier verdicts surfaced
//! per-run for queryable counts (e.g. "how often does Haiku trigger
//! revise?").

use anyhow::Result;
use rusqlite::params;

use super::Db;

#[derive(Debug, Clone)]
pub struct VerifierResultRow {
    pub run_id: String,
    pub status: String,
    pub findings_count: i64,
    pub findings_json: String,
    pub latency_ms: Option<i64>,
}

pub fn upsert_verifier_result(db: &Db, row: &VerifierResultRow) -> Result<()> {
    db.with_conn_rusqlite(|c| {
        c.execute(
            "INSERT INTO verifier_results (run_id, status, findings_count, findings_json, latency_ms) \
             VALUES (?1, ?2, ?3, ?4, ?5) \
             ON CONFLICT(run_id) DO UPDATE SET \
                 status = excluded.status, \
                 findings_count = excluded.findings_count, \
                 findings_json = excluded.findings_json, \
                 latency_ms = excluded.latency_ms",
            params![
                row.run_id,
                row.status,
                row.findings_count,
                row.findings_json,
                row.latency_ms,
            ],
        )?;
        Ok(())
    })
}

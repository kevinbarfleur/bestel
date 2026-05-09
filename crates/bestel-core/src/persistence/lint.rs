//! `lint_findings` table — one row per Sprint A linter failure attached to
//! a run. Sidecar JSON files (`<run_id>.lint.json`) remain authoritative;
//! this mirror exists so `query_runs_failing_identity` can JOIN.

use anyhow::Result;
use rusqlite::params;

use super::Db;

#[derive(Debug, Clone)]
pub struct LintFindingRow {
    pub run_id: String,
    pub rule: String,
    pub severity: String,
    pub message: String,
}

pub fn insert_lint_finding(db: &Db, row: &LintFindingRow) -> Result<()> {
    db.with_conn_rusqlite(|c| {
        c.execute(
            "INSERT INTO lint_findings (run_id, rule, severity, message) \
             VALUES (?1, ?2, ?3, ?4)",
            params![row.run_id, row.rule, row.severity, row.message],
        )?;
        Ok(())
    })
}

/// Replace all lint findings for `run_id` and re-insert from the report.
/// Idempotent — re-running the linter on the same run leaves the SQLite
/// table in sync with the latest sidecar JSON.
pub fn replace_lint_findings(
    db: &Db,
    run_id: &str,
    rows: &[LintFindingRow],
) -> Result<()> {
    db.with_conn_rusqlite(|c| {
        let tx = c.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM lint_findings WHERE run_id = ?1",
            params![run_id],
        )?;
        for row in rows {
            tx.execute(
                "INSERT INTO lint_findings (run_id, rule, severity, message) \
                 VALUES (?1, ?2, ?3, ?4)",
                params![row.run_id, row.rule, row.severity, row.message],
            )?;
        }
        tx.commit()?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::recorder::{ChatStats, PersistedRun};
    use crate::persistence::runs::insert_run;

    #[test]
    fn fk_links_finding_to_run() {
        let db = Db::open_in_memory().unwrap();
        let run = PersistedRun {
            id: "r1".into(),
            started_at: "2026-05-09T10:00:00Z".into(),
            ended_at: None,
            source: "ui".into(),
            provider: "anthropic".into(),
            model_id: "claude-haiku-4-5".into(),
            model_display_name: "Claude Haiku 4.5".into(),
            user_text: "build review".into(),
            user_attachments: Vec::new(),
            assistant_segments: Vec::new(),
            final_text: "draft without identity card".into(),
            stats: ChatStats::default(),
            error: None,
        };
        insert_run(&db, &run).unwrap();
        insert_lint_finding(
            &db,
            &LintFindingRow {
                run_id: "r1".into(),
                rule: "BUILD_IDENTITY_REQUIRED".into(),
                severity: "fail".into(),
                message: "missing Identity: line".into(),
            },
        )
        .unwrap();
        let count: i64 = db
            .with_conn_rusqlite(|c| {
                c.query_row("SELECT COUNT(*) FROM lint_findings", [], |r| r.get(0))
            })
            .unwrap();
        assert_eq!(count, 1);
    }
}

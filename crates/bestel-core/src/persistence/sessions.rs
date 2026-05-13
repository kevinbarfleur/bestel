//! `sessions` table — mirror of `session_notes::SessionNotes`.

use anyhow::{Context, Result};
use rusqlite::{params, OptionalExtension};

use super::Db;
use crate::llm::session_notes::SessionNotes;

#[derive(Debug, Clone)]
pub struct SessionRow {
    pub session_id: String,
    pub build_id: Option<String>,
    pub last_updated: String,
    pub current_diagnosis_json: String,
}

pub fn upsert_session(db: &Db, notes: &SessionNotes) -> Result<()> {
    let diag =
        serde_json::to_string(&notes.current_diagnosis).context("serialize current_diagnosis")?;
    let last_updated = notes
        .last_updated
        .map(|t| t.to_rfc3339())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    db.with_conn_rusqlite(|c| {
        c.execute(
            "INSERT INTO sessions (session_id, build_id, last_updated, current_diagnosis_json) \
             VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT(session_id) DO UPDATE SET \
                 build_id = excluded.build_id, \
                 last_updated = excluded.last_updated, \
                 current_diagnosis_json = excluded.current_diagnosis_json",
            params![notes.session_id, notes.build_id, last_updated, diag,],
        )?;
        Ok(())
    })
}

pub fn read_session_row(db: &Db, session_id: &str) -> Result<Option<SessionRow>> {
    db.with_conn_rusqlite(|c| {
        c.query_row(
            "SELECT session_id, build_id, last_updated, current_diagnosis_json \
             FROM sessions WHERE session_id = ?1",
            params![session_id],
            |r| {
                Ok(SessionRow {
                    session_id: r.get(0)?,
                    build_id: r.get(1)?,
                    last_updated: r.get(2)?,
                    current_diagnosis_json: r.get(3)?,
                })
            },
        )
        .optional()
        .map_err(rusqlite::Error::from)
    })
    .map_err(anyhow::Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::session_notes::{CurrentDiagnosis, SessionNotes};

    fn fixture(id: &str, build: &str) -> SessionNotes {
        SessionNotes {
            session_id: id.to_string(),
            build_id: Some(build.to_string()),
            last_updated: Some(chrono::Utc::now()),
            current_diagnosis: Some(CurrentDiagnosis {
                primary_bottleneck: "low spell suppression".into(),
                evidence: vec!["35% suppression on chest".into()],
                chosen_fix: "craft Saviour-tier base".into(),
                constraints: vec!["budget 5 div".into()],
            }),
        }
    }

    #[test]
    fn upsert_inserts_new_then_overwrites() {
        let db = Db::open_in_memory().unwrap();
        upsert_session(&db, &fixture("s1", "buildA")).unwrap();
        let row = read_session_row(&db, "s1").unwrap().unwrap();
        assert_eq!(row.build_id.as_deref(), Some("buildA"));

        upsert_session(&db, &fixture("s1", "buildB")).unwrap();
        let row2 = read_session_row(&db, "s1").unwrap().unwrap();
        assert_eq!(row2.build_id.as_deref(), Some("buildB"));
    }
}

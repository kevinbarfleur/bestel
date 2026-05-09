//! SQLite persistence for Build Sheets.
//!
//! The mirror table is `build_sheets` (see `persistence::migrations` v2).
//! Reads return raw rows — callers parse `payload` into [`super::BuildSheet`]
//! themselves so the parser can stay version-aware (sheets authored against
//! older `schema_version`s may need a migration step at read time).

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::persistence::Db;

use super::types::BuildSheet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetRow {
    pub id: String,
    pub fingerprint: String,
    pub pob_hash: String,
    pub name: String,
    pub schema_version: i64,
    pub payload: String,
    pub authored_at: String,
    pub updated_at: String,
    pub authored_in_chat: Option<String>,
    pub validated: bool,
}

impl SheetRow {
    pub fn parse_payload(&self) -> Result<BuildSheet> {
        serde_json::from_str(&self.payload).context("parse build_sheets.payload")
    }
}

/// Generate a sheet id from the current epoch (nanos — high enough resolution
/// that back-to-back inserts in the same fingerprint don't collide) plus a
/// short fingerprint head for human readability. The id is opaque and used
/// only for joins / deletes — chats reference it by fingerprint when
/// reattaching after a PoB swap.
fn sheet_id(fingerprint: &str) -> String {
    let now = Utc::now()
        .timestamp_nanos_opt()
        .unwrap_or_else(|| Utc::now().timestamp_micros());
    let head: String = fingerprint
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(12)
        .collect();
    format!("sheet-{now}-{head}")
}

/// Insert a brand-new sheet, returning the assigned id. Caller-side: build
/// the `BuildSheet`, JSON-serialize it into `payload`, then call this.
#[allow(clippy::too_many_arguments)]
pub fn insert_sheet(
    db: &Db,
    fingerprint: &str,
    pob_hash: &str,
    name: &str,
    payload: &BuildSheet,
    authored_in_chat: Option<&str>,
    validated: bool,
) -> Result<String> {
    let id = sheet_id(fingerprint);
    let now = Utc::now().to_rfc3339();
    let payload_json = serde_json::to_string(payload).context("serialize sheet payload")?;
    let schema_version = payload.schema_version.0 as i64;
    db.with_conn_rusqlite(|c| {
        c.execute(
            "INSERT INTO build_sheets (
                id, fingerprint, pob_hash, name, schema_version, payload,
                authored_at, updated_at, authored_in_chat, validated
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8, ?9)",
            params![
                id,
                fingerprint,
                pob_hash,
                name,
                schema_version,
                payload_json,
                now,
                authored_in_chat,
                validated as i64,
            ],
        )?;
        Ok(())
    })?;
    Ok(id)
}

/// Update an existing sheet's payload and hash (e.g. after a refresh
/// mini-interview rewrites a few sections). `validated=true` flips the row
/// out of "drafting" once the user confirms the last section.
pub fn update_sheet(
    db: &Db,
    id: &str,
    pob_hash: &str,
    payload: &BuildSheet,
    validated: bool,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let payload_json = serde_json::to_string(payload).context("serialize sheet payload")?;
    let schema_version = payload.schema_version.0 as i64;
    db.with_conn_rusqlite(|c| {
        c.execute(
            "UPDATE build_sheets
             SET pob_hash = ?2,
                 schema_version = ?3,
                 payload = ?4,
                 updated_at = ?5,
                 validated = ?6
             WHERE id = ?1",
            params![
                id,
                pob_hash,
                schema_version,
                payload_json,
                now,
                validated as i64,
            ],
        )?;
        Ok(())
    })?;
    Ok(())
}

/// Find the most-recently updated validated sheet for a given fingerprint.
/// Returns `None` if no validated sheet exists yet.
pub fn find_by_fingerprint(db: &Db, fingerprint: &str) -> Result<Option<SheetRow>> {
    db.with_conn_rusqlite(|c| {
        c.query_row(
            "SELECT id, fingerprint, pob_hash, name, schema_version, payload,
                    authored_at, updated_at, authored_in_chat, validated
             FROM build_sheets
             WHERE fingerprint = ?1 AND validated = 1
             ORDER BY updated_at DESC
             LIMIT 1",
            params![fingerprint],
            row_to_sheet,
        )
        .optional()
    })
}

/// List every sheet in the table (drafts + validated). Sorted by most-
/// recent `updated_at` first so the picker shows fresh authorings on top.
pub fn list_all_sheets(db: &Db) -> Result<Vec<SheetRow>> {
    db.with_conn_rusqlite(|c| {
        let mut stmt = c.prepare(
            "SELECT id, fingerprint, pob_hash, name, schema_version, payload,
                    authored_at, updated_at, authored_in_chat, validated
             FROM build_sheets
             ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map(params![], row_to_sheet)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    })
}

/// Delete a sheet by id. Returns `true` if a row was actually deleted,
/// `false` if no row matched the id (caller may already have refreshed).
pub fn delete_sheet(db: &Db, id: &str) -> Result<bool> {
    db.with_conn_rusqlite(|c| {
        let n = c.execute("DELETE FROM build_sheets WHERE id = ?1", params![id])?;
        Ok(n > 0)
    })
}

/// Read a single sheet by id.
pub fn get_by_id(db: &Db, id: &str) -> Result<Option<SheetRow>> {
    db.with_conn_rusqlite(|c| {
        c.query_row(
            "SELECT id, fingerprint, pob_hash, name, schema_version, payload,
                    authored_at, updated_at, authored_in_chat, validated
             FROM build_sheets
             WHERE id = ?1",
            params![id],
            row_to_sheet,
        )
        .optional()
    })
}

fn row_to_sheet(row: &rusqlite::Row) -> rusqlite::Result<SheetRow> {
    Ok(SheetRow {
        id: row.get(0)?,
        fingerprint: row.get(1)?,
        pob_hash: row.get(2)?,
        name: row.get(3)?,
        schema_version: row.get(4)?,
        payload: row.get(5)?,
        authored_at: row.get(6)?,
        updated_at: row.get(7)?,
        authored_in_chat: row.get(8)?,
        validated: row.get::<_, i64>(9)? != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sheets::types::BuildSheet;

    fn empty_sheet() -> BuildSheet {
        BuildSheet::with_default_sections()
    }

    #[test]
    fn insert_then_read_round_trip() {
        let db = Db::open_in_memory().expect("open mem db");
        let payload = empty_sheet();
        let id = insert_sheet(
            &db,
            "inquisitor:ice nova:brass dome+cospri's will",
            "abc123",
            "Spell Crit · Cold Conversion",
            &payload,
            Some("chat-1"),
            true,
        )
        .expect("insert");
        let row = get_by_id(&db, &id).expect("get").expect("some");
        assert_eq!(row.name, "Spell Crit · Cold Conversion");
        assert!(row.validated);
        assert_eq!(row.schema_version, 1);
        let parsed = row.parse_payload().expect("parse payload");
        assert_eq!(parsed.sections.len(), 6);
    }

    #[test]
    fn find_by_fingerprint_skips_unvalidated() {
        let db = Db::open_in_memory().expect("open mem db");
        let payload = empty_sheet();
        let _draft = insert_sheet(
            &db,
            "fp1",
            "hash-draft",
            "Draft sheet",
            &payload,
            None,
            false,
        )
        .expect("insert draft");
        // No validated row → returns None.
        assert!(find_by_fingerprint(&db, "fp1").expect("query").is_none());

        let _validated = insert_sheet(
            &db,
            "fp1",
            "hash-validated",
            "Validated sheet",
            &payload,
            None,
            true,
        )
        .expect("insert validated");
        let hit = find_by_fingerprint(&db, "fp1")
            .expect("query")
            .expect("found");
        assert_eq!(hit.name, "Validated sheet");
    }

    #[test]
    fn update_sheet_marks_validated() {
        let db = Db::open_in_memory().expect("open mem db");
        let payload = empty_sheet();
        let id = insert_sheet(&db, "fp2", "h0", "wip", &payload, None, false).expect("insert");
        update_sheet(&db, &id, "h1", &payload, true).expect("update");
        let row = get_by_id(&db, &id).expect("get").expect("some");
        assert!(row.validated);
        assert_eq!(row.pob_hash, "h1");
    }
}

//! CRUD over `build_registry` + `suggestion_dismissals`.

use anyhow::{Context, Result};
use rusqlite::{params, OptionalExtension};

use crate::persistence::Db;

use super::types::{RegistryEntry, RegistrySummary};

/// Insert a new registry entry. Returns the assigned row id. Caller is
/// responsible for computing the 5 signatures + canonical pob_hash + the
/// summary blob before calling this — keeps the store thin and lets the
/// IPC layer reuse the same canonical builder used by sheets.
#[allow(clippy::too_many_arguments)]
pub fn insert_entry(
    db: &Db,
    display_name: &str,
    game: &str,
    pob_path: &str,
    pob_hash: &str,
    identity_sig: &str,
    gear_sig: &str,
    tree_sig: &str,
    skill_sig: &str,
    config_sig: &str,
    linked_sheet_id: Option<&str>,
    summary: &RegistrySummary,
    now_ms: i64,
) -> Result<i64> {
    let summary_json =
        serde_json::to_string(summary).context("serialize registry summary")?;
    db.with_conn_rusqlite(|c| {
        c.execute(
            "INSERT INTO build_registry (
                display_name, game, pob_path, pob_hash,
                identity_sig, gear_sig, tree_sig, skill_sig, config_sig,
                linked_sheet_id, summary_json, last_seen_at, authored_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)",
            params![
                display_name,
                game,
                pob_path,
                pob_hash,
                identity_sig,
                gear_sig,
                tree_sig,
                skill_sig,
                config_sig,
                linked_sheet_id,
                summary_json,
                now_ms,
            ],
        )?;
        Ok(c.last_insert_rowid())
    })
}

/// Replace every signature + the pob_hash + summary for an existing entry,
/// bump `last_seen_at`. Called by `registry_refresh` when the user clicks
/// "Refresh from PoB file" and we re-parse the XML from disk.
pub fn update_entry_signatures(
    db: &Db,
    id: i64,
    pob_hash: &str,
    identity_sig: &str,
    gear_sig: &str,
    tree_sig: &str,
    skill_sig: &str,
    config_sig: &str,
    summary: &RegistrySummary,
    now_ms: i64,
) -> Result<()> {
    let summary_json =
        serde_json::to_string(summary).context("serialize registry summary")?;
    db.with_conn_rusqlite(|c| {
        c.execute(
            "UPDATE build_registry
             SET pob_hash = ?2,
                 identity_sig = ?3,
                 gear_sig = ?4,
                 tree_sig = ?5,
                 skill_sig = ?6,
                 config_sig = ?7,
                 summary_json = ?8,
                 last_seen_at = ?9
             WHERE id = ?1",
            params![
                id,
                pob_hash,
                identity_sig,
                gear_sig,
                tree_sig,
                skill_sig,
                config_sig,
                summary_json,
                now_ms,
            ],
        )?;
        Ok(())
    })
}

pub fn delete_entry(db: &Db, id: i64) -> Result<bool> {
    db.with_conn_rusqlite(|c| {
        let n = c.execute("DELETE FROM build_registry WHERE id = ?1", params![id])?;
        Ok(n > 0)
    })
}

pub fn list_entries(db: &Db) -> Result<Vec<RegistryEntry>> {
    db.with_conn_rusqlite(|c| {
        let mut stmt = c.prepare(
            "SELECT id, display_name, game, pob_path, pob_hash,
                    identity_sig, gear_sig, tree_sig, skill_sig, config_sig,
                    linked_sheet_id, summary_json, last_seen_at, authored_at
             FROM build_registry
             ORDER BY last_seen_at DESC",
        )?;
        let rows = stmt.query_map(params![], row_to_entry)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    })
}

pub fn get_entry(db: &Db, id: i64) -> Result<Option<RegistryEntry>> {
    db.with_conn_rusqlite(|c| {
        c.query_row(
            "SELECT id, display_name, game, pob_path, pob_hash,
                    identity_sig, gear_sig, tree_sig, skill_sig, config_sig,
                    linked_sheet_id, summary_json, last_seen_at, authored_at
             FROM build_registry
             WHERE id = ?1",
            params![id],
            row_to_entry,
        )
        .optional()
    })
}

pub fn get_pob_path(db: &Db, id: i64) -> Result<Option<String>> {
    db.with_conn_rusqlite(|c| {
        c.query_row(
            "SELECT pob_path FROM build_registry WHERE id = ?1",
            params![id],
            |r| r.get::<_, String>(0),
        )
        .optional()
    })
}

/// Bumps `last_seen_at`. Called when a chat selects a registry entry from
/// the dropdown so "recently used" rises in the picker order.
pub fn touch_last_seen(db: &Db, id: i64, now_ms: i64) -> Result<()> {
    db.with_conn_rusqlite(|c| {
        c.execute(
            "UPDATE build_registry SET last_seen_at = ?2 WHERE id = ?1",
            params![id, now_ms],
        )?;
        Ok(())
    })
}

/// Set the `dismissed_until` timestamp for a build's Path A suggestion
/// card. Re-dismissing extends the cooldown — last write wins.
pub fn suggestion_dismiss(db: &Db, pob_hash: &str, dismissed_until_ms: i64) -> Result<()> {
    db.with_conn_rusqlite(|c| {
        c.execute(
            "INSERT INTO suggestion_dismissals (build_pob_hash, dismissed_until)
             VALUES (?1, ?2)
             ON CONFLICT(build_pob_hash) DO UPDATE SET dismissed_until = excluded.dismissed_until",
            params![pob_hash, dismissed_until_ms],
        )?;
        Ok(())
    })
}

/// Returns `true` when the suggestion is currently dismissed.
pub fn suggestion_check(db: &Db, pob_hash: &str, now_ms: i64) -> Result<bool> {
    db.with_conn_rusqlite(|c| {
        let until: Option<i64> = c
            .query_row(
                "SELECT dismissed_until FROM suggestion_dismissals WHERE build_pob_hash = ?1",
                params![pob_hash],
                |r| r.get(0),
            )
            .optional()?;
        Ok(until.map(|u| u > now_ms).unwrap_or(false))
    })
}

fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<RegistryEntry> {
    let summary_json: String = row.get(11)?;
    let summary: RegistrySummary = serde_json::from_str(&summary_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            11,
            rusqlite::types::Type::Text,
            Box::new(e),
        )
    })?;
    Ok(RegistryEntry {
        id: row.get(0)?,
        display_name: row.get(1)?,
        game: row.get(2)?,
        pob_path: row.get(3)?,
        pob_hash: row.get(4)?,
        identity_sig: row.get(5)?,
        gear_sig: row.get(6)?,
        tree_sig: row.get(7)?,
        skill_sig: row.get(8)?,
        config_sig: row.get(9)?,
        linked_sheet_id: row.get(10)?,
        summary,
        last_seen_at: row.get(12)?,
        authored_at: row.get(13)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_summary() -> RegistrySummary {
        RegistrySummary {
            class: "Templar".to_string(),
            ascendancy: Some("Inquisitor".to_string()),
            level: Some(94),
            main_skill: Some("Penance Brand".to_string()),
            defining_uniques: vec!["Brass Dome".to_string(), "Cospri's Will".to_string()],
        }
    }

    fn seed(db: &Db, name: &str, now_ms: i64) -> i64 {
        insert_entry(
            db,
            name,
            "poe1",
            &format!("/builds/{name}.xml"),
            "pob_hash",
            "id_sig",
            "gear_sig",
            "tree_sig",
            "skill_sig",
            "config_sig",
            None,
            &fixture_summary(),
            now_ms,
        )
        .expect("insert ok")
    }

    #[test]
    fn insert_then_list_returns_entry() {
        let db = Db::open_in_memory().expect("mem db");
        let id = seed(&db, "Penance Inq", 1_000);
        let list = list_entries(&db).expect("list ok");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
        assert_eq!(list[0].display_name, "Penance Inq");
        assert_eq!(list[0].summary.class, "Templar");
    }

    #[test]
    fn list_ordered_by_last_seen_desc() {
        let db = Db::open_in_memory().expect("mem db");
        let a = seed(&db, "old", 1_000);
        let b = seed(&db, "new", 2_000);
        let list = list_entries(&db).expect("list ok");
        assert_eq!(list[0].id, b);
        assert_eq!(list[1].id, a);
    }

    #[test]
    fn touch_bumps_ordering() {
        let db = Db::open_in_memory().expect("mem db");
        let a = seed(&db, "a", 1_000);
        let _b = seed(&db, "b", 2_000);
        touch_last_seen(&db, a, 3_000).expect("touch ok");
        let list = list_entries(&db).expect("list ok");
        assert_eq!(list[0].id, a);
    }

    #[test]
    fn delete_removes_entry() {
        let db = Db::open_in_memory().expect("mem db");
        let id = seed(&db, "x", 1_000);
        assert!(delete_entry(&db, id).expect("del ok"));
        assert!(list_entries(&db).expect("list ok").is_empty());
        assert!(!delete_entry(&db, id).expect("del ok"));
    }

    #[test]
    fn suggestion_round_trip() {
        let db = Db::open_in_memory().expect("mem db");
        let now = 1_000_000;
        assert!(!suggestion_check(&db, "h1", now).expect("check"));
        suggestion_dismiss(&db, "h1", now + 86_400_000).expect("dismiss");
        assert!(suggestion_check(&db, "h1", now).expect("check"));
        assert!(!suggestion_check(&db, "h1", now + 86_400_001).expect("check"));
    }

    #[test]
    fn suggestion_redismiss_extends() {
        let db = Db::open_in_memory().expect("mem db");
        suggestion_dismiss(&db, "h1", 2_000).expect("dismiss");
        suggestion_dismiss(&db, "h1", 5_000).expect("redismiss");
        assert!(suggestion_check(&db, "h1", 3_000).expect("check"));
        assert!(suggestion_check(&db, "h1", 4_999).expect("check"));
        assert!(!suggestion_check(&db, "h1", 5_001).expect("check"));
    }
}

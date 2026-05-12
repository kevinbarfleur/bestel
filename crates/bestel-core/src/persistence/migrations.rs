//! Schema migration runner driven by `PRAGMA user_version`.
//!
//! Each entry in `MIGRATIONS` is a `(version, sql)` pair. The runner applies
//! every migration with version > current `user_version` in order, inside
//! one transaction per migration so a partial failure leaves the DB at a
//! known version. New schema changes append a new entry and bump
//! `LATEST_VERSION` — never edit a shipped migration in place.

use rusqlite::{params, Connection};

#[allow(dead_code)]
pub const LATEST_VERSION: i64 = 3;

const SCHEMA_V1: &str = r#"
CREATE TABLE runs (
    id                       TEXT PRIMARY KEY,
    started_at               TEXT NOT NULL,
    ended_at                 TEXT,
    source                   TEXT NOT NULL,
    provider                 TEXT NOT NULL,
    model_id                 TEXT NOT NULL,
    model_display_name       TEXT NOT NULL,
    user_text                TEXT NOT NULL,
    final_text               TEXT NOT NULL,
    error                    TEXT,
    elapsed_ms               INTEGER NOT NULL DEFAULT 0,
    text_bytes               INTEGER NOT NULL DEFAULT 0,
    reasoning_bytes          INTEGER NOT NULL DEFAULT 0,
    tool_calls               INTEGER NOT NULL DEFAULT 0,
    tool_done                INTEGER NOT NULL DEFAULT 0,
    tool_failed              INTEGER NOT NULL DEFAULT 0,
    input_tokens             INTEGER NOT NULL DEFAULT 0,
    cached_input_tokens      INTEGER NOT NULL DEFAULT 0,
    cache_creation_tokens    INTEGER NOT NULL DEFAULT 0,
    output_tokens            INTEGER NOT NULL DEFAULT 0,
    cost_usd                 REAL,
    verifier_status          TEXT,
    verifier_findings_count  INTEGER,
    raw_json                 BLOB NOT NULL
);
CREATE INDEX runs_provider_model ON runs(provider, model_id);
CREATE INDEX runs_started_at     ON runs(started_at);
CREATE INDEX runs_verifier       ON runs(verifier_status);

CREATE TABLE lint_findings (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id    TEXT NOT NULL,
    rule      TEXT NOT NULL,
    severity  TEXT NOT NULL,
    message   TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES runs(id) ON DELETE CASCADE
);
CREATE INDEX lint_findings_run ON lint_findings(run_id, rule);

CREATE TABLE verifier_results (
    run_id           TEXT PRIMARY KEY,
    status           TEXT NOT NULL,
    findings_count   INTEGER NOT NULL DEFAULT 0,
    findings_json    TEXT NOT NULL DEFAULT '[]',
    latency_ms       INTEGER,
    FOREIGN KEY (run_id) REFERENCES runs(id) ON DELETE CASCADE
);

CREATE TABLE sessions (
    session_id              TEXT PRIMARY KEY,
    build_id                TEXT,
    last_updated            TEXT NOT NULL,
    current_diagnosis_json  TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE builds_snapshot (
    build_id      TEXT PRIMARY KEY,
    source_path   TEXT NOT NULL,
    captured_at   TEXT NOT NULL,
    snapshot_json TEXT NOT NULL
);

CREATE TABLE kb_versions (
    doc_path        TEXT PRIMARY KEY,
    content_hash    TEXT NOT NULL,
    indexed_at      TEXT NOT NULL,
    dim             INTEGER NOT NULL,
    embedder_label  TEXT NOT NULL
);
"#;

/// v2 — Build Sheets feature (co-authored, validated build dossier).
///
/// `fingerprint` is the lookup key on PoB attach: ascendancy + main skill +
/// sorted defining uniques (fingerprint stays stable across gear churn).
/// `pob_hash` is sha256 of canonical PoB JSON — a hash mismatch with the same
/// fingerprint surfaces the "stale" state where Bestel proposes a refresh.
/// `payload` is the full sheet JSON (sections, intent, known_gaps, ...).
const SCHEMA_V2: &str = r#"
CREATE TABLE build_sheets (
    id                  TEXT PRIMARY KEY,
    fingerprint         TEXT NOT NULL,
    pob_hash            TEXT NOT NULL,
    name                TEXT NOT NULL,
    schema_version      INTEGER NOT NULL DEFAULT 1,
    payload             TEXT NOT NULL,
    authored_at         TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    authored_in_chat    TEXT,
    validated           INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX build_sheets_fingerprint ON build_sheets(fingerprint);
CREATE INDEX build_sheets_validated   ON build_sheets(validated);
"#;

/// v3 — Sprint v3 (registry + drift indicator).
///
/// Adds five orthogonal signature columns to `build_sheets` so the sheet
/// sidebar can render WHICH axis of the build drifted, not just the
/// binary `pob_hash_match`. Existing rows get NULL until the next time
/// they're loaded with a live PoB attached — `get_active_build_sheet_for_ui`
/// backfills them.
///
/// `build_registry` persists user-selected PoBs across sessions so a chat
/// can pick one from the dropdown without re-importing the XML. The 5
/// signatures are stored per entry so picker rows render a `DriftChipStrip`
/// at a glance.
///
/// `suggestion_dismissals` tracks per-build "hide for N days" state for
/// the Path A soft suggestion card.
const SCHEMA_V3: &str = r#"
ALTER TABLE build_sheets ADD COLUMN identity_sig TEXT;
ALTER TABLE build_sheets ADD COLUMN gear_sig     TEXT;
ALTER TABLE build_sheets ADD COLUMN tree_sig     TEXT;
ALTER TABLE build_sheets ADD COLUMN skill_sig    TEXT;
ALTER TABLE build_sheets ADD COLUMN config_sig   TEXT;

CREATE TABLE build_registry (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    display_name       TEXT NOT NULL,
    game               TEXT NOT NULL,
    pob_path           TEXT NOT NULL UNIQUE,
    pob_hash           TEXT NOT NULL,
    identity_sig       TEXT NOT NULL,
    gear_sig           TEXT NOT NULL,
    tree_sig           TEXT NOT NULL,
    skill_sig          TEXT NOT NULL,
    config_sig         TEXT NOT NULL,
    linked_sheet_id    TEXT REFERENCES build_sheets(id) ON DELETE SET NULL,
    summary_json       TEXT NOT NULL,
    last_seen_at       INTEGER NOT NULL,
    authored_at        INTEGER NOT NULL
);
CREATE INDEX build_registry_identity   ON build_registry(identity_sig);
CREATE INDEX build_registry_last_seen  ON build_registry(last_seen_at DESC);

CREATE TABLE suggestion_dismissals (
    build_pob_hash    TEXT PRIMARY KEY,
    dismissed_until   INTEGER NOT NULL
);
"#;

const MIGRATIONS: &[(i64, &str)] = &[(1, SCHEMA_V1), (2, SCHEMA_V2), (3, SCHEMA_V3)];

pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let current: i64 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap_or(0);
    for &(version, sql) in MIGRATIONS {
        if version <= current {
            continue;
        }
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(&format!("PRAGMA user_version = {version}"), params![])?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_db_lands_at_latest_version() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let v: i64 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(v, LATEST_VERSION);
    }

    #[test]
    fn second_migrate_is_noop() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
        let v: i64 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(v, LATEST_VERSION);
    }

    #[test]
    fn v3_creates_registry_and_dismissal_tables() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        // build_registry exists with all signature columns
        let registry_cols: Vec<String> = conn
            .prepare("PRAGMA table_info(build_registry)")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .map(|c| c.unwrap())
            .collect();
        for col in [
            "id", "display_name", "game", "pob_path", "pob_hash",
            "identity_sig", "gear_sig", "tree_sig", "skill_sig", "config_sig",
            "linked_sheet_id", "summary_json", "last_seen_at", "authored_at",
        ] {
            assert!(
                registry_cols.iter().any(|c| c == col),
                "build_registry missing column {col}"
            );
        }
        // suggestion_dismissals exists
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='suggestion_dismissals'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
        // build_sheets has the 5 signature columns
        let sheet_cols: Vec<String> = conn
            .prepare("PRAGMA table_info(build_sheets)")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .map(|c| c.unwrap())
            .collect();
        for col in ["identity_sig", "gear_sig", "tree_sig", "skill_sig", "config_sig"] {
            assert!(
                sheet_cols.iter().any(|c| c == col),
                "build_sheets missing column {col}"
            );
        }
    }
}

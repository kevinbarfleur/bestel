//! Schema migration runner driven by `PRAGMA user_version`.
//!
//! Each entry in `MIGRATIONS` is a `(version, sql)` pair. The runner applies
//! every migration with version > current `user_version` in order, inside
//! one transaction per migration so a partial failure leaves the DB at a
//! known version. New schema changes append a new entry and bump
//! `LATEST_VERSION` — never edit a shipped migration in place.

use rusqlite::{params, Connection};

#[allow(dead_code)]
pub const LATEST_VERSION: i64 = 2;

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

const MIGRATIONS: &[(i64, &str)] = &[(1, SCHEMA_V1), (2, SCHEMA_V2)];

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
}

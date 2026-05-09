//! Sprint G2 — SQLite mirror for structured persistence.
//!
//! Coexists with the legacy JSON writers (`recorder::save_run`,
//! `session_notes::write_notes`). The JSON path remains authoritative for
//! reads (dev panel, linter sidecar, eval-judge) so existing tooling keeps
//! working unchanged. The SQLite mirror is built for queryability:
//!
//!   sqlite3 ~/.bestel/runtime/bestel.sqlite3 \
//!     "SELECT id FROM runs WHERE provider='anthropic' AND model_id LIKE '%haiku%' \
//!      AND verifier_status='fail'"
//!
//! Tables (schema v1):
//!   - runs              — one row per persisted chat turn (mirror of PersistedRun).
//!   - lint_findings     — one row per lint failure produced by Sprint A linter.
//!   - verifier_results  — one row per Sprint G verifier verdict.
//!   - sessions          — mirror of session_notes JSON.
//!   - builds_snapshot   — point-in-time PoB snapshot keyed by build_id.
//!   - kb_versions       — Sprint E ingest fingerprint (per indexed doc).
//!
//! The schema is migrated forward via `PRAGMA user_version`. New migrations
//! append to `MIGRATIONS` and bump the version; old DBs catch up on next open.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use tracing::warn;

mod kb_versions;
mod lint;
mod migrations;
mod runs;
mod sessions;
mod verifier_results;

pub use kb_versions::{upsert_kb_version, KbVersionRow};
pub use lint::{insert_lint_finding, replace_lint_findings, LintFindingRow};
pub use runs::{
    insert_run, query_runs, query_runs_failing_identity, RunRow, RunsFilter,
};
pub use sessions::{read_session_row, upsert_session, SessionRow};
pub use verifier_results::{upsert_verifier_result, VerifierResultRow};

/// Default SQLite location, alongside the existing JSON debug-chats dir.
pub fn default_db_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".bestel").join("runtime").join("bestel.sqlite3"))
}

/// A blocking connection wrapped in a Mutex so we can call from async via
/// `tokio::task::spawn_blocking`. Cheap to clone; one shared writer + readers
/// is fine for the dev-panel access pattern (single user, no concurrency).
#[derive(Clone)]
pub struct Db {
    inner: Arc<Mutex<Connection>>,
}

impl Db {
    /// Open (or create) the SQLite file at the given path, run pending
    /// migrations to bring the schema to the latest version, and return a
    /// shared handle. Sets WAL journal mode + foreign_keys ON.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create db parent {}", parent.display()))?;
        }
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_URI,
        )
        .with_context(|| format!("open sqlite {}", path.display()))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .context("set journal_mode=WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .context("set synchronous=NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .context("set foreign_keys=ON")?;
        migrations::migrate(&conn).context("run migrations")?;
        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open an in-memory DB with the schema already migrated. Used by tests.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("open sqlite :memory:")?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .context("set foreign_keys=ON")?;
        migrations::migrate(&conn).context("run migrations")?;
        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open the default DB path. Returns `Ok(None)` when the home dir is not
    /// resolvable (rare on Windows / Linux but possible in sandboxed CI).
    pub fn open_default() -> Result<Option<Self>> {
        match default_db_path() {
            Some(p) => Ok(Some(Self::open(&p)?)),
            None => Ok(None),
        }
    }

    /// Run a closure with the locked connection. Most callers should use the
    /// typed helpers in submodules; this escape hatch exists for `bestel db
    /// query` raw SQL passthrough.
    pub fn with_conn<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let guard = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("db mutex poisoned"))?;
        f(&guard)
    }

    /// Convenience for the most common case: a closure that returns
    /// `rusqlite::Result<T>`. Wraps the rusqlite error into anyhow.
    pub fn with_conn_rusqlite<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> rusqlite::Result<T>,
    {
        self.with_conn(|c| f(c).map_err(anyhow::Error::from))
    }
}

/// Counts of every persisted table — surfaced by `bestel db stats`.
#[derive(Debug, Clone, Default)]
pub struct DbStats {
    pub runs: i64,
    pub lint_findings: i64,
    pub verifier_results: i64,
    pub sessions: i64,
    pub builds_snapshot: i64,
    pub kb_versions: i64,
    pub schema_version: i64,
    pub bytes_on_disk: u64,
}

/// Process-wide handle. Resolved lazily on first call to
/// [`global_db`] (or [`init_global_db`]) so the binary works even when
/// the home dir is unresolvable (the JSON path then carries the data).
static GLOBAL_DB: OnceLock<Option<Db>> = OnceLock::new();

/// Open (or get) the process-wide DB at the default location. Returns
/// `None` when the DB cannot be opened (no home dir, IO error). Logs a
/// warning on failure but never panics — callers fall back to JSON only.
pub fn global_db() -> Option<Db> {
    GLOBAL_DB
        .get_or_init(|| match Db::open_default() {
            Ok(opt) => opt,
            Err(e) => {
                warn!(error = ?e, "could not open global SQLite DB; mirror disabled");
                None
            }
        })
        .clone()
}

/// Force-initialise the global DB to a specific path. Useful for tests or
/// for binaries that want to override the default location. No-op if the
/// global has already been resolved.
pub fn init_global_db(path: impl AsRef<Path>) -> Result<()> {
    let db = Db::open(path)?;
    let _ = GLOBAL_DB.set(Some(db));
    Ok(())
}

impl Db {
    pub fn stats(&self, db_path: Option<&Path>) -> Result<DbStats> {
        self.with_conn(|c| {
            let mut s = DbStats::default();
            s.runs = c.query_row("SELECT COUNT(*) FROM runs", [], |r| r.get(0))?;
            s.lint_findings =
                c.query_row("SELECT COUNT(*) FROM lint_findings", [], |r| r.get(0))?;
            s.verifier_results =
                c.query_row("SELECT COUNT(*) FROM verifier_results", [], |r| r.get(0))?;
            s.sessions = c.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))?;
            s.builds_snapshot =
                c.query_row("SELECT COUNT(*) FROM builds_snapshot", [], |r| r.get(0))?;
            s.kb_versions = c.query_row("SELECT COUNT(*) FROM kb_versions", [], |r| r.get(0))?;
            s.schema_version = c
                .pragma_query_value(None, "user_version", |r| r.get(0))
                .unwrap_or(0);
            if let Some(path) = db_path {
                s.bytes_on_disk = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            }
            Ok(s)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory_runs_migrations() {
        let db = Db::open_in_memory().expect("open in-memory db");
        let stats = db.stats(None).expect("stats");
        assert_eq!(stats.schema_version, migrations::LATEST_VERSION);
        assert_eq!(stats.runs, 0);
    }

    #[test]
    fn opening_twice_is_idempotent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("bestel.sqlite3");
        {
            let _db = Db::open(&path).expect("first open");
        }
        let db2 = Db::open(&path).expect("re-open");
        let stats = db2.stats(Some(&path)).expect("stats");
        assert_eq!(stats.schema_version, migrations::LATEST_VERSION);
    }
}

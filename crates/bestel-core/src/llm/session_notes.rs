//! Session notes (Sprint G) — structured, per-session diagnostic memory.
//!
//! Stored at `~/.bestel/notes/<session_id>.json`. Schema is intentionally
//! narrow — Bestel is session-scoped (one PoB analysis per session); the
//! goal is to carry the *current diagnosis* across turns so a multi-turn
//! pivot ("→ what's the cheapest gear-only fix? → which item base?")
//! does not lose the turn-1 conclusion.
//!
//! This is the Anthropic *Effective Context Engineering* (Sept 2025)
//! "structured note-taking" pattern, deliberately NOT a vector store —
//! per the ROADMAP § Sprint G anti-pattern note: *"Vectorising user
//! conversation for long-term memory. Bestel is session-scoped — a
//! Markdown notes file per session suffices."*
//!
//! All I/O is best-effort: missing files return [`SessionNotes::default`]
//! and any parse / write error is logged via `tracing::warn` but never
//! propagated to the user.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::warn;

/// One session's structured diagnosis. Empty defaults are valid — a
/// fresh chat starts with no diagnosis and the verifier / agent fills
/// fields as the conversation progresses.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionNotes {
    pub session_id: String,
    /// Stable identifier of the build under analysis (today: hash of the
    /// PoB source file path). Lets the runtime verify that the cached
    /// diagnosis still belongs to the build the user has loaded.
    #[serde(default)]
    pub build_id: Option<String>,
    /// ISO-8601 UTC timestamp of the last write.
    #[serde(default)]
    pub last_updated: Option<DateTime<Utc>>,
    /// The actual diagnostic state.
    #[serde(default)]
    pub current_diagnosis: Option<CurrentDiagnosis>,
}

/// The frozen turn-1 conclusion. Re-read by `dispatch_get_active_build`
/// so it can echo back to the model on later turns.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CurrentDiagnosis {
    /// One short sentence: "physical mitigation is the weakest layer".
    pub primary_bottleneck: String,
    /// Per-axis evidence supporting the bottleneck. Free-form short
    /// strings; e.g. `["armour 3.4k → ~28% phys reduction at boss hit",
    /// "no spell suppression → spells uncapped"]`.
    #[serde(default)]
    pub evidence: Vec<String>,
    /// The fix path the agent committed to. Stays stable across turns
    /// unless explicitly revised.
    pub chosen_fix: String,
    /// Constraints the user imposed (budget, no-pivot, hardcore-friendly,
    /// etc.). Plain strings.
    #[serde(default)]
    pub constraints: Vec<String>,
}

/// `~/.bestel/notes/`.
pub fn default_notes_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("home dir not resolvable"))?;
    Ok(home.join(".bestel").join("notes"))
}

fn safe_session_filename(session_id: &str) -> Result<String> {
    if session_id.is_empty()
        || session_id.contains('/')
        || session_id.contains('\\')
        || session_id.contains("..")
        || session_id.contains('\0')
        || !session_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!("invalid session_id: {session_id:?}"));
    }
    Ok(format!("{session_id}.json"))
}

/// Read notes for `session_id`. Returns the default (empty) `SessionNotes`
/// when the file is missing or malformed; the read NEVER fails the
/// caller because we don't want a corrupted disk file to break the
/// chat loop.
pub fn read_notes(session_id: &str) -> SessionNotes {
    read_notes_at(default_notes_dir().ok().as_deref(), session_id)
}

pub fn read_notes_at(dir: Option<&Path>, session_id: &str) -> SessionNotes {
    let dir = match dir {
        Some(d) => d,
        None => return SessionNotes::default(),
    };
    let filename = match safe_session_filename(session_id) {
        Ok(n) => n,
        Err(e) => {
            warn!(?e, session_id, "invalid session_id; ignoring");
            return SessionNotes::default();
        }
    };
    let path = dir.join(filename);
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return SessionNotes::default(),
    };
    match serde_json::from_str::<SessionNotes>(&raw) {
        Ok(n) => n,
        Err(e) => {
            warn!(error = ?e, ?path, "session notes JSON malformed; using empty");
            SessionNotes::default()
        }
    }
}

/// Write notes for `session_id`. Errors propagate (callers usually
/// log+continue).
pub fn write_notes(session_id: &str, notes: &SessionNotes) -> Result<()> {
    let dir = default_notes_dir()?;
    write_notes_at(&dir, session_id, notes)
}

pub fn write_notes_at(dir: &Path, session_id: &str, notes: &SessionNotes) -> Result<()> {
    let filename = safe_session_filename(session_id)?;
    fs::create_dir_all(dir)
        .with_context(|| format!("create session notes dir {dir:?}"))?;
    let path = dir.join(filename);
    let mut snapshot = notes.clone();
    snapshot.session_id = session_id.to_string();
    snapshot.last_updated = Some(Utc::now());
    let json =
        serde_json::to_string_pretty(&snapshot).context("serialize SessionNotes")?;
    fs::write(&path, json).with_context(|| format!("write session notes {path:?}"))?;
    if let Some(db) = crate::persistence::global_db() {
        if let Err(e) = crate::persistence::upsert_session(&db, &snapshot) {
            warn!(error = ?e, session_id, "failed to mirror session notes to SQLite");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn read_missing_returns_default() {
        let tmp = TempDir::new().unwrap();
        let notes = read_notes_at(Some(tmp.path()), "some-session-id");
        assert_eq!(notes, SessionNotes::default());
    }

    #[test]
    fn write_then_read_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let mut notes = SessionNotes::default();
        notes.build_id = Some("build-abc".into());
        notes.current_diagnosis = Some(CurrentDiagnosis {
            primary_bottleneck: "physical mitigation".into(),
            evidence: vec!["armour 3.4k".into()],
            chosen_fix: "swap belt to Stygian Vise".into(),
            constraints: vec!["budget < 5 div".into()],
        });
        write_notes_at(tmp.path(), "abc", &notes).unwrap();
        let loaded = read_notes_at(Some(tmp.path()), "abc");
        assert_eq!(loaded.session_id, "abc");
        assert_eq!(loaded.build_id.as_deref(), Some("build-abc"));
        let diag = loaded.current_diagnosis.unwrap();
        assert_eq!(diag.primary_bottleneck, "physical mitigation");
        assert_eq!(diag.chosen_fix, "swap belt to Stygian Vise");
        assert!(loaded.last_updated.is_some());
    }

    #[test]
    fn malformed_json_returns_default() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("broken.json"), "{ not valid json").unwrap();
        let notes = read_notes_at(Some(tmp.path()), "broken");
        assert_eq!(notes, SessionNotes::default());
    }

    #[test]
    fn invalid_session_id_rejected_on_write() {
        let tmp = TempDir::new().unwrap();
        let err = write_notes_at(tmp.path(), "../etc/passwd", &SessionNotes::default()).unwrap_err();
        assert!(err.to_string().contains("invalid session_id"));
    }

    #[test]
    fn invalid_session_id_yields_default_on_read() {
        let tmp = TempDir::new().unwrap();
        let notes = read_notes_at(Some(tmp.path()), "with/slash");
        assert_eq!(notes, SessionNotes::default());
    }
}

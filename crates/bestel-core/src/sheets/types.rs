//! On-disk schema for a Build Sheet.
//!
//! `BuildSheet.payload` is what gets serialized into the SQLite `payload`
//! column as JSON. The shape is versioned via `SchemaVersion` so a future
//! interview / section change can land without breaking older rows.

use serde::{Deserialize, Serialize};

/// Bumped when the section set or any nested shape changes in a non
/// backward-compatible way. Old rows keep their original `schema_version` so
/// readers can pick a parser per version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaVersion(pub u32);

impl SchemaVersion {
    pub const CURRENT: SchemaVersion = SchemaVersion(1);
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::CURRENT
    }
}

/// One section of a Build Sheet. Each section starts as agent-drafted prose
/// (`drafted` true, `confirmed` false), then the user either confirms it
/// as-is or edits the `body` and we set `confirmed` to true. Order is fixed
/// in v1 (see brainstorm § "Section ordering"); v2 may pivot the order to
/// the user's question category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSheetSection {
    pub id: String,
    pub label: String,
    pub body: String,
    pub drafted: bool,
    pub confirmed: bool,
}

/// A defining item entry surfaced in the sheet sidebar. The role mirrors
/// the Identity-card grammar from `prompts/references/27_response_contracts.md`
/// so the same vocabulary is used end-to-end.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefiningItemEntry {
    pub name: String,
    pub role: String, // "engine" | "defining" | "amplifier" | "enabler"
    pub purpose: Option<String>,
}

/// Free-form constraints captured during the interview ("no Mageblood",
/// budget ≤ 50 div, "I don't enjoy melee", ...). Cheap to keep verbatim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentEntry {
    pub constraint: String,
}

/// Things the user explicitly flagged as known weaknesses. Surfaced in the
/// finalized chat answer ("read from sheet · known_gaps").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownGap {
    pub label: String,
    pub note: Option<String>,
}

/// Top-level sheet payload. JSON-serialized into SQLite `build_sheets.payload`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSheet {
    pub schema_version: SchemaVersion,
    pub sections: Vec<BuildSheetSection>,
    pub defining_items: Vec<DefiningItemEntry>,
    pub intent: Vec<IntentEntry>,
    pub known_gaps: Vec<KnownGap>,
}

impl BuildSheet {
    pub fn empty() -> Self {
        Self {
            schema_version: SchemaVersion::CURRENT,
            sections: Vec::new(),
            defining_items: Vec::new(),
            intent: Vec::new(),
            known_gaps: Vec::new(),
        }
    }

    /// Default 6-section interview structure (matches the design's stepper).
    pub fn with_default_sections() -> Self {
        let labels = [
            ("identity", "Identity"),
            ("archetype", "Archetype & skill"),
            ("damage", "Damage scaling"),
            ("defense", "Defense layers"),
            ("items", "Defining items"),
            ("intent", "Intent & goals"),
        ];
        let sections = labels
            .iter()
            .map(|(id, label)| BuildSheetSection {
                id: (*id).to_string(),
                label: (*label).to_string(),
                body: String::new(),
                drafted: false,
                confirmed: false,
            })
            .collect();
        Self {
            schema_version: SchemaVersion::CURRENT,
            sections,
            defining_items: Vec::new(),
            intent: Vec::new(),
            known_gaps: Vec::new(),
        }
    }
}

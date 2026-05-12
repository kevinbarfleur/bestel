//! Persisted shape of a registry entry. Serializable so the
//! `summary_json` blob in SQLite round-trips cleanly.

use serde::{Deserialize, Serialize};

/// The header summary saved alongside each registry entry. Keeps the
/// dropdown rows informative without forcing a re-parse of the PoB XML
/// every time the picker opens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySummary {
    pub class: String,
    pub ascendancy: Option<String>,
    pub level: Option<u32>,
    pub main_skill: Option<String>,
    /// Names of unique-rarity items extracted from the PoB at registration
    /// time. Drives the "Pillar of the Caged God · Inpulsa's · Ashes"
    /// line in the registry list rows.
    pub defining_uniques: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: i64,
    pub display_name: String,
    /// `"poe1"` or `"poe2"`.
    pub game: String,
    pub pob_path: String,
    pub pob_hash: String,
    pub identity_sig: String,
    pub gear_sig: String,
    pub tree_sig: String,
    pub skill_sig: String,
    pub config_sig: String,
    /// Optional FK to `build_sheets.id`. Bestel auto-links a registry
    /// entry to a matching sheet by identity_sig at insertion time, but
    /// the link may be cleared if the user deletes the sheet.
    pub linked_sheet_id: Option<String>,
    pub summary: RegistrySummary,
    pub last_seen_at: i64,
    pub authored_at: i64,
}

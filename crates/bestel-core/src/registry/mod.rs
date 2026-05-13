//! Persistent registry of PoB files the user has explicitly curated.
//!
//! Today (pre-v3) a PoB attaches to a single chat and a closing-then-reopening
//! Bestel forces a re-import. The registry persists each PoB the user
//! considers "their characters" so any chat (now or in the future) can pick
//! one from a dropdown without re-parsing the XML.
//!
//! Each entry carries the five orthogonal signatures from
//! `crate::pob::signatures::BuildSignatures` so the UI can show a glance-able
//! `DriftChipStrip` next to every registry row — when a chat picks a build
//! whose underlying PoB has drifted since registration, the user sees WHICH
//! axis drifted at registry pick-time, not after a full audit.
//!
//! Stored in `build_registry` (see `persistence::migrations` v3).

pub mod store;
pub mod types;

pub use store::{
    delete_entry, get_entry, get_pob_path, insert_entry, list_entries, suggestion_check,
    suggestion_dismiss, touch_last_seen, update_entry_signatures,
};
pub use types::{RegistryEntry, RegistrySummary};

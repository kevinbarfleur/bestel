//! Build Sheets — co-authored, validated build dossier reused across chats.
//!
//! A Build Sheet is the result of a short interview the agent conducts with
//! the user once a PoB is attached: it captures identity, archetype, damage
//! scaling, defense layers, defining items, and intent in a compact form so
//! every future chat about that build can read the sheet instead of replaying
//! the full PoB JSON. The motivation is dual: cost (cuts per-chat input by
//! ~85%) and accuracy (the personality of a build — purpose of an aura, role
//! of a unique — is captured once with user confirmation).
//!
//! Binding to a PoB uses two keys side-by-side:
//!   - `fingerprint`: stable across gear churn (ascendancy + main skill +
//!     sorted defining uniques). Used to find the existing sheet on attach.
//!   - `pob_hash`: sha256 of canonical PoB JSON. Used to detect drift; when
//!     the fingerprint matches but the hash differs, the sheet is "stale"
//!     and the agent proposes a mini-interview refresh.
//!
//! See `docs/brainstorm/build-sheets.md` for the full design rationale.

pub mod fingerprint;
pub mod store;
pub mod types;

pub use fingerprint::{compute_fingerprint, compute_fingerprint_from_pob, compute_pob_hash};
pub use store::SheetRow;
pub use types::{
    BuildSheet, BuildSheetSection, DefiningItemEntry, IntentEntry, KnownGap, SchemaVersion,
};

//! Deterministic mode classifier for a single agent turn.
//!
//! Runs in pure Rust BEFORE the first LLM call so the system prompt can
//! receive a `[Mode: ...]` runtime directive. The classifier embraces the
//! "decisions out of the model" pattern from the 2026-05-12 architecture
//! audit: routing a brief mechanical question vs a deep audit is a
//! deterministic pattern-match, not something a small model should reason
//! about per turn.
//!
//! Mode mapping (drives the SYSTEM_PROMPT `<answer_mode_router>`):
//!   - `BriefMechanic` — quick "what's my X / how many Y" question with a
//!     build attached. Answer directly from `get_active_build` (+ optional
//!     single `pob_calc`). Never propose a sheet interview.
//!   - `DeepAudit` — "review my build", "why am I dying", uber/pinnacle
//!     readiness, upgrade pathing. Existing flow (build-review skill).
//!   - `LegacyDiagnostic` — user explicitly opted out of the sheet
//!     ("skip the sheet", "fresh numbers", etc.). Run the 4-paragraph
//!     legacy diagnostic without authoring or reading a sheet.
//!   - `Refusal` — off-topic. Stay short, in-character.
//!   - `Default` — no chip rendered, classic behavior. Used when no build
//!     is attached and the question doesn't look mechanical.

use std::fmt;

/// Routing decision for a single user turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnMode {
    BriefMechanic,
    DeepAudit,
    LegacyDiagnostic,
    Refusal,
    Default,
}

impl TurnMode {
    /// Canonical wire string for the `[Mode: ...]` runtime tag and the
    /// `LlmDelta::ModeAssigned { mode }` payload.
    pub fn as_str(&self) -> &'static str {
        match self {
            TurnMode::BriefMechanic => "brief-mechanic",
            TurnMode::DeepAudit => "deep-audit",
            TurnMode::LegacyDiagnostic => "legacy-diagnostic",
            TurnMode::Refusal => "refusal",
            TurnMode::Default => "default",
        }
    }

    /// Returns true when the mode should propagate to the frontend as a
    /// `ModeChip`. The Default mode is implicit — no chip rendered.
    pub fn surfaces_to_user(&self) -> bool {
        !matches!(self, TurnMode::Default)
    }
}

impl fmt::Display for TurnMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// User says "skip the sheet", "fresh numbers no sheet", etc. Takes
/// precedence over every other cue — the user is explicitly overriding the
/// audit flow. Checked first in classify_turn.
const SKIP_CUES: &[&str] = &[
    "skip the sheet",
    "skip interview",
    "audit me from scratch",
    "audit from scratch",
    "fresh numbers no interview",
    "fresh numbers, no interview",
    "no interview",
    "no sheet",
    "without a sheet",
];

/// Trigger the deep audit flow (sheet authoring if not yet authored).
const AUDIT_CUES: &[&str] = &[
    "review my build",
    "audit my build",
    "audit my character",
    "why am i dying",
    "why do i die",
    "can i do uber",
    "can i do pinnacle",
    "ready for uber",
    "ready for pinnacle",
    "next upgrade",
    "what should i upgrade",
    "fix my build",
    "fix my defenses",
    "fix my defense",
    "is my build good",
    "rate my build",
    "improve my build",
    "build review",
];

/// Anchor verbs that very strongly hint a brief mechanical lookup question
/// about the active character. Cheap signals beat structure: "what's my X"
/// patterns route here regardless of subject (resists, suppression, EHP).
const BRIEF_CUES: &[&str] = &[
    "what's my",
    "what is my",
    "whats my",
    "how much",
    "how many",
    "how high",
    "how low",
    "am i capped",
    "am i over-capped",
    "am i overcapped",
    "do i have",
    "i have how",
    "what does my",
    "show me my",
    "tell me my",
];

/// Very narrow refusal sniff for the most blatant off-topic asks. The LLM
/// remains the primary refusal authority; this just lets us surface a
/// chip up front for the obvious cases.
const REFUSAL_CUES: &[&str] = &[
    "write me a poem",
    "tell me a joke",
    "what's the weather",
    "what is the weather",
];

/// Classify a single user turn. `user_message_lower` MUST already be
/// lowercased by the caller (typically `last_user_lower` in `anthropic.rs`).
/// `has_active_build` indicates a PoB is attached to the chat;
/// `has_active_sheet` indicates a validated Build Sheet exists for that
/// build.
pub fn classify_turn(
    user_message_lower: &str,
    has_active_build: bool,
    has_active_sheet: bool,
) -> TurnMode {
    let msg = user_message_lower;
    // Skip cues take absolute precedence — the user is opting out of the
    // sheet flow even when the rest of their message screams "audit".
    if contains_any(msg, SKIP_CUES) {
        return TurnMode::LegacyDiagnostic;
    }
    if contains_any(msg, REFUSAL_CUES) {
        return TurnMode::Refusal;
    }
    if contains_any(msg, AUDIT_CUES) {
        return TurnMode::DeepAudit;
    }
    // Without a build attached, mechanic-style questions are ambiguous —
    // they could be a wiki lookup. Stay in default so the LLM picks the
    // appropriate research path itself.
    if has_active_build {
        if contains_any(msg, BRIEF_CUES) {
            return TurnMode::BriefMechanic;
        }
        // If a sheet exists, default reads from the sheet rather than
        // opening a fresh interview — no chip needed.
        let _ = has_active_sheet;
    }
    TurnMode::Default
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cls(s: &str, build: bool) -> TurnMode {
        classify_turn(&s.to_ascii_lowercase(), build, false)
    }

    #[test]
    fn brief_resists_with_build_routes_to_brief_mechanic() {
        assert_eq!(cls("what's my fire res?", true), TurnMode::BriefMechanic);
        assert_eq!(cls("how much EHP do I have", true), TurnMode::BriefMechanic);
        assert_eq!(cls("am i capped on chaos", true), TurnMode::BriefMechanic);
    }

    #[test]
    fn brief_resists_without_build_stays_default() {
        assert_eq!(cls("what's my fire res?", false), TurnMode::Default);
    }

    #[test]
    fn audit_phrases_route_to_deep_audit() {
        assert_eq!(cls("review my build for uber elder", true), TurnMode::DeepAudit);
        assert_eq!(cls("why am I dying to shaper", true), TurnMode::DeepAudit);
        assert_eq!(cls("can I do pinnacle?", true), TurnMode::DeepAudit);
        assert_eq!(cls("what should I upgrade next", true), TurnMode::DeepAudit);
    }

    #[test]
    fn skip_phrases_win_over_audit_phrases() {
        assert_eq!(
            cls("review my build, skip the sheet", true),
            TurnMode::LegacyDiagnostic
        );
        assert_eq!(
            cls("audit my build — no interview please", true),
            TurnMode::LegacyDiagnostic
        );
    }

    #[test]
    fn refusal_cues_route_to_refusal() {
        assert_eq!(cls("tell me a joke", false), TurnMode::Refusal);
        assert_eq!(cls("what's the weather like", false), TurnMode::Refusal);
    }

    #[test]
    fn unrelated_question_stays_default() {
        assert_eq!(cls("what's the wiki say about Spell Suppression", false), TurnMode::Default);
    }

    #[test]
    fn mode_strings_are_kebab_case() {
        assert_eq!(TurnMode::BriefMechanic.as_str(), "brief-mechanic");
        assert_eq!(TurnMode::DeepAudit.as_str(), "deep-audit");
        assert_eq!(TurnMode::LegacyDiagnostic.as_str(), "legacy-diagnostic");
        assert_eq!(TurnMode::Refusal.as_str(), "refusal");
        assert_eq!(TurnMode::Default.as_str(), "default");
    }

    #[test]
    fn default_does_not_surface() {
        assert!(!TurnMode::Default.surfaces_to_user());
        assert!(TurnMode::BriefMechanic.surfaces_to_user());
        assert!(TurnMode::DeepAudit.surfaces_to_user());
        assert!(TurnMode::LegacyDiagnostic.surfaces_to_user());
        assert!(TurnMode::Refusal.surfaces_to_user());
    }
}

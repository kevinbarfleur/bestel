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
//!   - `BriefMechanic { quantitative }` — quick "what's my X / how many Y"
//!     question with a build attached. Answer directly from
//!     `get_active_build` (+ optional single `pob_calc`). Never propose a
//!     sheet interview. The `quantitative` sub-flag is true when the
//!     question explicitly asks for a number (Sprint v6 Reco 6); it routes
//!     to `force_mechanic_fetch` on iter 2 to guarantee a `pob_calc` before
//!     the model commits to a numeric claim.
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
    /// Brief "what's my X" lookup against the attached build. The
    /// `quantitative` sub-flag is set when the question asks for a number
    /// ("how much fire res", "what's my EHP") and triggers Sprint v6
    /// `force_mechanic_fetch` — a forced `pob_calc` on iter 2 so the model
    /// always grounds the answer in authoritative numbers. Non-quantitative
    /// BriefMechanic (e.g. "show me my main skill") keeps the legacy path.
    BriefMechanic {
        quantitative: bool,
    },
    DeepAudit,
    LegacyDiagnostic,
    Refusal,
    Default,
}

impl TurnMode {
    /// Canonical wire string for the `[Mode: ...]` runtime tag and the
    /// `LlmDelta::ModeAssigned { mode }` payload. The `quantitative`
    /// sub-flag is intentionally NOT surfaced to the UI — both sub-states
    /// render the same `brief-mechanic` chip so the user-facing taxonomy
    /// remains stable.
    pub fn as_str(&self) -> &'static str {
        match self {
            TurnMode::BriefMechanic { .. } => "brief-mechanic",
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

/// Sprint v6 Reco 6 — explicit number-asking interrogatives. When ANY of
/// these phrases appears, the question is asking for a numeric answer and
/// the BriefMechanic mode is promoted to `quantitative: true`. English-only;
/// translation extensions ship as plug-in packages later. The cues are
/// kept conservative (false positives = a wasted forced `pob_calc`, false
/// negatives = legacy path, no harm done).
const NUMERIC_INTERROGATIVES: &[&str] = &[
    "how much",
    "how many",
    "how fast",
    "how high",
    "how low",
    "what percent",
    "at level ",
    "at quality ",
    "am i capped",
    "am i overcapped",
    "am i over-capped",
    "am i over capped",
    "am i under-capped",
    "am i undercapped",
    "am i under capped",
];

/// "what's my X" patterns are quantitative only when X names a numeric
/// stat. "what's my main skill" / "what does my passive tree do" are
/// BriefMechanic but NOT quantitative — they ask for a label / structure,
/// not a number. The conservative subject list below sticks to PoE stats
/// whose canonical answer is a number (or a number + cap).
const NUMERIC_MY_TRIGGERS: &[&str] = &["what's my", "what is my", "whats my"];

const NUMERIC_SUBJECTS: &[&str] = &[
    "fire res",
    "cold res",
    "lightning res",
    "chaos res",
    "fire resistance",
    "cold resistance",
    "lightning resistance",
    "chaos resistance",
    "ehp",
    "max hit",
    "life",
    "mana",
    "energy shield",
    "spirit",
    "evasion",
    "armour",
    "armor",
    "dps",
    "damage",
    "crit chance",
    "crit multi",
    "crit multiplier",
    "spell suppression",
    "block chance",
    "spell block",
    "phys reduction",
    "physical reduction",
    "attack speed",
    "cast speed",
    "movement speed",
    "leech",
    "regen",
    "recovery rate",
    "elemental damage",
    "elemental res",
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
            let quantitative = is_numeric_mechanic_question(msg);
            return TurnMode::BriefMechanic { quantitative };
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

/// Sprint v6 Reco 6 — pure text predicate, independent of build state.
/// Returns true when the user's lowercased message looks like it asks for
/// a numeric answer. The caller (`classify_turn` or `anthropic.rs`) is
/// responsible for gating this on `has_active_build` and `TurnMode`.
///
/// Two layers:
///   1. `NUMERIC_INTERROGATIVES` — "how much / how many / how fast /
///      what percent / at level / at quality / am i capped / …".
///   2. `NUMERIC_MY_TRIGGERS` + `NUMERIC_SUBJECTS` — "what's my fire res",
///      "what is my EHP", etc. The subject list is conservative: only
///      stats whose canonical answer is a number, never a label.
pub fn is_numeric_mechanic_question(lower: &str) -> bool {
    if contains_any(lower, NUMERIC_INTERROGATIVES) {
        return true;
    }
    if contains_any(lower, NUMERIC_MY_TRIGGERS) && contains_any(lower, NUMERIC_SUBJECTS) {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cls(s: &str, build: bool) -> TurnMode {
        classify_turn(&s.to_ascii_lowercase(), build, false)
    }

    #[test]
    fn brief_resists_with_build_routes_to_brief_mechanic() {
        // Sprint v6 — the three legacy inputs all hit explicit numeric
        // triggers ("what's my fire res", "how much", "am i capped"), so
        // they MUST land in the `quantitative: true` sub-state. Verifying
        // the exact value (rather than `matches!`) keeps the assertion
        // load-bearing for `force_mechanic_fetch` gating.
        assert_eq!(
            cls("what's my fire res?", true),
            TurnMode::BriefMechanic { quantitative: true }
        );
        assert_eq!(
            cls("how much EHP do I have", true),
            TurnMode::BriefMechanic { quantitative: true }
        );
        assert_eq!(
            cls("am i capped on chaos", true),
            TurnMode::BriefMechanic { quantitative: true }
        );
    }

    #[test]
    fn brief_resists_without_build_stays_default() {
        assert_eq!(cls("what's my fire res?", false), TurnMode::Default);
    }

    #[test]
    fn brief_non_numeric_routes_to_brief_mechanic_non_quantitative() {
        // BRIEF_CUES match ("show me my", "tell me my", "what does my")
        // but the question doesn't ask for a number — the subject is a
        // label or structure. Quantitative sub-flag must stay false so
        // we don't force `pob_calc` on a non-numeric ask.
        assert_eq!(
            cls("show me my main skill", true),
            TurnMode::BriefMechanic {
                quantitative: false
            }
        );
        assert_eq!(
            cls("tell me my passives", true),
            TurnMode::BriefMechanic {
                quantitative: false
            }
        );
        assert_eq!(
            cls("what does my passive tree look like", true),
            TurnMode::BriefMechanic {
                quantitative: false
            }
        );
        assert_eq!(
            cls("do i have spell suppression", true),
            TurnMode::BriefMechanic {
                quantitative: false
            }
        );
    }

    #[test]
    fn is_numeric_mechanic_question_true_for_quantitative_asks() {
        // Sprint v6 Reco 6 — 10 explicit quantitative asks.
        let cases = [
            "how much fire res do i have",
            "how many endurance charges can i sustain",
            "how fast does this proc",
            "what percent of my life is leech",
            "at level 92 what's my dps",
            "at quality 20 how much damage do i lose",
            "am i capped on chaos res",
            "am i overcapped on fire",
            "how high can my crit chance go",
            "what's my fire resistance after merc lab",
        ];
        for c in cases {
            assert!(
                is_numeric_mechanic_question(&c.to_ascii_lowercase()),
                "expected quantitative=true for: {c}"
            );
        }
    }

    #[test]
    fn is_numeric_mechanic_question_false_for_non_quantitative() {
        // Sprint v6 Reco 6 — 10 mechanic / off-topic asks that must NOT
        // trip the quantitative sub-flag. False negatives are cheap (the
        // legacy path runs); false positives waste a forced pob_calc.
        let cases = [
            "review my build for uber elder",
            "tell me a joke",
            "explain how chill works",
            "what does my main skill do",
            "show me my main skill",
            "is this build good",
            "what's the wiki say about suppression",
            "do i have spell suppression",
            "tell me my passives",
            "audit my build",
        ];
        for c in cases {
            assert!(
                !is_numeric_mechanic_question(&c.to_ascii_lowercase()),
                "expected quantitative=false for: {c}"
            );
        }
    }

    #[test]
    fn audit_phrases_route_to_deep_audit() {
        assert_eq!(
            cls("review my build for uber elder", true),
            TurnMode::DeepAudit
        );
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
        assert_eq!(
            cls("what's the wiki say about Spell Suppression", false),
            TurnMode::Default
        );
    }

    #[test]
    fn mode_strings_are_kebab_case() {
        // Both `quantitative` sub-states render the same chip — the flag
        // is internal to `force_mechanic_fetch` and not part of the
        // user-facing taxonomy.
        assert_eq!(
            TurnMode::BriefMechanic { quantitative: true }.as_str(),
            "brief-mechanic"
        );
        assert_eq!(
            TurnMode::BriefMechanic {
                quantitative: false
            }
            .as_str(),
            "brief-mechanic"
        );
        assert_eq!(TurnMode::DeepAudit.as_str(), "deep-audit");
        assert_eq!(TurnMode::LegacyDiagnostic.as_str(), "legacy-diagnostic");
        assert_eq!(TurnMode::Refusal.as_str(), "refusal");
        assert_eq!(TurnMode::Default.as_str(), "default");
    }

    #[test]
    fn default_does_not_surface() {
        assert!(!TurnMode::Default.surfaces_to_user());
        assert!(TurnMode::BriefMechanic { quantitative: true }.surfaces_to_user());
        assert!(TurnMode::BriefMechanic {
            quantitative: false
        }
        .surfaces_to_user());
        assert!(TurnMode::DeepAudit.surfaces_to_user());
        assert!(TurnMode::LegacyDiagnostic.surfaces_to_user());
        assert!(TurnMode::Refusal.surfaces_to_user());
    }
}

//! Chain-of-Verification (CoVe factored) verifier.
//!
//! Replaces the Sprint G single-pass consistency check with a 3-step
//! retrieval-grounded pipeline (Meta, ACL 2024, arxiv 2309.11495):
//!
//! 1. **Extract**: atomize the draft into ≤ 7 verifiable claims (1 LLM call).
//! 2. **Gather evidence**: pull deterministic evidence per claim from the
//!    local KB (`kb::search`). If the KB is unavailable we fall through
//!    to mark every claim "unverified" and ship the draft as-is.
//! 3. **Judge**: classify each claim as ok / wrong / unverified against
//!    its evidence (1 batched LLM call). Wrongs include a one-line fix.
//!
//! When ≥ 1 claim is `wrong`, we run a 4th sub-call ("revise") that
//! rewrites the offending sentences in place, preserving paragraph
//! structure and Sources blocks. The revised text replaces the draft
//! in `run_verifier_pass`. If revise fails or the rewrite is empty,
//! we fall back to a `Pass` with diagnostic findings so the original
//! reply still ships.
//!
//! Cost note: 2-4 sub-calls per turn (extract + judge ± revise), each
//! 500-2000 tokens. Total overhead ~10-20% on a real turn vs the
//! single-pass Sprint G's ~5%. `settings::is_verify_enabled` lets
//! users opt out for cost-sensitive workflows; the heuristic
//! [`should_verify`] short-circuits trivial drafts before any API
//! call so the toggle is the only knob most users ever touch.

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Hard cap on claims extracted per draft. Anything beyond this gets
/// dropped at the extract step — we'd rather miss low-priority claims
/// than blow latency / cost. The model is instructed to rank by
/// importance.
const MAX_CLAIMS: usize = 7;

/// Per-claim evidence body cap. Anthropic charges per token and the
/// judge prompt batches all claims+evidence into one call, so each
/// extra char is paid 7×.
const MAX_EVIDENCE_PER_CLAIM_CHARS: usize = 500;

/// Whole-pipeline hard timeout. Three sub-calls + KB hits + revise can
/// stretch under load; this is the absolute cutoff after which we ship
/// the draft as-is rather than block the user.
const HARD_TIMEOUT_SECS: u64 = 12;

/// Top-K passed to `kb.search` per claim. Three hits are usually enough
/// to cover the canonical wiki page and one variant.
const KB_TOP_K_PER_CLAIM: usize = 3;

/// Verdict shape consumed by `anthropic.rs::run_verifier_pass`. The
/// first three fields preserve the Sprint G surface so the existing
/// 3-arm match (Pass / Revise / Fail) keeps working unchanged. The
/// last two are added by the CoVe refactor and surfaced to the UI as
/// "claims_checked / corrections_count" in the slim tool card.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifierVerdict {
    pub status: VerdictStatus,
    #[serde(default)]
    pub findings: Vec<VerifierFinding>,
    #[serde(default)]
    pub minimal_rewrite: String,
    #[serde(default)]
    pub claims_checked: Vec<VerifiedClaim>,
    #[serde(default)]
    pub corrections_count: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VerdictStatus {
    Pass,
    Revise,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifierFinding {
    pub category: String,
    pub issue: String,
    #[serde(default)]
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifiedClaim {
    /// Atomic claim sentence as the extractor distilled it.
    pub statement: String,
    /// Topic used for KB retrieval (e.g. "Brutality support gem").
    pub topic: String,
    /// Judge verdict against the gathered evidence.
    pub status: ClaimStatus,
    /// 1-2 sentence excerpt from the KB hit, truncated to
    /// [`MAX_EVIDENCE_PER_CLAIM_CHARS`]. Empty when the KB had no hit.
    pub evidence_excerpt: String,
    /// Single-sentence fix the judge proposed for `wrong` claims.
    /// `None` for `ok` and `unverified`.
    pub correction: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClaimStatus {
    Ok,
    Wrong,
    Unverified,
}

impl VerifierVerdict {
    /// Used as a fallback when any sub-call or parse fails. We DON'T
    /// promote a failed verifier into a false `revise` — we tag it
    /// `pass` so the user-facing answer survives, and surface the
    /// underlying error in `findings` for the dev panel.
    pub fn pass_with_note(note: impl Into<String>) -> Self {
        Self {
            status: VerdictStatus::Pass,
            findings: vec![VerifierFinding {
                category: "verifier_unavailable".into(),
                issue: note.into(),
                evidence: String::new(),
            }],
            minimal_rewrite: String::new(),
            claims_checked: Vec::new(),
            corrections_count: 0,
        }
    }

    /// Pass with no findings and no claims — used when `should_verify`
    /// short-circuits (cheap drafts that don't warrant verification).
    pub fn pass_empty() -> Self {
        Self {
            status: VerdictStatus::Pass,
            findings: Vec::new(),
            minimal_rewrite: String::new(),
            claims_checked: Vec::new(),
            corrections_count: 0,
        }
    }
}

// ============================================================================
// Heuristic gate — kept from Sprint G unchanged. Cheap pre-filter that runs
// before any API call. Returns true when the draft mentions:
// - a `Sources:` block (any answer that cites is worth verifying)
// - a numerical claim with PoE-context units (k DPS, m DPS, EHP, max-hit, life, ES)
// - a tier marker (T1-T17 maps, mod tier S/T1, t1 mod, etc.)
// - a patch version (`0.5`, `3.25`, `Settlers`, `Affliction`, etc.)
// - a `(cached)` marker (means engine fallback — verify the disclaimer)
// - a banned phrase (always verify, will be flagged `wrong`)
// - an `Identity:` line (always verify, build-grounded answer)
// ============================================================================

pub fn should_verify(text: &str) -> bool {
    let lower = text.to_lowercase();
    if lower.contains("sources:") {
        return true;
    }
    if lower.contains("(cached)") {
        return true;
    }
    if lower.contains("real dps")
        || lower.contains("actual dps")
        || lower.contains("live engine result")
        || lower.contains("verified dps")
    {
        return true;
    }
    if has_numerical_claim(&lower) {
        return true;
    }
    if has_tier_marker(text) {
        return true;
    }
    if has_patch_version(text) {
        return true;
    }
    if text.contains("Identity:") {
        return true;
    }
    false
}

/// Sprint `value-purge` — fetch-aware extension of [`should_verify`].
///
/// Forces a verifier pass when the draft contains a numerical claim about a
/// PoE mechanic but the agent did NOT call any of the fetch tools
/// (`wiki_parse`, `wiki_cargo`, `repoe_lookup`, `pob_calc`, `web_fetch`)
/// during the same turn. The bundled references no longer carry values, so
/// an unsourced number in the draft is, by definition, recalled from
/// training data — the exact failure mode the sprint is designed to catch.
///
/// Backward compatible: if `should_verify` already fires, we short-circuit
/// and return true (preserving every existing trigger). The fetch-aware
/// check only adds NEW triggers, never removes them.
pub fn should_verify_with_context(text: &str, fetch_tool_calls: usize) -> bool {
    if should_verify(text) {
        return true;
    }
    has_unsourced_numeric_claim(text, fetch_tool_calls)
}

/// Detects a numerical claim about a PoE mechanic in the draft without an
/// accompanying same-turn fetch. Cheap heuristic: regex-free, byte-scan
/// based, runs early-exit.
///
/// Returns `false` when at least one fetch tool was called this turn — the
/// claim is assumed grounded by that fetch and the existing pipeline
/// (extract → judge → revise) will validate the actual value.
///
/// Returns `true` only when:
/// 1. No fetch tool was called this turn, AND
/// 2. The draft contains a magnitude pattern (`\d+%`, `\d+ sec`, etc.), AND
/// 3. The draft also contains a mechanic keyword (cooldown, suppression,
///    mitigate, regen, …) close enough to the magnitude.
pub fn has_unsourced_numeric_claim(text: &str, fetch_tool_calls: usize) -> bool {
    if fetch_tool_calls > 0 {
        return false;
    }
    let lower = text.to_lowercase();
    let has_magnitude = has_magnitude_pattern(&lower);
    if !has_magnitude {
        return false;
    }
    // Generic mechanic keywords — when present alongside a magnitude in a
    // draft with zero fetches this turn, the number is almost certainly
    // recalled from training data.
    const MECHANIC_KEYWORDS: &[&str] = &[
        // Sprint G baseline keywords.
        "suppression",
        "mitigate",
        "mitigation",
        "reduction",
        "cooldown",
        "recovery",
        "charges",
        "regen",
        "cap on",
        "cap is",
        "penetration",
        "multiplier",
        "magnitude",
        "implicit",
        "tier",
        "more damage",
        "less damage",
        "more multiplier",
        "increased damage",
        "attack speed",
        "cast speed",
        "crit chance",
        "crit multi",
        "crit multiplier",
        // Sprint v6 (Reco 3) — build-behavioral magnitude claims. English-only;
        // translation extensions ship as plug-in packages later.
        // Mana axis.
        "mana sustain",
        "out of mana",
        "oom",
        "mana drain",
        "drain",
        // Survival axis.
        "sustain",
        "survive",
        "tank",
        "tankiness",
        "squishy",
        "fragile",
        // Burst / one-shot.
        "oneshot",
        "one-shot",
        "one shot",
        // Death predicates.
        "gets killed",
        "dies to",
        "killed by",
        "gets deleted",
        // Crowd control.
        "frozen",
        "stunned",
        "chilled",
        "shocked",
        "freeze threshold",
        "stun threshold",
        // Recovery verbs (`recovery` is above; these are conjugations).
        "recovers",
        "recover from",
        "recoup",
        // Resist caps.
        "capped",
        "uncapped",
        "over-capped",
    ];
    if MECHANIC_KEYWORDS.iter().any(|kw| lower.contains(kw)) {
        return true;
    }
    // Named value-bearing PoE entities — mirrors the anthropic.rs hit list
    // from the 2026-05-12 audit. Naming one of these in the same draft as
    // a magnitude is a textbook recall pattern (e.g. "Mind over Matter
    // diverts 30% of damage taken from mana") — caught here even when the
    // surrounding prose doesn't use a mechanic-keyword.
    const VALUE_ENTITIES: &[&str] = &[
        "soul of solaris",
        "mind over matter",
        "spell suppression",
        "spine bow",
        "bone helmet",
        "marble amulet",
        "trinity",
        "flame dash",
        "elemental overload",
        "cast on crit",
        "cast on critical",
        "righteous fire",
        "voices",
        "cluster jewel",
        "eldritch implicit",
        "watcher's eye",
        "tabula rasa",
        "lineage support",
        "resolute technique",
        "pain attunement",
        "fracturing orb",
        "awakener's orb",
    ];
    VALUE_ENTITIES.iter().any(|e| lower.contains(e))
}

/// Byte-scan helper: returns true if the text contains a numeric magnitude
/// followed (possibly across one whitespace) by a unit token. Matches
/// patterns like `40%`, `3.5 sec`, `8s`, `350% more`, `+30 to`, `(15-20)%`.
fn has_magnitude_pattern(lower: &str) -> bool {
    let bytes = lower.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i].is_ascii_digit() {
            let mut j = i + 1;
            while j < len && (bytes[j].is_ascii_digit() || bytes[j] == b'.') {
                j += 1;
            }
            if j < len && bytes[j] == b' ' {
                j += 1;
            }
            if j < len {
                let unit = bytes[j];
                if unit == b'%' || unit == b's' || unit == b'm' || unit == b'x' {
                    return true;
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }
    false
}

fn has_numerical_claim(lower: &str) -> bool {
    let units = [
        "dps",
        "ehp",
        "max-hit",
        "max hit",
        "armour",
        "evasion",
        "ms ttk",
        " k life",
        " k es",
        " hp",
        " life pool",
        " ehp pool",
        "pinnacle",
        "uber",
        " divines",
        " divine",
        " chaos ",
        " div ",
        " divines/h",
        " divines per hour",
        "per hour",
        " divs",
        " mirror",
    ];
    let has_unit = units.iter().any(|u| lower.contains(u));
    if !has_unit {
        return false;
    }
    lower.chars().any(|c| c.is_ascii_digit())
}

fn has_tier_marker(text: &str) -> bool {
    let t_digit = regex_lite_t_digit(text);
    if t_digit {
        return true;
    }
    let lower = text.to_lowercase();
    lower.contains("tier 1") || lower.contains("tier 2") || lower.contains("tier 3")
}

fn regex_lite_t_digit(text: &str) -> bool {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len.saturating_sub(1) {
        if (bytes[i] == b'T' || bytes[i] == b't') && bytes[i + 1].is_ascii_digit() {
            let prev_ok = i == 0 || !bytes[i - 1].is_ascii_alphabetic();
            if prev_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn has_patch_version(text: &str) -> bool {
    let lower = text.to_lowercase();
    let context = lower.contains("patch")
        || lower.contains("league")
        || lower.contains("poe1")
        || lower.contains("poe2")
        || lower.contains("settlers")
        || lower.contains("affliction")
        || lower.contains("kalandra")
        || lower.contains("ancestor")
        || lower.contains("necropolis");
    if !context {
        return false;
    }
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len.saturating_sub(2) {
        if bytes[i].is_ascii_digit() && bytes[i + 1] == b'.' && bytes[i + 2].is_ascii_digit() {
            return true;
        }
        i += 1;
    }
    false
}

// ============================================================================
// System prompts — hardcoded inline (NOT readable / overridable by the model).
// ============================================================================

const VERIFIER_EXTRACT_PROMPT: &str = r#"You are a claim extractor for Bestel, a Path of Exile assistant. Your job is to isolate atomic factual claims from a DRAFT answer that could be checked against the wiki / engine / kb.

SKIP these (do NOT extract):
- Advice, opinions, recommendations ("you should run X", "consider taking Y")
- Numbers that come from the player's PoB (already grounded by the engine)
- General mechanic descriptions that are textbook PoE ("crit chance caps at 95%")
- Hedged statements ("typically", "usually", "around")

EXTRACT these (the kind of thing that gets hallucinated):
- Specific item / unique / ascendancy node names ("Mind of the Council Foulborn variant")
- Numerical mechanics with concrete values ("Brutality lvl 20 = 59% more phys")
- Resource counts about the player ("you have 2 ascendancy points left", "at lvl 94 you need 2 more points")
- Patch-version specific behavior ("in Settlers 3.25, Brutality was buffed")
- Cross-references to gear or skills ("with Awakened Cast on Crit you get Y")

Each claim must be ATOMIC (one fact each) and SELF-CONTAINED (readable without context). Set "topic" to a short search query that would retrieve this claim's authoritative source from the wiki (e.g. "Brutality support gem", "PoE1 ascendancy points total").

Output ONLY this JSON shape, no prose, no markdown fences:

{"claims": [{"statement": "...", "topic": "..."}, ...]}

Cap: 7 claims max, ranked by importance (most likely-hallucinated first). If the draft has no extractable claims, return `{"claims": []}`."#;

const VERIFIER_JUDGE_PROMPT: &str = r#"You are a strict claim judge. For each numbered claim + evidence pair, decide:

- "ok": evidence directly supports the claim (numbers match, names match, behavior matches).
- "wrong": evidence contradicts the claim (different number, wrong name, wrong mechanic). Write a CORRECTION as a single replacement sentence.
- "unverified": evidence is empty, off-topic, or insufficient to judge (do NOT guess; default to unverified when in doubt).

Be strict on numbers. If the claim says "X = 59%" and the evidence says "X = 53%", that's WRONG, not unverified. If the claim says "Don't Panic Yet requires 100% physical conversion" and the evidence confirms it, that's OK.

Be conservative on names. If a unique name appears in the claim but not the evidence, prefer "unverified" unless the evidence explicitly says it doesn't exist.

The CORRECTION (when status=wrong) must be a single drop-in replacement sentence the reviser can splice into the draft verbatim. Keep PoE jargon intact, use proper item/skill case.

Output ONLY this JSON, no prose:

{"verdicts": [{"id": N, "status": "ok|wrong|unverified", "correction": "..."}, ...]}

`correction` is empty string for ok / unverified."#;

const VERIFIER_REVISE_PROMPT: &str = r#"You are revising a draft to fix specific factual claims. Apply each correction VERBATIM to the offending sentence in the draft. Rules:

- Splice corrections in place of the wrong sentence. Do NOT restructure paragraphs.
- Keep ALL other content identical: tone, headings, code blocks, Sources block, Identity line.
- Do NOT add new content, do NOT remove sources, do NOT add disclaimers.
- If a correction obsoletes a follow-up sentence, leave the follow-up unchanged unless the correction makes it nonsensical.
- Output the FULL revised draft text only. No prose explanation, no markdown fences around the whole output."#;

// ============================================================================
// Pipeline — internal data flow.
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
struct RawClaim {
    statement: String,
    topic: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RawClaimList {
    #[serde(default)]
    claims: Vec<RawClaim>,
}

#[derive(Debug, Clone)]
struct ClaimWithEvidence {
    claim: RawClaim,
    evidence: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RawVerdict {
    #[serde(default)]
    id: usize,
    status: ClaimStatus,
    #[serde(default)]
    correction: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RawVerdictList {
    #[serde(default)]
    verdicts: Vec<RawVerdict>,
}

// ============================================================================
// Sprint v6 Phase 4 — turn tool transcript.
// ============================================================================
//
// The CoVe verifier previously only saw the local KB (a static snapshot of
// conceptual mechanics). Build-specific claims like "you oom in 2s" or
// "your max hit is 4k" can't be checked against the KB — the authoritative
// source for those claims is the live `get_active_build` payload and any
// `pob_calc` echo from the same turn. Without that signal the judge had
// no choice but to mark them `unverified`, which the reviser then ignored.
//
// `ToolTranscript` parses the in-flight `messages` array (the same struct
// passed to the Anthropic API) and groups successful tool_result blocks
// by tool name. The judge prompt receives a topic-scoped slice via
// [`ToolTranscript::extract_relevant_fields`], and the pipeline can
// promote an `unverified` claim to `wrong` when the topic is build-
// specific AND the transcript carried a build payload AND the judge
// couldn't ground the claim. Backward-compat: passing `None` everywhere
// collapses to the Sprint G + value-purge behavior.

/// Maximum bytes of tool-output text included per claim in the judge
/// prompt. Keeps the batched judge call under ~8K tokens even with 7
/// claims × tool slices + KB evidence.
const MAX_TOOL_EVIDENCE_PER_CLAIM_CHARS: usize = 1500;

/// Maximum bytes stored per wiki / kb tool excerpt in the transcript.
/// Anthropic returns long text blobs; we cap at parse time to avoid the
/// promotion step ballooning the prompt regardless of relevance.
const MAX_TEXT_EXCERPT_CHARS: usize = 2000;

#[derive(Debug, Default, Clone)]
pub struct ToolTranscript {
    /// Most recent successful `get_active_build` payload (overwritten on
    /// repeat calls). `None` when no build is loaded or the call failed.
    pub active_build: Option<Value>,
    /// All `pob_calc` returns from the turn, in call order. Each entry is
    /// the parsed JSON response (or a fallback `Value::String` of the raw
    /// text when JSON parse fails).
    pub pob_calcs: Vec<Value>,
    /// `wiki_parse` + `wiki_cargo` + `wiki_search` + `wiki_synergies`
    /// excerpts, each capped at [`MAX_TEXT_EXCERPT_CHARS`].
    pub wiki_excerpts: Vec<String>,
    /// `repoe_lookup` + `repoe_mods_for_base` parsed rows.
    pub repoe_rows: Vec<Value>,
    /// `kb_search` text hits in retrieval order, each capped.
    pub kb_hits: Vec<String>,
}

impl ToolTranscript {
    /// Parse a turn's tool transcript from the Anthropic-shaped messages
    /// array. Walks every assistant `tool_use` and pairs it with the
    /// matching user `tool_result` by `id`. Failed or in-flight tool calls
    /// are skipped silently so a transcript built mid-turn never contains
    /// partial data.
    pub fn from_messages(messages: &[Value]) -> Self {
        let mut by_id: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for m in messages {
            let role = m.get("role").and_then(|v| v.as_str()).unwrap_or("");
            let Some(content) = m.get("content").and_then(|c| c.as_array()) else {
                continue;
            };
            if role == "assistant" {
                for block in content {
                    if block.get("type").and_then(|t| t.as_str()) != Some("tool_use") {
                        continue;
                    }
                    let id = block
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = block
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if !id.is_empty() && !name.is_empty() {
                        by_id.insert(id, name);
                    }
                }
            }
        }

        let mut out = ToolTranscript::default();
        for m in messages {
            let role = m.get("role").and_then(|v| v.as_str()).unwrap_or("");
            if role != "user" {
                continue;
            }
            let Some(content) = m.get("content").and_then(|c| c.as_array()) else {
                continue;
            };
            for block in content {
                if block.get("type").and_then(|t| t.as_str()) != Some("tool_result") {
                    continue;
                }
                let is_error = block
                    .get("is_error")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if is_error {
                    continue;
                }
                let id = block
                    .get("tool_use_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let Some(name) = by_id.get(id) else {
                    continue;
                };
                let text = tool_result_text(block);
                if text.trim().is_empty() {
                    continue;
                }
                out.absorb(name, &text);
            }
        }
        out
    }

    /// Returns true when ANY tool data is present. Cheap pre-check used
    /// by the pipeline to decide whether to bother including a
    /// `TURN TOOL OUTPUTS:` block in the composed evidence.
    pub fn has_any_data(&self) -> bool {
        self.active_build.is_some()
            || !self.pob_calcs.is_empty()
            || !self.wiki_excerpts.is_empty()
            || !self.repoe_rows.is_empty()
            || !self.kb_hits.is_empty()
    }

    /// Topic-scoped field extractor. Returns a compact string the judge
    /// prompt can ingest verbatim. Mapping by topic keyword:
    ///   - mana / oom / drain / out of mana → mana stats + reservations
    ///   - life / hp / sustain / regen / leech → life stats + recovery
    ///   - es / energy shield / shield → ES stats
    ///   - resist / res / capped / uncapped → resists block
    ///   - one-shot / oneshot / max hit / squishy → max-hit + EHP
    ///   - damage / dps / crit / hit → average_damage + crit_chance
    ///   - tree / cluster / jewel / passive / keystone → tree + jewels
    ///   - skill / gem / link → main_skill + skill_groups
    ///   - item / unique / gear → defining items
    ///   - (fallback) → head dump of key_stats + defences (capped)
    ///
    /// The returned string is empty when no relevant slice could be
    /// extracted. Cap: [`MAX_TOOL_EVIDENCE_PER_CLAIM_CHARS`] bytes.
    pub fn extract_relevant_fields(&self, claim_topic: &str) -> String {
        if !self.has_any_data() {
            return String::new();
        }
        let topic = claim_topic.to_lowercase();
        let mut buf = String::new();
        if let Some(build) = self.active_build.as_ref() {
            extract_build_slice(build, &topic, &mut buf);
        }
        for calc in &self.pob_calcs {
            extract_pob_calc_slice(calc, &topic, &mut buf);
        }
        for row in &self.repoe_rows {
            if topic_matches_any(
                &topic,
                &[
                    "mod", "tier", "implicit", "explicit", "affix", "prefix", "suffix",
                ],
            ) {
                push_with_label(&mut buf, "REPOE", row);
            }
        }
        for excerpt in &self.wiki_excerpts {
            if buf.len() >= MAX_TOOL_EVIDENCE_PER_CLAIM_CHARS {
                break;
            }
            // Wiki excerpts are off-topic noise unless the topic is generic
            // enough — skip when we already have build-side data. The
            // judge's KB-evidence block carries wiki sources separately.
            let _ = excerpt;
        }
        for hit in &self.kb_hits {
            if buf.len() >= MAX_TOOL_EVIDENCE_PER_CLAIM_CHARS {
                break;
            }
            let _ = hit;
        }
        truncate_inplace(&mut buf, MAX_TOOL_EVIDENCE_PER_CLAIM_CHARS);
        buf
    }

    fn absorb(&mut self, tool_name: &str, raw_content: &str) {
        match tool_name {
            "get_active_build" => {
                if let Ok(v) = serde_json::from_str::<Value>(raw_content) {
                    self.active_build = Some(v);
                }
            }
            "pob_calc" => {
                let v = serde_json::from_str::<Value>(raw_content).unwrap_or_else(|_| {
                    Value::String(truncate(raw_content, MAX_TEXT_EXCERPT_CHARS))
                });
                self.pob_calcs.push(v);
            }
            "wiki_parse" | "wiki_cargo" | "wiki_search" | "wiki_synergies" => {
                self.wiki_excerpts
                    .push(truncate(raw_content, MAX_TEXT_EXCERPT_CHARS));
            }
            "repoe_lookup" | "repoe_mods_for_base" | "poedb_lookup" => {
                if let Ok(v) = serde_json::from_str::<Value>(raw_content) {
                    self.repoe_rows.push(v);
                }
            }
            "kb_search" => {
                self.kb_hits
                    .push(truncate(raw_content, MAX_TEXT_EXCERPT_CHARS));
            }
            _ => {}
        }
    }
}

/// Returns true when a claim's topic refers to a build-specific quantity
/// that, in principle, can be checked against `get_active_build` and / or
/// `pob_calc`. Used by the promotion step: an `unverified` verdict on a
/// build-specific topic, when a build payload WAS available this turn,
/// indicates the model invented a number despite having the means to
/// verify — promote to `wrong`.
pub fn is_build_specific_topic(topic: &str) -> bool {
    let lower = topic.to_lowercase();
    const BUILD_KEYWORDS: &[&str] = &[
        // Player-stat keywords.
        "mana",
        "oom",
        "drain",
        "life",
        "hp",
        "energy shield",
        "spirit",
        "resist",
        " res ",
        "fire res",
        "cold res",
        "lightning res",
        "chaos res",
        "capped",
        "uncapped",
        "overcapped",
        "evasion",
        "armour",
        "armor",
        "max hit",
        "ehp",
        "one-shot",
        "oneshot",
        "one shot",
        "sustain",
        "leech",
        "regen",
        "recovery",
        "block",
        "spell suppression",
        "phys reduction",
        "physical reduction",
        // DPS-ish.
        "dps",
        "damage",
        "crit chance",
        "crit multi",
        "attack speed",
        "cast speed",
        "movement speed",
        // Build identity.
        "passive tree",
        "ascendancy",
        "keystone",
        "cluster jewel",
        "your build",
        "your character",
        "this build",
    ];
    BUILD_KEYWORDS.iter().any(|k| lower.contains(k))
}

fn tool_result_text(block: &Value) -> String {
    block
        .get("content")
        .map(|v| {
            if let Some(s) = v.as_str() {
                s.to_string()
            } else if let Some(arr) = v.as_array() {
                arr.iter()
                    .filter_map(|b| {
                        b.get("type")
                            .and_then(|t| t.as_str())
                            .filter(|t| *t == "text")
                            .and_then(|_| b.get("text").and_then(|t| t.as_str()))
                            .map(|s| s.to_string())
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                v.to_string()
            }
        })
        .unwrap_or_default()
}

fn extract_build_slice(build: &Value, topic: &str, buf: &mut String) {
    // The `get_active_build` payload is a structured JSON: top-level
    // {identity, key_stats, defences, tree, jewels, skill_groups, items,
    // …}. We slice by topic keyword. Each slice path is best-effort —
    // missing keys produce no output rather than failing.
    let want_mana = topic_matches_any(
        topic,
        &["mana", "oom", "drain", "out of mana", "reservation"],
    );
    let want_life = topic_matches_any(
        topic,
        &[
            "life", "hp", "sustain", "leech", "regen", "recovery", "recoup",
        ],
    );
    let want_es = topic_matches_any(topic, &["energy shield", " es ", "shield"]);
    let want_resists = topic_matches_any(
        topic,
        &["resist", " res ", "capped", "uncapped", "overcapped"],
    );
    let want_maxhit = topic_matches_any(
        topic,
        &[
            "one-shot", "oneshot", "one shot", "max hit", "ehp", "squishy", "fragile",
        ],
    );
    let want_damage = topic_matches_any(
        topic,
        &[
            "dps",
            "damage",
            "crit",
            "hit chance",
            "attack speed",
            "cast speed",
        ],
    );
    let want_tree = topic_matches_any(
        topic,
        &[
            "tree",
            "passive",
            "ascendancy",
            "keystone",
            "notable",
            "cluster",
            "jewel",
        ],
    );
    let want_skill = topic_matches_any(topic, &["skill", "gem", "link", "support"]);
    let want_items = topic_matches_any(
        topic,
        &["item", "unique", "gear", "weapon", "armour", "armor"],
    );

    let any_specific = want_mana
        || want_life
        || want_es
        || want_resists
        || want_maxhit
        || want_damage
        || want_tree
        || want_skill
        || want_items;

    let key_stats = build.get("key_stats");
    let defences = build.get("defences");

    if want_mana {
        if let Some(ks) = key_stats {
            push_filtered_fields(buf, "MANA", ks, &["mana", "reservation"]);
        }
    }
    if want_life {
        if let Some(d) = defences {
            push_filtered_fields(
                buf,
                "LIFE",
                d,
                &["life", "leech", "regen", "recoup", "recovery"],
            );
        }
    }
    if want_es {
        if let Some(d) = defences {
            push_filtered_fields(buf, "ENERGY_SHIELD", d, &["energy_shield", "es_"]);
        }
    }
    if want_resists {
        if let Some(d) = defences {
            push_filtered_fields(
                buf,
                "RESISTS",
                d,
                &["fire", "cold", "lightning", "chaos", "res", "cap"],
            );
        }
    }
    if want_maxhit {
        if let Some(d) = defences {
            push_filtered_fields(
                buf,
                "DEFENCE",
                d,
                &["max_hit", "ehp", "armour", "evasion", "block"],
            );
        }
    }
    if want_damage {
        if let Some(ks) = key_stats {
            push_filtered_fields(
                buf,
                "DAMAGE",
                ks,
                &["damage", "dps", "crit", "speed", "hit", "average"],
            );
        }
    }
    if want_tree {
        if let Some(t) = build.get("tree") {
            push_with_label(buf, "TREE", t);
        }
        if let Some(j) = build.get("jewels") {
            push_with_label(buf, "JEWELS", j);
        }
        if let Some(ks) = build.get("allocated_keystones") {
            push_with_label(buf, "KEYSTONES", ks);
        }
        if let Some(n) = build.get("allocated_notables") {
            push_with_label(buf, "NOTABLES", n);
        }
    }
    if want_skill {
        if let Some(g) = build.get("skill_groups") {
            push_with_label(buf, "SKILL_GROUPS", g);
        }
        if let Some(ms) = build.get("main_skill") {
            push_with_label(buf, "MAIN_SKILL", ms);
        }
    }
    if want_items {
        if let Some(i) = build.get("items") {
            push_with_label(buf, "ITEMS", i);
        }
    }

    // Fallback head-dump when no slice matched: hand the judge a compact
    // identity + key_stats + defences view so it can at least sanity-check
    // simple claims. Capped tight so generic topics don't dominate the
    // prompt. Skipped entirely when at least one specific slice fired.
    if !any_specific {
        if let Some(id) = build.get("identity") {
            push_with_label(buf, "IDENTITY", id);
        }
        if let Some(ks) = key_stats {
            push_with_label(buf, "KEY_STATS", ks);
        }
        if let Some(d) = defences {
            push_with_label(buf, "DEFENCES", d);
        }
    }
}

fn extract_pob_calc_slice(calc: &Value, topic: &str, buf: &mut String) {
    // pob_calc returns the same key_stats / defences shape (echo of
    // PathOfBuilding's StatsTable). Reuse the same slicer for symmetry —
    // any matching topic pulls the same field set as get_active_build.
    extract_build_slice(calc, topic, buf);
}

fn topic_matches_any(topic: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| topic.contains(n))
}

fn push_filtered_fields(buf: &mut String, label: &str, obj: &Value, key_substrings: &[&str]) {
    let Some(map) = obj.as_object() else {
        return;
    };
    let mut found: Vec<(String, &Value)> = Vec::new();
    for (k, v) in map {
        let kl = k.to_lowercase();
        if key_substrings.iter().any(|s| kl.contains(s)) {
            found.push((k.clone(), v));
        }
    }
    if found.is_empty() {
        return;
    }
    if !buf.is_empty() {
        buf.push('\n');
    }
    buf.push_str(label);
    buf.push_str(": ");
    let mut first = true;
    for (k, v) in found {
        if !first {
            buf.push_str(", ");
        }
        first = false;
        buf.push_str(&k);
        buf.push('=');
        buf.push_str(&v.to_string());
    }
}

fn push_with_label(buf: &mut String, label: &str, v: &Value) {
    if !buf.is_empty() {
        buf.push('\n');
    }
    buf.push_str(label);
    buf.push_str(": ");
    let s = v.to_string();
    // Cap aggressively to avoid one section dominating the budget.
    if s.len() > 600 {
        buf.push_str(&s[..600]);
        buf.push('…');
    } else {
        buf.push_str(&s);
    }
}

fn truncate_inplace(s: &mut String, max: usize) {
    if s.len() <= max {
        return;
    }
    let mut end = max;
    while !s.is_char_boundary(end) && end > 0 {
        end -= 1;
    }
    s.truncate(end);
    s.push('…');
}

// ============================================================================
// Helpers.
// ============================================================================

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        // Truncate on a char boundary to avoid panicking on UTF-8.
        let mut end = max;
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        format!("{}…", &s[..end])
    }
}

fn strip_code_fences(s: &str) -> &str {
    let trimmed = s.trim();
    if let Some(rest) = trimmed.strip_prefix("```json") {
        return rest.trim_end_matches("```").trim();
    }
    if let Some(rest) = trimmed.strip_prefix("```") {
        return rest.trim_end_matches("```").trim();
    }
    trimmed
}

/// POST one verifier sub-call and pull the assistant text out of the
/// Anthropic-shaped response. All sub-calls share this transport.
async fn post_one(
    http: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    api_version: &str,
    body: Value,
) -> Result<String> {
    let resp = http
        .post(endpoint)
        .header("anthropic-version", api_version)
        .header("x-api-key", api_key)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .with_context(|| "verifier sub-call HTTP error")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let raw = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "verifier sub-call HTTP {status}: {}",
            truncate(&raw, 200)
        ));
    }
    let parsed: Value = resp
        .json()
        .await
        .with_context(|| "verifier sub-call JSON parse")?;
    let text = parsed
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| {
            arr.iter().find_map(|block| {
                block
                    .get("type")
                    .and_then(|t| t.as_str())
                    .filter(|t| *t == "text")
                    .and_then(|_| block.get("text").and_then(|t| t.as_str()))
            })
        })
        .unwrap_or("")
        .to_string();
    if text.trim().is_empty() {
        return Err(anyhow!("verifier sub-call returned empty content"));
    }
    Ok(text)
}

fn build_extract_body(model: &str, draft: &str) -> Value {
    let user_payload =
        format!("DRAFT:\n{draft}\n\n---\n\nReturn ONLY the JSON object {{\"claims\": [...]}}.");
    json!({
        "model": model,
        "max_tokens": 1024,
        "system": [{"type": "text", "text": VERIFIER_EXTRACT_PROMPT}],
        "messages": [{"role": "user", "content": user_payload}],
    })
}

fn build_judge_body(model: &str, claims_with_evidence: &[ClaimWithEvidence]) -> Value {
    let mut payload = String::new();
    for (i, ce) in claims_with_evidence.iter().enumerate() {
        payload.push_str(&format!(
            "CLAIM {}: {}\nEVIDENCE: {}\n\n",
            i,
            ce.claim.statement,
            if ce.evidence.is_empty() {
                "(no evidence found in KB)"
            } else {
                ce.evidence.as_str()
            }
        ));
    }
    payload.push_str("Return ONLY the JSON object {\"verdicts\": [...]}.");
    json!({
        "model": model,
        "max_tokens": 2048,
        "system": [{"type": "text", "text": VERIFIER_JUDGE_PROMPT}],
        "messages": [{"role": "user", "content": payload}],
    })
}

fn build_revise_body(model: &str, draft: &str, corrections: &[(usize, &VerifiedClaim)]) -> Value {
    let mut corr_block = String::new();
    for (_id, c) in corrections {
        corr_block.push_str(&format!(
            "- Wrong: {}\n  Fix: {}\n",
            c.statement,
            c.correction.clone().unwrap_or_default()
        ));
    }
    let user_payload = format!(
        "DRAFT:\n{draft}\n\n---\n\nCORRECTIONS TO APPLY:\n{corr_block}\n---\n\nReturn the full revised draft text only."
    );
    json!({
        "model": model,
        "max_tokens": 4096,
        "system": [{"type": "text", "text": VERIFIER_REVISE_PROMPT}],
        "messages": [{"role": "user", "content": user_payload}],
    })
}

async fn extract_claims(
    http: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    api_version: &str,
    model: &str,
    draft: &str,
) -> Result<Vec<RawClaim>> {
    let body = build_extract_body(model, draft);
    let text = post_one(http, endpoint, api_key, api_version, body).await?;
    let parsed: RawClaimList = serde_json::from_str(strip_code_fences(&text))
        .with_context(|| format!("extract parse: {}", truncate(&text, 200)))?;
    let mut claims = parsed.claims;
    if claims.len() > MAX_CLAIMS {
        claims.truncate(MAX_CLAIMS);
    }
    Ok(claims)
}

/// Pulls per-claim evidence from the local KB AND (Sprint v6 Phase 4) the
/// turn's tool transcript. KB hits give conceptual context; the transcript
/// gives build-specific ground truth. When `transcript` is `Some`, the
/// composed evidence string is:
///
/// ```text
/// TURN TOOL OUTPUTS:
/// {topic-scoped slice of active_build + pob_calcs + repoe rows}
///
/// KB EVIDENCE (background):
/// {kb hits, best score first}
/// ```
///
/// Either block is omitted when its source produced nothing. When
/// `transcript` is `None`, the behavior is bit-for-bit identical to the
/// Sprint G pipeline — only the KB block ships.
async fn gather_evidence(
    claims: Vec<RawClaim>,
    transcript: Option<&ToolTranscript>,
) -> Vec<ClaimWithEvidence> {
    let kb = crate::llm::kb::global();
    let mut out: Vec<ClaimWithEvidence> = Vec::with_capacity(claims.len());
    for c in claims {
        let kb_text = if let Some(kb) = kb.as_ref() {
            match kb.search(&c.topic, KB_TOP_K_PER_CLAIM, None, &[]).await {
                Ok(hits) if !hits.is_empty() => {
                    let mut acc = String::new();
                    for h in hits {
                        if !acc.is_empty() {
                            acc.push_str("\n---\n");
                        }
                        acc.push_str(&h.chunk.body);
                        if acc.len() >= MAX_EVIDENCE_PER_CLAIM_CHARS {
                            break;
                        }
                    }
                    truncate(&acc, MAX_EVIDENCE_PER_CLAIM_CHARS)
                        .trim_end_matches('…')
                        .to_string()
                }
                _ => String::new(),
            }
        } else {
            String::new()
        };
        let tool_text = transcript
            .map(|t| t.extract_relevant_fields(&c.topic))
            .unwrap_or_default();
        let evidence = compose_evidence(&tool_text, &kb_text);
        out.push(ClaimWithEvidence { claim: c, evidence });
    }
    out
}

/// Combine tool transcript slice + KB hits into a single evidence string
/// the judge prompt formats verbatim. The order matters: tool outputs
/// come first because they're authoritative for build-specific topics;
/// KB sits as background.
fn compose_evidence(tool_text: &str, kb_text: &str) -> String {
    match (tool_text.trim().is_empty(), kb_text.trim().is_empty()) {
        (true, true) => String::new(),
        (false, true) => format!("TURN TOOL OUTPUTS:\n{tool_text}"),
        (true, false) => kb_text.to_string(),
        (false, false) => {
            format!("TURN TOOL OUTPUTS:\n{tool_text}\n\nKB EVIDENCE (background):\n{kb_text}")
        }
    }
}

async fn judge_batch(
    http: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    api_version: &str,
    model: &str,
    claims_with_evidence: &[ClaimWithEvidence],
) -> Result<Vec<RawVerdict>> {
    if claims_with_evidence.is_empty() {
        return Ok(Vec::new());
    }
    let body = build_judge_body(model, claims_with_evidence);
    let text = post_one(http, endpoint, api_key, api_version, body).await?;
    let parsed: RawVerdictList = serde_json::from_str(strip_code_fences(&text))
        .with_context(|| format!("judge parse: {}", truncate(&text, 200)))?;
    Ok(parsed.verdicts)
}

async fn revise_draft(
    http: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    api_version: &str,
    model: &str,
    draft: &str,
    corrections: &[(usize, &VerifiedClaim)],
) -> Result<String> {
    let body = build_revise_body(model, draft, corrections);
    let text = post_one(http, endpoint, api_key, api_version, body).await?;
    let stripped = strip_code_fences(&text).to_string();
    if stripped.trim().is_empty() {
        return Err(anyhow!("revise returned empty draft"));
    }
    Ok(stripped)
}

// ============================================================================
// Public entry point.
// ============================================================================

/// Runs the full CoVe-factored pipeline against a draft and returns a
/// verdict the caller should consume in the same way as Sprint G:
/// - `Pass` → ship the draft as-is (or with the audit trail in
///   `claims_checked` for the UI).
/// - `Revise` → swap in `minimal_rewrite` (which already preserves
///   structure) before showing to the user.
/// - `Fail` → currently never returned by this pipeline (we always
///   degrade gracefully to `Pass` with diagnostics). The variant
///   stays in the enum for forward compatibility.
///
/// Errors at any step never propagate: every failure path collapses
/// into [`VerifierVerdict::pass_with_note`] so the user-facing reply
/// always ships. The whole pipeline is wrapped in a hard timeout
/// ([`HARD_TIMEOUT_SECS`]).
pub async fn verify_factored(
    http: &reqwest::Client,
    messages_endpoint: &str,
    api_key: &str,
    api_version: &str,
    model: &str,
    user_question: &str,
    draft: &str,
    fetch_tool_calls: usize,
    transcript: Option<&ToolTranscript>,
) -> VerifierVerdict {
    if draft.trim().is_empty() {
        return VerifierVerdict::pass_with_note("draft was empty; nothing to verify");
    }
    if !should_verify_with_context(draft, fetch_tool_calls) {
        return VerifierVerdict::pass_empty();
    }
    let _ = user_question; // reserved for future judge prompts
    let pipeline = run_pipeline(
        http,
        messages_endpoint,
        api_key,
        api_version,
        model,
        draft,
        transcript,
    );
    match tokio::time::timeout(Duration::from_secs(HARD_TIMEOUT_SECS), pipeline).await {
        Ok(verdict) => verdict,
        Err(_) => {
            VerifierVerdict::pass_with_note(format!("verifier hard timeout > {HARD_TIMEOUT_SECS}s"))
        }
    }
}

async fn run_pipeline(
    http: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    api_version: &str,
    model: &str,
    draft: &str,
    transcript: Option<&ToolTranscript>,
) -> VerifierVerdict {
    let started = std::time::Instant::now();

    // Phase 1 — extract.
    let claims = match extract_claims(http, endpoint, api_key, api_version, model, draft).await {
        Ok(v) if v.is_empty() => {
            tracing::info!(
                target: "bestel.verifier",
                phase = "extract",
                claims = 0,
                latency_ms = started.elapsed().as_millis() as u64,
                "no claims extracted, passing through"
            );
            return VerifierVerdict::pass_empty();
        }
        Ok(v) => {
            tracing::info!(
                target: "bestel.verifier",
                phase = "extract",
                claims = v.len(),
                latency_ms = started.elapsed().as_millis() as u64,
                "claims extracted"
            );
            v
        }
        Err(e) => {
            tracing::warn!(
                target: "bestel.verifier",
                phase = "extract",
                error = %e,
                "extract failed"
            );
            return VerifierVerdict::pass_with_note(format!("extract failed: {e}"));
        }
    };

    // Phase 2 — gather evidence (deterministic, no LLM call, infallible).
    let evidence_started = std::time::Instant::now();
    let with_evidence = gather_evidence(claims, transcript).await;
    let with_evidence_count = with_evidence
        .iter()
        .filter(|c| !c.evidence.is_empty())
        .count();
    tracing::info!(
        target: "bestel.verifier",
        phase = "evidence",
        claims = with_evidence.len(),
        with_evidence = with_evidence_count,
        latency_ms = evidence_started.elapsed().as_millis() as u64,
        "evidence gathered"
    );

    // Phase 3 — judge batch.
    let judge_started = std::time::Instant::now();
    let raw_verdicts =
        match judge_batch(http, endpoint, api_key, api_version, model, &with_evidence).await {
            Ok(v) => {
                tracing::info!(
                    target: "bestel.verifier",
                    phase = "judge",
                    verdicts = v.len(),
                    latency_ms = judge_started.elapsed().as_millis() as u64,
                    "judge completed"
                );
                v
            }
            Err(e) => {
                tracing::warn!(
                    target: "bestel.verifier",
                    phase = "judge",
                    error = %e,
                    "judge failed"
                );
                return VerifierVerdict::pass_with_note(format!("judge failed: {e}"));
            }
        };

    // Compose the audit list. Pad / truncate verdicts to match claims —
    // the judge sometimes drops or duplicates entries; default missing
    // ones to `unverified`.
    let mut claims_checked: Vec<VerifiedClaim> = Vec::with_capacity(with_evidence.len());
    for (i, ce) in with_evidence.iter().enumerate() {
        let v = raw_verdicts.iter().find(|v| v.id == i);
        let (status, correction) = match v {
            Some(v) => {
                let corr = if matches!(v.status, ClaimStatus::Wrong) && !v.correction.is_empty() {
                    Some(v.correction.clone())
                } else {
                    None
                };
                (v.status, corr)
            }
            None => (ClaimStatus::Unverified, None),
        };
        claims_checked.push(VerifiedClaim {
            statement: ce.claim.statement.clone(),
            topic: ce.claim.topic.clone(),
            status,
            evidence_excerpt: ce.evidence.clone(),
            correction,
        });
    }

    // Sprint v6 Phase 4 — promotion of `unverified` to `wrong`.
    //
    // When a claim's topic is build-specific (mana / life / max hit / etc.),
    // AND the turn carried a successful `get_active_build` payload, AND the
    // judge couldn't find evidence (status=unverified), then the model
    // either invented the claim despite having the means to verify it, or
    // misread its own tool output. Either way it's a `wrong` for the
    // reviser to clean up. We synthesize a redacting correction so the
    // revise step has a fix to splice in. KB-only topics (skill mechanics,
    // game rules) keep their `unverified` status because the absence of
    // KB evidence is normal for niche queries.
    let transcript_has_build = transcript
        .map(|t| t.active_build.is_some())
        .unwrap_or(false);
    if transcript_has_build {
        let mut promoted = 0usize;
        for c in claims_checked.iter_mut() {
            if matches!(c.status, ClaimStatus::Unverified) && is_build_specific_topic(&c.topic) {
                c.status = ClaimStatus::Wrong;
                if c.correction.is_none() {
                    c.correction = Some(
                        "I should not make this specific claim without grounding it in your build's actual numbers — drop the claim or hedge it explicitly.".to_string(),
                    );
                }
                promoted += 1;
            }
        }
        if promoted > 0 {
            tracing::info!(
                target: "bestel.verifier",
                phase = "promote",
                promoted,
                "promoted unverified build-specific claims to wrong"
            );
        }
    }

    let wrongs: Vec<(usize, &VerifiedClaim)> = claims_checked
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c.status, ClaimStatus::Wrong))
        .collect();
    let corrections_count = wrongs.len();

    if corrections_count == 0 {
        tracing::info!(
            target: "bestel.verifier",
            phase = "compose",
            claims = claims_checked.len(),
            corrections = 0,
            total_latency_ms = started.elapsed().as_millis() as u64,
            "verdict: pass (no corrections needed)"
        );
        return VerifierVerdict {
            status: VerdictStatus::Pass,
            findings: Vec::new(),
            minimal_rewrite: String::new(),
            claims_checked,
            corrections_count: 0,
        };
    }

    // Phase 4 — revise (only when ≥ 1 wrong).
    let revise_started = std::time::Instant::now();
    let rewrite =
        match revise_draft(http, endpoint, api_key, api_version, model, draft, &wrongs).await {
            Ok(r) => {
                tracing::info!(
                    target: "bestel.verifier",
                    phase = "revise",
                    corrections = corrections_count,
                    rewrite_chars = r.len(),
                    latency_ms = revise_started.elapsed().as_millis() as u64,
                    "revise completed"
                );
                r
            }
            Err(e) => {
                tracing::warn!(
                    target: "bestel.verifier",
                    phase = "revise",
                    corrections = corrections_count,
                    error = %e,
                    "revise failed, falling back to pass"
                );
                return VerifierVerdict {
                    status: VerdictStatus::Pass,
                    findings: vec![VerifierFinding {
                        category: "revise_failed".into(),
                        issue: e.to_string(),
                        evidence: String::new(),
                    }],
                    minimal_rewrite: String::new(),
                    claims_checked,
                    corrections_count,
                };
            }
        };

    let findings: Vec<VerifierFinding> = claims_checked
        .iter()
        .filter(|c| matches!(c.status, ClaimStatus::Wrong))
        .map(|c| VerifierFinding {
            category: "incorrect_claim".into(),
            issue: c.statement.clone(),
            evidence: c.correction.clone().unwrap_or_default(),
        })
        .collect();

    let verdict = VerifierVerdict {
        status: VerdictStatus::Revise,
        findings,
        minimal_rewrite: rewrite,
        claims_checked,
        corrections_count,
    };
    tracing::info!(
        target: "bestel.verifier",
        phase = "compose",
        claims = verdict.claims_checked.len(),
        corrections = verdict.corrections_count,
        total_latency_ms = started.elapsed().as_millis() as u64,
        "verdict: revise"
    );
    verdict
}

// ============================================================================
// Tests.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_verify_skips_brief_mechanics_answer() {
        let text = "Spell suppression caps at 100% chance. The cap is the maximum, not the value.";
        assert!(!should_verify(text));
    }

    #[test]
    fn should_verify_fires_on_sources_block() {
        let text = "Some answer.\n\nSources:\n- [Wiki: foo](https://example.com)";
        assert!(should_verify(text));
    }

    #[test]
    fn should_verify_fires_on_dps_claim() {
        let text = "Your build does 4.2M DPS against pinnacle targets.";
        assert!(should_verify(text));
    }

    #[test]
    fn should_verify_fires_on_cached_marker() {
        let text = "Combined DPS reads 137,200 (cached) — see disclaimer above.";
        assert!(should_verify(text));
    }

    #[test]
    fn should_verify_fires_on_banned_phrases() {
        for s in [
            "your real DPS is",
            "actual DPS",
            "live engine result",
            "verified DPS",
        ] {
            assert!(should_verify(s), "expected trigger on: {s}");
        }
    }

    #[test]
    fn should_verify_fires_on_identity_card() {
        let text = "Identity: defense=life, hit_model=crit, mechanic=self-cast.";
        assert!(should_verify(text));
    }

    #[test]
    fn should_verify_fires_on_tier_marker() {
        assert!(should_verify("Run T16 maps for the drop chance."));
        assert!(should_verify("It's a tier 1 mod with weight 1000."));
    }

    #[test]
    fn should_verify_fires_on_patch_version() {
        assert!(should_verify(
            "This was changed in PoE2 0.5 — atlas towers replace scarabs."
        ));
        assert!(should_verify("Settlers 3.25 patch added recombinators."));
    }

    #[test]
    fn pass_with_note_keeps_status_pass() {
        let v = VerifierVerdict::pass_with_note("api unreachable");
        assert_eq!(v.status, VerdictStatus::Pass);
        assert_eq!(v.findings.len(), 1);
        assert_eq!(v.findings[0].category, "verifier_unavailable");
        assert!(v.claims_checked.is_empty());
        assert_eq!(v.corrections_count, 0);
    }

    #[test]
    fn unsourced_numeric_claim_fires_when_no_fetch() {
        // Plain mechanic + magnitude + zero fetch tools = should trigger.
        assert!(has_unsourced_numeric_claim(
            "Mind over Matter diverts 30% of damage taken from mana.",
            0,
        ));
        assert!(has_unsourced_numeric_claim(
            "Flame Dash has 3 charges with 10s cooldown.",
            0,
        ));
        assert!(has_unsourced_numeric_claim(
            "Trinity Support grants 50% elemental penetration at 3 stacks.",
            0,
        ));
    }

    #[test]
    fn unsourced_numeric_claim_silent_when_fetch_happened() {
        // Same drafts but ≥1 fetch tool call this turn → trust the pipeline.
        assert!(!has_unsourced_numeric_claim(
            "Mind over Matter diverts 30% of damage taken from mana.",
            1,
        ));
        assert!(!has_unsourced_numeric_claim(
            "Flame Dash has 3 charges with 10s cooldown.",
            2,
        ));
    }

    #[test]
    fn unsourced_numeric_claim_silent_on_pure_narrative() {
        // No magnitude / no mechanic keyword → does not trigger even with
        // zero fetches.
        assert!(!has_unsourced_numeric_claim(
            "Your build feels strong on bosses.",
            0,
        ));
        assert!(!has_unsourced_numeric_claim(
            "Consider running Pathfinder over Trickster for survivability.",
            0,
        ));
    }

    // Sprint v6 (Reco 3) — broadened mechanic keyword coverage. Each test
    // pairs a build-behavioral keyword from the new groups with a magnitude
    // (`%`, `s`, `m`, `x`) that the existing `has_magnitude_pattern` accepts.

    #[test]
    fn unsourced_numeric_claim_fires_on_oom_keyword() {
        assert!(has_unsourced_numeric_claim(
            "You oom in 2 seconds during long boss fights.",
            0,
        ));
    }

    #[test]
    fn unsourced_numeric_claim_fires_on_drain_keyword() {
        assert!(has_unsourced_numeric_claim(
            "Mana drain hits 80% of your pool every 3s.",
            0,
        ));
    }

    #[test]
    fn unsourced_numeric_claim_fires_on_survival_keywords() {
        assert!(has_unsourced_numeric_claim(
            "You survive 5s of sustained DoT before going down.",
            0,
        ));
        assert!(has_unsourced_numeric_claim(
            "Squishy with under 4000 max hit — you die in 1s.",
            0,
        ));
    }

    #[test]
    fn unsourced_numeric_claim_fires_on_one_shot_keyword() {
        assert!(has_unsourced_numeric_claim(
            "Sirus pizza one-shots you at 30s downtime.",
            0,
        ));
    }

    #[test]
    fn unsourced_numeric_claim_fires_on_cc_keyword() {
        assert!(has_unsourced_numeric_claim(
            "You get frozen for 4s on hit without freeze threshold.",
            0,
        ));
    }

    #[test]
    fn unsourced_numeric_claim_fires_on_capped_keyword() {
        assert!(has_unsourced_numeric_claim(
            "Your resists capped at 75% across the board.",
            0,
        ));
    }

    #[test]
    fn unsourced_numeric_claim_fires_on_death_predicate() {
        assert!(has_unsourced_numeric_claim(
            "You die in 2s — gets killed by every map boss.",
            0,
        ));
    }

    #[test]
    fn unsourced_numeric_claim_silent_on_new_keywords_with_fetch() {
        // Same broadened triggers — should NOT fire when a fetch happened.
        assert!(!has_unsourced_numeric_claim(
            "You oom in 2 seconds during long boss fights.",
            1,
        ));
        assert!(!has_unsourced_numeric_claim(
            "Your resists capped at 75%.",
            2,
        ));
    }

    #[test]
    fn should_verify_with_context_or_logic() {
        // Existing trigger still fires regardless of fetch count.
        assert!(should_verify_with_context(
            "Identity: defense=life, hit_model=crit.",
            5,
        ));
        // New trigger fires when fetch count is zero.
        assert!(should_verify_with_context(
            "Spell Suppression caps at 50% prevented.",
            0,
        ));
        // Both negative → returns false.
        assert!(!should_verify_with_context(
            "Glad to hear the build is feeling smoother now.",
            0,
        ));
    }

    #[test]
    fn pass_empty_no_findings_no_claims() {
        let v = VerifierVerdict::pass_empty();
        assert_eq!(v.status, VerdictStatus::Pass);
        assert!(v.findings.is_empty());
        assert!(v.claims_checked.is_empty());
        assert_eq!(v.corrections_count, 0);
    }

    #[test]
    fn verified_claim_serde_roundtrip() {
        let c = VerifiedClaim {
            statement: "Brutality lvl 20 = 59% more phys".into(),
            topic: "Brutality support gem".into(),
            status: ClaimStatus::Wrong,
            evidence_excerpt: "Brutality at level 20 grants 53% more physical damage.".into(),
            correction: Some("Brutality lvl 20 = 53% more phys (60% only with awakened).".into()),
        };
        let s = serde_json::to_string(&c).unwrap();
        assert!(s.contains("\"status\":\"wrong\""));
        let back: VerifiedClaim = serde_json::from_str(&s).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn claim_status_lowercase_serde() {
        assert_eq!(serde_json::to_string(&ClaimStatus::Ok).unwrap(), "\"ok\"");
        assert_eq!(
            serde_json::to_string(&ClaimStatus::Wrong).unwrap(),
            "\"wrong\""
        );
        assert_eq!(
            serde_json::to_string(&ClaimStatus::Unverified).unwrap(),
            "\"unverified\""
        );
    }

    #[test]
    fn verdict_serde_includes_new_fields() {
        let v = VerifierVerdict {
            status: VerdictStatus::Revise,
            findings: vec![],
            minimal_rewrite: "rewritten".into(),
            claims_checked: vec![VerifiedClaim {
                statement: "x".into(),
                topic: "x".into(),
                status: ClaimStatus::Ok,
                evidence_excerpt: String::new(),
                correction: None,
            }],
            corrections_count: 0,
        };
        let s = serde_json::to_string(&v).unwrap();
        assert!(s.contains("claims_checked"));
        assert!(s.contains("corrections_count"));
    }

    #[test]
    fn old_verdict_shape_still_parses() {
        // Backwards-compat: a Sprint G verdict (no claims_checked /
        // corrections_count) must still deserialize cleanly. The
        // missing fields default to empty.
        let json = r#"{"status":"pass","findings":[],"minimal_rewrite":""}"#;
        let v: VerifierVerdict = serde_json::from_str(json).unwrap();
        assert_eq!(v.status, VerdictStatus::Pass);
        assert!(v.claims_checked.is_empty());
        assert_eq!(v.corrections_count, 0);
    }

    #[test]
    fn truncate_handles_utf8_boundary() {
        // Multibyte char at the boundary must not panic.
        let s = "abc\u{1F600}xyz";
        let t = truncate(s, 4);
        assert!(t.ends_with('…'));
    }

    #[test]
    fn strip_code_fences_handles_json_block() {
        let s = "```json\n{\"a\":1}\n```";
        assert_eq!(strip_code_fences(s), "{\"a\":1}");
    }

    #[test]
    fn strip_code_fences_handles_plain_block() {
        let s = "```\n{\"a\":1}\n```";
        assert_eq!(strip_code_fences(s), "{\"a\":1}");
    }

    // ============================================================================
    // Sprint v6 Phase 4 — ToolTranscript / promotion tests.
    // ============================================================================

    fn build_msgs_with_active_build(payload: &Value) -> Vec<Value> {
        // Anthropic-shaped messages: one assistant turn requesting
        // `get_active_build`, one user turn returning the tool_result.
        vec![
            json!({
                "role": "assistant",
                "content": [{
                    "type": "tool_use",
                    "id": "toolu_1",
                    "name": "get_active_build",
                    "input": {}
                }]
            }),
            json!({
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": "toolu_1",
                    "content": payload.to_string()
                }]
            }),
        ]
    }

    #[test]
    fn tool_transcript_parses_get_active_build_payload() {
        let payload = json!({
            "identity": {"class": "Inquisitor", "level": 92},
            "key_stats": {"mana": 1234, "mana_unreserved": 234, "average_damage": 50000},
            "defences": {"life": 4500, "fire_res": 75, "fire_res_cap": 75, "max_hit_fire": 4200}
        });
        let msgs = build_msgs_with_active_build(&payload);
        let t = ToolTranscript::from_messages(&msgs);
        assert!(t.active_build.is_some());
        assert!(t.pob_calcs.is_empty());
        assert!(t.has_any_data());
    }

    #[test]
    fn tool_transcript_skips_error_results() {
        let msgs = vec![
            json!({
                "role": "assistant",
                "content": [{
                    "type": "tool_use",
                    "id": "toolu_err",
                    "name": "get_active_build",
                    "input": {}
                }]
            }),
            json!({
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": "toolu_err",
                    "is_error": true,
                    "content": "build not loaded"
                }]
            }),
        ];
        let t = ToolTranscript::from_messages(&msgs);
        assert!(t.active_build.is_none());
        assert!(!t.has_any_data());
    }

    #[test]
    fn tool_transcript_collects_multiple_pob_calcs() {
        let msgs = vec![
            json!({"role":"assistant","content":[{"type":"tool_use","id":"a","name":"pob_calc","input":{}}]}),
            json!({"role":"user","content":[{"type":"tool_result","tool_use_id":"a","content": "{\"key_stats\":{\"average_damage\":1000}}"}]}),
            json!({"role":"assistant","content":[{"type":"tool_use","id":"b","name":"pob_calc","input":{}}]}),
            json!({"role":"user","content":[{"type":"tool_result","tool_use_id":"b","content": "{\"key_stats\":{\"average_damage\":2000}}"}]}),
        ];
        let t = ToolTranscript::from_messages(&msgs);
        assert_eq!(t.pob_calcs.len(), 2);
    }

    #[test]
    fn extract_relevant_fields_mana_topic_pulls_mana_only() {
        let payload = json!({
            "identity": {"class": "X"},
            "key_stats": {
                "mana": 1234,
                "mana_unreserved": 234,
                "mana_reservation_total_percent": 60,
                "average_damage": 50000,
                "crit_chance": 95.0
            },
            "defences": {"life": 4500, "fire_res": 75}
        });
        let t = ToolTranscript::from_messages(&build_msgs_with_active_build(&payload));
        let s = t.extract_relevant_fields("mana sustain");
        assert!(s.contains("MANA"), "expected MANA section in: {s}");
        assert!(s.contains("mana"), "expected mana key in: {s}");
        assert!(
            !s.contains("crit_chance"),
            "should NOT include unrelated key crit_chance: {s}"
        );
    }

    #[test]
    fn extract_relevant_fields_resist_topic_pulls_resists_only() {
        let payload = json!({
            "key_stats": {"mana": 100},
            "defences": {
                "fire_res": 75,
                "fire_res_cap": 75,
                "cold_res": 75,
                "chaos_res": -42,
                "max_hit_fire": 4200,
                "life": 4500
            }
        });
        let t = ToolTranscript::from_messages(&build_msgs_with_active_build(&payload));
        let s = t.extract_relevant_fields("fire resistance capped");
        assert!(s.contains("RESISTS"));
        assert!(s.contains("fire_res"));
        // life is not a resist key — should not appear.
        assert!(!s.contains("life"));
    }

    #[test]
    fn extract_relevant_fields_unknown_topic_uses_fallback_head_dump() {
        let payload = json!({
            "identity": {"class": "Templar"},
            "key_stats": {"mana": 100},
            "defences": {"life": 5000}
        });
        let t = ToolTranscript::from_messages(&build_msgs_with_active_build(&payload));
        let s = t.extract_relevant_fields("some niche claim");
        assert!(s.contains("IDENTITY") || s.contains("KEY_STATS") || s.contains("DEFENCES"));
    }

    #[test]
    fn extract_relevant_fields_empty_transcript_returns_empty() {
        let t = ToolTranscript::default();
        assert!(t.extract_relevant_fields("mana").is_empty());
    }

    #[test]
    fn is_build_specific_topic_fires_on_player_stats() {
        for topic in [
            "mana sustain in long fights",
            "fire resistance",
            "max hit on fire damage",
            "life leech rate",
            "your build dps",
            "passive tree options",
            "spell suppression cap",
        ] {
            assert!(
                is_build_specific_topic(topic),
                "expected build-specific for: {topic}"
            );
        }
    }

    #[test]
    fn is_build_specific_topic_skips_game_mechanics() {
        for topic in [
            "Brutality support gem behavior",
            "Spectres list",
            "Maven mechanic explanation",
            "Awakened Cast on Crit support",
        ] {
            assert!(
                !is_build_specific_topic(topic),
                "expected game-mechanic for: {topic}"
            );
        }
    }

    #[test]
    fn compose_evidence_keeps_old_shape_when_no_tool_data() {
        // Backward-compat: when transcript is empty the evidence is just
        // the KB text — no `TURN TOOL OUTPUTS:` wrapper.
        let s = compose_evidence("", "Brutality grants 53% more phys.");
        assert_eq!(s, "Brutality grants 53% more phys.");
    }

    #[test]
    fn compose_evidence_wraps_when_both_present() {
        let s = compose_evidence("MANA: mana=1234", "Brutality grants 53%.");
        assert!(s.contains("TURN TOOL OUTPUTS:"));
        assert!(s.contains("MANA: mana=1234"));
        assert!(s.contains("KB EVIDENCE (background):"));
        assert!(s.contains("Brutality grants 53%."));
    }

    #[test]
    fn compose_evidence_tool_only_drops_kb_header() {
        let s = compose_evidence("MANA: mana=1234", "");
        assert!(s.starts_with("TURN TOOL OUTPUTS:"));
        assert!(!s.contains("KB EVIDENCE"));
    }
}

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
                if unit == b'%'
                    || unit == b's'
                    || unit == b'm'
                    || unit == b'x'
                {
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
        "dps", "ehp", "max-hit", "max hit", "armour", "evasion", "ms ttk",
        " k life", " k es", " hp", " life pool", " ehp pool", "pinnacle",
        "uber", " divines", " divine", " chaos ", " div ", " divines/h",
        " divines per hour", "per hour", " divs", " mirror",
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
    let user_payload = format!(
        "DRAFT:\n{draft}\n\n---\n\nReturn ONLY the JSON object {{\"claims\": [...]}}."
    );
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

/// Pulls per-claim evidence from the local KB. KB hits are concatenated
/// (best score first) then truncated to [`MAX_EVIDENCE_PER_CLAIM_CHARS`].
/// If the KB is unavailable (not bootstrapped, ungetheable), every claim
/// returns empty evidence — they'll be classified `unverified` downstream.
async fn gather_evidence(claims: Vec<RawClaim>) -> Vec<ClaimWithEvidence> {
    let kb = crate::llm::kb::global();
    let mut out: Vec<ClaimWithEvidence> = Vec::with_capacity(claims.len());
    for c in claims {
        let evidence = if let Some(kb) = kb.as_ref() {
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
        out.push(ClaimWithEvidence { claim: c, evidence });
    }
    out
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
    );
    match tokio::time::timeout(Duration::from_secs(HARD_TIMEOUT_SECS), pipeline).await {
        Ok(verdict) => verdict,
        Err(_) => VerifierVerdict::pass_with_note(format!(
            "verifier hard timeout > {HARD_TIMEOUT_SECS}s"
        )),
    }
}

async fn run_pipeline(
    http: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    api_version: &str,
    model: &str,
    draft: &str,
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
    let with_evidence = gather_evidence(claims).await;
    let with_evidence_count = with_evidence.iter().filter(|c| !c.evidence.is_empty()).count();
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
    let raw_verdicts = match judge_batch(
        http,
        endpoint,
        api_key,
        api_version,
        model,
        &with_evidence,
    )
    .await
    {
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
    let rewrite = match revise_draft(
        http,
        endpoint,
        api_key,
        api_version,
        model,
        draft,
        &wrongs,
    )
    .await
    {
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
        for s in ["your real DPS is", "actual DPS", "live engine result", "verified DPS"] {
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
        assert!(should_verify("This was changed in PoE2 0.5 — atlas towers replace scarabs."));
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
        assert_eq!(serde_json::to_string(&ClaimStatus::Wrong).unwrap(), "\"wrong\"");
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
}

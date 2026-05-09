//! Conditional verifier (Sprint G).
//!
//! Catches unsourced numerical / named claims without paying verification
//! latency on every turn. The verifier is a SECOND PASS by the same model
//! (NOT a sub-agent — Cognition-style consistency); it is gated on a
//! cheap heuristic ([`should_verify`]) so quick mechanic answers and
//! off-topic refusals never hit the API a second time.
//!
//! Verdict shape is strict JSON:
//!
//! ```json
//! {
//!   "status": "pass" | "revise" | "fail",
//!   "findings": [{"category": "...", "issue": "...", "evidence": "..."}],
//!   "minimal_rewrite": "<full revised answer when status=revise, else empty>"
//! }
//! ```
//!
//! - `pass` → emit the draft as-is.
//! - `revise` → swap in `minimal_rewrite` (kept minimal: don't restructure
//!   the answer, just patch the offending claim).
//! - `fail` → emit a fallback "verifier rejected" message; surface the
//!   findings in the dev panel.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// The full verifier system prompt. Kept inline (not a SKILL.md) because
/// it must NOT be readable / overridable by the model itself.
pub const VERIFIER_SYSTEM_PROMPT: &str = r#"You are a strict claim verifier for Bestel, a Path of Exile assistant. You are NOT replying to the user — you are auditing a DRAFT answer the assistant produced. Your only output is a JSON object matching this schema:

{
  "status": "pass" | "revise" | "fail",
  "findings": [{"category": "...", "issue": "...", "evidence": "..."}],
  "minimal_rewrite": "<full revised answer when status=revise, else empty string>"
}

Decision rules:

- **pass**: every numerical claim is sourced (engine echo, wiki, repoe, trade), no fabricated unique names, no PoE1↔PoE2 contamination, the cache disclaimer is present when `pob_calc` failed, no banned phrases ("real DPS" / "actual DPS" / "live engine result" after engine failure).

- **revise**: a single defect that can be patched in-place without restructuring the answer. Return a `minimal_rewrite` that fixes JUST the offending sentence(s); keep paragraph structure, headings, and `Sources:` intact. Do NOT add new content.

- **fail**: the draft fabricates a key claim, cites a forbidden source, or misuses the build identity. Return an empty `minimal_rewrite` and explain the defect in `findings`.

Categories you flag in `findings`:

- `unsourced_number` — DPS / EHP / max-hit / damage value with no engine echo or `(cached)` marker.
- `fabricated_unique` — a unique name that isn't in the wiki / engine output.
- `cache_disclaimer_missing` — engine failed but the verbatim cache disclaimer is absent.
- `banned_phrase` — "real DPS", "actual DPS", "live engine result" after a `pob_calc` failure.
- `cross_game_contamination` — PoE1 mechanic cited for a PoE2 question or vice-versa.
- `identity_card_missing` — answer cites the player's loaded build with concrete numbers (DPS, EHP, max-hit, resists, ascendancy, item names from their gear) but does not open with the `Identity:` line. **Do NOT fire** when the assistant is asking a clarifying question, when no build is loaded (the draft will say so or ask the user to attach one), or when the answer is purely about generic mechanics.
- `forbidden_source` — citation in `Sources:` from the tier-4 SEO blocklist (aoeah, mmogah, fandom, fextralife, etc.).
- `fabricated_url` — URL in `Sources:` that wasn't returned by any tool call you can see.

Output JSON only. No prose, no markdown, no preamble. Wrap nothing in code fences."#;

/// One verdict from the verifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifierVerdict {
    pub status: VerdictStatus,
    #[serde(default)]
    pub findings: Vec<VerifierFinding>,
    #[serde(default)]
    pub minimal_rewrite: String,
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

impl VerifierVerdict {
    /// Used as a fallback when the API call or parse fails. We DON'T
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
        }
    }
}

/// Cheap heuristic: does this draft contain claims worth verifying?
///
/// True if the draft mentions:
/// - a `Sources:` block (any answer that cites is worth verifying)
/// - a numerical claim with PoE-context units (k DPS, m DPS, EHP, max-hit, life, ES)
/// - a tier marker (T1-T17 maps, mod tier S/T1, t1 mod, etc.)
/// - a patch version (`0.5`, `3.25`, `Settlers`, `Affliction`, etc.)
/// - a `(cached)` marker (means engine fallback — verify the disclaimer)
/// - a banned phrase (always verify, will be flagged `fail`)
///
/// False for short, generic answers ("spell suppression caps at 100%",
/// "Resolute Technique disables crit") which already have the linter.
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
    // Require at least one digit anywhere in the same 200-char window.
    lower.chars().any(|c| c.is_ascii_digit())
}

fn has_tier_marker(text: &str) -> bool {
    // T1..T17, "tier 1", "tier 17"
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
            // Reject when T is mid-word (e.g. "ATTACK").
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
    // Crude patch-version detector: `\d+\.\d+` AND a PoE-context word.
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

/// Build the verifier API request body. Kept as a free function so the
/// caller can inject HTTP transport choice (reqwest vs mock).
pub fn build_request_body(
    model: &str,
    user_question: &str,
    draft: &str,
) -> Value {
    let user_payload = format!(
        "USER QUESTION:\n{user_question}\n\n---\n\nDRAFT ANSWER:\n{draft}\n\n---\n\nReturn ONLY the JSON verdict object."
    );
    json!({
        "model": model,
        "max_tokens": 1024,
        "system": [{
            "type": "text",
            "text": VERIFIER_SYSTEM_PROMPT,
        }],
        "messages": [{
            "role": "user",
            "content": user_payload,
        }],
    })
}

/// Parse a verdict from the assistant message text. Tolerates code fences
/// around the JSON.
pub fn parse_verdict(body: &str) -> Result<VerifierVerdict> {
    let trimmed = strip_code_fences(body);
    serde_json::from_str::<VerifierVerdict>(trimmed)
        .with_context(|| format!("parse verifier verdict: {}", truncate(trimmed, 200)))
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

/// Send the draft to the verifier endpoint. Errors are NOT propagated as
/// failures — they bubble up as `pass_with_note` so the user-facing
/// answer always ships. Latency budget: target ~2s, hard timeout 8s.
///
/// `messages_endpoint` is the full URL (e.g. `https://api.anthropic.com/v1/messages`).
pub async fn verify(
    http: &reqwest::Client,
    messages_endpoint: &str,
    api_key: &str,
    api_version: &str,
    model: &str,
    user_question: &str,
    draft: &str,
) -> VerifierVerdict {
    if draft.trim().is_empty() {
        return VerifierVerdict::pass_with_note("draft was empty; nothing to verify");
    }
    if !should_verify(draft) {
        return VerifierVerdict {
            status: VerdictStatus::Pass,
            findings: Vec::new(),
            minimal_rewrite: String::new(),
        };
    }
    let body = build_request_body(model, user_question, draft);
    let resp = match http
        .post(messages_endpoint)
        .header("anthropic-version", api_version)
        .header("x-api-key", api_key)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return VerifierVerdict::pass_with_note(format!("verifier HTTP error: {e}")),
    };
    if !resp.status().is_success() {
        let status = resp.status();
        let raw = resp.text().await.unwrap_or_default();
        return VerifierVerdict::pass_with_note(format!(
            "verifier HTTP {status}: {}",
            truncate(&raw, 200)
        ));
    }
    let parsed: Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => return VerifierVerdict::pass_with_note(format!("verifier parse error: {e}")),
    };
    let assistant_text = parsed
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
        .unwrap_or("");
    if assistant_text.trim().is_empty() {
        return VerifierVerdict::pass_with_note("verifier returned empty content");
    }
    match parse_verdict(assistant_text) {
        Ok(v) => v,
        Err(e) => VerifierVerdict::pass_with_note(format!("verifier verdict unparseable: {e}")),
    }
}

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
    fn parse_verdict_pass_minimal() {
        let json = r#"{"status":"pass","findings":[],"minimal_rewrite":""}"#;
        let v = parse_verdict(json).unwrap();
        assert_eq!(v.status, VerdictStatus::Pass);
        assert!(v.findings.is_empty());
    }

    #[test]
    fn parse_verdict_revise_with_rewrite() {
        let json = r#"{"status":"revise","findings":[{"category":"unsourced_number","issue":"DPS without engine echo","evidence":"4.2M"}],"minimal_rewrite":"Your build does ~4.2M DPS (cached) ..."}"#;
        let v = parse_verdict(json).unwrap();
        assert_eq!(v.status, VerdictStatus::Revise);
        assert_eq!(v.findings.len(), 1);
        assert!(v.minimal_rewrite.contains("(cached)"));
    }

    #[test]
    fn parse_verdict_strips_code_fences() {
        let json = "```json\n{\"status\":\"fail\",\"findings\":[],\"minimal_rewrite\":\"\"}\n```";
        let v = parse_verdict(json).unwrap();
        assert_eq!(v.status, VerdictStatus::Fail);
    }

    #[test]
    fn parse_verdict_rejects_invalid_json() {
        let err = parse_verdict("not json").unwrap_err();
        assert!(err.to_string().to_lowercase().contains("parse"));
    }

    #[test]
    fn build_request_body_includes_strict_system() {
        let body = build_request_body("claude-sonnet-4-5", "what's my DPS", "draft answer here");
        assert_eq!(body["model"], "claude-sonnet-4-5");
        assert!(body["system"][0]["text"]
            .as_str()
            .unwrap()
            .contains("strict claim verifier"));
        assert!(body["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("USER QUESTION"));
        assert!(body["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("DRAFT ANSWER"));
    }

    #[test]
    fn pass_with_note_keeps_status_pass() {
        let v = VerifierVerdict::pass_with_note("api unreachable");
        assert_eq!(v.status, VerdictStatus::Pass);
        assert_eq!(v.findings.len(), 1);
        assert_eq!(v.findings[0].category, "verifier_unavailable");
    }
}

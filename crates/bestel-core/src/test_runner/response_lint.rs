//! Deterministic response linter — Sprint A (Quality Gate).
//!
//! Converts the failure modes observed in the 2026-05-08 export into runtime
//! assertions over a [`PersistedRun`]. Each rule is independent and produces
//! at most one [`LintFinding`] per run; the final [`LintReport`] aggregates
//! them so the dev panel and `bestel run-battery --strict` can render a
//! summary without re-implementing the rules.
//!
//! Severity tiers:
//! - `Fail` = grammar-of-output bugs that block the answer (`<thinking>`
//!   leak, missing identity card, fabricated citation, "real DPS" after
//!   engine failure, panel sidecar misplaced, etc.).
//! - `Warn` = process-narration tells (`Let me fetch...`) that we want to
//!   visualise but not gate on yet.
//! - `Pass` is implicit (rules that did not fire produce no finding).

use serde::{Deserialize, Serialize};

use crate::llm::recorder::{PersistedRun, PersistedSegment};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintFinding {
    pub id: String,
    pub severity: FindingSeverity,
    pub message: String,
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LintReport {
    pub findings: Vec<LintFinding>,
}

impl LintReport {
    pub fn warn_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::Warn)
            .count()
    }

    pub fn fail_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::Fail)
            .count()
    }

    pub fn has_failures(&self) -> bool {
        self.fail_count() > 0
    }
}

/// Run all 11 lints against a [`PersistedRun`].
pub fn lint_run(run: &PersistedRun) -> LintReport {
    let mut findings = Vec::new();
    let final_text = &run.final_text;

    if let Some(f) = check_no_thinking_tags(final_text) {
        findings.push(f);
    }
    if let Some(f) = check_no_process_narration(final_text) {
        findings.push(f);
    }
    if let Some(f) = check_build_identity_required(run, final_text) {
        findings.push(f);
    }
    if let Some(f) = check_pob_calc_failure_no_real_number(run, final_text) {
        findings.push(f);
    }
    if let Some(f) = check_panel_data_first(final_text) {
        findings.push(f);
    }
    if let Some(f) = check_panel_marker_payload_match(final_text) {
        findings.push(f);
    }
    if let Some(f) = check_sources_fetched_only(run, final_text) {
        findings.push(f);
    }
    if let Some(f) = check_no_internal_doc_as_final_source(final_text) {
        findings.push(f);
    }
    if let Some(f) = check_no_blocked_source(final_text) {
        findings.push(f);
    }
    if let Some(f) = check_failed_raw_data_no_table(run, final_text) {
        findings.push(f);
    }
    if let Some(f) = check_calcs_echo_required(run, final_text) {
        findings.push(f);
    }

    LintReport { findings }
}

/// Domains rejected by `web_fetch` (mirrored from `bestel-core::llm::tools`).
/// Kept local to avoid making `tools.rs` internals pub.
const BLOCKED_HOSTS: &[&str] = &[
    "aoeah.com",
    "mmogah.com",
    "iggm.com",
    "ggwtb.com",
    "boostmatch.com",
    "sportskeeda.com",
    "gamewatcher.com",
    "switchbladegaming.com",
    "dotesports.com",
    "gamerant.com",
    "fandom.com",
    "fextralife.com",
];

// ---------------------------------------------------------------------------
// Rule helpers
// ---------------------------------------------------------------------------

fn finding(
    id: &'static str,
    severity: FindingSeverity,
    message: impl Into<String>,
    evidence: Option<String>,
) -> LintFinding {
    LintFinding {
        id: id.to_string(),
        severity,
        message: message.into(),
        evidence,
    }
}

fn first_n_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

// ---------------------------------------------------------------------------
// 1. NO_THINKING_TAGS
// ---------------------------------------------------------------------------

fn check_no_thinking_tags(final_text: &str) -> Option<LintFinding> {
    let lower = final_text.to_ascii_lowercase();
    let needles = ["<thinking>", "</thinking>", "<reasoning>", "</reasoning>"];
    for needle in needles {
        if let Some(pos) = lower.find(needle) {
            let evidence = first_n_chars(&final_text[pos..], 120);
            return Some(finding(
                "NO_THINKING_TAGS",
                FindingSeverity::Fail,
                format!("Reasoning tag '{needle}' leaked into the user-facing answer."),
                Some(evidence),
            ));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// 2. NO_PROCESS_NARRATION
// ---------------------------------------------------------------------------

fn check_no_process_narration(final_text: &str) -> Option<LintFinding> {
    let needles_ci: &[&str] = &[
        "let me fetch",
        "let me search",
        "let me look up",
        "let me check",
        "i'll search",
        "i'll fetch",
        "i'll look up",
        "i'll check",
        "i will search",
        "i will fetch",
        "i need to search",
        "i need to fetch",
        "je vais chercher",
        "je vais récupérer",
        "je vais consulter",
        "laissez-moi chercher",
    ];
    let lower = final_text.to_ascii_lowercase();
    for needle in needles_ci {
        if let Some(pos) = lower.find(needle) {
            let evidence = first_n_chars(&final_text[pos..], 120);
            return Some(finding(
                "NO_PROCESS_NARRATION",
                FindingSeverity::Warn,
                format!("Process-narration phrase '{needle}' leaked into the answer."),
                Some(evidence),
            ));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// 3. BUILD_IDENTITY_REQUIRED
// ---------------------------------------------------------------------------

fn tool_succeeded(run: &PersistedRun, name: &str) -> bool {
    run.assistant_segments.iter().any(|s| match s {
        PersistedSegment::Tool {
            name: n, status, ..
        } => n == name && status == "done",
        _ => false,
    })
}

fn tool_failed(run: &PersistedRun, name: &str) -> bool {
    run.assistant_segments.iter().any(|s| match s {
        PersistedSegment::Tool {
            name: n, status, ..
        } => n == name && status == "failed",
        _ => false,
    })
}

/// True when `get_active_build` ran but reported the structured
/// `"status":"no_build"` payload (i.e. nothing is actually loaded). The
/// answer cannot lead with an Identity card in that case, so the
/// `BUILD_IDENTITY_REQUIRED` rule must skip.
fn tool_returned_no_build(run: &PersistedRun) -> bool {
    run.assistant_segments.iter().any(|s| match s {
        PersistedSegment::Tool {
            name, outputs, ..
        } if name == "get_active_build" => outputs
            .iter()
            .any(|o| o.contains("\"status\":\"no_build\"") || o.contains("\"status\": \"no_build\"")),
        _ => false,
    })
}

fn check_build_identity_required(run: &PersistedRun, final_text: &str) -> Option<LintFinding> {
    if !tool_succeeded(run, "get_active_build") {
        return None;
    }
    // The tool may "succeed" while reporting `"status":"no_build"` — that
    // happens when no PoB is attached. The Identity card cannot be
    // produced from nothing, so skip the rule entirely in that case.
    if tool_returned_no_build(run) {
        return None;
    }
    // Sprint D — `get_active_build` is now force-called on iteration 1 of
    // every build-active session, so this rule fires on every build-loaded
    // run regardless of question intent. We only enforce the identity card
    // when the answer actually engages with the player's specific build.
    // Pure mechanics answers ("Spell suppression caps at 100%.") do not
    // need to lead with `Identity:` even though `get_active_build` ran.
    if !answer_is_build_specific(final_text) {
        return None;
    }
    // The card must appear in the first ~500 characters (covers panel-data
    // sidecar prefix + a normal opening sentence) and start the
    // "Identity: defense=..., hit_model=..., mechanic=..." pattern.
    let head = first_n_chars(final_text, 500);
    let head_lower = head.to_ascii_lowercase();
    let has_keyword = head_lower.contains("identity:")
        && head_lower.contains("defense=")
        && head_lower.contains("hit_model=")
        && head_lower.contains("mechanic=");
    if has_keyword {
        return None;
    }
    Some(finding(
        "BUILD_IDENTITY_REQUIRED",
        FindingSeverity::Fail,
        "get_active_build succeeded but the answer does not lead with the \
         'Identity: defense=..., hit_model=..., mechanic=...' card."
            .to_string(),
        Some(first_n_chars(final_text, 200)),
    ))
}

/// True when the answer text references the player's specific build —
/// stats, items, ascendancy, layout — as opposed to a generic mechanics
/// or vocabulary answer. Used to gate `BUILD_IDENTITY_REQUIRED` so the
/// Sprint D forced `get_active_build` call doesn't make every Brief
/// mechanics answer trip the linter.
fn answer_is_build_specific(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    const PERSONAL_MARKERS: &[&str] = &[
        "your build",
        "your character",
        "your dps",
        "your ehp",
        "your hp",
        "your tree",
        "your gear",
        "your gem",
        "your skill",
        "your ascend",
        "your main",
        "your weapon",
        "your flask",
        "your jewel",
        "your aura",
        "your defense",
        "your defence",
        "your offense",
        "your offence",
        "your block",
        "your spell suppression",
        "your suppress",
        "your resistance",
        "your resist",
        "your mana",
        "your life",
        "your build's",
        "the build",
        "this build",
        "your setup",
        "your loadout",
        "main skill",
    ];
    PERSONAL_MARKERS.iter().any(|m| lower.contains(m))
}

// ---------------------------------------------------------------------------
// 4. POB_CALC_FAILURE_NO_REAL_NUMBER
// ---------------------------------------------------------------------------

fn check_pob_calc_failure_no_real_number(
    run: &PersistedRun,
    final_text: &str,
) -> Option<LintFinding> {
    if !tool_failed(run, "pob_calc") {
        return None;
    }
    let lower = final_text.to_ascii_lowercase();
    let claims: &[&str] = &["real dps", "true dps", "actual dps", "live dps", "current dps"];
    let disclaimers: &[&str] = &[
        "cache",
        "cached",
        "stale",
        "engine failed",
        "engine error",
        "could not recompute",
        "cannot recompute",
        "without recompute",
        "pob_calc failed",
    ];

    for claim in claims {
        if let Some(pos) = lower.find(claim) {
            let window_start = pos.saturating_sub(120);
            let window_end = (pos + claim.len() + 120).min(lower.len());
            let window = &lower[window_start..window_end];
            let disclaimed = disclaimers.iter().any(|d| window.contains(d));
            if !disclaimed {
                return Some(finding(
                    "POB_CALC_FAILURE_NO_REAL_NUMBER",
                    FindingSeverity::Fail,
                    format!(
                        "pob_calc failed but the answer claims '{claim}' without a cache/stale/engine-failed disclaimer in context."
                    ),
                    Some(first_n_chars(&final_text[window_start..window_end], 240)),
                ));
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// 5. PANEL_DATA_FIRST
// ---------------------------------------------------------------------------

const PANEL_OPEN: &str = "\u{27E6}panel"; // ⟦panel
const PANEL_DATA_OPEN: &str = "\u{27E6}panel-data\u{27E7}"; // ⟦panel-data⟧
const PANEL_DATA_CLOSE: &str = "\u{27E6}/panel-data\u{27E7}"; // ⟦/panel-data⟧

fn has_panel_marker(final_text: &str) -> bool {
    // Any `⟦panel:` or `⟦panel*:` (but not `⟦panel-data⟧`).
    let mut search_from = 0;
    while let Some(rel) = final_text[search_from..].find(PANEL_OPEN) {
        let pos = search_from + rel;
        let after = &final_text[pos + PANEL_OPEN.len()..];
        if after.starts_with(':') || after.starts_with("*:") {
            return true;
        }
        search_from = pos + PANEL_OPEN.len();
    }
    false
}

fn check_panel_data_first(final_text: &str) -> Option<LintFinding> {
    if !has_panel_marker(final_text) {
        return None;
    }
    let trimmed = final_text.trim_start();
    if trimmed.starts_with(PANEL_DATA_OPEN) {
        return None;
    }
    Some(finding(
        "PANEL_DATA_FIRST",
        FindingSeverity::Fail,
        "Message uses ⟦panel:…⟧ markers but does not begin with the ⟦panel-data⟧ sidecar.".to_string(),
        Some(first_n_chars(trimmed, 160)),
    ))
}

// ---------------------------------------------------------------------------
// 6. PANEL_MARKER_PAYLOAD_MATCH
// ---------------------------------------------------------------------------

fn extract_panel_marker_names(final_text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut cursor = 0;
    while let Some(rel) = final_text[cursor..].find(PANEL_OPEN) {
        let pos = cursor + rel;
        let after = &final_text[pos + PANEL_OPEN.len()..];
        // Skip ⟦panel-data⟧ sidecar opens/closes.
        if after.starts_with("-data") || after.starts_with("-data\u{27E7}") {
            cursor = pos + PANEL_OPEN.len();
            continue;
        }
        // Match `:type:name⟧` or `*:type:name⟧`.
        let rest = after.strip_prefix('*').unwrap_or(after);
        let rest = match rest.strip_prefix(':') {
            Some(r) => r,
            None => {
                cursor = pos + PANEL_OPEN.len();
                continue;
            }
        };
        if let Some(end) = rest.find('\u{27E7}') {
            let body = &rest[..end];
            // body = "type:name"
            if let Some((_ty, name)) = body.split_once(':') {
                names.push(name.trim().to_string());
            }
            cursor = pos + PANEL_OPEN.len() + 1 + end + 1;
        } else {
            break;
        }
    }
    names
}

fn extract_panel_data_keys(final_text: &str) -> Vec<String> {
    let Some(start) = final_text.find(PANEL_DATA_OPEN) else {
        return Vec::new();
    };
    let after = start + PANEL_DATA_OPEN.len();
    let Some(close_rel) = final_text[after..].find(PANEL_DATA_CLOSE) else {
        return Vec::new();
    };
    let body = &final_text[after..after + close_rel];
    let body_trim = body.trim();
    let body_trim = body_trim
        .strip_prefix("```json")
        .unwrap_or(body_trim)
        .trim_start_matches('\n')
        .trim_end_matches("```")
        .trim();
    let parsed: serde_json::Value = match serde_json::from_str(body_trim) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let mut keys = Vec::new();
    if let Some(obj) = parsed.as_object() {
        for k in obj.keys() {
            keys.push(k.clone());
        }
    }
    keys
}

fn check_panel_marker_payload_match(final_text: &str) -> Option<LintFinding> {
    let markers = extract_panel_marker_names(final_text);
    if markers.is_empty() {
        return None;
    }
    let keys = extract_panel_data_keys(final_text);
    let mut missing: Vec<String> = Vec::new();
    for m in &markers {
        if !keys.iter().any(|k| k == m) {
            missing.push(m.clone());
        }
    }
    if missing.is_empty() {
        return None;
    }
    Some(finding(
        "PANEL_MARKER_PAYLOAD_MATCH",
        FindingSeverity::Fail,
        format!(
            "Panel markers without matching ⟦panel-data⟧ keys: {}",
            missing.join(", ")
        ),
        Some(format!("markers={markers:?} keys={keys:?}")),
    ))
}

// ---------------------------------------------------------------------------
// 7. SOURCES_FETCHED_ONLY
// ---------------------------------------------------------------------------

fn extract_sources_urls(final_text: &str) -> Vec<String> {
    // Heuristic: scan from the last "Sources" or "Sources:" header onward.
    let lower = final_text.to_ascii_lowercase();
    let mut anchor = None;
    for needle in ["\nsources:", "\nsources\n", "\n## sources", "\n**sources**"] {
        if let Some(p) = lower.rfind(needle) {
            anchor = Some(p);
            break;
        }
    }
    let region = match anchor {
        Some(p) => &final_text[p..],
        None => return Vec::new(),
    };
    let mut urls = Vec::new();
    let bytes = region.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"http://") || bytes[i..].starts_with(b"https://") {
            let mut j = i;
            while j < bytes.len() && !is_url_terminator(bytes[j] as char) {
                j += 1;
            }
            if let Ok(s) = std::str::from_utf8(&bytes[i..j]) {
                let trimmed = s.trim_end_matches(|c: char| c == '.' || c == ',' || c == ')' || c == ']');
                urls.push(trimmed.to_string());
            }
            i = j;
        } else {
            i += 1;
        }
    }
    urls
}

fn is_url_terminator(c: char) -> bool {
    c.is_whitespace() || c == '<' || c == '>' || c == '"' || c == '\'' || c == '|'
}

fn collect_tool_output_urls(run: &PersistedRun) -> Vec<String> {
    let mut urls = Vec::new();
    for seg in &run.assistant_segments {
        if let PersistedSegment::Tool {
            name,
            detail,
            outputs,
            ..
        } = seg
        {
            if matches!(
                name.as_str(),
                "web_fetch" | "wiki_parse" | "wiki_search" | "wiki_synergies" | "wiki_cargo"
            ) {
                if let Some(d) = detail {
                    extract_urls_into(d, &mut urls);
                }
                for chunk in outputs {
                    extract_urls_into(chunk, &mut urls);
                }
            }
        }
    }
    urls
}

fn extract_urls_into(text: &str, out: &mut Vec<String>) {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"http://") || bytes[i..].starts_with(b"https://") {
            let mut j = i;
            while j < bytes.len() && !is_url_terminator(bytes[j] as char) {
                j += 1;
            }
            if let Ok(s) = std::str::from_utf8(&bytes[i..j]) {
                out.push(s.to_string());
            }
            i = j;
        } else {
            i += 1;
        }
    }
}

fn url_host(url: &str) -> Option<String> {
    let after = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let host_end = after
        .find(|c: char| c == '/' || c == '?' || c == '#')
        .unwrap_or(after.len());
    let host = &after[..host_end];
    if host.is_empty() {
        None
    } else {
        Some(host.to_ascii_lowercase())
    }
}

fn check_sources_fetched_only(run: &PersistedRun, final_text: &str) -> Option<LintFinding> {
    let cited = extract_sources_urls(final_text);
    if cited.is_empty() {
        return None;
    }
    let fetched = collect_tool_output_urls(run);
    let fetched_hosts: Vec<String> = fetched.iter().filter_map(|u| url_host(u)).collect();
    let mut fabricated: Vec<String> = Vec::new();
    for c in &cited {
        let host = match url_host(c) {
            Some(h) => h,
            None => continue,
        };
        // Match by host: the model often reformats the path, so we accept
        // any URL whose host is part of the same domain that was actually
        // fetched in this turn.
        let host_seen = fetched_hosts.iter().any(|fh| fh == &host || fh.ends_with(&format!(".{host}")) || host.ends_with(&format!(".{fh}")));
        if !host_seen {
            fabricated.push(c.clone());
        }
    }
    if fabricated.is_empty() {
        return None;
    }
    Some(finding(
        "SOURCES_FETCHED_ONLY",
        FindingSeverity::Fail,
        format!(
            "Sources cite URL(s) whose host was never fetched this turn: {}",
            fabricated.join(", ")
        ),
        Some(format!(
            "fetched_hosts={fetched_hosts:?} cited={cited:?}"
        )),
    ))
}

// ---------------------------------------------------------------------------
// 8. NO_INTERNAL_DOC_AS_FINAL_SOURCE
// ---------------------------------------------------------------------------

fn check_no_internal_doc_as_final_source(final_text: &str) -> Option<LintFinding> {
    // Only consider mentions inside a Sources / citation context, not body
    // explanations that legitimately reference our internal taxonomy.
    let lower = final_text.to_ascii_lowercase();
    let anchors = [
        "\nsources:",
        "\nsources\n",
        "\n## sources",
        "\n**sources**",
        "[source]",
        "(source:",
    ];
    let mut anchor = None;
    for needle in anchors {
        if let Some(p) = lower.rfind(needle) {
            anchor = Some(p);
            break;
        }
    }
    let region = match anchor {
        Some(p) => &final_text[p..],
        None => return None,
    };
    let region_lower = region.to_ascii_lowercase();
    let needles = [
        "prompts/references/",
        "references/00_",
        "references/01_",
        "references/02_",
        "references/03_",
        "references/04_",
        "references/05_",
        "references/06_",
        "references/07_",
        "references/08_",
        "references/09_",
        "references/10_",
        "references/11_",
        "references/12_",
        "references/13_",
        "references/14_",
        "references/15_",
        "references/16_",
        "references/17_",
        "references/18_",
        "references/19_",
        "references/20_",
        "references/21_",
        "references/22_",
        "references/23_",
        "references/24_",
        "references/25_",
        "references/26_",
        "docs/references/internal",
        "internal:references",
    ];
    for n in needles {
        if let Some(pos) = region_lower.find(n) {
            let evidence = first_n_chars(&region[pos..], 160);
            return Some(finding(
                "NO_INTERNAL_DOC_AS_FINAL_SOURCE",
                FindingSeverity::Fail,
                "Internal reference document cited as a live user-facing source.".to_string(),
                Some(evidence),
            ));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// 9. NO_BLOCKED_SOURCE
// ---------------------------------------------------------------------------

fn check_no_blocked_source(final_text: &str) -> Option<LintFinding> {
    let mut urls = Vec::new();
    extract_urls_into(final_text, &mut urls);
    for url in &urls {
        let Some(host) = url_host(url) else { continue };
        if BLOCKED_HOSTS
            .iter()
            .any(|b| host == *b || host.ends_with(&format!(".{b}")))
        {
            return Some(finding(
                "NO_BLOCKED_SOURCE",
                FindingSeverity::Fail,
                format!("Answer cites a blocked source host '{host}'."),
                Some(url.clone()),
            ));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// 10. FAILED_RAW_DATA_NO_TABLE
// ---------------------------------------------------------------------------

fn check_failed_raw_data_no_table(run: &PersistedRun, final_text: &str) -> Option<LintFinding> {
    let raw_failed = tool_failed(run, "repoe_lookup")
        || tool_failed(run, "wiki_cargo")
        || tool_failed(run, "wiki_parse")
        || tool_failed(run, "web_fetch");
    if !raw_failed {
        return None;
    }
    let lower = final_text.to_ascii_lowercase();
    let uncertainty_markers = [
        "could not confirm",
        "cannot confirm",
        "could not retrieve",
        "cannot retrieve",
        "source unavailable",
        "source failed",
        "not in the current knowledge base",
        "cannot give an exact",
        "cannot give exact",
        "without source",
    ];
    let has_uncertainty = uncertainty_markers.iter().any(|m| lower.contains(m));
    if has_uncertainty {
        return None;
    }
    // Heuristic table detection: any line containing both `|` and a tier or
    // ilvl token, twice, indicates a fabricated mod table.
    let mut hits = 0usize;
    let mut evidence = None;
    for line in final_text.lines() {
        if !line.contains('|') {
            continue;
        }
        let l = line.to_ascii_lowercase();
        let tier_hit = l.contains(" t1 ")
            || l.contains("|t1|")
            || l.contains(" t2 ")
            || l.contains("|t2|")
            || l.contains(" t3 ")
            || l.contains("|t3|");
        let ilvl_hit = l.contains("ilvl ") || l.contains("item level ") || l.contains("|ilvl|");
        if tier_hit || ilvl_hit {
            hits += 1;
            if evidence.is_none() {
                evidence = Some(line.to_string());
            }
            if hits >= 2 {
                return Some(finding(
                    "FAILED_RAW_DATA_NO_TABLE",
                    FindingSeverity::Fail,
                    "Raw-data tool failed but the answer prints an exact tier/ilvl table without an uncertainty marker.".to_string(),
                    evidence,
                ));
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// 11. CALCS_ECHO_REQUIRED
// ---------------------------------------------------------------------------

fn check_calcs_echo_required(run: &PersistedRun, final_text: &str) -> Option<LintFinding> {
    if !tool_succeeded(run, "pob_calc") {
        return None;
    }
    let lower = final_text.to_ascii_lowercase();
    let echoes = [
        "enemyisboss",
        "enemy is boss",
        "boss",
        "pinnacle",
        "active skill",
        "main skill",
        "configured",
        "config:",
        "calcs:",
    ];
    let any_echo = echoes.iter().any(|e| lower.contains(e));
    if any_echo {
        return None;
    }
    Some(finding(
        "CALCS_ECHO_REQUIRED",
        FindingSeverity::Fail,
        "pob_calc succeeded but the answer does not surface the Calcs echo (boss / Pinnacle / active skill / config flags).".to_string(),
        Some(first_n_chars(final_text, 240)),
    ))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::recorder::ChatStats;

    fn run_with(final_text: &str, segments: Vec<PersistedSegment>) -> PersistedRun {
        PersistedRun {
            id: "test".into(),
            started_at: "2026-05-08T00:00:00Z".into(),
            ended_at: None,
            source: "test".into(),
            provider: "anthropic".into(),
            model_id: "claude-sonnet-4-5".into(),
            model_display_name: "Claude Sonnet 4.5".into(),
            user_text: "irrelevant".into(),
            user_attachments: Vec::new(),
            assistant_segments: segments,
            final_text: final_text.into(),
            stats: ChatStats::default(),
            error: None,
        }
    }

    fn tool_seg(name: &str, status: &str) -> PersistedSegment {
        PersistedSegment::Tool {
            id: format!("t-{name}"),
            name: name.into(),
            detail: None,
            outputs: Vec::new(),
            status: status.into(),
            summary: None,
        }
    }

    fn tool_seg_with_outputs(name: &str, status: &str, outputs: Vec<String>) -> PersistedSegment {
        PersistedSegment::Tool {
            id: format!("t-{name}"),
            name: name.into(),
            detail: None,
            outputs,
            status: status.into(),
            summary: None,
        }
    }

    #[test]
    fn flags_thinking_tag_leak() {
        let run = run_with(
            "Sure thing. <thinking>Let me think about this</thinking> Mageblood is a heavy belt.",
            vec![],
        );
        let report = lint_run(&run);
        assert!(report.findings.iter().any(|f| f.id == "NO_THINKING_TAGS"));
        assert!(report.has_failures());
    }

    #[test]
    fn flags_process_narration_as_warn() {
        let run = run_with("Let me fetch the wiki page first.", vec![]);
        let report = lint_run(&run);
        let f = report
            .findings
            .iter()
            .find(|f| f.id == "NO_PROCESS_NARRATION")
            .expect("warn finding");
        assert_eq!(f.severity, FindingSeverity::Warn);
    }

    #[test]
    fn build_identity_required_passes_with_card() {
        let run = run_with(
            "Identity: defense=hybrid, hit_model=crit, mechanic=self-cast. Defining uniques: …",
            vec![tool_seg("get_active_build", "done")],
        );
        let report = lint_run(&run);
        assert!(!report
            .findings
            .iter()
            .any(|f| f.id == "BUILD_IDENTITY_REQUIRED"));
    }

    #[test]
    fn build_identity_required_fails_without_card() {
        let run = run_with(
            "Your build looks fine, but I would consider swapping helm.",
            vec![tool_seg("get_active_build", "done")],
        );
        let report = lint_run(&run);
        assert!(report
            .findings
            .iter()
            .any(|f| f.id == "BUILD_IDENTITY_REQUIRED" && f.severity == FindingSeverity::Fail));
    }

    #[test]
    fn build_identity_skipped_on_brief_mechanics_answer_with_forced_get_active_build() {
        // Sprint D: `get_active_build` is force-called on every build-active
        // session. A Brief mechanics answer (no personal-build markers) must
        // not trip BUILD_IDENTITY_REQUIRED even though the tool ran.
        let run = run_with(
            "Spell suppression caps at 100% effective; the tree alone gets you to 100% with \
             Wind Dancer + suppress nodes.",
            vec![tool_seg("get_active_build", "done")],
        );
        let report = lint_run(&run);
        assert!(!report
            .findings
            .iter()
            .any(|f| f.id == "BUILD_IDENTITY_REQUIRED"));
    }

    #[test]
    fn build_identity_skipped_when_get_active_build_returns_no_build_status() {
        // Sprint H follow-up: `get_active_build` reports {"status":"no_build"}
        // when nothing is attached. Even on a vague-build question
        // ("my build feels like ass"), there's no Identity to print —
        // the rule must skip.
        let run = run_with(
            "Aye exile — what hurts most? Defenses, damage, or just slow farming?",
            vec![tool_seg_with_outputs(
                "get_active_build",
                "done",
                vec![
                    "{\"status\":\"no_build\",\"message\":\"No Path of Building build is currently loaded.\"}".into(),
                ],
            )],
        );
        let report = lint_run(&run);
        assert!(!report
            .findings
            .iter()
            .any(|f| f.id == "BUILD_IDENTITY_REQUIRED"));
    }

    #[test]
    fn build_identity_required_fires_on_build_specific_answer() {
        // Sprint D: even with the marker refinement, an answer that names
        // the player's build ("your build", "your DPS") still requires the
        // identity card when `get_active_build` succeeded.
        let run = run_with(
            "Your DPS is held back by missing aura effect on the helm enchant.",
            vec![tool_seg("get_active_build", "done")],
        );
        let report = lint_run(&run);
        assert!(report
            .findings
            .iter()
            .any(|f| f.id == "BUILD_IDENTITY_REQUIRED" && f.severity == FindingSeverity::Fail));
    }

    #[test]
    fn pob_calc_failure_real_dps_claim_fails() {
        let run = run_with(
            "Your real DPS against Pinnacle is around 14M.",
            vec![tool_seg("pob_calc", "failed")],
        );
        let report = lint_run(&run);
        assert!(report
            .findings
            .iter()
            .any(|f| f.id == "POB_CALC_FAILURE_NO_REAL_NUMBER"));
    }

    #[test]
    fn pob_calc_failure_with_disclaimer_passes() {
        let run = run_with(
            "The real DPS could not be recomputed (engine failed); the cached PoB headline is 14M but treat it as stale.",
            vec![tool_seg("pob_calc", "failed")],
        );
        let report = lint_run(&run);
        assert!(!report
            .findings
            .iter()
            .any(|f| f.id == "POB_CALC_FAILURE_NO_REAL_NUMBER"));
    }

    #[test]
    fn panel_data_first_passes_when_sidecar_leads() {
        let body = "\u{27E6}panel-data\u{27E7}\n```json\n{\"Mageblood\": {}}\n```\n\u{27E6}/panel-data\u{27E7}\n\nMageblood \u{27E6}panel*:item-card:Mageblood\u{27E7} is a Heavy Belt.";
        let run = run_with(body, vec![]);
        let report = lint_run(&run);
        assert!(!report.findings.iter().any(|f| f.id == "PANEL_DATA_FIRST"));
    }

    #[test]
    fn panel_data_first_fails_when_text_precedes() {
        let body = "Mageblood \u{27E6}panel*:item-card:Mageblood\u{27E7} is a Heavy Belt.";
        let run = run_with(body, vec![]);
        let report = lint_run(&run);
        assert!(report.findings.iter().any(|f| f.id == "PANEL_DATA_FIRST"));
    }

    #[test]
    fn panel_marker_payload_match_flags_missing_key() {
        let body = "\u{27E6}panel-data\u{27E7}\n```json\n{\"Mageblood\": {}}\n```\n\u{27E6}/panel-data\u{27E7}\nThe \u{27E6}panel*:item-card:Carcass Jack\u{27E7} provides AoE.";
        let run = run_with(body, vec![]);
        let report = lint_run(&run);
        let f = report
            .findings
            .iter()
            .find(|f| f.id == "PANEL_MARKER_PAYLOAD_MATCH")
            .expect("missing-key finding");
        assert!(f.message.contains("Carcass Jack"));
    }

    #[test]
    fn sources_fetched_only_flags_unfetched_host() {
        let body = "Mageblood is great.\n\nSources:\n- https://www.poewiki.net/wiki/Mageblood";
        let run = run_with(
            body,
            vec![tool_seg_with_outputs(
                "wiki_parse",
                "done",
                vec!["fetched https://www.poe2wiki.net/wiki/Spirit body...".into()],
            )],
        );
        let report = lint_run(&run);
        assert!(report
            .findings
            .iter()
            .any(|f| f.id == "SOURCES_FETCHED_ONLY"));
    }

    #[test]
    fn sources_fetched_only_passes_when_host_matches() {
        let body = "Mageblood is great.\n\nSources:\n- https://www.poewiki.net/wiki/Mageblood";
        let run = run_with(
            body,
            vec![tool_seg_with_outputs(
                "wiki_parse",
                "done",
                vec!["fetched https://www.poewiki.net/wiki/Mageblood body...".into()],
            )],
        );
        let report = lint_run(&run);
        assert!(!report
            .findings
            .iter()
            .any(|f| f.id == "SOURCES_FETCHED_ONLY"));
    }

    #[test]
    fn no_internal_doc_as_final_source_flags_internal_path() {
        let body = "The model.\n\nSources: prompts/references/24_patch_history_meta.md";
        let run = run_with(body, vec![]);
        let report = lint_run(&run);
        assert!(report
            .findings
            .iter()
            .any(|f| f.id == "NO_INTERNAL_DOC_AS_FINAL_SOURCE"));
    }

    #[test]
    fn no_blocked_source_flags_fandom() {
        let body = "Check https://pathofexile.fandom.com/wiki/Divine_Flesh for details.";
        let run = run_with(body, vec![]);
        let report = lint_run(&run);
        assert!(report
            .findings
            .iter()
            .any(|f| f.id == "NO_BLOCKED_SOURCE"));
    }

    #[test]
    fn failed_raw_data_no_table_flags_fabricated_table() {
        let body = "| Mod | Tier | ilvl |\n|---|---|---|\n| Life | T1 | ilvl 86 |\n| ES | T1 | ilvl 86 |";
        let run = run_with(body, vec![tool_seg("repoe_lookup", "failed")]);
        let report = lint_run(&run);
        assert!(report
            .findings
            .iter()
            .any(|f| f.id == "FAILED_RAW_DATA_NO_TABLE"));
    }

    #[test]
    fn failed_raw_data_no_table_passes_with_uncertainty() {
        let body = "I cannot confirm the exact mod pool because repoe_lookup failed. Roughly: T1 life, T1 ES, T1 evasion.";
        let run = run_with(body, vec![tool_seg("repoe_lookup", "failed")]);
        let report = lint_run(&run);
        assert!(!report
            .findings
            .iter()
            .any(|f| f.id == "FAILED_RAW_DATA_NO_TABLE"));
    }

    #[test]
    fn calcs_echo_required_flags_silent_dps() {
        let body = "Total DPS sits around 137,000.";
        let run = run_with(body, vec![tool_seg("pob_calc", "done")]);
        let report = lint_run(&run);
        assert!(report
            .findings
            .iter()
            .any(|f| f.id == "CALCS_ECHO_REQUIRED"));
    }

    #[test]
    fn calcs_echo_required_passes_with_boss_flag() {
        let body = "Total DPS against Pinnacle boss with active skill 'Penance Brand' is 137,000.";
        let run = run_with(body, vec![tool_seg("pob_calc", "done")]);
        let report = lint_run(&run);
        assert!(!report
            .findings
            .iter()
            .any(|f| f.id == "CALCS_ECHO_REQUIRED"));
    }

    #[test]
    fn clean_run_produces_no_findings() {
        let body = "\u{27E6}panel-data\u{27E7}\n```json\n{\"Mageblood\": {\"slot\": \"belt\"}}\n```\n\u{27E6}/panel-data\u{27E7}\n\nIdentity: defense=hybrid, hit_model=crit, mechanic=self-cast. Defining uniques: Mageblood (defining). Conversion: none.\n\nThe \u{27E6}panel*:item-card:Mageblood\u{27E7} pushes flask uptime against Pinnacle bosses with active skill Penance Brand.\n\nSources:\nhttps://www.poewiki.net/wiki/Mageblood";
        let run = run_with(
            body,
            vec![
                tool_seg("get_active_build", "done"),
                tool_seg("pob_calc", "done"),
                tool_seg_with_outputs(
                    "wiki_parse",
                    "done",
                    vec!["fetched https://www.poewiki.net/wiki/Mageblood".into()],
                ),
            ],
        );
        let report = lint_run(&run);
        assert!(
            !report.has_failures(),
            "clean run should have no fail findings, got {:?}",
            report.findings
        );
    }
}

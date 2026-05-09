//! Hybrid binding strategy:
//!
//!   - `fingerprint`: ascendancy + main-skill display name + sorted defining
//!     uniques. Stable across gear churn (one Marble Amulet swap leaves the
//!     fingerprint unchanged), so it's the lookup key on PoB attach. Two
//!     PoBs with the same fingerprint usually mean the same character.
//!
//!   - `pob_hash`: sha256 of the canonicalized PoB JSON. Differs as soon as
//!     any tracked field changes. When fingerprint matches but hash differs,
//!     we're in the "stale" state — same character, drifted gear.
//!
//! The PoB struct (`crate::pob::PobBuild`) drives both: callers pass the
//! parsed build plus the list of items the agent flagged as "defining" in
//! the Identity card, and we hash from there. We don't hash the raw XML
//! because callers may have already parsed and lost trailing whitespace
//! differences that don't actually change the build.

use sha2::{Digest, Sha256};

/// Canonicalize the inputs that constitute a build's identity and return a
/// fingerprint string of the form `<ascendancy>:<main_skill>:<defining_uniques>`
/// where the uniques are lowercased, sorted, and joined by `+`. This is a
/// stable shape so two callers passing the same set of inputs in different
/// orders produce the same fingerprint.
pub fn compute_fingerprint(
    ascendancy: &str,
    main_skill: &str,
    defining_uniques: &[String],
) -> String {
    let mut uniques: Vec<String> = defining_uniques
        .iter()
        .map(|u| u.trim().to_ascii_lowercase())
        .filter(|u| !u.is_empty())
        .collect();
    uniques.sort();
    uniques.dedup();
    let uniques_part = uniques.join("+");
    let asc = ascendancy.trim().to_ascii_lowercase();
    let skill = main_skill.trim().to_ascii_lowercase();
    format!("{asc}:{skill}:{uniques_part}")
}

/// Convenience wrapper that derives `compute_fingerprint`'s inputs straight
/// from a parsed `PobBuild`. Pulls all unique-rarity item names from the
/// build's items array — NOT just the role-tagged ones from the agent's
/// Identity card. The role tags (engine / defining / amplifier / enabler)
/// are an output concern that the LLM decides post-hoc; the fingerprint is
/// an identity-matching concern that must be deterministic from the parsed
/// build alone, so a future re-attach of the same PoB produces the same
/// string and `find_by_fingerprint` hits.
///
/// Returns `None` when the build lacks an ascendancy or main skill — the
/// fingerprint isn't usable in that case.
pub fn compute_fingerprint_from_pob(pob: &crate::pob::PobBuild) -> Option<String> {
    let asc = pob.ascendancy.as_deref().unwrap_or("").trim();
    let skill = pob.main_skill.as_deref().unwrap_or("").trim();
    if asc.is_empty() || skill.is_empty() {
        return None;
    }
    let uniques: Vec<String> = pob
        .items
        .iter()
        .filter(|i| i.rarity.as_deref().map(|r| r.eq_ignore_ascii_case("UNIQUE")).unwrap_or(false))
        .filter_map(|i| i.name.clone())
        .collect();
    Some(compute_fingerprint(asc, skill, &uniques))
}

/// SHA-256 the canonical JSON the caller provides. We hash a *string* rather
/// than re-serializing in here so the caller controls canonicalization (key
/// order, whitespace) — typically `serde_json::to_string` of the parsed
/// `PobBuild`. Returns lowercase hex.
pub fn compute_pob_hash(canonical_json: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(canonical_json.as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(digest.len() * 2);
    for b in digest.iter() {
        use std::fmt::Write;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_order_insensitive() {
        let a = compute_fingerprint(
            "Inquisitor",
            "Ice Nova of Frostbolts",
            &[
                "Brass Dome".to_string(),
                "Cospri's Will".to_string(),
            ],
        );
        let b = compute_fingerprint(
            "Inquisitor",
            "Ice Nova of Frostbolts",
            &[
                "Cospri's Will".to_string(),
                "Brass Dome".to_string(),
            ],
        );
        assert_eq!(a, b);
    }

    #[test]
    fn fingerprint_is_case_insensitive() {
        let a = compute_fingerprint(
            "Inquisitor",
            "Ice Nova of Frostbolts",
            &["Brass Dome".to_string()],
        );
        let b = compute_fingerprint(
            "INQUISITOR",
            "ice nova of frostbolts",
            &["BRASS DOME".to_string()],
        );
        assert_eq!(a, b);
    }

    #[test]
    fn fingerprint_dedupes_repeated_uniques() {
        let f = compute_fingerprint(
            "Inquisitor",
            "Ice Nova of Frostbolts",
            &[
                "Brass Dome".to_string(),
                "Brass Dome".to_string(),
            ],
        );
        assert!(f.ends_with(":brass dome"));
    }

    #[test]
    fn pob_hash_changes_with_input() {
        let a = compute_pob_hash(r#"{"life":4820}"#);
        let b = compute_pob_hash(r#"{"life":4821}"#);
        assert_ne!(a, b);
        assert_eq!(a.len(), 64);
    }
}

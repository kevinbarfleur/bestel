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

use serde_json::Value;
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
        .filter(|i| {
            i.rarity
                .as_deref()
                .map(|r| r.eq_ignore_ascii_case("UNIQUE"))
                .unwrap_or(false)
        })
        .filter_map(|i| i.name.clone())
        .collect();
    Some(compute_fingerprint(asc, skill, &uniques))
}

/// SHA-256 the canonical JSON the caller provides. We hash a *string* rather
/// than re-serializing in here so the caller controls canonicalization (key
/// order, whitespace) — typically `serde_json::to_string` of the parsed
/// `PobBuild`. Returns lowercase hex.
///
/// Prefer `compute_pob_hash_from_build` for `PobBuild` inputs — it handles
/// the items-vec reorder sensitivity that bit us before. This raw entry
/// stays for callers that hash other kinds of canonical strings.
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

/// Produce the canonical JSON used for hashing. Items are sorted by
/// `(rarity, name, base, whitespace-normalised raw)` so a PoB resave that
/// happens to write items in a different order does NOT flip the hash. All
/// other fields are serde-serialised verbatim — they already use
/// `BTreeMap` for stat / config maps, so they're stable.
pub fn canonicalise_for_hash(pob: &crate::pob::PobBuild) -> Value {
    let mut clone = pob.clone();
    clone.items.sort_by(|a, b| {
        let key = |it: &crate::pob::PobItem| {
            (
                it.rarity.clone().unwrap_or_default().to_ascii_lowercase(),
                it.name.clone().unwrap_or_default().to_ascii_lowercase(),
                it.base.clone().unwrap_or_default().to_ascii_lowercase(),
                normalize_whitespace(&it.raw),
            )
        };
        key(a).cmp(&key(b))
    });
    serde_json::to_value(&clone).unwrap_or(Value::Null)
}

/// Hash a `PobBuild` after canonicalising it. Cheap (one clone + sort + JSON
/// serialise + sha256) and the only correct path for sheet drift detection
/// — `serde_json::to_string(&build)` directly is order-fragile.
pub fn compute_pob_hash_from_build(pob: &crate::pob::PobBuild) -> String {
    let canonical = serde_json::to_string(&canonicalise_for_hash(pob)).unwrap_or_default();
    compute_pob_hash(&canonical)
}

fn normalize_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_was_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_was_space && !out.is_empty() {
                out.push(' ');
                prev_was_space = true;
            }
        } else {
            out.push(ch);
            prev_was_space = false;
        }
    }
    if out.ends_with(' ') {
        out.pop();
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
            &["Brass Dome".to_string(), "Cospri's Will".to_string()],
        );
        let b = compute_fingerprint(
            "Inquisitor",
            "Ice Nova of Frostbolts",
            &["Cospri's Will".to_string(), "Brass Dome".to_string()],
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
            &["Brass Dome".to_string(), "Brass Dome".to_string()],
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

    #[test]
    fn normalize_whitespace_collapses_runs() {
        assert_eq!(normalize_whitespace("a  b\nc\t\td"), "a b c d");
        assert_eq!(
            normalize_whitespace("  leading   middle trailing  "),
            "leading middle trailing"
        );
        assert_eq!(normalize_whitespace(""), "");
    }

    #[test]
    fn compute_pob_hash_from_build_is_item_order_insensitive() {
        // Two PoB builds that differ only in item-array order should
        // produce identical hashes. Before canonicalisation, a PoB resave
        // that reorders items flipped the hash and broke sheet drift
        // detection.
        let xml_a = br#"<?xml version="1.0"?>
<PathOfBuilding>
  <Build level="92" className="Witch" mainSocketGroup="1"/>
  <Items activeItemSet="1">
    <Item id="1">Rarity: UNIQUE
Mageblood
Heavy Belt</Item>
    <Item id="2">Rarity: UNIQUE
Watcher's Eye
Prismatic Jewel</Item>
    <ItemSet id="1"><Slot name="Belt" itemId="1"/></ItemSet>
  </Items>
</PathOfBuilding>"#;
        let xml_b = br#"<?xml version="1.0"?>
<PathOfBuilding>
  <Build level="92" className="Witch" mainSocketGroup="1"/>
  <Items activeItemSet="1">
    <Item id="2">Rarity: UNIQUE
Watcher's Eye
Prismatic Jewel</Item>
    <Item id="1">Rarity: UNIQUE
Mageblood
Heavy Belt</Item>
    <ItemSet id="1"><Slot name="Belt" itemId="1"/></ItemSet>
  </Items>
</PathOfBuilding>"#;
        let a =
            crate::pob::parser::parse_bytes(xml_a, std::path::PathBuf::from("same.xml")).unwrap();
        let b =
            crate::pob::parser::parse_bytes(xml_b, std::path::PathBuf::from("same.xml")).unwrap();
        // Direct serialization differs because Vec<PobItem> order differs.
        assert_ne!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
        // Canonicalised hash is identical.
        assert_eq!(
            compute_pob_hash_from_build(&a),
            compute_pob_hash_from_build(&b)
        );
    }

    #[test]
    fn compute_pob_hash_from_build_detects_real_changes() {
        // Sanity: actually changing an item's mod text should flip the hash.
        let xml_a = br#"<?xml version="1.0"?>
<PathOfBuilding>
  <Build level="92" className="Witch" mainSocketGroup="1"/>
  <Items activeItemSet="1">
    <Item id="1">Rarity: RARE
Mind Star
Praetor Crown
+100 to maximum Life</Item>
  </Items>
</PathOfBuilding>"#;
        let xml_b = br#"<?xml version="1.0"?>
<PathOfBuilding>
  <Build level="92" className="Witch" mainSocketGroup="1"/>
  <Items activeItemSet="1">
    <Item id="1">Rarity: RARE
Mind Star
Praetor Crown
+120 to maximum Life</Item>
  </Items>
</PathOfBuilding>"#;
        let a =
            crate::pob::parser::parse_bytes(xml_a, std::path::PathBuf::from("same.xml")).unwrap();
        let b =
            crate::pob::parser::parse_bytes(xml_b, std::path::PathBuf::from("same.xml")).unwrap();
        assert_ne!(
            compute_pob_hash_from_build(&a),
            compute_pob_hash_from_build(&b)
        );
    }
}

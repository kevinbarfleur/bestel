//! Five SHA-256 signatures over a parsed `PobBuild` that together describe
//! every axis of build drift relevant to a Build Sheet. Each signature is
//! orthogonal: a passive respec flips only `tree_sig`, a flask swap flips
//! only `gear_sig`, etc.
//!
//! Signatures live next to `pob_hash` (the canonical-JSON SHA-256 in
//! `sheets::fingerprint::compute_pob_hash`) but are NOT redundant with it.
//! `pob_hash` is one-bit "anything changed"; these five answer "what kind
//! of thing changed", which is what the drift indicator surfaces to the
//! user.
//!
//! Stable ordering matters: when the same build is hashed twice the same
//! bytes must come out. Every collection input is sorted by a deterministic
//! key before hashing.
//!
//! NOT a privacy-sensitive hash. These are visible in the SQLite registry
//! and surfaced to the agent; they describe build shape, not identity.

use sha2::{Digest, Sha256};

use super::PobBuild;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSignatures {
    pub identity: String,
    pub tree: String,
    pub gear: String,
    pub skill: String,
    pub config: String,
}

impl BuildSignatures {
    pub fn from_build(pob: &PobBuild) -> Self {
        Self {
            identity: identity_sig(pob),
            tree: tree_sig(pob),
            gear: gear_sig(pob),
            skill: skill_sig(pob),
            config: config_sig(pob),
        }
    }
}

/// Class + ascendancy + main-skill name. Flips when the player effectively
/// rerolled the character. The most stable signature: gear churn, tree
/// respecs, and config tweaks leave it unchanged.
pub fn identity_sig(pob: &PobBuild) -> String {
    let class = pob.class.trim().to_ascii_lowercase();
    let asc = pob.ascendancy.as_deref().unwrap_or("").trim().to_ascii_lowercase();
    let skill = pob.main_skill.as_deref().unwrap_or("").trim().to_ascii_lowercase();
    let game = pob.game.label();
    hash_str(&format!("{game}|{class}|{asc}|{skill}"))
}

/// Allocated passive node ids + class internals + mastery picks. A single
/// new node flips this signature. Mastery effects matter because they
/// frequently change build identity (e.g. "% spell suppression" mastery vs
/// "% mana" mastery on the same anchor).
pub fn tree_sig(pob: &PobBuild) -> String {
    let mut nodes = pob.allocated_nodes.clone();
    nodes.sort_unstable();
    nodes.dedup();
    let nodes_part = nodes
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let mut masteries: Vec<String> = pob
        .mastery_picks
        .iter()
        .map(|m| match m {
            super::MasteryPick::Poe1 { node_id, effect_id } => format!("p1:{node_id}:{effect_id}"),
            super::MasteryPick::Poe2 { effect_id } => format!("p2:{effect_id}"),
        })
        .collect();
    masteries.sort();
    masteries.dedup();
    let mastery_part = masteries.join(",");
    let class_id = pob.tree.class_id.map(|v| v.to_string()).unwrap_or_default();
    let asc_id = pob
        .tree.ascend_class_id
        .map(|v| v.to_string())
        .unwrap_or_default();
    hash_str(&format!(
        "tree|c={class_id}|a={asc_id}|n={nodes_part}|m={mastery_part}"
    ))
}

/// Item identity: every unique by name + every rare's slot-keyed canonical
/// body. Rares hash via their `raw` blob's normalized whitespace so a save
/// that only retypes a mod doesn't flip the signature, but a tier upgrade
/// does. Uniques are name-only because the game guarantees their stats by
/// name — re-rolling Mageblood's implicits doesn't change "this is Mageblood".
pub fn gear_sig(pob: &PobBuild) -> String {
    let mut entries: Vec<String> = pob
        .items
        .iter()
        .map(|i| {
            let rarity = i
                .rarity
                .as_deref()
                .unwrap_or("")
                .to_ascii_uppercase();
            if rarity == "UNIQUE" {
                let name = i.name.as_deref().unwrap_or("").trim().to_ascii_lowercase();
                format!("u:{name}")
            } else {
                let base = i.base.as_deref().unwrap_or("").trim().to_ascii_lowercase();
                let body_hash = hash_str(&normalize_ws(&i.raw));
                format!("r:{rarity}:{base}:{body_hash}")
            }
        })
        .filter(|s| !s.is_empty())
        .collect();
    entries.sort();
    entries.dedup();
    let mut slots: Vec<String> = pob
        .slot_map
        .iter()
        .map(|(k, v)| format!("{}:{}", k.to_ascii_lowercase(), v))
        .collect();
    slots.sort();
    let slots_part = slots.join(",");
    hash_str(&format!("gear|{}|slots={slots_part}", entries.join(";")))
}

/// Main-skill gem + supports + gem levels. Catches "I swapped Hypothermia
/// for Slower Projectiles" cleanly without flipping for an unrelated tree
/// respec. Includes both main and ancillary skill groups because changing
/// an aura support (Generosity vs not) can flip build math.
pub fn skill_sig(pob: &PobBuild) -> String {
    let mut groups: Vec<String> = pob
        .skill_groups
        .iter()
        .map(|g| {
            let mut gems: Vec<String> = g
                .gems
                .iter()
                .filter(|gem| !gem.name.trim().is_empty())
                .map(|gem| {
                    let name = gem.name.trim().to_ascii_lowercase();
                    let level = gem.level.unwrap_or(0);
                    let enabled = if gem.enabled { "1" } else { "0" };
                    format!("{name}@{level}/{enabled}")
                })
                .collect();
            gems.sort();
            gems.dedup();
            let label = g.label.trim().to_ascii_lowercase();
            let main = if g.is_main { "M" } else { "S" };
            let slot = g.slot.as_deref().unwrap_or("").to_ascii_lowercase();
            format!("{main}:{label}:{slot}:[{}]", gems.join(","))
        })
        .filter(|s| !s.is_empty())
        .collect();
    groups.sort();
    groups.dedup();
    hash_str(&format!("skill|{}", groups.join(";")))
}

/// Calcs config: boss profile, charges, flasks, buffs, custom mods. Flips
/// when the user changes "what fight am I simming". Doesn't include any
/// gear or skill state — those have their own signatures.
pub fn config_sig(pob: &PobBuild) -> String {
    let charges = format!(
        "p={}/{}|f={}/{}|e={}/{}",
        pob.charges.power.current,
        pob.charges.power.max,
        pob.charges.frenzy.current,
        pob.charges.frenzy.max,
        pob.charges.endurance.current,
        pob.charges.endurance.max
    );
    let mut inputs: Vec<String> = pob
        .config
        .inputs
        .iter()
        .map(|(k, v)| format!("{}={}", k.to_ascii_lowercase(), v.trim().to_ascii_lowercase()))
        .collect();
    inputs.sort();
    let inputs_part = inputs.join(",");
    let mut custom = pob.config.custom_mods.clone();
    custom.sort();
    custom.dedup();
    let custom_part = custom.join(";");
    let mut buffs = pob.buffs.buffs.clone();
    buffs.sort();
    buffs.dedup();
    let mut combat = pob.buffs.combat.clone();
    combat.sort();
    combat.dedup();
    let mut curses = pob.buffs.curses.clone();
    curses.sort();
    curses.dedup();
    let buffs_part = format!(
        "b=[{}]|c=[{}]|x=[{}]",
        buffs.join(","),
        combat.join(","),
        curses.join(",")
    );
    let active_set = pob
        .config
        .active_set_id
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();
    hash_str(&format!(
        "config|set={active_set}|{charges}|in={inputs_part}|mods={custom_part}|{buffs_part}"
    ))
}

/// SHA-256 the given string, return 64-char lowercase hex.
fn hash_str(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(digest.len() * 2);
    for b in digest.iter() {
        use std::fmt::Write;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

/// Collapse all runs of whitespace to single spaces and trim. Stable canon
/// for an item's `raw` text so re-saving the same build in PoB doesn't flip
/// `gear_sig` due to formatting noise.
fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pob::parser::parse_bytes;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PobBuild {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("tests")
            .join("fixtures")
            .join("pob")
            .join(name);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| {
            panic!("fixture {} not found: {e}", path.display());
        });
        parse_bytes(&bytes, path).expect("parse")
    }

    #[test]
    fn signatures_are_deterministic_on_same_build() {
        let a = fixture("poe1_inquisitor.xml");
        let b = fixture("poe1_inquisitor.xml");
        let sa = BuildSignatures::from_build(&a);
        let sb = BuildSignatures::from_build(&b);
        assert_eq!(sa, sb);
    }

    #[test]
    fn signatures_are_64_char_hex() {
        let b = fixture("poe1_inquisitor.xml");
        let s = BuildSignatures::from_build(&b);
        for sig in [&s.identity, &s.tree, &s.gear, &s.skill, &s.config] {
            assert_eq!(sig.len(), 64, "sig should be 64 hex chars: {sig}");
            assert!(sig.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn tree_sig_flips_when_node_added() {
        let mut b = fixture("poe1_inquisitor.xml");
        let before = tree_sig(&b);
        b.allocated_nodes.push(99_999_999);
        let after = tree_sig(&b);
        assert_ne!(before, after);
    }

    #[test]
    fn identity_sig_stable_across_gear_changes() {
        let mut b = fixture("poe1_inquisitor.xml");
        let before = identity_sig(&b);
        b.items.clear();
        let after = identity_sig(&b);
        assert_eq!(before, after);
    }

    #[test]
    fn skill_sig_flips_when_main_skill_renamed() {
        let mut b = fixture("poe1_inquisitor.xml");
        let before = skill_sig(&b);
        if let Some(g) = b.skill_groups.iter_mut().find(|g| g.is_main) {
            g.label = format!("{}_renamed", g.label);
        }
        let after = skill_sig(&b);
        assert_ne!(before, after);
    }

    #[test]
    fn config_sig_flips_when_charges_change() {
        let mut b = fixture("poe1_inquisitor.xml");
        let before = config_sig(&b);
        b.charges.power.current = b.charges.power.current.wrapping_add(1);
        let after = config_sig(&b);
        assert_ne!(before, after);
    }

    #[test]
    fn gear_sig_flips_when_unique_swapped() {
        let mut b = fixture("poe1_inquisitor.xml");
        let before = gear_sig(&b);
        if let Some(u) = b
            .items
            .iter_mut()
            .find(|i| i.rarity.as_deref().map(|r| r.eq_ignore_ascii_case("UNIQUE")).unwrap_or(false))
        {
            u.name = u.name.as_ref().map(|n| format!("{n} (Replica)"));
        }
        let after = gear_sig(&b);
        assert_ne!(before, after);
    }
}

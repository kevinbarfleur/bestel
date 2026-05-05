//! Static dictionary of passive-tree node metadata, embedded at compile-time.
//!
//! Powers the build-panel hover tooltips and the rich notable/keystone
//! lists. Source data lives at `crates/bestel-core/data/passive_nodes_*.json`
//! and is hand-curated for the most popular nodes — see
//! `download-passive-nodes.ps1` for the (placeholder) regen script.
//!
//! The XML's `<Spec nodes="N1,N2,...">` list is just numeric ids ; this
//! module is what turns those into human-readable keystone names with
//! descriptions for the UI.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::PoeVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Keystone,
    Notable,
    Normal,
    Mastery,
    JewelSocket,
    AscendancyKeystone,
    AscendancyNotable,
    AscendancySmall,
}

impl NodeKind {
    pub fn label(self) -> &'static str {
        match self {
            NodeKind::Keystone => "Keystone",
            NodeKind::Notable => "Notable",
            NodeKind::Normal => "Passive",
            NodeKind::Mastery => "Mastery",
            NodeKind::JewelSocket => "Jewel socket",
            NodeKind::AscendancyKeystone => "Ascendancy keystone",
            NodeKind::AscendancyNotable => "Ascendancy notable",
            NodeKind::AscendancySmall => "Ascendancy",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeInfo {
    pub id: u32,
    pub name: String,
    pub kind: NodeKind,
    pub description: String,
    #[serde(default)]
    pub ascendancy: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NodeFile {
    nodes: Vec<NodeInfo>,
}

const POE1_BYTES: &[u8] = include_bytes!("../../data/passive_nodes_poe1.json");
const POE2_BYTES: &[u8] = include_bytes!("../../data/passive_nodes_poe2.json");

static POE1_DICT: OnceLock<HashMap<u32, NodeInfo>> = OnceLock::new();
static POE2_DICT: OnceLock<HashMap<u32, NodeInfo>> = OnceLock::new();

fn dict_for(game: PoeVersion) -> &'static HashMap<u32, NodeInfo> {
    let (cell, bytes) = match game {
        PoeVersion::Poe1 => (&POE1_DICT, POE1_BYTES),
        PoeVersion::Poe2 => (&POE2_DICT, POE2_BYTES),
    };
    cell.get_or_init(|| match serde_json::from_slice::<NodeFile>(bytes) {
        Ok(f) => f
            .nodes
            .into_iter()
            .map(|n| (n.id, n))
            .collect::<HashMap<_, _>>(),
        Err(e) => {
            eprintln!(
                "[bestel-core] failed to load passive-node dict for {:?}: {}",
                game, e
            );
            HashMap::new()
        }
    })
}

/// Look up a single node by id. Returns `None` if the id isn't in the seed
/// dictionary (which only covers the most popular nodes — see
/// `download-passive-nodes.ps1` for full coverage).
pub fn lookup_node(game: PoeVersion, id: u32) -> Option<&'static NodeInfo> {
    dict_for(game).get(&id)
}

fn filter_kind<'a>(
    game: PoeVersion,
    ids: &'a [u32],
    pred: impl Fn(NodeKind) -> bool + 'a,
) -> Vec<&'static NodeInfo> {
    let dict = dict_for(game);
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for id in ids {
        if !seen.insert(*id) {
            continue;
        }
        if let Some(info) = dict.get(id) {
            if pred(info.kind) {
                out.push(info);
            }
        }
    }
    out
}

/// Keystones in the player's allocation, in the order they appear.
pub fn keystones_in(game: PoeVersion, ids: &[u32]) -> Vec<&'static NodeInfo> {
    filter_kind(game, ids, |k| k == NodeKind::Keystone)
}

/// Notables in the player's allocation, in the order they appear.
pub fn notables_in(game: PoeVersion, ids: &[u32]) -> Vec<&'static NodeInfo> {
    filter_kind(game, ids, |k| k == NodeKind::Notable)
}

/// Ascendancy nodes (keystones + notables + small) in the player's allocation.
pub fn ascendancy_in(game: PoeVersion, ids: &[u32]) -> Vec<&'static NodeInfo> {
    filter_kind(game, ids, |k| {
        matches!(
            k,
            NodeKind::AscendancyKeystone
                | NodeKind::AscendancyNotable
                | NodeKind::AscendancySmall
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_poe1_seed() {
        let info = lookup_node(PoeVersion::Poe1, 5934).expect("Resolute Technique");
        assert_eq!(info.name, "Resolute Technique");
        assert_eq!(info.kind, NodeKind::Keystone);
        assert!(info.description.contains("Evaded"));
    }

    #[test]
    fn loads_poe2_seed() {
        let info = lookup_node(PoeVersion::Poe2, 1143).expect("PoE2 Resolute Technique");
        assert_eq!(info.kind, NodeKind::Keystone);
    }

    #[test]
    fn filters_keystones() {
        let ids = vec![5934, 65532, 9408, 38085]; // RT, MoM, Doryani's Lesson, Acrobatics
        let ks = keystones_in(PoeVersion::Poe1, &ids);
        assert_eq!(ks.len(), 3);
        assert!(ks.iter().any(|n| n.name == "Resolute Technique"));
        assert!(ks.iter().any(|n| n.name == "Mind Over Matter"));
        assert!(ks.iter().any(|n| n.name == "Acrobatics"));
    }

    #[test]
    fn filters_ascendancy() {
        let ids = vec![49254, 16221, 5934];
        let asc = ascendancy_in(PoeVersion::Poe1, &ids);
        assert!(asc.iter().any(|n| n.name == "Inevitable Judgement"));
        assert!(asc.iter().any(|n| n.name == "Sanctuary"));
        // Resolute Technique is a regular keystone, not an ascendancy node.
        assert!(asc.iter().all(|n| n.name != "Resolute Technique"));
    }

    #[test]
    fn unknown_id_returns_none() {
        assert!(lookup_node(PoeVersion::Poe1, 999_999).is_none());
    }
}

use std::path::{Path, PathBuf};

use super::PoeVersion;

#[derive(Debug, Clone)]
pub struct PobInstall {
    pub game: PoeVersion,
    pub builds_dir: PathBuf,
}

pub fn find_pob_dirs() -> Vec<PobInstall> {
    let docs = match dirs::document_dir() {
        Some(p) => p,
        None => return Vec::new(),
    };

    let candidates = [
        (PoeVersion::Poe1, "Path of Building"),
        (PoeVersion::Poe1, "Path of Building Community"),
        (PoeVersion::Poe2, "Path of Building (PoE2)"),
        (PoeVersion::Poe2, "Path of Building Community (PoE2)"),
    ];

    let mut found: Vec<PobInstall> = Vec::new();
    for (game, sub) in candidates {
        let dir = docs.join(sub).join("Builds");
        if dir.is_dir() {
            found.push(PobInstall {
                game,
                builds_dir: dir,
            });
        }
    }
    found
}

pub fn most_recent_build(dir: &Path) -> Option<PathBuf> {
    let mut best: Option<(PathBuf, std::time::SystemTime)> = None;
    let walker = walk_xml(dir);
    for path in walker {
        if let Ok(meta) = std::fs::metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                match &best {
                    Some((_, t)) if *t >= mtime => {}
                    _ => best = Some((path, mtime)),
                }
            }
        }
    }
    best.map(|(p, _)| p)
}

fn walk_xml(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_xml_inner(dir, &mut out);
    out
}

fn walk_xml_inner(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_xml_inner(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("xml") {
            out.push(path);
        }
    }
}

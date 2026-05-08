//! Ensures every required snapshot blob under `src/sources/snapshots/`
//! exists at compile time. If a real snapshot is missing (e.g. before the
//! first run of `scripts/refresh-snapshots.ps1`), this writes a tiny
//! zstd-compressed `{}` stub so `include_bytes!` resolves cleanly.
//!
//! The runtime catalogue treats an empty-object stub as "no entries" — the
//! refresh task populates real data on first online launch.

use std::fs;
use std::path::Path;

fn main() {
    let stub = zstd::encode_all(b"{}".as_ref(), 19).expect("zstd-compress empty stub");
    let snapshots: &[&str] = &[
        "src/sources/snapshots/poe1/mods.json.zst",
        "src/sources/snapshots/poe1/base_items.json.zst",
        "src/sources/snapshots/poe1/gems.json.zst",
        "src/sources/snapshots/poe1/uniques.json.zst",
        "src/sources/snapshots/poe1/cluster_jewels.json.zst",
        "src/sources/snapshots/poe1/essences.json.zst",
        "src/sources/snapshots/poe1/fossils.json.zst",
        "src/sources/snapshots/poe1/stat_translations.json.zst",
        "src/sources/snapshots/poe2/mods.json.zst",
        "src/sources/snapshots/poe2/base_items.json.zst",
        "src/sources/snapshots/poe2/gems.json.zst",
        "src/sources/snapshots/poe2/uniques.json.zst",
        "src/sources/snapshots/poe2/stat_translations.json.zst",
        "src/sources/snapshots/trade_stats/poe1.json.zst",
        "src/sources/snapshots/trade_stats/poe2.json.zst",
    ];
    for rel in snapshots {
        let path = Path::new(rel);
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create snapshot parent dir");
            }
            fs::write(path, &stub).expect("write snapshot stub");
            println!("cargo:warning=wrote stub {}", path.display());
        }
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

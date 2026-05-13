//! Diagnose why `repoe_lookup category=base_items name="Stellar Amulet"`
//! returned no results in the chat UI. Prints:
//! - Total entry count in the bundled poe1 base_items snapshot.
//! - First 3 raw entries (so we can see the actual schema).
//! - Direct lookup_by_id for the canonical Stellar Amulet metadata path.
//! - lookup_by_name "Stellar Amulet" with limit=5 and the resulting scores.

use bestel_core::sources::repoe::{Category, Game, RepoeClient};
use bestel_core::sources::FileCache;

fn main() -> anyhow::Result<()> {
    // Use the real on-disk cache so we hit the refreshed snapshot, not the
    // bundled fallback.
    let cache_dir = FileCache::default_dir();
    println!("cache_dir = {}", cache_dir.display());
    std::fs::create_dir_all(&cache_dir)?;
    let client = RepoeClient::new(FileCache::new(cache_dir));
    let snap = client.snapshot(Game::Poe1, Category::BaseItems)?;
    println!(
        "snapshot source={:?} fetched_at={} entries={}",
        snap.source,
        snap.fetched_at,
        snap.entries.len()
    );

    println!("\n--- first 3 raw entries (key + first 200 chars of value) ---");
    for (i, (k, v)) in snap.entries.iter().take(3).enumerate() {
        let s = serde_json::to_string(v).unwrap_or_default();
        let cut = if s.len() > 200 { &s[..200] } else { &s };
        println!("[{i}] {k}\n    {cut}");
    }

    println!("\n--- direct id lookup ---");
    let id = "Metadata/Items/Amulets/AmuletStellar";
    match client.lookup_by_id(Game::Poe1, Category::BaseItems, id) {
        Ok(r) => println!(
            "by_id '{id}': {} entries returned, first: {}",
            r.entries.len(),
            r.entries
                .first()
                .map(|e| serde_json::to_string(&e.value).unwrap_or_default())
                .unwrap_or_else(|| "<none>".into())
        ),
        Err(e) => println!("by_id '{id}': ERROR {e}"),
    }

    println!("\n--- fuzzy name lookup ---");
    let r = client.lookup_by_name(Game::Poe1, Category::BaseItems, "Stellar Amulet", 5)?;
    println!("hits: {}", r.entries.len());
    for e in &r.entries {
        let preview = serde_json::to_string(&e.value).unwrap_or_default();
        let preview = if preview.len() > 150 {
            &preview[..150]
        } else {
            &preview
        };
        println!("  score={:.3}  id={}  value={}", e.score, e.id, preview);
    }

    println!("\n--- scan: any key matching 'AmuletStellar' substring ---");
    let mut matches: Vec<&String> = snap
        .entries
        .keys()
        .filter(|k| k.to_lowercase().contains("amuletstellar"))
        .collect();
    matches.sort();
    for k in matches.iter().take(10) {
        println!("  {k}");
    }

    println!("\n--- scan: any value containing 'Stellar Amulet' ---");
    let mut name_matches: Vec<&String> = snap
        .entries
        .iter()
        .filter(|(_, v)| {
            serde_json::to_string(v)
                .map(|s| s.contains("Stellar Amulet"))
                .unwrap_or(false)
        })
        .map(|(k, _)| k)
        .collect();
    name_matches.sort();
    for k in name_matches.iter().take(10) {
        println!("  {k}");
    }
    println!("(total name-match: {})", name_matches.len());

    println!("\n--- all amulet base ids ---");
    let mut amulet_keys: Vec<&String> = snap
        .entries
        .keys()
        .filter(|k| k.contains("/Amulet"))
        .collect();
    amulet_keys.sort();
    for k in &amulet_keys {
        let name = snap
            .entries
            .get(*k)
            .and_then(|v| v.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("<no name>");
        println!("  {k:60}  name={name}");
    }
    println!("(total amulet entries: {})", amulet_keys.len());

    println!("\n--- search 'Stellar' anywhere in keys or values ---");
    let mut hits = 0usize;
    for (k, v) in &snap.entries {
        let blob = serde_json::to_string(v).unwrap_or_default();
        if k.contains("Stellar") || blob.contains("Stellar") {
            hits += 1;
            if hits <= 10 {
                let cut = if blob.len() > 200 {
                    &blob[..200]
                } else {
                    &blob
                };
                println!("  {k}\n    {cut}");
            }
        }
    }
    println!("(total 'Stellar' hits in base_items: {hits})");

    Ok(())
}

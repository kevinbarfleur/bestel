//! Live smoke test for Phase 3 source clients.
//! `cargo run --example sources_smoke -p bestel-core`
//!
//! Hits real endpoints (wiki + trade data). Prints summaries; no asserts —
//! this is a manual sanity check, not a unit test.

use anyhow::Result;
use bestel_core::pob::PoeVersion;
use bestel_core::sources::{FileCache, PoeHttpClient, TradeClient, WikiClient};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let http = PoeHttpClient::new()?;
    println!("User-Agent: {}", http.user_agent());

    let cache_dir = std::env::temp_dir().join("bestel-smoke-cache");
    let cache = FileCache::new(cache_dir.clone());
    println!("Cache dir: {}\n", cache_dir.display());

    // === Wiki search + parse ===
    let wiki = WikiClient::new(http.clone(), cache.clone(), PoeVersion::Poe1);
    println!("=== wiki_search('Resistance', poe1) ===");
    match wiki.search("Resistance", 3).await {
        Ok(hits) => {
            for h in &hits {
                println!("  · {}\n    {}\n    {:.120}…", h.title, h.url, h.snippet);
            }
            if let Some(top) = hits.first() {
                println!("\n=== wiki_parse('{}') ===", top.title);
                match wiki.parse(&top.title).await {
                    Ok(p) => {
                        println!("  url: {}", p.url);
                        println!("  sections: {}", p.sections.len());
                        println!(
                            "  text preview: {:.300}…",
                            p.plain_text.chars().take(300).collect::<String>()
                        );
                    }
                    Err(e) => println!("  parse error: {e}"),
                }
            }
        }
        Err(e) => println!("  search error: {e}"),
    }

    // === Wiki cargo ===
    println!("\n=== wiki_cargo(items, name='Tabula Rasa') ===");
    match wiki
        .cargo(
            "items",
            &["name", "base_item", "drop_level"],
            Some("name=\"Tabula Rasa\""),
            5,
        )
        .await
    {
        Ok(rows) => {
            println!("  rows: {}", rows.len());
            for r in rows.iter().take(3) {
                println!("  · {}", r);
            }
        }
        Err(e) => println!("  cargo error: {e}"),
    }

    // === Trade leagues + resolve ===
    let trade = TradeClient::new(http.clone(), cache.clone(), PoeVersion::Poe1);
    println!("\n=== trade leagues (poe1) ===");
    match trade.leagues().await {
        Ok(ls) => {
            for l in ls.iter().take(5) {
                println!("  · {l}");
            }
        }
        Err(e) => println!("  leagues error: {e}"),
    }

    println!("\n=== trade_resolve_stats('maximum life', prefer_pseudo=true) ===");
    match trade.resolve("maximum life", true, 5).await {
        Ok(hits) => {
            for h in &hits {
                println!("  · [{}] {} → {}", h.stat_type, h.text, h.id);
            }
        }
        Err(e) => println!("  resolve error: {e}"),
    }

    println!("\nDone.");
    Ok(())
}

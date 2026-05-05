//! Live smoke test for `find_synergies`.
//! `cargo run --example synergies_smoke -p bestel-core --release [-- topic [game] [limit]]`
//!
//! Hits the real PoE wiki. Prints the top synergies as a markdown list.

use anyhow::Result;
use bestel_core::llm::wiki::find_synergies;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let topic = args.next().unwrap_or_else(|| "Divine Flesh".to_string());
    let game = args.next().unwrap_or_else(|| "poe1".to_string());
    let limit: usize = args
        .next()
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(40);

    println!("find_synergies(topic={topic:?}, game={game}, limit={limit})\n");
    let json = find_synergies(&topic, &game, limit).await?;
    let parsed: serde_json::Value = serde_json::from_str(&json)?;
    println!("source: {}", parsed["source"].as_str().unwrap_or(""));
    println!("count : {}\n", parsed["count"]);
    if let Some(arr) = parsed["results"].as_array() {
        for (i, r) in arr.iter().enumerate() {
            let title = r["title"].as_str().unwrap_or("");
            let kind = r["kind"].as_str().unwrap_or("");
            let ann = r["annotation"].as_str();
            let url = r["url"].as_str().unwrap_or("");
            let badge = match ann {
                Some(a) => format!(" [{a}]"),
                None => String::new(),
            };
            println!("{:>3}. ({kind}){badge} {title}\n     {url}", i + 1);
        }
    }
    Ok(())
}

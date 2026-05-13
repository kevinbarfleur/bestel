//! `find_synergies` — query the wiki's `Special:WhatLinksHere` reverse-link
//! index to surface pages that *interact with* a topic (uniques, keystones,
//! cluster jewel notables, ascendancy nodes) even when the user did not name
//! them. The plain wiki page only mentions sources/upgrades; the reverse-link
//! index is the closest thing PoE has to a synergy graph.

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde_json::json;

const USER_AGENT: &str = concat!(
    "bestel/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/kevinbarfleur/bestel)"
);

pub async fn find_synergies(topic: &str, game: &str, limit: usize) -> Result<String> {
    let topic = topic.trim();
    if topic.is_empty() {
        return Err(anyhow!("topic must not be empty"));
    }
    let host = match game.to_ascii_lowercase().as_str() {
        "poe2" | "poe 2" | "pathofexile2" => "poe2wiki.net",
        _ => "poewiki.net",
    };

    let title = topic.replace(' ', "_");
    let url = format!(
        "https://www.{host}/index.php?title=Special:WhatLinksHere/{}&limit=500",
        urlencoding::encode(&title),
    );

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .build()
        .context("build http client")?;

    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?
        .error_for_status()
        .with_context(|| format!("status from {url}"))?;
    let html = resp.text().await.context("read body")?;

    let entries = parse_whatlinkshere(&html);
    let kept: Vec<Entry> = entries
        .into_iter()
        .filter(|e| keep_entry(&e.title))
        .take(limit)
        .collect();

    let host_for_url = host;
    let results: Vec<_> = kept
        .iter()
        .map(|e| {
            let slug = e.title.replace(' ', "_");
            json!({
                "title": e.title,
                "kind": classify(&e.title),
                "annotation": e.annotation,
                "url": format!("https://www.{host_for_url}/wiki/{slug}"),
            })
        })
        .collect();

    let total = results.len();
    let payload = json!({
        "topic": topic,
        "game": if host == "poe2wiki.net" { "poe2" } else { "poe1" },
        "source": url,
        "count": total,
        "results": results,
        "note": "These are pages that link TO the topic on the wiki. They include uniques, passive skills, cluster jewel notables, mechanics pages, modifiers, and category pages. Use this to surface synergies the user did not name. Open the most relevant ones with web_fetch to confirm the interaction."
    });
    Ok(serde_json::to_string(&payload).unwrap_or_default())
}

#[derive(Debug)]
struct Entry {
    title: String,
    annotation: Option<&'static str>,
}

fn parse_whatlinkshere(html: &str) -> Vec<Entry> {
    let Some(list_start) = html.find("mw-whatlinkshere-list") else {
        return Vec::new();
    };
    let after = &html[list_start..];
    let list_end = after
        .find("</ul>")
        .map(|e| list_start + e)
        .unwrap_or(html.len());
    let block = &html[list_start..list_end];

    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();

    for li in block.split("<li") {
        let Some(href_start_off) = li.find("href=\"/wiki/") else {
            continue;
        };
        let href_start = href_start_off + "href=\"/wiki/".len();
        let Some(href_end_off) = li[href_start..].find('"') else {
            continue;
        };
        let raw_slug = &li[href_start..href_start + href_end_off];

        // Drop fragment anchors (`#section`) and query strings.
        let raw_slug = raw_slug.split('#').next().unwrap_or(raw_slug);
        let raw_slug = raw_slug.split('?').next().unwrap_or(raw_slug);
        if raw_slug.is_empty() {
            continue;
        }

        let title = urlencoding::decode(raw_slug)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| raw_slug.to_string())
            .replace('_', " ");

        if !seen.insert(title.clone()) {
            continue;
        }

        let annotation = if li.contains("(transclusion)") {
            Some("transclusion")
        } else if li.contains("(redirect page)") {
            Some("redirect")
        } else {
            None
        };

        out.push(Entry { title, annotation });
    }

    out
}

fn keep_entry(title: &str) -> bool {
    let lower = title.to_ascii_lowercase();
    const SKIP_NS: &[&str] = &[
        "user:",
        "user talk:",
        "talk:",
        "file:",
        "file talk:",
        "template:",
        "template talk:",
        "category:",
        "category talk:",
        "help:",
        "help talk:",
        "special:",
        "module:",
        "module talk:",
        "mediawiki:",
        "mediawiki talk:",
        "path of exile wiki:",
        "path of exile 2 wiki:",
    ];
    !SKIP_NS.iter().any(|p| lower.starts_with(p))
}

fn classify(title: &str) -> &'static str {
    let lower = title.to_ascii_lowercase();
    if lower.starts_with("passive skill:") {
        "passive_skill"
    } else if lower.starts_with("modifier:") {
        "modifier"
    } else if lower.starts_with("monster:") {
        "monster"
    } else if lower.starts_with("npc:") {
        "npc"
    } else if lower.starts_with("legacy:") {
        "legacy"
    } else if lower.starts_with("recipe:") {
        "recipe"
    } else {
        "page"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_extracts_titles_dedup_and_skips_meta() {
        let html = r#"
            <html><body>
            <ul class="mw-whatlinkshere-list">
                <li><a href="/wiki/The_Fourth_Vow" title="The Fourth Vow">The Fourth Vow</a></li>
                <li><a href="/wiki/Mahuxotl%27s_Machination" title="Mahuxotl's Machination">Mahuxotl's Machination</a></li>
                <li><a href="/wiki/Passive_Skill:Affliction~notable~born~of~chaos~">Born of Chaos</a> (transclusion)</li>
                <li><a href="/wiki/The_Fourth_Vow" title="The Fourth Vow">The Fourth Vow</a></li>
                <li><a href="/wiki/Special:WhatLinksHere/Foo">noise</a></li>
                <li><a href="/wiki/Template:Item">tpl</a></li>
            </ul>
            </body></html>
        "#;
        let entries = parse_whatlinkshere(html);
        let kept: Vec<&Entry> = entries.iter().filter(|e| keep_entry(&e.title)).collect();
        assert!(kept.iter().any(|e| e.title == "The Fourth Vow"));
        assert!(kept.iter().any(|e| e.title == "Mahuxotl's Machination"));
        assert!(kept.iter().any(|e| {
            e.title == "Passive Skill:Affliction~notable~born~of~chaos~"
                && e.annotation == Some("transclusion")
        }));
        assert!(!kept.iter().any(|e| e.title.starts_with("Special:")));
        assert!(!kept.iter().any(|e| e.title.starts_with("Template:")));
        assert_eq!(
            kept.iter().filter(|e| e.title == "The Fourth Vow").count(),
            1,
            "dedup failed"
        );
    }

    #[test]
    fn classify_namespace_prefixes() {
        assert_eq!(classify("Passive Skill:foo"), "passive_skill");
        assert_eq!(classify("Modifier:bar"), "modifier");
        assert_eq!(classify("Monster:Sirus"), "monster");
        assert_eq!(classify("The Fourth Vow"), "page");
    }
}

//! PoEDB / POE2DB. Phase 3 keeps this as a link-only stub. Phase 4 will
//! add HTML scraping with caching once we know which pages the LLM needs.

use crate::pob::PoeVersion;

pub fn page_url(game: PoeVersion, slug: &str) -> String {
    let host = match game {
        PoeVersion::Poe1 => "poedb.tw/us",
        PoeVersion::Poe2 => "poe2db.tw/us",
    };
    format!("https://{host}/{slug}")
}

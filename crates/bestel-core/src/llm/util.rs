use serde_json::Value;

const MAX_QUERY_LEN: usize = 60;
const MAX_WHERE_LEN: usize = 80;

/// Format the tool call arguments as a compact human-readable detail
/// string for the delta stream. Per-tool match arms produce a tailored
/// view (just the title for `wiki_parse`, the query for `kb_search`,
/// etc.); unknown tools fall back to a generic `key=value` joiner.
///
/// Kept short and one-line so the UI can render it inline next to the
/// tool name without truncation drama.
pub fn summarize_tool_args(name: &str, args: &Value) -> Option<String> {
    let obj = args.as_object()?;
    if obj.is_empty() {
        return None;
    }

    let s_str = |key: &str| -> Option<&str> { obj.get(key)?.as_str() };
    let s_owned = |key: &str| -> Option<String> { s_str(key).map(|s| s.to_string()) };

    match name {
        "wiki_search" => s_owned("query").map(|q| quote_truncate(&q, MAX_QUERY_LEN)),
        "wiki_parse" => s_owned("title"),
        "wiki_cargo" => {
            let table = s_owned("table");
            let where_clause = s_owned("where").map(|w| truncate(&w, MAX_WHERE_LEN));
            match (table, where_clause) {
                (Some(t), Some(w)) => Some(format!("{t}: {w}")),
                (Some(t), None) => Some(t),
                (None, Some(w)) => Some(w),
                (None, None) => None,
            }
        }
        "kb_search" => {
            let q = s_owned("query")?;
            let k = obj
                .get("top_k")
                .and_then(|v| v.as_u64());
            Some(match k {
                Some(k) => format!("{} (top_k={k})", quote_truncate(&q, MAX_QUERY_LEN)),
                None => quote_truncate(&q, MAX_QUERY_LEN),
            })
        }
        "load_skill" => s_owned("name").or_else(|| s_owned("skill")),
        "pob_calc" => {
            let category = s_owned("category");
            let calcs_n = obj
                .get("calcs")
                .and_then(|v| v.as_object())
                .map(|m| m.len())
                .filter(|n| *n > 0);
            match (category, calcs_n) {
                (Some(c), Some(n)) => Some(format!("{c} +{n} overrides")),
                (Some(c), None) => Some(c),
                (None, Some(n)) => Some(format!("+{n} overrides")),
                (None, None) => None,
            }
        }
        "pob_engine_facts" => obj
            .get("categories")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .filter(|s| !s.is_empty()),
        "read_internal_reference" => s_owned("rel_path").or_else(|| s_owned("path")),
        "repoe_lookup" => {
            let game = s_owned("game");
            let category = s_owned("category");
            let id = s_owned("id").or_else(|| s_owned("name"));
            match (game, category, id) {
                (Some(g), Some(c), Some(i)) => Some(format!("{g}:{c} {i}")),
                (None, Some(c), Some(i)) => Some(format!("{c} {i}")),
                (Some(g), Some(c), None) => Some(format!("{g}:{c}")),
                (_, _, Some(i)) => Some(i),
                _ => None,
            }
        }
        "repoe_resolve" => s_owned("tag"),
        "trade_resolve_stats" => s_owned("phrase").map(|p| quote_truncate(&p, MAX_QUERY_LEN)),
        "trade_search_url" => {
            // The `query_body` is a deeply-nested JSON blob; a 60-char
            // truncation hides exactly what we need at a glance ("how many
            // stat filters? which category?"). Extract the structural shape
            // instead, so the debug-chat log surfaces an actionable summary.
            let league = s_owned("league").unwrap_or_else(|| "?".to_string());
            let game = s_owned("game").unwrap_or_else(|| "poe1".to_string());
            let body = obj.get("query_body");
            let query = body.and_then(|b| b.get("query"));
            let stats_count = query
                .and_then(|q| q.get("stats"))
                .and_then(|s| s.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|g| g.get("filters").and_then(|f| f.as_array()))
                        .map(|f| f.len())
                        .sum::<usize>()
                })
                .unwrap_or(0);
            let category = query
                .and_then(|q| q.get("filters"))
                .and_then(|f| f.get("type_filters"))
                .and_then(|t| t.get("filters"))
                .and_then(|f| f.get("category"))
                .and_then(|c| c.get("option"))
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());
            let cat_part = category
                .map(|c| format!(", category={c}"))
                .unwrap_or_default();
            Some(format!(
                "{game}/{league} · {stats_count} stat filter(s){cat_part}"
            ))
        }
        "web_fetch" => s_owned("url").map(|u| hostname_or_url(&u)),
        "get_active_build" => None,
        "sheet_propose_section" => s_owned("section_id"),
        "sheet_ask" => s_owned("question_id").or_else(|| s_owned("title").map(|t| truncate(&t, 60))),
        "sheet_finalize_request" => s_owned("name"),
        "get_active_build_sheet" => s_owned("fingerprint").map(|f| truncate(&f, 60)),
        _ => generic_join(args),
    }
}

fn quote_truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() > max {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        format!("\"{out}\"")
    } else {
        format!("\"{s}\"")
    }
}

fn truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() > max {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    } else {
        s.to_string()
    }
}

fn hostname_or_url(url: &str) -> String {
    let after_scheme = url
        .find("://")
        .map(|i| &url[i + 3..])
        .unwrap_or(url);
    let host = after_scheme.split(['/', '?', '#']).next().unwrap_or(after_scheme);
    let host = host.split('@').next_back().unwrap_or(host);
    let host = host.split(':').next().unwrap_or(host);
    if host.is_empty() {
        truncate(url, MAX_QUERY_LEN)
    } else {
        host.to_string()
    }
}

fn generic_join(args: &Value) -> Option<String> {
    let obj = args.as_object()?;
    if obj.is_empty() {
        return None;
    }
    let mut parts: Vec<String> = Vec::new();
    for (k, v) in obj.iter() {
        let val_str = match v {
            Value::String(s) => quote_truncate(s, MAX_QUERY_LEN),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            other => truncate(&other.to_string(), MAX_QUERY_LEN),
        };
        parts.push(format!("{k}={val_str}"));
        if parts.len() == 3 {
            break;
        }
    }
    Some(parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn wiki_parse_returns_title() {
        let s = summarize_tool_args("wiki_parse", &json!({"title": "Resolute Technique", "game": "poe2"}));
        assert_eq!(s.as_deref(), Some("Resolute Technique"));
    }

    #[test]
    fn kb_search_includes_top_k() {
        let s = summarize_tool_args("kb_search", &json!({"query": "spell suppression cap", "top_k": 5}));
        assert_eq!(s.as_deref(), Some("\"spell suppression cap\" (top_k=5)"));
    }

    #[test]
    fn load_skill_returns_name() {
        let s = summarize_tool_args("load_skill", &json!({"name": "build-review"}));
        assert_eq!(s.as_deref(), Some("build-review"));
    }

    #[test]
    fn read_internal_reference_returns_path() {
        let s = summarize_tool_args(
            "read_internal_reference",
            &json!({"rel_path": "references/poe2/05_atlas.md"}),
        );
        assert_eq!(s.as_deref(), Some("references/poe2/05_atlas.md"));
    }

    #[test]
    fn web_fetch_returns_hostname() {
        let s = summarize_tool_args(
            "web_fetch",
            &json!({"url": "https://www.poewiki.net/wiki/Resolute_Technique"}),
        );
        assert_eq!(s.as_deref(), Some("www.poewiki.net"));
    }

    #[test]
    fn pob_calc_with_overrides() {
        let s = summarize_tool_args(
            "pob_calc",
            &json!({"category": "Player", "calcs": {"flask_uptime": 0.8, "boss": true}}),
        );
        assert_eq!(s.as_deref(), Some("Player +2 overrides"));
    }

    #[test]
    fn unknown_tool_falls_back_to_generic() {
        let s = summarize_tool_args(
            "mystery_tool",
            &json!({"foo": "bar", "n": 42}),
        );
        let s = s.unwrap();
        assert!(s.contains("foo=\"bar\""));
        assert!(s.contains("n=42"));
    }

    #[test]
    fn empty_args_returns_none() {
        assert_eq!(summarize_tool_args("anything", &json!({})), None);
    }

    #[test]
    fn trade_search_url_summarizes_structure() {
        let s = summarize_tool_args(
            "trade_search_url",
            &json!({
                "game": "poe1",
                "league": "Mirage",
                "query_body": {
                    "query": {
                        "status": {"option": "online"},
                        "stats": [
                            {"type": "and", "filters": [
                                {"id": "explicit.stat_2974417149", "value": {"min": 140}},
                                {"id": "pseudo.pseudo_total_attack_speed", "value": {"min": 20}}
                            ]}
                        ],
                        "filters": {
                            "type_filters": {"filters": {"category": {"option": "weapon.onesword"}}}
                        }
                    }
                }
            }),
        );
        assert_eq!(
            s.as_deref(),
            Some("poe1/Mirage · 2 stat filter(s), category=weapon.onesword"),
        );
    }

    #[test]
    fn trade_search_url_no_category() {
        let s = summarize_tool_args(
            "trade_search_url",
            &json!({
                "game": "poe2",
                "league": "Standard",
                "query_body": {"query": {"stats": [{"type": "and", "filters": [{"id": "x"}]}]}}
            }),
        );
        assert_eq!(s.as_deref(), Some("poe2/Standard · 1 stat filter(s)"));
    }

    #[test]
    fn get_active_build_returns_none_even_with_args() {
        assert_eq!(
            summarize_tool_args("get_active_build", &json!({"refresh": true})),
            None
        );
    }
}

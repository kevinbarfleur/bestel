use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;

use super::{PobBuild, PobItem, PobSkillGem, PobSkillGroup, PoeVersion};

pub fn parse_file(path: &Path) -> Result<PobBuild> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("reading PoB file {}", path.display()))?;
    parse_bytes(&bytes, path.to_path_buf())
}

pub fn parse_bytes(bytes: &[u8], source: PathBuf) -> Result<PobBuild> {
    let xml = String::from_utf8_lossy(bytes);
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(false);

    let mut game: Option<PoeVersion> = None;
    let mut build_attrs: BTreeMap<String, String> = BTreeMap::new();
    let mut stats: BTreeMap<String, f64> = BTreeMap::new();
    let mut notes = String::new();
    let mut items: Vec<PobItem> = Vec::new();
    let mut skill_groups: Vec<PobSkillGroup> = Vec::new();
    let mut passive_tree_url: Option<String> = None;

    let mut in_build = false;
    let mut in_notes = false;
    let mut in_items = false;
    let mut in_skills = false;
    let mut in_url = false;

    let mut current_item: Option<(String, String)> = None;
    let mut current_skill: Option<PobSkillGroup> = None;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = std::str::from_utf8(e.name().as_ref())?.to_string();
                match name.as_str() {
                    "PathOfBuilding" => {
                        if game.is_none() {
                            game = Some(PoeVersion::Poe1);
                        }
                    }
                    "PathOfBuilding2" => {
                        game = Some(PoeVersion::Poe2);
                    }
                    "Build" => {
                        in_build = true;
                        for a in e.attributes().flatten() {
                            let k = std::str::from_utf8(a.key.as_ref())?.to_string();
                            let v = a.unescape_value().unwrap_or_default().to_string();
                            build_attrs.insert(k, v);
                        }
                    }
                    "Notes" => {
                        in_notes = true;
                    }
                    "Items" => {
                        in_items = true;
                    }
                    "Skills" => {
                        in_skills = true;
                    }
                    "Item" if in_items => {
                        let id = e
                            .attributes()
                            .flatten()
                            .find(|a| a.key.as_ref() == b"id")
                            .map(|a| a.unescape_value().unwrap_or_default().to_string())
                            .unwrap_or_default();
                        current_item = Some((id, String::new()));
                    }
                    "Skill" if in_skills => {
                        let mut label = String::new();
                        let mut is_main = false;
                        for a in e.attributes().flatten() {
                            match a.key.as_ref() {
                                b"label" => {
                                    label = a.unescape_value().unwrap_or_default().to_string();
                                }
                                b"mainActiveSkill" => {
                                    let v = a.unescape_value().unwrap_or_default();
                                    is_main = v.as_ref() == "1" || v.as_ref() == "true";
                                }
                                _ => {}
                            }
                        }
                        current_skill = Some(PobSkillGroup {
                            label,
                            is_main,
                            gems: Vec::new(),
                        });
                    }
                    "URL" => {
                        in_url = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(e)) => {
                let name = std::str::from_utf8(e.name().as_ref())?.to_string();
                match name.as_str() {
                    "PlayerStat" if in_build => {
                        let mut stat_key = String::new();
                        let mut stat_val: Option<f64> = None;
                        for a in e.attributes().flatten() {
                            match a.key.as_ref() {
                                b"stat" => {
                                    stat_key =
                                        a.unescape_value().unwrap_or_default().to_string();
                                }
                                b"value" => {
                                    stat_val = a
                                        .unescape_value()
                                        .ok()
                                        .and_then(|v| v.parse::<f64>().ok());
                                }
                                _ => {}
                            }
                        }
                        if !stat_key.is_empty() {
                            if let Some(v) = stat_val {
                                stats.insert(stat_key, v);
                            }
                        }
                    }
                    "Gem" if current_skill.is_some() => {
                        let mut name_spec = String::new();
                        let mut level: Option<u32> = None;
                        let mut quality: Option<u32> = None;
                        let mut enabled = true;
                        for a in e.attributes().flatten() {
                            match a.key.as_ref() {
                                b"nameSpec" => {
                                    name_spec =
                                        a.unescape_value().unwrap_or_default().to_string();
                                }
                                b"level" => {
                                    level = a
                                        .unescape_value()
                                        .ok()
                                        .and_then(|v| v.parse::<u32>().ok());
                                }
                                b"quality" => {
                                    quality = a
                                        .unescape_value()
                                        .ok()
                                        .and_then(|v| v.parse::<u32>().ok());
                                }
                                b"enabled" => {
                                    let v = a.unescape_value().unwrap_or_default();
                                    enabled = v.as_ref() != "false";
                                }
                                _ => {}
                            }
                        }
                        if let Some(group) = current_skill.as_mut() {
                            if !name_spec.is_empty() {
                                group.gems.push(PobSkillGem {
                                    name: name_spec,
                                    level,
                                    quality,
                                    enabled,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(t)) => {
                let text = t.unescape().unwrap_or_default().to_string();
                if in_notes {
                    notes.push_str(&text);
                } else if let Some((_, raw)) = current_item.as_mut() {
                    raw.push_str(&text);
                } else if in_url {
                    if passive_tree_url.is_none() {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            passive_tree_url = Some(trimmed.to_string());
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = std::str::from_utf8(e.name().as_ref())?.to_string();
                match name.as_str() {
                    "Build" => in_build = false,
                    "Notes" => in_notes = false,
                    "Items" => in_items = false,
                    "Skills" => in_skills = false,
                    "URL" => in_url = false,
                    "Item" => {
                        if let Some((id, raw)) = current_item.take() {
                            let item = parse_item_text(id, raw);
                            items.push(item);
                        }
                    }
                    "Skill" => {
                        if let Some(group) = current_skill.take() {
                            if !group.gems.is_empty() {
                                skill_groups.push(group);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("parse error at {}: {}", reader.buffer_position(), e)),
            _ => {}
        }
        buf.clear();
    }

    let game = game.ok_or_else(|| anyhow!("not a Path of Building XML (no root)"))?;

    let class = build_attrs
        .get("className")
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());
    let ascendancy = build_attrs.get("ascendClassName").cloned();
    let level = build_attrs
        .get("level")
        .and_then(|v| v.parse::<u32>().ok());
    let target_version = build_attrs.get("targetVersion").cloned();
    let main_socket_group = build_attrs
        .get("mainSocketGroup")
        .and_then(|v| v.parse::<usize>().ok());

    if let Some(idx) = main_socket_group {
        let target = idx.saturating_sub(1);
        if let Some(g) = skill_groups.get_mut(target) {
            g.is_main = true;
        }
    }

    let main_skill = pick_main_skill(&skill_groups, main_socket_group);

    Ok(PobBuild {
        source_file: source,
        game,
        class,
        ascendancy,
        level,
        target_version,
        stats,
        notes: notes.trim().to_string(),
        main_skill,
        skill_groups,
        items,
        passive_tree_url,
    })
}

fn pick_main_skill(
    groups: &[PobSkillGroup],
    main_socket_group: Option<usize>,
) -> Option<String> {
    let candidate: Option<&PobSkillGroup> = main_socket_group
        .and_then(|idx| groups.get(idx.saturating_sub(1)))
        .or_else(|| groups.iter().find(|g| g.is_main))
        .or_else(|| groups.first());

    candidate.and_then(|g| {
        g.gems
            .iter()
            .find(|gem| gem.enabled && !is_support_name(&gem.name))
            .map(|gem| gem.name.clone())
            .or_else(|| g.gems.first().map(|gem| gem.name.clone()))
    })
}

fn is_support_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("support") || lower.starts_with("supported by")
}

fn parse_item_text(id: String, raw: String) -> PobItem {
    let mut rarity: Option<String> = None;
    let mut name: Option<String> = None;
    let mut base: Option<String> = None;

    let mut lines = raw.lines().map(|l| l.trim()).filter(|l| !l.is_empty());

    if let Some(first) = lines.next() {
        if let Some(rest) = first.strip_prefix("Rarity:") {
            rarity = Some(rest.trim().to_string());
        }
    }

    let next1 = lines.next().map(|s| s.to_string());
    let next2 = lines.next().map(|s| s.to_string());

    match (rarity.as_deref(), next1, next2) {
        (Some(r), Some(n1), Some(n2))
            if r.eq_ignore_ascii_case("RARE") || r.eq_ignore_ascii_case("UNIQUE") =>
        {
            name = Some(n1);
            base = Some(n2);
        }
        (_, Some(n1), _) => {
            base = Some(n1);
        }
        _ => {}
    }

    PobItem {
        id,
        rarity,
        name,
        base,
        raw: raw.trim().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_poe1_root() {
        let xml = br#"<?xml version="1.0"?><PathOfBuilding><Build level="92" className="Witch" ascendClassName="Necromancer"><PlayerStat stat="Life" value="3500"/></Build><Notes>hello</Notes></PathOfBuilding>"#;
        let b = parse_bytes(xml, PathBuf::from("test.xml")).unwrap();
        assert_eq!(b.game, PoeVersion::Poe1);
        assert_eq!(b.class, "Witch");
        assert_eq!(b.ascendancy.as_deref(), Some("Necromancer"));
        assert_eq!(b.level, Some(92));
        assert_eq!(b.life(), Some(3500.0));
        assert_eq!(b.notes, "hello");
    }

    #[test]
    fn parses_poe2_root() {
        let xml = br#"<?xml version="1.0"?><PathOfBuilding2><Build level="50" className="Druid" ascendClassName="Shaman"><PlayerStat stat="TotalDPS" value="40000"/></Build></PathOfBuilding2>"#;
        let b = parse_bytes(xml, PathBuf::from("test.xml")).unwrap();
        assert_eq!(b.game, PoeVersion::Poe2);
        assert_eq!(b.class, "Druid");
        assert_eq!(b.dps(), Some(40000.0));
    }
}

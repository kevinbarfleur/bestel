use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;

use super::{
    JewelPlacement, MasteryPick, NotesSection, PobBuffs, PobBuild, PobBuildSummary, PobCharges,
    PobConfig, PobDefenses, PobItem, PobSkillGem, PobSkillGroup, PobSkillSet, PobTattoo, PobTree,
    PobTreeSpec, PoePantheon, PoeVersion,
};

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
    let mut buffs = PobBuffs::default();
    let mut config = PobConfig::default();
    let mut tree = PobTree::default();
    let mut tattoos: Vec<PobTattoo> = Vec::new();
    let mut allocated_nodes: Vec<u32> = Vec::new();
    let mut mastery_picks: Vec<MasteryPick> = Vec::new();
    let mut jewel_placements: Vec<JewelPlacement> = Vec::new();
    let mut spectres: Vec<String> = Vec::new();
    let mut import_link: Option<String> = None;
    let mut active_item_set: Option<String> = None;
    let mut item_sets: Vec<(String, BTreeMap<String, String>)> = Vec::new();

    let mut in_build = false;
    let mut in_notes = false;
    let mut in_items = false;
    let mut in_skills = false;
    let mut in_url = false;
    let mut current_config_set: Option<String> = None;
    let mut current_item: Option<(String, String)> = None;
    let mut current_skill: Option<PobSkillGroup> = None;
    let mut current_item_set: Option<(String, BTreeMap<String, String>)> = None;
    let mut current_override: Option<(u32, String, String)> = None; // (node_id, dn, body)

    // Sprint v5 — multi-set / multi-spec awareness. The parser no longer
    // flattens every `<Skill>` under every `<SkillSet>` into one bucket.
    // It tracks which set/spec the player tagged active and emits skills /
    // sockets / tattoos / nodes belonging to the active branch into the
    // legacy fields. Inactive branches are recorded as metadata-only
    // entries in `skill_sets[]` / `tree_specs[]` so the LLM can describe
    // "you have a boss setup too" without their full content polluting
    // signatures or render.
    let mut active_skill_set_id: Option<String> = None;
    let mut current_skill_set_id: Option<String> = None;
    let mut current_skill_set_title: Option<String> = None;
    let mut skill_set_metas: Vec<(String, Option<String>)> = Vec::new(); // (id, title)
    let mut active_spec_index: Option<u32> = None;
    let mut spec_index_counter: u32 = 0;
    let mut current_spec_title: Option<String> = None;
    let mut spec_metas: Vec<(u32, Option<String>)> = Vec::new();

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
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"activeItemSet" {
                                active_item_set =
                                    Some(a.unescape_value().unwrap_or_default().to_string());
                            }
                        }
                    }
                    "Skills" => {
                        in_skills = true;
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"activeSkillSet" {
                                active_skill_set_id =
                                    Some(a.unescape_value().unwrap_or_default().to_string());
                            }
                        }
                    }
                    "Tree" => {
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"activeSpec" {
                                let v = a.unescape_value().unwrap_or_default().to_string();
                                if let Ok(n) = v.parse::<u32>() {
                                    active_spec_index = Some(n);
                                }
                            }
                        }
                    }
                    "SkillSet" if in_skills => {
                        let mut id = String::new();
                        let mut title: Option<String> = None;
                        for a in e.attributes().flatten() {
                            let v = a.unescape_value().unwrap_or_default().to_string();
                            match a.key.as_ref() {
                                b"id" => id = v,
                                b"title" => {
                                    if !v.is_empty() {
                                        title = Some(v);
                                    }
                                }
                                _ => {}
                            }
                        }
                        if !id.is_empty() {
                            current_skill_set_id = Some(id.clone());
                            current_skill_set_title = title.clone();
                            // Record the metadata in document order so the
                            // post-pass can emit a non-active placeholder
                            // entry for each named set.
                            if !skill_set_metas.iter().any(|(existing, _)| existing == &id) {
                                skill_set_metas.push((id, title));
                            }
                        }
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
                    "ItemSet" if in_items => {
                        let id = e
                            .attributes()
                            .flatten()
                            .find(|a| a.key.as_ref() == b"id")
                            .map(|a| a.unescape_value().unwrap_or_default().to_string())
                            .unwrap_or_default();
                        current_item_set = Some((id, BTreeMap::new()));
                    }
                    "Skill" if in_skills => {
                        let mut label = String::new();
                        let mut is_main = false;
                        let mut slot: Option<String> = None;
                        for a in e.attributes().flatten() {
                            let v = a.unescape_value().unwrap_or_default().to_string();
                            match a.key.as_ref() {
                                b"label" => label = v,
                                b"mainActiveSkill" => {
                                    is_main = v == "1" || v == "true";
                                }
                                b"slot" => {
                                    if !v.is_empty() && v != "nil" {
                                        slot = Some(v);
                                    }
                                }
                                _ => {}
                            }
                        }
                        current_skill = Some(PobSkillGroup {
                            label,
                            is_main,
                            gems: Vec::new(),
                            slot,
                        });
                    }
                    "URL" => {
                        in_url = true;
                    }
                    "Config" => {
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"activeConfigSet" {
                                config.active_set_id =
                                    Some(a.unescape_value().unwrap_or_default().to_string());
                            }
                        }
                    }
                    "ConfigSet" => {
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"id" {
                                current_config_set =
                                    Some(a.unescape_value().unwrap_or_default().to_string());
                            }
                        }
                    }
                    "Spec" => {
                        spec_index_counter += 1;
                        // Capture title attribute (PoB calls it `title` on
                        // some specs). Used by the metadata-only entries
                        // we emit for inactive specs.
                        let mut title: Option<String> = None;
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"title" {
                                let v = a.unescape_value().unwrap_or_default().to_string();
                                if !v.is_empty() {
                                    title = Some(v);
                                }
                            }
                        }
                        current_spec_title = title.clone();
                        if !spec_metas.iter().any(|(idx, _)| *idx == spec_index_counter) {
                            spec_metas.push((spec_index_counter, title));
                        }
                        // Only absorb the active spec's attrs into the
                        // legacy fields. Old PoB builds with a single Spec
                        // and no `activeSpec` attribute still keep the
                        // existing behaviour (absorb the only spec).
                        let belongs_to_active = match active_spec_index {
                            Some(active) => active == spec_index_counter,
                            None => true,
                        };
                        if belongs_to_active {
                            absorb_spec_attrs(
                                &e,
                                &mut tree,
                                &mut allocated_nodes,
                                &mut mastery_picks,
                            );
                        }
                    }
                    // PoE1 tattoo override: `<Override dn="..." nodeId="..." ...>{stat text}</Override>`
                    "Override" => {
                        let mut node_id: u32 = 0;
                        let mut dn = String::new();
                        for a in e.attributes().flatten() {
                            let v = a.unescape_value().unwrap_or_default().to_string();
                            match a.key.as_ref() {
                                b"dn" => dn = v,
                                b"nodeId" => node_id = v.parse().unwrap_or(0),
                                _ => {}
                            }
                        }
                        if !dn.is_empty() {
                            current_override = Some((node_id, dn, String::new()));
                        }
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
                    "Buffs" if in_build => {
                        for a in e.attributes().flatten() {
                            let v = a.unescape_value().unwrap_or_default().to_string();
                            let parsed: Vec<String> = v
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                            match a.key.as_ref() {
                                b"combatList" => buffs.combat = parsed,
                                b"buffList" => buffs.buffs = parsed,
                                b"curseList" => buffs.curses = parsed,
                                _ => {}
                            }
                        }
                    }
                    "Spectre" if in_build => {
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"id" {
                                let v = a.unescape_value().unwrap_or_default().to_string();
                                if !v.is_empty() {
                                    spectres.push(v);
                                }
                            }
                        }
                    }
                    "Gem" if current_skill.is_some() => {
                        let mut gem = PobSkillGem {
                            enabled: true,
                            ..Default::default()
                        };
                        let mut had_minion_attr = false;
                        for a in e.attributes().flatten() {
                            let v = a.unescape_value().unwrap_or_default().to_string();
                            match a.key.as_ref() {
                                b"nameSpec" => gem.name = v,
                                b"level" => gem.level = v.parse().ok(),
                                b"quality" => gem.quality = v.parse().ok(),
                                b"enabled" => gem.enabled = v != "false",
                                b"skillId" => gem.skill_id = Some(v),
                                b"variantId" => gem.variant_id = Some(v),
                                b"gemId" => gem.gem_id = Some(v),
                                b"statSetIndex" => gem.stat_set_index = v.parse().ok(),
                                b"skillMinion" | b"skillMinionSkill" => {
                                    had_minion_attr = true;
                                }
                                _ => {}
                            }
                        }
                        gem.is_minion = had_minion_attr;
                        if let Some(group) = current_skill.as_mut() {
                            if !gem.name.is_empty() {
                                group.gems.push(gem);
                            }
                        }
                    }
                    "Input" => {
                        let mut input_name = String::new();
                        let mut value = String::new();
                        for a in e.attributes().flatten() {
                            let v = a.unescape_value().unwrap_or_default().to_string();
                            match a.key.as_ref() {
                                b"name" => input_name = v,
                                b"boolean" => value = v,
                                b"number" => value = v,
                                b"string" => value = v,
                                _ => {}
                            }
                        }
                        if input_name == "customMods" {
                            for line in value.lines().map(|l| l.trim()) {
                                if !line.is_empty() {
                                    config.custom_mods.push(line.to_string());
                                }
                            }
                        } else if !input_name.is_empty() {
                            config.inputs.insert(input_name, value);
                        }
                    }
                    "Placeholder" => {
                        let mut input_name = String::new();
                        let mut value = String::new();
                        for a in e.attributes().flatten() {
                            let v = a.unescape_value().unwrap_or_default().to_string();
                            match a.key.as_ref() {
                                b"name" => input_name = v,
                                b"boolean" | b"number" | b"string" => value = v,
                                _ => {}
                            }
                        }
                        if !input_name.is_empty() {
                            config.placeholders.insert(input_name, value);
                        }
                    }
                    "Spec" => {
                        spec_index_counter += 1;
                        let mut title: Option<String> = None;
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"title" {
                                let v = a.unescape_value().unwrap_or_default().to_string();
                                if !v.is_empty() {
                                    title = Some(v);
                                }
                            }
                        }
                        if !spec_metas.iter().any(|(idx, _)| *idx == spec_index_counter) {
                            spec_metas.push((spec_index_counter, title));
                        }
                        let belongs_to_active = match active_spec_index {
                            Some(active) => active == spec_index_counter,
                            None => true,
                        };
                        if belongs_to_active {
                            absorb_spec_attrs(
                                &e,
                                &mut tree,
                                &mut allocated_nodes,
                                &mut mastery_picks,
                            );
                        }
                    }
                    "WeaponSet1" => {
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"nodes" {
                                let v = a.unescape_value().unwrap_or_default();
                                tree.weapon_set_1_node_count = Some(
                                    v.split(',').filter(|s| !s.is_empty()).count() as u32,
                                );
                            }
                        }
                    }
                    "WeaponSet2" => {
                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"nodes" {
                                let v = a.unescape_value().unwrap_or_default();
                                tree.weapon_set_2_node_count = Some(
                                    v.split(',').filter(|s| !s.is_empty()).count() as u32,
                                );
                            }
                        }
                    }
                    "Socket" => {
                        let mut node_id: u32 = 0;
                        let mut item_id: u32 = 0;
                        for a in e.attributes().flatten() {
                            let v = a.unescape_value().unwrap_or_default().to_string();
                            match a.key.as_ref() {
                                b"nodeId" => node_id = v.parse().unwrap_or(0),
                                b"itemId" => item_id = v.parse().unwrap_or(0),
                                _ => {}
                            }
                        }
                        if node_id != 0 && item_id != 0 {
                            jewel_placements.push(JewelPlacement { node_id, item_id });
                        }
                    }
                    "Slot" if current_item_set.is_some() => {
                        let mut slot_name = String::new();
                        let mut item_id = String::new();
                        for a in e.attributes().flatten() {
                            let v = a.unescape_value().unwrap_or_default().to_string();
                            match a.key.as_ref() {
                                b"name" => slot_name = v,
                                b"itemId" => item_id = v,
                                _ => {}
                            }
                        }
                        if let Some((_, map)) = current_item_set.as_mut() {
                            if !slot_name.is_empty() && !item_id.is_empty() && item_id != "0" {
                                map.insert(slot_name, item_id);
                            }
                        }
                    }
                    "Import" => {
                        for a in e.attributes().flatten() {
                            // ⚠ NEVER read lastAccountHash / lastCharacterHash — PII.
                            if a.key.as_ref() == b"importLink" {
                                let v = a.unescape_value().unwrap_or_default().to_string();
                                if v.starts_with("https://pobb.in/")
                                    || v.starts_with("http://pobb.in/")
                                {
                                    import_link = Some(v);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(t)) => {
                let text = t.unescape().unwrap_or_default().to_string();
                if let Some((_, _, body)) = current_override.as_mut() {
                    body.push_str(&text);
                } else if in_notes {
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
                    "ConfigSet" => current_config_set = None,
                    "Item" => {
                        if let Some((id, raw)) = current_item.take() {
                            let item = parse_item_text(id, raw);
                            items.push(item);
                        }
                    }
                    "ItemSet" => {
                        if let Some((id, map)) = current_item_set.take() {
                            item_sets.push((id, map));
                        }
                    }
                    "Skill" => {
                        if let Some(group) = current_skill.take() {
                            if !group.gems.is_empty() {
                                // Skip skills from non-active SkillSet
                                // wrappers. Old PoB builds without any
                                // <SkillSet> element still flatten one
                                // implicit set (both ids are None).
                                let belongs_to_active = match (
                                    &current_skill_set_id,
                                    &active_skill_set_id,
                                ) {
                                    (Some(cur), Some(active)) => cur == active,
                                    _ => true,
                                };
                                if belongs_to_active {
                                    skill_groups.push(group);
                                }
                            }
                        }
                    }
                    "SkillSet" => {
                        current_skill_set_id = None;
                        current_skill_set_title = None;
                    }
                    "Spec" => {
                        current_spec_title = None;
                    }
                    "Override" => {
                        if let Some((node_id, dn, body)) = current_override.take() {
                            let body = body.trim().to_string();
                            tattoos.push(PobTattoo {
                                node_id,
                                display_name: dn,
                                body,
                            });
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

    let _ = current_config_set;
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

    let pantheon = PoePantheon {
        major: build_attrs
            .get("pantheonMajorGod")
            .filter(|s| !s.is_empty() && s.as_str() != "None")
            .cloned(),
        minor: build_attrs
            .get("pantheonMinorGod")
            .filter(|s| !s.is_empty() && s.as_str() != "None")
            .cloned(),
        bandit: build_attrs
            .get("bandit")
            .filter(|s| !s.is_empty() && s.as_str() != "None")
            .cloned(),
    };

    if let Some(idx) = main_socket_group {
        let target = idx.saturating_sub(1);
        if let Some(g) = skill_groups.get_mut(target) {
            g.is_main = true;
        }
    }

    let main_skill = pick_main_skill(&skill_groups, main_socket_group);
    let charges = derive_charges(&stats);
    let defenses = derive_defenses(&stats);
    let stripped_notes = strip_pob_color_codes(&notes);
    let notes_sections = split_notes_sections(&stripped_notes);

    // Pick the active ItemSet (or the first one) and surface its slot map.
    let slot_map = pick_active_slot_map(&item_sets, active_item_set.as_deref());

    // Build the metadata-only skill_sets / tree_specs surface. The active
    // entry carries the parsed groups / tree (so the LLM can describe
    // "you have a Mapping setup and a Boss setup; this answer is about
    // the Mapping one"); inactive entries carry only id + title. We avoid
    // re-collecting inactive content here to keep the JSON tight and
    // signatures stable.
    let skill_sets: Vec<PobSkillSet> = skill_set_metas
        .iter()
        .map(|(id, title)| {
            let is_active = active_skill_set_id
                .as_deref()
                .map(|active| active == id)
                .unwrap_or(false);
            PobSkillSet {
                id: id.clone(),
                title: title.clone(),
                is_active,
                groups: if is_active {
                    skill_groups.clone()
                } else {
                    Vec::new()
                },
            }
        })
        .collect();
    let tree_specs: Vec<PobTreeSpec> = spec_metas
        .iter()
        .map(|(idx, title)| {
            let is_active = match active_spec_index {
                Some(active) => active == *idx,
                None => spec_metas.len() == 1,
            };
            PobTreeSpec {
                id: idx.to_string(),
                title: title.clone(),
                is_active,
                tree: if is_active {
                    tree.clone()
                } else {
                    PobTree::default()
                },
                allocated_nodes: if is_active {
                    allocated_nodes.clone()
                } else {
                    Vec::new()
                },
                mastery_picks: if is_active {
                    mastery_picks.clone()
                } else {
                    Vec::new()
                },
            }
        })
        .collect();
    let _ = current_spec_title;
    let _ = current_skill_set_title;

    Ok(PobBuild {
        source_file: source,
        game,
        class,
        ascendancy,
        level,
        target_version,
        stats,
        notes: stripped_notes.trim().to_string(),
        notes_sections,
        main_skill,
        skill_groups,
        items,
        passive_tree_url,
        charges,
        buffs,
        config,
        tree,
        defenses,
        slot_map,
        pantheon,
        tattoos,
        allocated_nodes,
        mastery_picks,
        jewel_placements,
        spectres,
        import_link,
        skill_sets,
        tree_specs,
    })
}

/// Lightweight summary parser. Reads only as far as the closing `</Build>` tag
/// — picks up className, ascendClassName, level, mainSocketGroup index — so we
/// can populate the build picker without parsing every item, gem, and stat.
pub fn parse_summary(path: &Path) -> Result<PobBuildSummary> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("summary read {}", path.display()))?;
    let xml = String::from_utf8_lossy(&bytes);
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);

    let mut game: Option<PoeVersion> = None;
    let mut build_attrs: BTreeMap<String, String> = BTreeMap::new();
    let mut first_gem_name: Option<String> = None;
    let mut in_skills = false;
    let mut in_skill_block = false;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                match std::str::from_utf8(e.name().as_ref())? {
                    "PathOfBuilding" if game.is_none() => game = Some(PoeVersion::Poe1),
                    "PathOfBuilding2" => game = Some(PoeVersion::Poe2),
                    "Build" => {
                        for a in e.attributes().flatten() {
                            let k = std::str::from_utf8(a.key.as_ref())?.to_string();
                            let v = a.unescape_value().unwrap_or_default().to_string();
                            build_attrs.insert(k, v);
                        }
                    }
                    "Skills" => in_skills = true,
                    "Skill" if in_skills => in_skill_block = true,
                    _ => {}
                }
            }
            Ok(Event::Empty(e)) => {
                if std::str::from_utf8(e.name().as_ref())? == "Gem"
                    && in_skill_block
                    && first_gem_name.is_none()
                {
                    for a in e.attributes().flatten() {
                        if a.key.as_ref() == b"nameSpec" {
                            let n = a.unescape_value().unwrap_or_default().to_string();
                            if !n.is_empty() && !is_support_name(&n) {
                                first_gem_name = Some(n);
                                break;
                            }
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                if std::str::from_utf8(e.name().as_ref())? == "Skill" {
                    in_skill_block = false;
                    if first_gem_name.is_some() {
                        break;
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("summary parse: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    let game = game.ok_or_else(|| anyhow!("summary: not a PoB XML"))?;
    let class = build_attrs
        .get("className")
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());
    let ascendancy = build_attrs.get("ascendClassName").cloned();
    let level = build_attrs.get("level").and_then(|v| v.parse::<u32>().ok());

    Ok(PobBuildSummary {
        path: path.to_path_buf(),
        game,
        class,
        ascendancy,
        level,
        main_skill_hint: first_gem_name,
        mtime: std::fs::metadata(path).ok().and_then(|m| m.modified().ok()),
    })
}

fn absorb_spec_attrs(
    e: &quick_xml::events::BytesStart<'_>,
    tree: &mut PobTree,
    allocated: &mut Vec<u32>,
    masteries: &mut Vec<MasteryPick>,
) {
    for a in e.attributes().flatten() {
        let k = a.key.as_ref();
        let v = a.unescape_value().unwrap_or_default().to_string();
        match k {
            b"treeVersion" => tree.version = Some(v),
            b"classId" => tree.class_id = v.parse().ok(),
            b"ascendClassId" => tree.ascend_class_id = v.parse().ok(),
            b"classInternalId" => tree.class_internal_id = v.parse().ok(),
            b"ascendancyInternalId" => tree.ascendancy_internal_id = Some(v),
            b"nodes" => {
                tree.node_count = v.split(',').filter(|s| !s.is_empty()).count() as u32;
                if allocated.is_empty() {
                    for piece in v.split(',') {
                        if let Ok(n) = piece.trim().parse::<u32>() {
                            allocated.push(n);
                        }
                    }
                }
            }
            b"masteryEffects" => {
                tree.mastery_count = v.split(',').filter(|s| !s.is_empty()).count() as u32;
                if masteries.is_empty() {
                    parse_mastery_effects(&v, masteries);
                }
            }
            _ => {}
        }
    }
}

/// Parse the `masteryEffects` attribute. Two formats:
///   PoE1: `{nodeId,effectId},{nodeId,effectId},...`
///   PoE2: `effectId,effectId,...`
fn parse_mastery_effects(raw: &str, out: &mut Vec<MasteryPick>) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }
    if trimmed.contains('{') {
        // PoE1 brace form `{nodeId,effectId},{...}` — comma split is
        // ambiguous, so we walk char by char and collect tokens between
        // each pair of braces.
        let mut depth = 0;
        let mut cur = String::new();
        for ch in trimmed.chars() {
            match ch {
                '{' => {
                    depth = 1;
                    cur.clear();
                }
                '}' => {
                    if depth == 1 {
                        let parts: Vec<u32> = cur
                            .split(',')
                            .filter_map(|p| p.trim().parse::<u32>().ok())
                            .collect();
                        if parts.len() == 2 {
                            out.push(MasteryPick::Poe1 {
                                node_id: parts[0],
                                effect_id: parts[1],
                            });
                        }
                        cur.clear();
                    }
                    depth = 0;
                }
                _ if depth == 1 => cur.push(ch),
                _ => {}
            }
        }
    } else {
        // PoE2 plain comma form
        for piece in trimmed.split(',') {
            if let Ok(n) = piece.trim().parse::<u32>() {
                out.push(MasteryPick::Poe2 { effect_id: n });
            }
        }
    }
}

fn pick_active_slot_map(
    item_sets: &[(String, BTreeMap<String, String>)],
    active_id: Option<&str>,
) -> BTreeMap<String, String> {
    if let Some(id) = active_id {
        if let Some((_, map)) = item_sets.iter().find(|(set_id, _)| set_id == id) {
            return map.clone();
        }
    }
    item_sets
        .first()
        .map(|(_, m)| m.clone())
        .unwrap_or_default()
}

/// Strip PoB's in-text colour codes : `^xRRGGBB` (hex foreground), `^N`
/// (palette index 0..9). Otherwise keep notes content untouched.
pub fn strip_pob_color_codes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '^' {
            match chars.peek() {
                Some('x') | Some('X') => {
                    chars.next();
                    for _ in 0..6 {
                        if matches!(
                            chars.peek(),
                            Some(c) if c.is_ascii_hexdigit()
                        ) {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                Some(c) if c.is_ascii_digit() => {
                    chars.next();
                }
                _ => {
                    out.push(ch);
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
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

fn derive_charges(stats: &BTreeMap<String, f64>) -> PobCharges {
    let mut c = PobCharges::default();
    if let Some(v) = stats.get("PowerCharges") {
        c.power.current = *v as u32;
    }
    if let Some(v) = stats.get("PowerChargesMax") {
        c.power.max = *v as u32;
    }
    if let Some(v) = stats.get("FrenzyCharges") {
        c.frenzy.current = *v as u32;
    }
    if let Some(v) = stats.get("FrenzyChargesMax") {
        c.frenzy.max = *v as u32;
    }
    if let Some(v) = stats.get("EnduranceCharges") {
        c.endurance.current = *v as u32;
    }
    if let Some(v) = stats.get("EnduranceChargesMax") {
        c.endurance.max = *v as u32;
    }
    c
}

fn derive_defenses(stats: &BTreeMap<String, f64>) -> PobDefenses {
    let g = |k: &str| stats.get(k).copied();
    PobDefenses {
        life: g("Life"),
        mana: g("Mana"),
        energy_shield: g("EnergyShield"),
        spirit: g("Spirit"),
        armour: g("Armour"),
        evasion: g("Evasion"),
        physical_dr: g("PhysicalDamageReduction"),
        spell_suppression: g("EffectiveSpellSuppressionChance"),
        block_chance: g("EffectiveBlockChance"),
        attack_dodge: g("AttackDodgeChance"),
        spell_dodge: g("SpellDodgeChance"),
        fire_resist: g("FireResist"),
        cold_resist: g("ColdResist"),
        lightning_resist: g("LightningResist"),
        chaos_resist: g("ChaosResist"),
        fire_max_hit: g("FireMaximumHitTaken"),
        cold_max_hit: g("ColdMaximumHitTaken"),
        lightning_max_hit: g("LightningMaximumHitTaken"),
        physical_max_hit: g("PhysicalMaximumHitTaken"),
        chaos_max_hit: g("ChaosMaximumHitTaken"),
        total_ehp: g("TotalEHP"),
    }
}

/// Split the free-form Notes string into headed sections. PoB users typically
/// mark sections with `==`, `# `, or all-caps lines. We do a simple heuristic:
/// any line that starts with `==` or `#` opens a new section.
fn split_notes_sections(notes: &str) -> Vec<NotesSection> {
    let mut out: Vec<NotesSection> = Vec::new();
    let mut current = NotesSection::default();
    for line in notes.lines() {
        let trimmed = line.trim();
        let is_heading = trimmed.starts_with("==")
            || trimmed.starts_with("# ")
            || trimmed.starts_with("## ");
        if is_heading {
            let body = current.body.trim().to_string();
            if !current.heading.is_empty() || !body.is_empty() {
                out.push(NotesSection {
                    heading: current.heading.clone(),
                    body,
                });
            }
            let heading = trimmed
                .trim_start_matches('=')
                .trim_start_matches('#')
                .trim()
                .to_string();
            current = NotesSection {
                heading,
                body: String::new(),
            };
        } else {
            current.body.push_str(line);
            current.body.push('\n');
        }
    }
    let body = current.body.trim().to_string();
    if !current.heading.is_empty() || !body.is_empty() {
        out.push(NotesSection {
            heading: current.heading,
            body,
        });
    }
    out
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
        assert_eq!(b.defenses.life, Some(3500.0));
    }

    #[test]
    fn parses_poe2_root() {
        let xml = br#"<?xml version="1.0"?><PathOfBuilding2><Build level="50" className="Druid" ascendClassName="Shaman"><PlayerStat stat="TotalDPS" value="40000"/></Build></PathOfBuilding2>"#;
        let b = parse_bytes(xml, PathBuf::from("test.xml")).unwrap();
        assert_eq!(b.game, PoeVersion::Poe2);
        assert_eq!(b.class, "Druid");
        assert_eq!(b.dps(), Some(40000.0));
    }

    #[test]
    fn parses_charges_buffs_config_tree() {
        let xml = br#"<?xml version="1.0"?>
<PathOfBuilding2>
  <Build level="87" className="Druid" ascendClassName="Shaman" mainSocketGroup="1">
    <PlayerStat stat="EnduranceCharges" value="3"/>
    <PlayerStat stat="EnduranceChargesMax" value="3"/>
    <PlayerStat stat="PowerChargesMax" value="3"/>
    <PlayerStat stat="FrenzyChargesMax" value="3"/>
    <PlayerStat stat="FireResist" value="77"/>
    <Buffs combatList="3 Endurance Charges, Onslaught" buffList="Herald of Ash, Rage" curseList=""/>
  </Build>
  <Config activeConfigSet="1">
    <ConfigSet id="1">
      <Input name="conditionIgnited" boolean="true"/>
      <Input name="enemyDistance" number="10"/>
      <Input name="customMods" string="+5 to Strength&#10;+10 to Dex"/>
      <Placeholder name="enemyFireResist" number="50"/>
      <Placeholder name="enemyLevel" number="82"/>
    </ConfigSet>
  </Config>
  <Tree activeSpec="1">
    <Spec nodes="1,2,3,4,5" treeVersion="0_4" classId="4" ascendClassId="2" masteryEffects="10,20"/>
  </Tree>
</PathOfBuilding2>"#;
        let b = parse_bytes(xml, PathBuf::from("test.xml")).unwrap();
        assert_eq!(b.charges.power.max, 3);
        assert_eq!(b.charges.endurance.current, 3);
        assert_eq!(b.charges.endurance.max, 3);
        assert_eq!(b.buffs.combat, vec!["3 Endurance Charges", "Onslaught"]);
        assert_eq!(b.buffs.buffs, vec!["Herald of Ash", "Rage"]);
        assert!(b.buffs.curses.is_empty());
        assert_eq!(b.config.active_set_id.as_deref(), Some("1"));
        assert_eq!(b.config.inputs.get("conditionIgnited").map(String::as_str), Some("true"));
        assert_eq!(b.config.inputs.get("enemyDistance").map(String::as_str), Some("10"));
        assert_eq!(b.config.placeholders.get("enemyFireResist").map(String::as_str), Some("50"));
        assert_eq!(b.config.placeholders.get("enemyLevel").map(String::as_str), Some("82"));
        assert_eq!(b.config.custom_mods, vec!["+5 to Strength", "+10 to Dex"]);
        assert_eq!(b.tree.version.as_deref(), Some("0_4"));
        assert_eq!(b.tree.class_id, Some(4));
        assert_eq!(b.tree.ascend_class_id, Some(2));
        assert_eq!(b.tree.node_count, 5);
        assert_eq!(b.tree.mastery_count, 2);
        assert_eq!(b.defenses.fire_resist, Some(77.0));
        assert_eq!(b.allocated_nodes, vec![1, 2, 3, 4, 5]);
        assert_eq!(b.mastery_picks.len(), 2);
        assert!(matches!(b.mastery_picks[0], MasteryPick::Poe2 { effect_id: 10 }));
    }

    #[test]
    fn parses_gem_metadata() {
        let xml = br#"<?xml version="1.0"?>
<PathOfBuilding2>
  <Build level="50" className="Druid" mainSocketGroup="1"/>
  <Skills>
    <SkillSet id="1">
      <Skill mainActiveSkill="1" label="Main">
        <Gem nameSpec="Furious Slam" skillId="FuriousSlamPlayer" variantId="FuriousSlam" gemId="Metadata/Items/Gems/SkillGemFuriousSlam" level="16" quality="20" enabled="true"/>
        <Gem nameSpec="Pounce" skillId="WolfPouncePlayer" gemId="Metadata/Items/Gems/SkillGemWolfPounce" skillMinion="WolfMinion" skillMinionSkill="1" level="3" enabled="true"/>
      </Skill>
    </SkillSet>
  </Skills>
</PathOfBuilding2>"#;
        let b = parse_bytes(xml, PathBuf::from("t.xml")).unwrap();
        assert_eq!(b.skill_groups.len(), 1);
        let g = &b.skill_groups[0];
        assert!(g.is_main);
        assert_eq!(g.gems.len(), 2);
        assert_eq!(g.gems[0].skill_id.as_deref(), Some("FuriousSlamPlayer"));
        assert_eq!(g.gems[0].variant_id.as_deref(), Some("FuriousSlam"));
        assert_eq!(g.gems[0].gem_id.as_deref().unwrap(), "Metadata/Items/Gems/SkillGemFuriousSlam");
        assert!(!g.gems[0].is_minion);
        assert!(g.gems[1].is_minion);
    }

    #[test]
    fn splits_notes_sections() {
        let raw = "intro line\n== Build Goals\nthing 1\nthing 2\n== Pob Notes\nbody";
        let sections = split_notes_sections(raw);
        // first chunk has empty heading + intro line
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].heading, "");
        assert!(sections[0].body.contains("intro line"));
        assert_eq!(sections[1].heading, "Build Goals");
        assert!(sections[1].body.contains("thing 1"));
        assert_eq!(sections[2].heading, "Pob Notes");
        assert_eq!(sections[2].body, "body");
    }

    #[test]
    fn parser_respects_active_skill_set_and_active_spec() {
        // Two SkillSets + two Specs. The parser must:
        //   - Surface ONLY active set's groups into legacy skill_groups.
        //   - Surface ONLY active spec's nodes into legacy allocated_nodes.
        //   - Emit metadata-only entries for non-active sets / specs in
        //     skill_sets[] / tree_specs[] with is_active=false.
        let xml = br#"<?xml version="1.0"?>
<PathOfBuilding>
  <Build level="92" className="Witch" mainSocketGroup="1"/>
  <Tree activeSpec="2">
    <Spec title="Leveling" nodes="1,2,3" treeVersion="3_25" classId="1"/>
    <Spec title="Final" nodes="100,200,300,400" treeVersion="3_25" classId="1"/>
  </Tree>
  <Skills activeSkillSet="2">
    <SkillSet id="1" title="Mapping">
      <Skill mainActiveSkill="1" label="Map">
        <Gem nameSpec="Hatred" enabled="true"/>
      </Skill>
    </SkillSet>
    <SkillSet id="2" title="Boss">
      <Skill mainActiveSkill="1" label="Boss">
        <Gem nameSpec="Penance Brand of Dissipation" enabled="true"/>
      </Skill>
    </SkillSet>
  </Skills>
</PathOfBuilding>"#;
        let b = parse_bytes(xml, PathBuf::from("t.xml")).unwrap();
        // Active set's gem only.
        assert_eq!(b.skill_groups.len(), 1);
        assert_eq!(
            b.skill_groups[0].gems[0].name,
            "Penance Brand of Dissipation"
        );
        // Both sets are exposed; only the second is active.
        assert_eq!(b.skill_sets.len(), 2);
        let active = b.skill_sets.iter().find(|s| s.is_active).unwrap();
        assert_eq!(active.id, "2");
        assert_eq!(active.title.as_deref(), Some("Boss"));
        let inactive = b.skill_sets.iter().find(|s| !s.is_active).unwrap();
        assert_eq!(inactive.id, "1");
        assert_eq!(inactive.title.as_deref(), Some("Mapping"));
        // Inactive entries do NOT carry the gems — metadata only.
        assert!(inactive.groups.is_empty());

        // Active spec's nodes only.
        assert_eq!(b.allocated_nodes, vec![100, 200, 300, 400]);
        assert_eq!(b.tree_specs.len(), 2);
        let active_spec = b.tree_specs.iter().find(|s| s.is_active).unwrap();
        assert_eq!(active_spec.id, "2");
        assert_eq!(active_spec.title.as_deref(), Some("Final"));
        assert_eq!(active_spec.allocated_nodes, vec![100, 200, 300, 400]);
        let inactive_spec = b.tree_specs.iter().find(|s| !s.is_active).unwrap();
        assert_eq!(inactive_spec.id, "1");
        assert!(inactive_spec.allocated_nodes.is_empty());
    }

    #[test]
    fn parses_pantheon_bandit_skill_slot_tattoos_socket_spectre_import() {
        let xml = br#"<?xml version="1.0"?>
<PathOfBuilding>
  <Build level="100" className="Templar" ascendClassName="Inquisitor"
         pantheonMajorGod="Solaris" pantheonMinorGod="Ralakesh" bandit="Alira"
         mainSocketGroup="1">
    <Spectre id="Metadata/Monsters/X/Y"/>
  </Build>
  <Import importLink="https://pobb.in/pob/abc123"
          lastAccountHash="should_be_ignored" lastCharacterHash="ignore_too"/>
  <Tree activeSpec="1">
    <Spec nodes="100,200,300" masteryEffects="{100,5},{200,7}" treeVersion="3_25" classId="5"/>
  </Tree>
  <Skills>
    <SkillSet id="1">
      <Skill mainActiveSkill="1" slot="Helmet" label="">
        <Gem nameSpec="Penance Brand" enabled="true"/>
      </Skill>
    </SkillSet>
  </Skills>
  <Items activeItemSet="1">
    <Item id="1">Rarity: RARE
Mind Star
Praetor Crown</Item>
    <Item id="2">Rarity: UNIQUE
Watcher's Eye
Prismatic Jewel</Item>
    <ItemSet id="1">
      <Slot name="Helmet" itemId="1"/>
      <Slot name="Empty Slot" itemId="0"/>
    </ItemSet>
  </Items>
</PathOfBuilding>"#;
        let b = parse_bytes(xml, PathBuf::from("t.xml")).unwrap();
        assert_eq!(b.pantheon.major.as_deref(), Some("Solaris"));
        assert_eq!(b.pantheon.minor.as_deref(), Some("Ralakesh"));
        assert_eq!(b.pantheon.bandit.as_deref(), Some("Alira"));
        assert_eq!(b.spectres, vec!["Metadata/Monsters/X/Y"]);
        assert_eq!(b.import_link.as_deref(), Some("https://pobb.in/pob/abc123"));
        assert_eq!(b.allocated_nodes, vec![100, 200, 300]);
        assert_eq!(b.mastery_picks.len(), 2);
        if let MasteryPick::Poe1 { node_id, effect_id } = &b.mastery_picks[0] {
            assert_eq!(*node_id, 100);
            assert_eq!(*effect_id, 5);
        } else {
            panic!("expected PoE1 mastery shape");
        }
        assert_eq!(b.skill_groups.len(), 1);
        assert_eq!(b.skill_groups[0].slot.as_deref(), Some("Helmet"));
        assert_eq!(b.slot_map.get("Helmet").map(String::as_str), Some("1"));
        assert!(!b.slot_map.contains_key("Empty Slot"));
    }

    #[test]
    fn strips_pob_color_codes() {
        let s = "^xFFFF77Title:^7 body ^7more ^x70FF70green^7";
        let cleaned = strip_pob_color_codes(s);
        assert_eq!(cleaned, "Title: body more green");
    }

    #[test]
    fn rejects_non_pobbin_import_links() {
        let xml = br#"<?xml version="1.0"?>
<PathOfBuilding>
  <Build level="1" className="Witch"/>
  <Import importLink="https://evil.example.com/leak" lastAccountHash="abc"/>
</PathOfBuilding>"#;
        let b = parse_bytes(xml, PathBuf::from("t.xml")).unwrap();
        assert!(b.import_link.is_none());
    }

    #[test]
    fn parses_jewel_sockets() {
        let xml = br#"<?xml version="1.0"?>
<PathOfBuilding>
  <Build level="1" className="Witch"/>
  <Tree>
    <Spec nodes="1" treeVersion="3_25" classId="0">
      <Sockets>
        <Socket nodeId="2311" itemId="13"/>
        <Socket nodeId="9408" itemId="1"/>
      </Sockets>
    </Spec>
  </Tree>
</PathOfBuilding>"#;
        let b = parse_bytes(xml, PathBuf::from("t.xml")).unwrap();
        assert_eq!(b.jewel_placements.len(), 2);
        assert_eq!(b.jewel_placements[0].node_id, 2311);
        assert_eq!(b.jewel_placements[0].item_id, 13);
    }

    #[test]
    fn parses_tattoos() {
        let xml = br#"<?xml version="1.0"?>
<PathOfBuilding>
  <Build level="1" className="Templar"/>
  <Tree>
    <Spec nodes="1" treeVersion="3_25" classId="5">
      <Overrides>
        <Override dn="Tattoo of the Valako Stormrider" nodeId="26270">+6% to Lightning Resistance</Override>
        <Override dn="Tattoo of the Hinekora Storyteller" nodeId="51923">+3% to Chaos Resistance</Override>
      </Overrides>
    </Spec>
  </Tree>
</PathOfBuilding>"#;
        let b = parse_bytes(xml, PathBuf::from("t.xml")).unwrap();
        assert_eq!(b.tattoos.len(), 2);
        assert_eq!(b.tattoos[0].node_id, 26270);
        assert_eq!(b.tattoos[0].display_name, "Tattoo of the Valako Stormrider");
        assert_eq!(b.tattoos[0].body, "+6% to Lightning Resistance");
    }
}

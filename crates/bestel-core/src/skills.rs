//! Bestel skills (Sprint F).
//!
//! A "skill" is a folder under `prompts/skills/<name>/` containing a
//! `SKILL.md` with TOML frontmatter (delimited by `+++`) plus an optional
//! `templates/` subdir of reusable parameterised prompt fragments. The
//! skill's `description` (~100 words) lives in the cached BP4 system block
//! so the model decides when to invoke it; the skill BODY is loaded on
//! demand via the `load_skill` tool — progressive disclosure rather than
//! permanent context.
//!
//! Format:
//!
//! ```text
//! +++
//! name = "build-review"
//! description = "Use when ..."
//! when_to_use = ["...", "..."]
//! trigger_examples = ["...", "..."]
//! +++
//!
//! # Skill body (markdown)
//! ...
//! ```
//!
//! TOML (rather than YAML) avoids pulling a YAML parser; the project
//! already depends on `toml`.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

const FRONTMATTER_DELIM: &str = "+++";

/// Bundled skill = (rel_path, raw_text). Mirrors `BUNDLED_REFERENCES`.
/// Entries point at files with the canonical `<name>/SKILL.md` shape OR
/// `<name>/templates/<template>.md` for the reusable fragments.
pub const BUNDLED_SKILLS: &[(&str, &str)] = &[
    (
        "build-review/SKILL.md",
        include_str!("../../../prompts/skills/build-review/SKILL.md"),
    ),
    (
        "build-review/templates/quick_audit.md",
        include_str!("../../../prompts/skills/build-review/templates/quick_audit.md"),
    ),
    (
        "craft-audit/SKILL.md",
        include_str!("../../../prompts/skills/craft-audit/SKILL.md"),
    ),
    (
        "craft-audit/templates/workflow.md",
        include_str!("../../../prompts/skills/craft-audit/templates/workflow.md"),
    ),
    (
        "mapping-strategy/SKILL.md",
        include_str!("../../../prompts/skills/mapping-strategy/SKILL.md"),
    ),
    (
        "mapping-strategy/templates/farm_plan.md",
        include_str!("../../../prompts/skills/mapping-strategy/templates/farm_plan.md"),
    ),
];

/// Parsed frontmatter shape. `name` and `description` are required;
/// the optional list fields default to empty.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub when_to_use: Vec<String>,
    #[serde(default)]
    pub trigger_examples: Vec<String>,
}

/// One fully-parsed skill: frontmatter + body + indexed templates.
#[derive(Debug, Clone, Serialize)]
pub struct Skill {
    pub frontmatter: SkillFrontmatter,
    pub body: String,
    /// Map of template stem → full markdown content. Stable iteration order
    /// for deterministic dispatch.
    pub templates: BTreeMap<String, String>,
}

/// Compact summary used by the BP4 descriptions block + the `load_skill`
/// tool's enum schema.
#[derive(Debug, Clone, Serialize)]
pub struct SkillSummary {
    pub name: String,
    pub description: String,
    pub trigger_examples: Vec<String>,
    pub templates: Vec<String>,
}

/// Parse one `SKILL.md` raw text into the structured form. Errors out
/// when the frontmatter is missing, malformed, or the required fields
/// are absent.
pub fn parse_skill_md(raw: &str) -> Result<(SkillFrontmatter, String)> {
    let trimmed = raw.trim_start_matches('\u{feff}');
    let after_open = match trimmed.strip_prefix(FRONTMATTER_DELIM) {
        Some(s) => s,
        None => {
            return Err(anyhow!(
                "SKILL.md must open with `+++` frontmatter delimiter"
            ));
        }
    };
    // Skip the newline immediately after `+++`.
    let after_open = after_open.strip_prefix("\r\n").or_else(|| after_open.strip_prefix('\n'))
        .unwrap_or(after_open);
    let close_idx = after_open
        .find(&format!("\n{FRONTMATTER_DELIM}"))
        .ok_or_else(|| anyhow!("SKILL.md frontmatter missing closing `+++`"))?;
    let frontmatter_text = &after_open[..close_idx];
    let after_close = &after_open[close_idx + 1 + FRONTMATTER_DELIM.len()..];
    let body = after_close
        .trim_start_matches(|c: char| c == '\r' || c == '\n')
        .to_string();
    // Normalize CRLF → LF before TOML parse. On Windows checkouts git
    // converts line endings to CRLF by default ; we found that
    // `frontmatter_text` ends with a bare `\r` (the `\r\n+++` was
    // partially consumed: we searched for `\n+++` so the `\n` was the
    // delimiter and the `\r` stayed in `frontmatter_text`). The TOML
    // parser rejects a bare `\r` after a closing bracket with
    // "expected newline". Stripping all `\r` is the simplest fix that
    // works regardless of where the orphan came from. Linux/LF
    // checkouts have no `\r` at all so the .replace is a no-op alloc.
    let frontmatter_normalized = frontmatter_text.replace('\r', "");
    let frontmatter: SkillFrontmatter = toml::from_str(&frontmatter_normalized)
        .with_context(|| format!("parse skill frontmatter:\n{frontmatter_normalized}"))?;
    if frontmatter.name.trim().is_empty() {
        return Err(anyhow!("skill `name` must be non-empty"));
    }
    if frontmatter.description.trim().is_empty() {
        return Err(anyhow!("skill `description` must be non-empty"));
    }
    Ok((frontmatter, body))
}

/// Read and assemble every bundled skill. Returns one [`Skill`] per
/// `<name>/SKILL.md` entry, with sibling `templates/*.md` files attached.
pub fn list_bundled_skills() -> Result<Vec<Skill>> {
    let mut by_name: BTreeMap<String, (Option<(SkillFrontmatter, String)>, BTreeMap<String, String>)> =
        BTreeMap::new();
    for (rel, content) in BUNDLED_SKILLS {
        let (skill_dir, file) = match rel.split_once('/') {
            Some((dir, rest)) => (dir, rest),
            None => continue,
        };
        let entry = by_name
            .entry(skill_dir.to_string())
            .or_insert_with(|| (None, BTreeMap::new()));
        if file == "SKILL.md" {
            entry.0 = Some(parse_skill_md(content)?);
        } else if let Some(template_path) = file.strip_prefix("templates/") {
            let stem = template_path
                .strip_suffix(".md")
                .unwrap_or(template_path)
                .to_string();
            entry.1.insert(stem, (*content).to_string());
        }
    }
    let mut out = Vec::with_capacity(by_name.len());
    for (dir_name, (parsed, templates)) in by_name {
        let (frontmatter, body) = parsed
            .ok_or_else(|| anyhow!("skill folder `{dir_name}` is missing SKILL.md"))?;
        out.push(Skill {
            frontmatter,
            body,
            templates,
        });
    }
    Ok(out)
}

/// Cached compact view of every bundled skill — used by the
/// `load_skill` tool schema (enum) and the BP4 descriptions block.
pub fn list_skill_summaries() -> Result<Vec<SkillSummary>> {
    let skills = list_bundled_skills()?;
    Ok(skills
        .into_iter()
        .map(|s| SkillSummary {
            name: s.frontmatter.name,
            description: s.frontmatter.description,
            trigger_examples: s.frontmatter.trigger_examples,
            templates: s.templates.keys().cloned().collect(),
        })
        .collect())
}

/// Skill names are the part before `/SKILL.md` in [`BUNDLED_SKILLS`].
/// Stable, alphabetically sorted.
pub fn bundled_skill_names() -> Vec<String> {
    let mut names: Vec<String> = BUNDLED_SKILLS
        .iter()
        .filter_map(|(rel, _)| rel.split_once('/').map(|(d, _)| d.to_string()))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    names.sort();
    names
}

/// Load one skill body by `name`, preferring the user's
/// `~/.bestel/skills/<name>/SKILL.md` and falling back to bundled. Returns
/// the FULL parsed skill so callers can present templates too.
pub fn load_skill(name: &str) -> Result<Skill> {
    if !is_safe_skill_name(name) {
        return Err(anyhow!(
            "invalid skill name '{name}' (path traversal / unsafe characters)"
        ));
    }
    if let Ok(disk_root) = skills_dir() {
        let skill_md = disk_root.join(name).join("SKILL.md");
        if let Ok(raw) = fs::read_to_string(&skill_md) {
            let (frontmatter, body) = parse_skill_md(&raw)
                .with_context(|| format!("parse user skill {skill_md:?}"))?;
            let templates = read_disk_templates(&disk_root.join(name).join("templates"));
            return Ok(Skill {
                frontmatter,
                body,
                templates,
            });
        }
    }
    let skills = list_bundled_skills()?;
    skills
        .into_iter()
        .find(|s| s.frontmatter.name == name)
        .ok_or_else(|| {
            anyhow!(
                "unknown skill '{name}'. Available: {}",
                bundled_skill_names().join(", ")
            )
        })
}

/// Body of the BP4 cached descriptions block. ~300 tokens for 3 skills.
/// Format: one heading per skill + frontmatter description + invocation
/// hint. The model uses this to decide when to call `load_skill`.
pub fn descriptions_block() -> String {
    let summaries = match list_skill_summaries() {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    let mut out = String::from(
        "## Available skills (loaded on demand via `load_skill`)\n\nProgressive-disclosure workflows. The descriptions below stay in your context; the bodies load only when you call `load_skill(name)`.\n\n",
    );
    for s in summaries {
        out.push_str(&format!(
            "### `{name}`\n{desc}\n\n**Triggers**: {triggers}.\n\n",
            name = s.name,
            desc = s.description.trim(),
            triggers = if s.trigger_examples.is_empty() {
                "(see SKILL.md when_to_use)".to_string()
            } else {
                s.trigger_examples
                    .iter()
                    .map(|t| format!("\"{t}\""))
                    .collect::<Vec<_>>()
                    .join("; ")
            }
        ));
    }
    out.push_str("Cap: 1 skill per turn (2 if the user explicitly asks for a full audit).\n");
    out
}

fn is_safe_skill_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains("..")
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn skills_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("home dir not resolvable"))?;
    Ok(home.join(".bestel").join("skills"))
}

fn read_disk_templates(dir: &PathBuf) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let read_dir = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return out,
    };
    for entry in read_dir.flatten() {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let stem = match p.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        if let Ok(text) = fs::read_to_string(&p) {
            out.insert(stem, text);
        }
    }
    out
}

/// Seed bundled skills onto disk under `~/.bestel/skills/` if missing.
/// Mirrors `prompts::seed_references_if_missing`. Idempotent.
pub fn seed_skills_if_missing() -> Result<()> {
    let root = skills_dir()?;
    fs::create_dir_all(&root)?;
    for (rel, content) in BUNDLED_SKILLS {
        let dest = root.join(rel);
        if dest.exists() {
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, content)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_skill_md_extracts_frontmatter_and_body() {
        let raw = "+++\nname = \"foo\"\ndescription = \"bar\"\nwhen_to_use = [\"a\"]\n+++\n\n# Body\nhello\n";
        let (fm, body) = parse_skill_md(raw).unwrap();
        assert_eq!(fm.name, "foo");
        assert_eq!(fm.description, "bar");
        assert_eq!(fm.when_to_use, vec!["a".to_string()]);
        assert!(body.starts_with("# Body"));
    }

    #[test]
    fn parse_skill_md_rejects_missing_frontmatter() {
        let err = parse_skill_md("# no frontmatter").unwrap_err();
        assert!(err.to_string().contains("+++"));
    }

    #[test]
    fn parse_skill_md_rejects_unclosed_frontmatter() {
        let err = parse_skill_md("+++\nname = \"x\"\ndescription = \"y\"\n").unwrap_err();
        assert!(err.to_string().to_lowercase().contains("closing"));
    }

    #[test]
    fn parse_skill_md_rejects_empty_name() {
        let err = parse_skill_md("+++\nname = \"\"\ndescription = \"x\"\n+++\nbody\n").unwrap_err();
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn list_bundled_skills_returns_three_entries_with_templates() {
        let skills = list_bundled_skills().unwrap();
        let names: Vec<_> = skills.iter().map(|s| s.frontmatter.name.as_str()).collect();
        assert!(names.contains(&"build-review"));
        assert!(names.contains(&"craft-audit"));
        assert!(names.contains(&"mapping-strategy"));
        let build_review = skills
            .iter()
            .find(|s| s.frontmatter.name == "build-review")
            .unwrap();
        assert!(build_review.templates.contains_key("quick_audit"));
    }

    #[test]
    fn descriptions_block_lists_three_named_skills() {
        let block = descriptions_block();
        assert!(block.contains("build-review"));
        assert!(block.contains("craft-audit"));
        assert!(block.contains("mapping-strategy"));
        assert!(block.contains("load_skill"));
    }

    #[test]
    fn load_skill_rejects_path_traversal() {
        for unsafe_name in &["../etc", "build-review/..", "/abs", "with space"] {
            let err = load_skill(unsafe_name).unwrap_err();
            assert!(err.to_string().contains("invalid"));
        }
    }

    #[test]
    fn load_skill_returns_bundled_when_disk_missing() {
        let skill = load_skill("build-review").unwrap();
        assert_eq!(skill.frontmatter.name, "build-review");
        assert!(!skill.body.is_empty());
    }

    #[test]
    fn load_skill_unknown_lists_alternatives() {
        let err = load_skill("not-a-real-skill").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("build-review"));
        assert!(msg.contains("craft-audit"));
    }
}

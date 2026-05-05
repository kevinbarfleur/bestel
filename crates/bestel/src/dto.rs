use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use bestel_core::llm::detect::{Detection, Probe};
use bestel_core::llm::models::{CostTier, ModelProfile, ProviderKind, SpeedTier};
use bestel_core::llm::{Attachment, LlmDelta, ToolStatus};
use bestel_core::pob::dict::{keystones_in, notables_in};
use bestel_core::pob::{PobBuild, PobBuildSummary, PoeVersion};

#[derive(Debug, Clone, Deserialize)]
pub struct AttachmentDto {
    pub name: String,
    pub mime: String,
    pub data_base64: String,
}

impl From<&AttachmentDto> for Attachment {
    fn from(a: &AttachmentDto) -> Self {
        Attachment {
            name: a.name.clone(),
            mime: a.mime.clone(),
            data_base64: a.data_base64.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PobBuildSummaryDto {
    pub path: String,
    pub file_name: String,
    pub game: &'static str,
    pub class: String,
    pub ascendancy: Option<String>,
    pub level: Option<u32>,
    pub main_skill_hint: Option<String>,
    pub mtime_ms: Option<u64>,
    pub header: String,
}

impl From<&PobBuildSummary> for PobBuildSummaryDto {
    fn from(s: &PobBuildSummary) -> Self {
        let file_name = s
            .path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self {
            path: s.path.display().to_string(),
            file_name,
            game: game_label(s.game),
            class: s.class.clone(),
            ascendancy: s.ascendancy.clone(),
            level: s.level,
            main_skill_hint: s.main_skill_hint.clone(),
            mtime_ms: s.mtime.and_then(systime_ms),
            header: s.header(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ResistanceDto {
    pub name: &'static str,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillGemDto {
    pub name: String,
    pub level: Option<u32>,
    pub quality: Option<u32>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillGroupDto {
    pub label: String,
    pub is_main: bool,
    pub slot: Option<String>,
    pub gems: Vec<SkillGemDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemSummaryDto {
    pub id: String,
    pub slot: Option<String>,
    pub rarity: Option<String>,
    pub name: Option<String>,
    pub base: Option<String>,
    /// Full PoB-format text (rarity + name + base + mods + sockets + item level).
    /// Used by the frontend to render in-game-style tooltips.
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PassiveNodeDto {
    pub id: u32,
    pub name: String,
    pub kind: &'static str,
    pub description: Option<String>,
    pub ascendancy: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PobBuildDto {
    pub source_file: String,
    pub file_name: String,
    pub game: &'static str,
    pub class: String,
    pub ascendancy: Option<String>,
    pub level: Option<u32>,
    pub summary_line: String,
    pub main_skill: Option<String>,
    pub life: Option<f64>,
    pub mana: Option<f64>,
    pub energy_shield: Option<f64>,
    pub ehp: Option<f64>,
    pub dps: Option<f64>,
    pub resistances: Vec<ResistanceDto>,
    pub skill_groups: Vec<SkillGroupDto>,
    pub items: Vec<ItemSummaryDto>,
    pub passive_tree_url: Option<String>,
    pub notes: String,
    pub allocated_keystones: Vec<PassiveNodeDto>,
    pub allocated_notables: Vec<PassiveNodeDto>,
}

impl From<&PobBuild> for PobBuildDto {
    fn from(b: &PobBuild) -> Self {
        let file_name = b
            .source_file
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();

        let resistances = b
            .resistances()
            .iter()
            .map(|(name, value)| ResistanceDto {
                name,
                value: *value,
            })
            .collect();

        let skill_groups = b
            .skill_groups
            .iter()
            .map(|g| SkillGroupDto {
                label: g.label.clone(),
                is_main: g.is_main,
                slot: g.slot.clone(),
                gems: g
                    .gems
                    .iter()
                    .map(|gem| SkillGemDto {
                        name: gem.name.clone(),
                        level: gem.level,
                        quality: gem.quality,
                        enabled: gem.enabled,
                    })
                    .collect(),
            })
            .collect();

        let items = b
            .items
            .iter()
            .map(|it| ItemSummaryDto {
                id: it.id.clone(),
                slot: b
                    .slot_map
                    .iter()
                    .find_map(|(slot, item_id)| (*item_id == it.id).then(|| slot.clone())),
                rarity: it.rarity.clone(),
                name: it.name.clone(),
                base: it.base.clone(),
                raw_text: it.raw.clone(),
            })
            .collect();

        let allocated_keystones = keystones_in(b.game, &b.allocated_nodes)
            .into_iter()
            .map(passive_node_dto_from)
            .collect();

        let allocated_notables = notables_in(b.game, &b.allocated_nodes)
            .into_iter()
            .map(passive_node_dto_from)
            .collect();

        Self {
            source_file: b.source_file.display().to_string(),
            file_name,
            game: game_label(b.game),
            class: b.class.clone(),
            ascendancy: b.ascendancy.clone(),
            level: b.level,
            summary_line: b.summary_line(),
            main_skill: b.main_skill.clone(),
            life: b.life(),
            mana: b.mana(),
            energy_shield: b.energy_shield(),
            ehp: b.ehp(),
            dps: b.dps(),
            resistances,
            skill_groups,
            items,
            passive_tree_url: b.passive_tree_url.clone(),
            notes: b.notes.clone(),
            allocated_keystones,
            allocated_notables,
        }
    }
}

fn passive_node_dto_from(n: &bestel_core::pob::dict::NodeInfo) -> PassiveNodeDto {
    PassiveNodeDto {
        id: n.id,
        name: n.name.clone(),
        kind: n.kind.label(),
        description: if n.description.is_empty() {
            None
        } else {
            Some(n.description.clone())
        },
        ascendancy: n.ascendancy.clone(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelProfileDto {
    pub id: &'static str,
    pub provider: &'static str,
    pub model_id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub speed: &'static str,
    pub cost: &'static str,
    pub cost_per_mtok: Option<(f32, f32)>,
}

impl From<&ModelProfile> for ModelProfileDto {
    fn from(p: &ModelProfile) -> Self {
        Self {
            id: p.id,
            provider: provider_kind_label(p.provider),
            model_id: p.model_id,
            display_name: p.display_name,
            description: p.description,
            speed: speed_label(p.speed),
            cost: cost_label(p.cost),
            cost_per_mtok: p.cost_per_mtok,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeDto {
    pub name: &'static str,
    pub installed: bool,
    pub version: Option<String>,
    pub note: Option<String>,
}

impl From<&Probe> for ProbeDto {
    fn from(p: &Probe) -> Self {
        Self {
            name: p.name,
            installed: p.installed,
            version: p.version.clone(),
            note: p.note.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DetectionDto {
    pub active_provider: Option<String>,
    pub probes: Vec<ProbeDto>,
}

impl From<&Detection> for DetectionDto {
    fn from(d: &Detection) -> Self {
        Self {
            active_provider: d.provider.as_ref().map(|p| p.label()),
            probes: d.probes.iter().map(ProbeDto::from).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DeltaEvent {
    Text {
        session_id: u64,
        text: String,
    },
    ReasoningBegin {
        session_id: u64,
    },
    ReasoningDelta {
        session_id: u64,
        text: String,
    },
    ReasoningEnd {
        session_id: u64,
    },
    ToolBegin {
        session_id: u64,
        id: String,
        name: String,
        detail: Option<String>,
    },
    ToolOutput {
        session_id: u64,
        id: String,
        chunk: String,
    },
    ToolEnd {
        session_id: u64,
        id: String,
        status: &'static str,
        summary: Option<String>,
    },
    MessageEnd {
        session_id: u64,
    },
    Error {
        session_id: u64,
        message: String,
    },
    Cancelled {
        session_id: u64,
    },
    Completed {
        session_id: u64,
    },
}

impl DeltaEvent {
    pub fn from_delta(session_id: u64, delta: LlmDelta) -> Self {
        match delta {
            LlmDelta::TextDelta(text) => DeltaEvent::Text { session_id, text },
            LlmDelta::ReasoningBegin => DeltaEvent::ReasoningBegin { session_id },
            LlmDelta::ReasoningDelta(text) => DeltaEvent::ReasoningDelta { session_id, text },
            LlmDelta::ReasoningEnd => DeltaEvent::ReasoningEnd { session_id },
            LlmDelta::ToolBegin { id, name, detail } => DeltaEvent::ToolBegin {
                session_id,
                id,
                name,
                detail,
            },
            LlmDelta::ToolOutput { id, chunk } => DeltaEvent::ToolOutput {
                session_id,
                id,
                chunk,
            },
            LlmDelta::ToolEnd { id, status, summary } => DeltaEvent::ToolEnd {
                session_id,
                id,
                status: tool_status_label(status),
                summary,
            },
            LlmDelta::MessageEnd => DeltaEvent::MessageEnd { session_id },
            LlmDelta::Error(message) => DeltaEvent::Error {
                session_id,
                message,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildEvent {
    pub summary: PobBuildSummaryDto,
    pub build: PobBuildDto,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderStatusEvent {
    pub detection: DetectionDto,
}

fn game_label(g: PoeVersion) -> &'static str {
    match g {
        PoeVersion::Poe1 => "poe1",
        PoeVersion::Poe2 => "poe2",
    }
}

fn provider_kind_label(p: ProviderKind) -> &'static str {
    match p {
        ProviderKind::Anthropic => "anthropic",
        ProviderKind::CodexCli => "codex_cli",
        ProviderKind::ClaudeCli => "claude_cli",
        ProviderKind::Ollama => "ollama",
    }
}

fn speed_label(s: SpeedTier) -> &'static str {
    match s {
        SpeedTier::Fast => "fast",
        SpeedTier::Balanced => "balanced",
        SpeedTier::Heavy => "heavy",
    }
}

fn cost_label(c: CostTier) -> &'static str {
    match c {
        CostTier::Free => "free",
        CostTier::Cheap => "cheap",
        CostTier::Mid => "mid",
        CostTier::Premium => "premium",
        CostTier::Subscription => "subscription",
    }
}

fn tool_status_label(s: ToolStatus) -> &'static str {
    match s {
        ToolStatus::Running => "running",
        ToolStatus::Done => "done",
        ToolStatus::Failed => "failed",
    }
}

fn systime_ms(t: SystemTime) -> Option<u64> {
    t.duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64)
}

/// Helper used by `PobBuildSummary` synthesised from a freshly parsed `PobBuild`.
pub fn summary_dto_from_build(b: &PobBuild, mtime_ms: Option<u64>) -> PobBuildSummaryDto {
    let file_name = b
        .source_file
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_default();
    PobBuildSummaryDto {
        path: b.source_file.display().to_string(),
        file_name,
        game: game_label(b.game),
        class: b.class.clone(),
        ascendancy: b.ascendancy.clone(),
        level: b.level,
        main_skill_hint: b.main_skill.clone(),
        mtime_ms,
        header: b.summary_line(),
    }
}

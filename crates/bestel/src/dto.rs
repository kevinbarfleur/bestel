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

/// Lightweight summary of a stored Build Sheet — what the BuildPicker
/// needs to render the row list (name, fingerprint, version, timestamps).
/// `payload` is intentionally NOT included: full sheet bodies are fetched
/// on demand via `get_build_sheet` when the user selects a row.
#[derive(Debug, Clone, Serialize)]
pub struct BuildSheetSummaryDto {
    pub id: String,
    pub fingerprint: String,
    pub pob_hash: String,
    pub name: String,
    pub schema_version: i64,
    pub authored_at: String,
    pub updated_at: String,
    pub authored_in_chat: Option<String>,
    pub validated: bool,
}

/// Full Build Sheet detail — summary fields + the parsed payload as a
/// JSON value. Used by the BuildPicker's right-pane preview and any
/// future "view full" modal that wants to read sections directly.
#[derive(Debug, Clone, Serialize)]
pub struct BuildSheetDetailDto {
    pub id: String,
    pub fingerprint: String,
    pub pob_hash: String,
    pub name: String,
    pub schema_version: i64,
    pub authored_at: String,
    pub updated_at: String,
    pub authored_in_chat: Option<String>,
    pub validated: bool,
    pub payload: serde_json::Value,
}

/// One verified claim as surfaced to the chat tool card. Mirrors
/// `bestel_core::llm::verifier::VerifiedClaim` but keeps the DTO crate
/// boundary clean.
#[derive(Debug, Clone, Serialize)]
pub struct VerifiedClaimDto {
    pub statement: String,
    pub topic: String,
    /// `"ok" | "wrong" | "unverified"` — same lowercase variants as the
    /// core enum so the frontend can render with a matching pill.
    pub status: String,
    pub evidence_excerpt: String,
    pub correction: Option<String>,
}

impl From<bestel_core::llm::verifier::VerifiedClaim> for VerifiedClaimDto {
    fn from(c: bestel_core::llm::verifier::VerifiedClaim) -> Self {
        let status = match c.status {
            bestel_core::llm::verifier::ClaimStatus::Ok => "ok",
            bestel_core::llm::verifier::ClaimStatus::Wrong => "wrong",
            bestel_core::llm::verifier::ClaimStatus::Unverified => "unverified",
        };
        Self {
            statement: c.statement,
            topic: c.topic,
            status: status.to_string(),
            evidence_excerpt: c.evidence_excerpt,
            correction: c.correction,
        }
    }
}

/// Status of one whitelisted API key env var, as exposed to the picker UI.
/// `masked_preview` is `Some("sk-ant-...4f2c")` when we can safely echo a
/// short fingerprint, `Some("(from environment)")` for OS-env-only keys
/// (we never echo the raw OS value), or `None` when the key is unset.
#[derive(Debug, Clone, Serialize)]
pub struct KeyStatusDto {
    pub env_name: String,
    pub set: bool,
    pub masked_preview: Option<String>,
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
    /// Effective cap for this resistance (75 by default, higher when the
    /// build raises the max via Purity, Aegis Aurora, etc.). Read from the
    /// PoB `{Element}ResistMax` stat if present, falls back to 75.
    pub cap: f64,
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
    /// PoE2 only — `Spirit` reservation pool. Populated from the parsed
    /// defenses block.
    pub spirit: Option<f64>,
    /// PoE1 evasion rating. Populated from the parsed defenses block.
    pub evasion: Option<f64>,
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

        // Resolve the per-element cap from PoB stats if present.
        // PoB exports `{Element}ResistMax` only when the build's max is raised
        // above the default 75 (Purity of Fire, Aegis Aurora, etc.).
        let cap_for = |element: &str| -> f64 {
            let stat_key = match element {
                "fire" => "FireResistMax",
                "cold" => "ColdResistMax",
                "lightning" => "LightningResistMax",
                "chaos" => "ChaosResistMax",
                _ => return 75.0,
            };
            b.stat(stat_key).unwrap_or(75.0)
        };

        let resistances = b
            .resistances()
            .iter()
            .map(|(name, value)| ResistanceDto {
                name,
                value: *value,
                cap: cap_for(name),
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
            spirit: b.defenses.spirit,
            evasion: b.defenses.evasion,
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
    pub id: String,
    pub provider: &'static str,
    pub model_id: String,
    pub display_name: String,
    pub description: String,
    pub speed: &'static str,
    pub cost: &'static str,
    pub cost_per_mtok: Option<(f32, f32)>,
    /// When set, the picker should check this specific env var probe for
    /// availability instead of the generic provider probe. Used for
    /// Anthropic-compatible third-party endpoints (e.g. DeepSeek) that
    /// require their own API key.
    pub api_key_env: Option<String>,
    /// Optional link to the official model documentation page.
    pub info_url: Option<String>,
    /// Optional link to where the user can obtain or manage the API key.
    pub api_key_url: Option<String>,
    /// Whether the model accepts image attachments.
    pub vision_capable: bool,
}

impl From<&ModelProfile> for ModelProfileDto {
    fn from(p: &ModelProfile) -> Self {
        Self {
            id: p.id.clone(),
            provider: provider_kind_label(p.provider),
            model_id: p.model_id.clone(),
            display_name: p.display_name.clone(),
            description: p.description.clone(),
            speed: speed_label(p.speed),
            cost: cost_label(p.cost),
            cost_per_mtok: p.cost_per_mtok,
            api_key_env: p.api_key_env.clone(),
            info_url: p.info_url.clone(),
            api_key_url: p.api_key_url.clone(),
            vision_capable: p.vision_capable,
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
    ToolDetailUpdate {
        session_id: u64,
        id: String,
        summary_input: String,
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
    Usage {
        session_id: u64,
        input_tokens: u64,
        cached_input_tokens: u64,
        cache_creation_tokens: u64,
        output_tokens: u64,
        cost_usd: Option<f64>,
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
    /// Verifier verdict — Chain-of-Verification factored pipeline. Carries
    /// the legacy Sprint G fields (`status`, `findings_count`,
    /// `findings_summary`) for the dev panel plus the per-claim audit list
    /// (`claims_checked`, `corrections_count`) that drives the slim chat
    /// tool card. Empty `claims_checked` means the heuristic skipped (cheap
    /// draft) or the toggle is off.
    Verifier {
        session_id: u64,
        status: String,
        findings_count: usize,
        findings_summary: String,
        claims_checked: Vec<VerifiedClaimDto>,
        corrections_count: usize,
    },
    /// Sprint v6 Phase 2 — live response-lint findings (warn-only). Emitted
    /// when at least one of the 12 lint rules fired on this turn's
    /// `PersistedRun`. Only used for data collection in the dev panel;
    /// Phase 3 will turn this into a gating signal.
    LintFindings {
        session_id: u64,
        findings: Vec<bestel_core::test_runner::response_lint::LintFinding>,
    },
    /// Build Sheets — agent-drafted (or re-drafted) section. Front-end
    /// patches the matching `BSDraftedCard` in place.
    SheetDraftUpdate {
        session_id: u64,
        section_id: String,
        title: String,
        body: String,
        confirmed: bool,
    },
    /// Build Sheets — questions_v2-style picker for a personality question.
    SheetAskUser {
        session_id: u64,
        question_id: String,
        title: String,
        subtitle: Option<String>,
        options: Vec<String>,
        multi: bool,
        has_other: bool,
    },
    /// Build Sheets — one-shot interview opened by the agent after deep PoB
    /// analysis. Frontend renders a single `BSInterviewPanel` with all
    /// sections + questions in one tall card. The payload is opaque JSON
    /// (validated server-side at dispatch time, see ref 32 for shape).
    SheetInterviewOpen {
        session_id: u64,
        payload: serde_json::Value,
    },
    /// Build Sheets — sheet finalized + persisted. Frontend renders a small
    /// `BSSheetSavedBanner` segment in the chat timeline.
    SheetFinalized {
        session_id: u64,
        sheet_id: String,
        name: String,
    },
    /// Build Sheets — a validated sheet became active for this chat. Fired
    /// after both finalize and lookup; populates the sidebar
    /// `BSLinkedSheetCard` via `sheet.loadActiveSheet(...)` on the frontend.
    SheetLoaded {
        session_id: u64,
        sheet_id: String,
        fingerprint: String,
        name: String,
        pob_hash: String,
        stale: bool,
        authored_at: String,
        updated_at: String,
        schema_version: i64,
        payload: serde_json::Value,
    },
    /// Sprint v3 — deterministic mode classification emitted once per turn
    /// before the first LLM call. Renders a `ModeChip` over the assistant
    /// message for non-default modes (brief-mechanic, deep-audit,
    /// legacy-diagnostic, refusal).
    ModeAssigned {
        session_id: u64,
        mode: String,
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
            LlmDelta::ToolDetailUpdate { id, summary_input } => DeltaEvent::ToolDetailUpdate {
                session_id,
                id,
                summary_input,
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
            LlmDelta::Usage(stats) => DeltaEvent::Usage {
                session_id,
                input_tokens: stats.input_tokens,
                cached_input_tokens: stats.cached_input_tokens,
                cache_creation_tokens: stats.cache_creation_tokens,
                output_tokens: stats.output_tokens,
                cost_usd: stats.cost_usd,
            },
            LlmDelta::Verifier(v) => {
                let status = match v.status {
                    bestel_core::llm::verifier::VerdictStatus::Pass => "pass",
                    bestel_core::llm::verifier::VerdictStatus::Revise => "revise",
                    bestel_core::llm::verifier::VerdictStatus::Fail => "fail",
                };
                let summary = if v.findings.is_empty() {
                    String::new()
                } else {
                    v.findings
                        .iter()
                        .map(|f| format!("{}: {}", f.category, f.issue))
                        .collect::<Vec<_>>()
                        .join("; ")
                };
                let claims_checked: Vec<VerifiedClaimDto> = v
                    .claims_checked
                    .into_iter()
                    .map(VerifiedClaimDto::from)
                    .collect();
                DeltaEvent::Verifier {
                    session_id,
                    status: status.to_string(),
                    findings_count: v.findings.len(),
                    findings_summary: summary,
                    claims_checked,
                    corrections_count: v.corrections_count,
                }
            }
            LlmDelta::SheetDraftUpdate {
                section_id,
                title,
                body,
                confirmed,
            } => DeltaEvent::SheetDraftUpdate {
                session_id,
                section_id,
                title,
                body,
                confirmed,
            },
            LlmDelta::SheetAskUser {
                question_id,
                title,
                subtitle,
                options,
                multi,
                has_other,
            } => DeltaEvent::SheetAskUser {
                session_id,
                question_id,
                title,
                subtitle,
                options,
                multi,
                has_other,
            },
            LlmDelta::SheetInterviewOpen { payload } => DeltaEvent::SheetInterviewOpen {
                session_id,
                payload,
            },
            LlmDelta::SheetFinalized { sheet_id, name } => DeltaEvent::SheetFinalized {
                session_id,
                sheet_id,
                name,
            },
            LlmDelta::SheetLoaded {
                sheet_id,
                fingerprint,
                name,
                pob_hash,
                stale,
                authored_at,
                updated_at,
                schema_version,
                payload,
            } => DeltaEvent::SheetLoaded {
                session_id,
                sheet_id,
                fingerprint,
                name,
                pob_hash,
                stale,
                authored_at,
                updated_at,
                schema_version,
                payload,
            },
            LlmDelta::ModeAssigned { mode } => DeltaEvent::ModeAssigned { session_id, mode },
            LlmDelta::LintFindings { findings } => DeltaEvent::LintFindings {
                session_id,
                findings,
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

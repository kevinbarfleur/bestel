export type Game = 'poe1' | 'poe2';

export interface AttachmentDto {
  name: string;
  mime: string;
  data_base64: string;
}

export interface PobBuildSummaryDto {
  path: string;
  file_name: string;
  game: Game;
  class: string;
  ascendancy: string | null;
  level: number | null;
  main_skill_hint: string | null;
  mtime_ms: number | null;
  header: string;
}

/** Lightweight Build Sheet row used by the BuildPicker's sheets pane.
 *  Mirror of `crates/bestel/src/dto.rs::BuildSheetSummaryDto`. The full
 *  payload is fetched on demand via `getBuildSheet`. */
export interface BuildSheetSummaryDto {
  id: string;
  fingerprint: string;
  pob_hash: string;
  name: string;
  schema_version: number;
  authored_at: string;
  updated_at: string;
  authored_in_chat: string | null;
  validated: boolean;
}

/** Full Build Sheet detail — summary fields + parsed payload (sections,
 *  defining_items, intent, known_gaps). */
export interface BuildSheetDetailDto extends BuildSheetSummaryDto {
  payload: unknown;
}

export interface ResistanceDto {
  name: 'fire' | 'cold' | 'lightning' | 'chaos';
  value: number | null;
  /** Effective cap (75 by default, higher when the build raises the max). */
  cap: number;
}

export interface SkillGemDto {
  name: string;
  level: number | null;
  quality: number | null;
  enabled: boolean;
}

export interface SkillGroupDto {
  label: string;
  is_main: boolean;
  slot: string | null;
  gems: SkillGemDto[];
}

export interface ItemSummaryDto {
  id: string;
  slot: string | null;
  rarity: string | null;
  name: string | null;
  base: string | null;
  raw_text: string;
}

export interface PassiveNodeDto {
  id: number;
  name: string;
  kind: string;
  description: string | null;
  ascendancy: string | null;
}

export interface PobBuildDto {
  source_file: string;
  file_name: string;
  game: Game;
  class: string;
  ascendancy: string | null;
  level: number | null;
  summary_line: string;
  main_skill: string | null;
  life: number | null;
  mana: number | null;
  energy_shield: number | null;
  /** PoE2 only — Spirit reservation pool. */
  spirit: number | null;
  /** PoE1 evasion rating. */
  evasion: number | null;
  ehp: number | null;
  dps: number | null;
  resistances: ResistanceDto[];
  skill_groups: SkillGroupDto[];
  items: ItemSummaryDto[];
  passive_tree_url: string | null;
  notes: string;
  allocated_keystones: PassiveNodeDto[];
  allocated_notables: PassiveNodeDto[];
}

export type ProviderKindLabel = 'anthropic' | 'ollama';
export type SpeedLabel = 'fast' | 'balanced' | 'heavy';
export type CostLabel = 'free' | 'cheap' | 'mid' | 'premium' | 'subscription';

export interface ModelProfileDto {
  id: string;
  provider: ProviderKindLabel;
  model_id: string;
  display_name: string;
  description: string;
  speed: SpeedLabel;
  cost: CostLabel;
  cost_per_mtok: [number, number] | null;
  api_key_env: string | null;
  /** Validated official link for model docs / product page. */
  info_url: string | null;
  /** Validated official link to obtain or manage the API key. */
  api_key_url: string | null;
  /** Whether this model accepts image attachments (vision input). */
  vision_capable: boolean;
}

export interface ProbeDto {
  name: string;
  installed: boolean;
  version: string | null;
  note: string | null;
}

export interface KeyStatusDto {
  env_name: string;
  set: boolean;
  /** Either a masked fingerprint like "sk-ant-...4f2c", the literal
   * "(from environment)" for OS-env-only keys (no preview echoed),
   * or null when the key is unset. */
  masked_preview: string | null;
}

export interface DetectionDto {
  active_provider: string | null;
  probes: ProbeDto[];
}

export interface SettingsDto {
  schema_version: number;
  verify_enabled: boolean;
}

export interface BuildEvent {
  summary: PobBuildSummaryDto;
  build: PobBuildDto;
}

export interface ProviderStatusEvent {
  detection: DetectionDto;
}

export type ToolStatus = 'running' | 'done' | 'failed';

export interface DebugRunStats {
  elapsed_ms: number;
  text_bytes: number;
  reasoning_bytes: number;
  tool_calls: number;
  tool_done: number;
  tool_failed: number;
  // Sprint C cache telemetry. Older persisted runs without these fields
  // deserialize to 0 / null thanks to serde defaults on the Rust side.
  input_tokens?: number;
  cached_input_tokens?: number;
  cache_creation_tokens?: number;
  output_tokens?: number;
  cost_usd?: number | null;
}

export interface DebugRunAttachment {
  name: string;
  mime: string;
}

export type DebugRunSegment =
  | { kind: 'text'; text: string }
  | { kind: 'reasoning'; text: string }
  | {
      kind: 'tool';
      id: string;
      name: string;
      detail: string | null;
      outputs: string[];
      status: ToolStatus;
      summary: string | null;
    };

export interface DebugRunDto {
  id: string;
  started_at: string;
  ended_at: string | null;
  source: string;
  provider: string;
  model_id: string;
  model_display_name: string;
  user_text: string;
  user_attachments: DebugRunAttachment[];
  assistant_segments: DebugRunSegment[];
  final_text: string;
  stats: DebugRunStats;
  error: string | null;
}

export type LintSeverity = 'fail' | 'warn';

export interface LintFinding {
  id: string;
  severity: LintSeverity;
  message: string;
  evidence: string | null;
}

export interface LintReport {
  findings: LintFinding[];
}

export interface UsageStats {
  input_tokens: number;
  cached_input_tokens: number;
  cache_creation_tokens: number;
  output_tokens: number;
  cost_usd: number | null;
}

/** One claim audited by the CoVe verifier. Mirrors `VerifiedClaimDto`
 * on the Rust side. The slim chat tool card shows a roll-up; clicking
 * expand reveals the per-claim list using these. */
export interface VerifiedClaimDto {
  statement: string;
  topic: string;
  /** Lowercase variant from the Rust enum: matches the pill colour. */
  status: 'ok' | 'wrong' | 'unverified';
  evidence_excerpt: string;
  correction: string | null;
}

export type LlmDeltaEvent =
  | { kind: 'text'; session_id: number; text: string }
  | { kind: 'reasoning_begin'; session_id: number }
  | { kind: 'reasoning_delta'; session_id: number; text: string }
  | { kind: 'reasoning_end'; session_id: number }
  | { kind: 'tool_begin'; session_id: number; id: string; name: string; detail: string | null }
  | { kind: 'tool_detail_update'; session_id: number; id: string; summary_input: string }
  | { kind: 'tool_output'; session_id: number; id: string; chunk: string }
  | { kind: 'tool_end'; session_id: number; id: string; status: ToolStatus; summary: string | null }
  | ({ kind: 'usage'; session_id: number } & UsageStats)
  | { kind: 'message_end'; session_id: number }
  | { kind: 'error'; session_id: number; message: string }
  | { kind: 'cancelled'; session_id: number }
  | { kind: 'completed'; session_id: number }
  | {
      kind: 'sheet_draft_update';
      session_id: number;
      section_id: string;
      title: string;
      body: string;
      confirmed: boolean;
    }
  | {
      kind: 'sheet_ask_user';
      session_id: number;
      question_id: string;
      title: string;
      subtitle: string | null;
      options: string[];
      multi: boolean;
      has_other: boolean;
    }
  | {
      kind: 'sheet_interview_open';
      session_id: number;
      payload: {
        sections: { id: string; title: string; draft_body: string }[];
        questions: {
          question_id: string;
          section_id: string;
          title: string;
          subtitle?: string;
          options: string[];
          multi?: boolean;
          has_other?: boolean;
        }[];
        notes_prompt: string;
      };
    }
  | {
      kind: 'sheet_finalized';
      session_id: number;
      sheet_id: string;
      name: string;
    }
  | {
      kind: 'sheet_loaded';
      session_id: number;
      sheet_id: string;
      fingerprint: string;
      name: string;
      pob_hash: string;
      stale: boolean;
      authored_at: string;
      updated_at: string;
      schema_version: number;
      payload: unknown;
    }
  | {
      kind: 'verifier';
      session_id: number;
      /** `pass` | `revise` | `fail`. Pass with empty `claims_checked`
       * means the heuristic skipped or the toggle was off — no card. */
      status: string;
      findings_count: number;
      findings_summary: string;
      claims_checked: VerifiedClaimDto[];
      corrections_count: number;
    };

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

export type ProviderKindLabel = 'anthropic' | 'codex_cli' | 'claude_cli' | 'ollama';
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

export interface UsageStats {
  input_tokens: number;
  cached_input_tokens: number;
  cache_creation_tokens: number;
  output_tokens: number;
  cost_usd: number | null;
}

export type LlmDeltaEvent =
  | { kind: 'text'; session_id: number; text: string }
  | { kind: 'reasoning_begin'; session_id: number }
  | { kind: 'reasoning_delta'; session_id: number; text: string }
  | { kind: 'reasoning_end'; session_id: number }
  | { kind: 'tool_begin'; session_id: number; id: string; name: string; detail: string | null }
  | { kind: 'tool_output'; session_id: number; id: string; chunk: string }
  | { kind: 'tool_end'; session_id: number; id: string; status: ToolStatus; summary: string | null }
  | ({ kind: 'usage'; session_id: number } & UsageStats)
  | { kind: 'message_end'; session_id: number }
  | { kind: 'error'; session_id: number; message: string }
  | { kind: 'cancelled'; session_id: number }
  | { kind: 'completed'; session_id: number };

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

export type ProviderKindLabel = 'anthropic' | 'codex_cli' | 'claude_cli';
export type SpeedLabel = 'fast' | 'balanced' | 'heavy';
export type CostLabel = 'cheap' | 'mid' | 'premium' | 'subscription';

export interface ModelProfileDto {
  id: string;
  provider: ProviderKindLabel;
  model_id: string;
  display_name: string;
  description: string;
  speed: SpeedLabel;
  cost: CostLabel;
  cost_per_mtok: [number, number] | null;
}

export interface ProbeDto {
  name: string;
  installed: boolean;
  version: string | null;
  note: string | null;
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

export type LlmDeltaEvent =
  | { kind: 'text'; session_id: number; text: string }
  | { kind: 'reasoning_begin'; session_id: number }
  | { kind: 'reasoning_delta'; session_id: number; text: string }
  | { kind: 'reasoning_end'; session_id: number }
  | { kind: 'tool_begin'; session_id: number; id: string; name: string; detail: string | null }
  | { kind: 'tool_output'; session_id: number; id: string; chunk: string }
  | { kind: 'tool_end'; session_id: number; id: string; status: ToolStatus; summary: string | null }
  | { kind: 'message_end'; session_id: number }
  | { kind: 'error'; session_id: number; message: string }
  | { kind: 'cancelled'; session_id: number }
  | { kind: 'completed'; session_id: number };

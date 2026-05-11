<script setup lang="ts">
import { computed, ref } from 'vue';

import type { ToolSegment } from '../../../stores/chat';
import ArtShell from './ArtShell.vue';
import ArtHead from './ArtHead.vue';

const props = defineProps<{ segment: ToolSegment }>();

/**
 * Collapsed by default. The PoB import payload is ~30 KB of JSON live
 * and gets truncated to 5 KB by `persistReplacer` for localStorage,
 * which then breaks `JSON.parse(output)` after a reload → fallback to
 * raw text dump in the chat. The mini-card (rows from a successful
 * parse) is still shown when available, but the raw body — and the
 * "JSON unavailable" placeholder when parse fails — is only revealed
 * when the user clicks Expand. Matches the chevron pattern of other
 * tool segments. */
const expanded = ref(false);

const parsed = computed(() => {
  if (!props.segment.output) return null;
  try {
    return JSON.parse(props.segment.output);
  } catch {
    return null;
  }
});

const summaryLine = computed(() => {
  const p = parsed.value;
  if (!p || typeof p !== 'object') return null;
  const head = p.summary_line ?? p.header ?? p.class ?? null;
  return typeof head === 'string' ? head : null;
});

interface LeaderRow {
  k: string;
  v: string;
  vColor?: string;
}

const rows = computed<LeaderRow[]>(() => {
  const p = parsed.value;
  if (!p) return [];
  const out: LeaderRow[] = [];
  if (p.life != null || p.energy_shield != null || p.mana != null) {
    const passives = p.passive_count ?? p.passives ?? null;
    if (passives != null) out.push({ k: 'passives', v: fmt(passives) });
  }
  const items = Array.isArray(p.items) ? p.items.length : null;
  if (items != null) out.push({ k: 'items', v: `${items} / 12` });
  const gemCount = Array.isArray(p.skill_groups)
    ? p.skill_groups.reduce((acc: number, g: { gems?: unknown[] }) => acc + (g.gems?.length ?? 0), 0)
    : null;
  if (gemCount != null && gemCount > 0) out.push({ k: 'gems', v: String(gemCount) });
  if (p.dps != null) {
    out.push({ k: 'pdps', v: fmtBig(p.dps), vColor: 'var(--el-lit-deep)' });
  }
  return out;
});

const skillName = computed<string | null>(() => {
  const p = parsed.value;
  if (!p?.main_skill) return null;
  return typeof p.main_skill === 'string' ? p.main_skill : null;
});

function fmt(v: number): string {
  if (v >= 1000) return `${(v / 1000).toFixed(1)}k`;
  return Math.round(v).toString();
}
function fmtBig(v: number): string {
  if (v >= 1_000_000) return `${(v / 1_000_000).toFixed(2)}M`;
  if (v >= 1000) return `${(v / 1000).toFixed(1)}k`;
  return Math.round(v).toString();
}

const status = computed(() =>
  props.segment.status === 'running'
    ? { label: '· running', tone: 'ink' as const }
    : props.segment.status === 'done'
    ? { label: '✓ parsed', tone: 'good' as const }
    : { label: '✗ failed', tone: 'bad' as const },
);

const meta = computed(() => {
  const parts: string[] = [];
  if (skillName.value) parts.push(`· ${skillName.value}`);
  if (summaryLine.value) parts.push(`· ${summaryLine.value}`);
  return parts.join(' ');
});
</script>

<template>
  <ArtShell accent="lit">
    <ArtHead
      kind="Path of Building"
      title="Path of Building"
      :meta="meta"
      :status="status"
    />
    <div v-if="rows.length" class="pob-rows">
      <div v-for="r in rows" :key="r.k" class="leader">
        <span class="leader__k">{{ r.k }}</span>
        <span class="leader__dots" />
        <span class="leader__v" :style="r.vColor ? { color: r.vColor } : {}">{{ r.v }}</span>
      </div>
    </div>
    <button
      v-if="segment.output"
      type="button"
      class="pob-toggle"
      @click="expanded = !expanded"
    >
      {{ expanded ? '− hide raw payload' : '+ show raw payload' }}
    </button>
    <pre v-if="expanded && segment.output" class="pob-raw">{{ segment.output }}</pre>
  </ArtShell>
</template>

<style scoped>
.pob-rows {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.leader {
  display: flex;
  align-items: baseline;
  gap: 6px;
}
.leader__k {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.10em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 500;
  flex-shrink: 0;
  white-space: nowrap;
}
.leader__dots {
  flex: 1;
  min-width: 12px;
  border-bottom: 1px dotted var(--paper-line);
  transform: translateY(-3px);
}
.leader__v {
  font-family: var(--hand-display);
  font-size: 12px;
  font-weight: 600;
  color: var(--ink);
  flex: 0 0 auto;
  white-space: nowrap;
}
.pob-toggle {
  margin-top: 8px;
  background: transparent;
  border: none;
  color: var(--ink-faint);
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.10em;
  text-transform: uppercase;
  cursor: pointer;
  padding: 2px 4px;
  align-self: flex-start;
}
.pob-toggle:hover {
  color: var(--ink-soft);
}
.pob-raw {
  margin: 6px 0 0 0;
  padding: 8px 10px;
  font-family: var(--mono, ui-monospace, SFMono-Regular, monospace);
  font-size: 10px;
  line-height: 1.4;
  color: var(--ink-faint);
  background: var(--paper-tint, rgba(0, 0, 0, 0.03));
  border: 1px dotted var(--paper-line);
  border-radius: 4px;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 320px;
  overflow-y: auto;
}
</style>

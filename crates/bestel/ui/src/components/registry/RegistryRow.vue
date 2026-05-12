<script setup lang="ts">
import { computed } from 'vue';
import DriftChipStrip, { type DriftSignatures } from '../sheet/DriftChipStrip.vue';
import FreshPill from '../sheet/FreshPill.vue';
import RunicIcon from '../runic/RunicIcon.vue';
import type { RegistryEntryDto } from '../../api/types';

interface Props {
  entry: RegistryEntryDto;
  compact?: boolean;
  /** Optional drift signatures derived by comparing the entry's stored
   *  signatures against the currently-attached PoB. When omitted, the
   *  row collapses to a fresh pill or "no sheet" caption. */
  driftSigs?: DriftSignatures | null;
}
const props = withDefaults(defineProps<Props>(), { compact: false, driftSigs: null });
defineEmits<{
  (e: 'open'): void;
  (e: 'menu'): void;
  (e: 'author-sheet'): void;
  (e: 'open-sheet'): void;
}>();

const summaryLine = computed(() => {
  const s = props.entry.summary;
  const lvl = s.level ? `lvl ${s.level} · ` : '';
  const skillTail = s.main_skill ? ` · main skill ${s.main_skill}` : '';
  const asc = s.ascendancy ?? '—';
  const uniques = s.defining_uniques.length > 0 ? ` · ${s.defining_uniques.slice(0, 3).join(' · ')}` : '';
  return `${s.class} · ${asc} · ${lvl}${asc}${skillTail}${uniques}`.replace(/^\s+|\s+$/g, '');
});

const pobHashShort = computed(() => {
  const h = props.entry.pob_hash;
  if (h.length <= 12) return h;
  return `${h.slice(0, 4)}…${h.slice(-4)}`;
});

const hasSheet = computed(() => props.entry.linked_sheet_id !== null);
const allMatch = computed(() => {
  const d = props.driftSigs;
  if (!d) return true;
  return (['identity', 'tree', 'gear', 'skill', 'config'] as const).every((k) => d[k] === 'match');
});
</script>

<template>
  <div class="registry-row" :class="{ 'registry-row--compact': compact }">
    <div class="registry-row__line registry-row__line--title">
      <span class="registry-row__game">{{ entry.game }}</span>
      <span class="registry-row__name">{{ entry.display_name }}</span>
      <span class="registry-row__last-seen">
        last seen <strong>{{ new Date(entry.last_seen_at).toLocaleString() }}</strong>
      </span>
      <button class="registry-row__icon-btn" type="button" aria-label="Open chat with this build" @click="$emit('open')">
        <RunicIcon name="open" :size="14" />
      </button>
      <button class="registry-row__icon-btn" type="button" aria-label="More actions" @click="$emit('menu')">
        ⋯
      </button>
    </div>
    <div class="registry-row__summary">
      {{ entry.summary.class }} · {{ entry.summary.ascendancy ?? '—' }}
      <template v-if="entry.summary.level"> · lvl {{ entry.summary.level }}</template>
      <template v-if="entry.summary.main_skill">
        · main skill <strong>{{ entry.summary.main_skill }}</strong>
      </template>
      <template v-if="entry.summary.defining_uniques.length">
        · {{ entry.summary.defining_uniques.slice(0, 3).join(' · ') }}
      </template>
    </div>
    <div class="registry-row__line registry-row__line--meta">
      <template v-if="!hasSheet">
        <span class="registry-row__no-sheet">
          No build sheet yet —
          <button type="button" class="registry-row__link" @click="$emit('author-sheet')">author one</button>
        </span>
      </template>
      <template v-else>
        <span class="registry-row__sheet-label">
          Sheet
          <button type="button" class="registry-row__link" @click="$emit('open-sheet')">id={{ entry.linked_sheet_id }}</button>
        </span>
        <FreshPill v-if="allMatch || !driftSigs" dense />
        <DriftChipStrip v-else :sigs="driftSigs" dense />
      </template>
      <span class="registry-row__spacer" />
      <span class="registry-row__hash">pob_hash {{ pobHashShort }}</span>
    </div>
  </div>
</template>

<style scoped>
.registry-row {
  padding: 14px 18px;
  border: 1px solid var(--paper-line);
  border-radius: 5px;
  background: var(--paper);
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.registry-row--compact {
  padding: 10px 16px;
  gap: 6px;
}
.registry-row__line {
  display: flex;
  align-items: center;
  gap: 12px;
}
.registry-row__line--title {
  align-items: baseline;
}
.registry-row__game {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  color: var(--ink-faint);
  padding: 2px 6px;
  border: 1px solid var(--paper-line);
  border-radius: 3px;
  flex: none;
}
.registry-row__name {
  font-size: 18px;
  font-weight: 700;
  color: var(--ink);
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.registry-row--compact .registry-row__name {
  font-size: 16px;
}
.registry-row__last-seen {
  font-size: 13px;
  color: var(--ink-soft);
  flex: none;
}
.registry-row__last-seen strong {
  color: var(--ink);
  font-weight: 500;
}
.registry-row__icon-btn {
  width: 28px;
  height: 28px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  color: var(--ink-soft);
  border: 1px solid transparent;
  border-radius: 4px;
  cursor: pointer;
  font-size: 18px;
  font-weight: 700;
}
.registry-row__icon-btn:hover {
  background: var(--paper-shade);
  color: var(--ink);
}
.registry-row__summary {
  font-size: 14px;
  color: var(--ink-soft);
}
.registry-row__summary strong {
  color: var(--ink);
  font-weight: 600;
}
.registry-row__line--meta {
  flex-wrap: wrap;
}
.registry-row__no-sheet {
  font-size: 13px;
  color: var(--ink-faint);
}
.registry-row__sheet-label {
  font-size: 13px;
  color: var(--ink-soft);
}
.registry-row__link {
  background: transparent;
  border: none;
  padding: 0;
  cursor: pointer;
  font-family: var(--hand);
  font-size: 13px;
  font-weight: 500;
  color: var(--amber);
  text-decoration: underline;
  text-decoration-style: dotted;
  text-decoration-color: var(--amber-soft);
  text-underline-offset: 3px;
}
.registry-row__spacer {
  flex: 1;
}
.registry-row__hash {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: var(--ink-faint);
}
</style>

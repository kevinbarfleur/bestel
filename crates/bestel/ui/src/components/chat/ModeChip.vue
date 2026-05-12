<script setup lang="ts">
import { computed } from 'vue';

export type TurnMode = 'brief-mechanic' | 'deep-audit' | 'legacy-diagnostic' | 'refusal';

interface Palette {
  variant: 'audit' | 'legacy' | 'refusal' | 'brief';
  label: string;
}

const PALETTE: Record<TurnMode, Palette> = {
  'brief-mechanic': { variant: 'brief', label: 'brief mechanic · direct answer' },
  'deep-audit': { variant: 'audit', label: 'deep build audit · sheet authoring' },
  'legacy-diagnostic': { variant: 'legacy', label: 'legacy diagnostic · no sheet' },
  refusal: { variant: 'refusal', label: 'refusal · out-of-scope' },
};

const props = defineProps<{ mode: TurnMode }>();
const palette = computed(() => PALETTE[props.mode]);
</script>

<template>
  <div class="mode-chip-row">
    <span class="mode-chip" :class="`mode-chip--${palette.variant}`">
      <span class="mode-chip__bullet" aria-hidden="true">
        <svg viewBox="0 0 12 12" fill="none">
          <circle cx="6" cy="6" r="4" fill="currentColor" opacity="0.35" />
          <circle cx="6" cy="6" r="2.4" fill="currentColor" />
        </svg>
      </span>
      <span class="mode-chip__label">mode</span>
      <span class="mode-chip__sep">·</span>
      <span class="mode-chip__value">{{ palette.label }}</span>
    </span>
    <slot name="action" />
  </div>
</template>

<style scoped>
.mode-chip-row {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.mode-chip {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 3px 10px;
  border-radius: 12px;
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  font-weight: 700;
  line-height: 1;
  border: 1px solid currentColor;
}
.mode-chip--audit {
  color: var(--amber);
  background: var(--amber-glow);
  border-color: var(--amber);
}
.mode-chip--legacy {
  color: var(--ink-soft);
  background: var(--paper-shade);
  border-color: var(--ink-faint);
}
.mode-chip--refusal {
  color: var(--bad);
  background: rgba(164, 72, 72, 0.08);
  border-color: var(--bad);
}
.mode-chip--brief {
  color: var(--good);
  background: rgba(84, 124, 74, 0.10);
  border-color: var(--good);
}
.mode-chip__bullet svg {
  width: 9px;
  height: 9px;
  display: block;
}
.mode-chip__sep {
  opacity: 0.45;
}
</style>

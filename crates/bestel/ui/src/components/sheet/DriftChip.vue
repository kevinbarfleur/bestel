<script setup lang="ts">
import { computed } from 'vue';

export type DriftAxis = 'identity' | 'tree' | 'gear' | 'skill' | 'config';
export type DriftState = 'match' | 'drift' | 'na';

interface Props {
  kind: DriftAxis;
  state: DriftState;
  dense?: boolean;
  iconsOnly?: boolean;
}
const props = withDefaults(defineProps<Props>(), { dense: false, iconsOnly: false });
defineEmits<{ (e: 'click'): void }>();

const LABELS: Record<DriftAxis, string> = {
  identity: 'identity',
  tree: 'tree',
  gear: 'gear',
  skill: 'skill',
  config: 'config',
};
const TIPS: Record<DriftAxis, string> = {
  identity: 'Class + ascendancy + main skill. A change here means the build is effectively a different character.',
  tree: 'Passive tree allocations + masteries + cluster jewels.',
  gear: 'Unique-rarity items + significant rare-slot mods.',
  skill: 'Main skill gem + support gems + gem levels.',
  config: 'Calcs config — charges, boss type, flasks, conditional buffs.',
};

const stateClass = computed(() => `drift-chip drift-chip--${props.state}` + (props.dense ? ' drift-chip--dense' : ''));
const tooltip = computed(
  () => `${LABELS[props.kind]} signature — ${props.state.toUpperCase()}. ${TIPS[props.kind]}`,
);
</script>

<template>
  <button
    type="button"
    :class="stateClass"
    :title="tooltip"
    @click="$emit('click')"
  >
    <span v-if="state === 'drift'" class="drift-chip__icon" aria-hidden="true">
      <svg viewBox="0 0 12 12" fill="none" focusable="false">
        <path d="M6 1L11 11H1L6 1Z" fill="currentColor" opacity="0.18" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
        <path d="M6 5v3M6 9.5v0.1" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      </svg>
    </span>
    <span v-else-if="state === 'na'" class="drift-chip__na" aria-hidden="true">n/a</span>
    <span v-else class="drift-chip__dot" aria-hidden="true" />
    <span v-if="!iconsOnly" class="drift-chip__label">{{ LABELS[kind] }}</span>
  </button>
</template>

<style scoped>
.drift-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 8px;
  border-radius: 3px;
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  font-weight: 600;
  background: transparent;
  border: 1px solid var(--paper-line);
  color: var(--ink-soft);
  cursor: pointer;
  line-height: 1;
}
.drift-chip--dense {
  gap: 4px;
  padding: 1px 6px;
  font-size: 10px;
}
.drift-chip--match {
  background: var(--paper-shade);
}
.drift-chip--drift {
  background: var(--amber-glow);
  border: 1px solid var(--note);
  color: var(--note);
  font-weight: 700;
}
.drift-chip--na {
  border: 1px dashed var(--ink-ghost);
  color: var(--ink-faint);
}
.drift-chip__icon svg {
  width: 10px;
  height: 10px;
}
.drift-chip--dense .drift-chip__icon svg {
  width: 8px;
  height: 8px;
}
.drift-chip__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.5;
}
.drift-chip--dense .drift-chip__dot {
  width: 5px;
  height: 5px;
}
.drift-chip__na {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  opacity: 0.7;
}
.drift-chip--dense .drift-chip__na {
  font-size: 9px;
}
</style>

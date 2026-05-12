<script setup lang="ts">
import { computed } from 'vue';
import DriftChip, { type DriftAxis, type DriftState } from './DriftChip.vue';
import FreshPill from './FreshPill.vue';

export type DriftSignatures = Record<DriftAxis, DriftState>;

interface Props {
  sigs: DriftSignatures;
  dense?: boolean;
  iconsOnly?: boolean;
  /** Optional trailing label rendered to the right of the strip (e.g. "all match"). */
  label?: string;
}
const props = withDefaults(defineProps<Props>(), { dense: false, iconsOnly: false, label: '' });
const emit = defineEmits<{ (e: 'chip-click', axis: DriftAxis): void }>();

// Impact order: identity drift is the most severe, config drift the most local.
const ORDER: DriftAxis[] = ['identity', 'tree', 'gear', 'skill', 'config'];

const allMatch = computed(() => ORDER.every((k) => props.sigs[k] === 'match'));
</script>

<template>
  <span v-if="allMatch" class="drift-strip drift-strip--collapsed">
    <FreshPill :dense="dense" />
  </span>
  <span v-else class="drift-strip" :class="{ 'drift-strip--dense': dense }">
    <DriftChip
      v-for="axis in ORDER"
      :key="axis"
      :kind="axis"
      :state="sigs[axis] ?? 'match'"
      :dense="dense"
      :icons-only="iconsOnly"
      @click="emit('chip-click', axis)"
    />
    <span v-if="label" class="drift-strip__label">{{ label }}</span>
  </span>
</template>

<style scoped>
.drift-strip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.drift-strip--dense {
  gap: 4px;
}
.drift-strip__label {
  font-size: 12px;
  color: var(--ink-soft);
  margin-left: 4px;
}
</style>

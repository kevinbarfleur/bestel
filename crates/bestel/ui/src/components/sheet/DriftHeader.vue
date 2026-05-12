<script setup lang="ts">
import { computed } from 'vue';
import type { DriftAxis } from './DriftChip.vue';
import type { DriftSignatures } from './DriftChipStrip.vue';

const props = defineProps<{ sigs: DriftSignatures }>();

const ORDER: DriftAxis[] = ['identity', 'tree', 'gear', 'skill', 'config'];

const drifted = computed<DriftAxis[]>(() => ORDER.filter((k) => props.sigs[k] === 'drift'));

const sentence = computed(() => {
  const d = drifted.value;
  if (d.length === 0) return null;
  if (d.includes('identity')) {
    return {
      lead: "Your build's core identity changed",
      tail: ' since the sheet was authored — you should re-author the sheet.',
    };
  }
  if (d.length === 1) {
    switch (d[0]) {
      case 'tree':
        return { tail: 'Passive tree changed since authoring. The agent will re-derive scaling and may re-frame the archetype.' };
      case 'gear':
        return { tail: 'Items moved around since authoring. Intent and archetype still apply; the agent will re-derive numbers off the live PoB.' };
      case 'config':
        return { tail: 'Calcs config (charges, boss type, flasks) changed. Numbers will be re-derived.' };
      case 'skill':
        return { tail: 'Skill setup changed (gem, supports, or levels). The agent will re-derive offense.' };
      default:
        return null;
    }
  }
  return {
    lead: d.join(' + '),
    tail: ' drifted since authoring. The agent will re-derive numbers and may re-frame the archetype.',
  };
});
</script>

<template>
  <div v-if="sentence" class="drift-header">
    <strong v-if="sentence.lead" class="drift-header__lead">{{ sentence.lead }}</strong>
    <span>{{ sentence.tail }}</span>
  </div>
  <div v-else class="drift-header">
    This sheet still matches your current PoB. The agent will reuse it as-is.
  </div>
</template>

<style scoped>
.drift-header {
  font-size: 14px;
  color: var(--ink-soft);
  line-height: 1.55;
  max-width: 640px;
}
.drift-header__lead {
  color: var(--ink);
  font-weight: 600;
}
</style>

<!--
  Coloured availability pip with optional label.
  - kind="on"   → green (good) with a subtle halo
  - kind="warn" → amber-yellow (note)
  - kind="off"  → ink-faint, no halo

  Used in picker list rows ("API key set" / "no key" / "CLI detected").
-->
<script setup lang="ts">
import { computed } from 'vue';

type Kind = 'on' | 'warn' | 'off';

const props = withDefaults(
  defineProps<{
    kind?: Kind;
    label?: string;
  }>(),
  { kind: 'on' },
);

const dotColor = computed(() => {
  switch (props.kind) {
    case 'on':
      return 'var(--good)';
    case 'warn':
      return 'var(--note)';
    case 'off':
    default:
      return 'var(--ink-faint)';
  }
});
const halo = computed(() => (props.kind === 'on' ? '0 0 0 2px rgba(46, 107, 58, 0.18)' : 'none'));
</script>

<template>
  <span class="pdot">
    <span
      class="pdot__pip"
      :style="{ background: dotColor, boxShadow: halo }"
    />
    <span v-if="label" class="pdot__label">{{ label }}</span>
  </span>
</template>

<style scoped>
.pdot {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.pdot__pip {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex: none;
}
.pdot__label {
  font-size: 14px;
  color: var(--ink-soft);
}
</style>

<!--
  Small-caps section header with the ◆ rune mark. Used inside picker
  detail panes ("API key", "Specs & pricing", "About", etc.). Mirrors
  upstream `SectionHead` (picker-primitives.jsx:21-44).

  The `accent` flag turns the label amber-on-amber for the priority-1
  section (API key in the model picker) — visually distinct from the
  ink-soft "background" sections like Specs and About.
-->
<script setup lang="ts">
withDefaults(
  defineProps<{
    accent?: boolean;
    right?: string;
  }>(),
  { accent: false },
);
</script>

<template>
  <div class="psh">
    <span class="psh__rune">◆</span>
    <span class="psh__label" :class="{ 'psh__label--accent': accent }">
      <slot />
    </span>
    <span class="psh__rule" />
    <span v-if="right" class="psh__right">{{ right }}</span>
  </div>
</template>

<style scoped>
.psh {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-bottom: 6px;
  margin-bottom: 10px;
  border-bottom: 1px solid var(--paper-line);
}
.psh__rune {
  color: var(--amber);
  opacity: 0.65;
  font-size: 13px;
}
.psh__label {
  font-family: var(--label);
  font-size: 13px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 600;
}
.psh__label--accent {
  color: var(--amber);
}
.psh__rule {
  flex: 1;
  height: 1px;
  background: var(--paper-line);
}
.psh__right {
  font-size: 14px;
  color: var(--ink-soft);
  font-weight: 400;
}
</style>

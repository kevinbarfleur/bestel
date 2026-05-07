<script setup lang="ts">
defineProps<{
  /** Use amber title color (for high-priority sections like API key). */
  accent?: boolean;
}>();
</script>

<template>
  <header class="section-head">
    <span class="section-head__rune" aria-hidden="true">◆</span>
    <span
      class="section-head__title"
      :class="{ 'section-head__title--accent': accent }"
    >
      <slot />
    </span>
    <span class="section-head__hairline" aria-hidden="true" />
    <span v-if="$slots.right" class="section-head__right">
      <slot name="right" />
    </span>
  </header>
</template>

<style scoped>
.section-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-bottom: 6px;
  margin-bottom: 10px;
  border-bottom: 1px solid var(--paper-line);
}
.section-head__rune {
  color: var(--amber);
  opacity: 0.65;
  font-size: 12px;
  flex: none;
}
.section-head__title {
  font-family: var(--label);
  font-size: var(--fs-caps);
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: var(--fw-semibold);
  white-space: nowrap;
}
.section-head__title--accent {
  color: var(--amber);
}
.section-head__hairline {
  flex: 1;
  height: 1px;
  background: var(--paper-line);
}
.section-head__right {
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
  font-weight: var(--fw-regular);
  white-space: nowrap;
}
</style>

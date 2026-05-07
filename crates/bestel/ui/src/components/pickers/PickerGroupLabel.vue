<script setup lang="ts">
defineProps<{
  /** Optional count to display at the end of the row (e.g. number of items). */
  count?: number;
}>();
</script>

<template>
  <header class="group-label">
    <span class="group-label__caption"><slot /></span>
    <span class="group-label__hairline" aria-hidden="true" />
    <span v-if="count != null" class="group-label__count">{{ count }}</span>
  </header>
</template>

<style scoped>
.group-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 14px 16px 6px;
  background: var(--paper);
  position: sticky;
  top: 0;
  z-index: 1;
}
.group-label:first-of-type { padding-top: 8px; }

.group-label__caption {
  font-family: var(--label);
  font-size: var(--fs-caps);
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: var(--fw-semibold);
}
.group-label__hairline {
  flex: 1;
  height: 1px;
  background: var(--paper-line);
}
.group-label__count {
  font-family: var(--hand);
  font-size: var(--fs-caps);
  color: var(--ink-faint);
}
</style>

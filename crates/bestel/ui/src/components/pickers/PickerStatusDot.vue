<script setup lang="ts">
withDefaults(
  defineProps<{
    /** Availability state. on = ready, warn = degraded, off = unavailable. */
    kind?: 'on' | 'warn' | 'off';
    /** Optional inline label to the right of the dot. */
    label?: string;
  }>(),
  { kind: 'on' },
);
</script>

<template>
  <span class="status-dot" :class="`status-dot--${kind}`">
    <span class="status-dot__dot" aria-hidden="true" />
    <span v-if="label" class="status-dot__label">{{ label }}</span>
  </span>
</template>

<style scoped>
.status-dot {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
}
.status-dot__dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex: none;
}
.status-dot--on .status-dot__dot {
  background: var(--good);
  box-shadow: 0 0 0 2px rgba(84, 124, 74, 0.18);
}
.status-dot--warn .status-dot__dot {
  background: var(--note);
}
.status-dot--off .status-dot__dot {
  background: var(--ink-faint);
}
.status-dot__label {
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}
</style>

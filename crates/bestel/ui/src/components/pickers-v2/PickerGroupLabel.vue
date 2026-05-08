<!--
  Sticky group header inside a picker sidebar, e.g. "Anthropic API · 3"
  above the Anthropic models. Stays pinned during scroll thanks to
  position: sticky. The right-side count is optional and renders ink-faint.
-->
<script setup lang="ts">
defineProps<{
  count?: number | string;
}>();
</script>

<template>
  <div class="pgl">
    <span class="pgl__label">
      <slot />
    </span>
    <span class="pgl__rule" />
    <span v-if="count !== undefined" class="pgl__count">{{ count }}</span>
  </div>
</template>

<style scoped>
.pgl {
  padding: 14px 16px 6px;
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--paper);
  position: sticky;
  top: 0;
  z-index: 1;
}
.pgl__label {
  font-family: var(--label);
  font-size: 12px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 600;
}
.pgl__rule {
  flex: 1;
  height: 1px;
  background: var(--paper-line);
}
.pgl__count {
  font-size: 13px;
  color: var(--ink-faint);
  font-variant-numeric: tabular-nums;
}
</style>

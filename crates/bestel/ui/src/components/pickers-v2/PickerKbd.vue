<!--
  Inline keyboard-cap badge. Sits inside primary buttons (`⏎`),
  shortcut hints in modal footers (`↑↓`, `esc`), and the Settings
  Shortcuts pane. Matches the upstream `Kbd` from
  picker-primitives.jsx: 12 px label, 1 px ink-faint border, paper-shade
  fill in the dark variant.
-->
<script setup lang="ts">
withDefaults(
  defineProps<{
    onDark?: boolean;
  }>(),
  { onDark: false },
);
</script>

<template>
  <span class="kbd" :class="{ 'kbd--on-dark': onDark }">
    <slot />
  </span>
</template>

<style scoped>
.kbd {
  font-family: var(--label);
  font-size: 12px;
  padding: 2px 6px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  color: var(--ink-soft);
  line-height: 1.3;
  letter-spacing: 0.02em;
  white-space: nowrap;
}
.kbd--on-dark {
  border-color: rgba(244, 241, 234, 0.45);
  color: rgba(244, 241, 234, 0.85);
}
</style>

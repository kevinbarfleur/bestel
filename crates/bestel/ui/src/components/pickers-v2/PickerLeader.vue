<!--
  Leader row — a key on the left, a dotted rule, a value on the right.
  Reads like an almanac entry. The mono variant uses JetBrains Mono for
  technical values (model ids, paths, hashes). `accent` lets callers
  paint the value in the elemental / good / bad palette.
-->
<script setup lang="ts">
withDefaults(
  defineProps<{
    k: string;
    v: string | number;
    mono?: boolean;
    accent?: string;
  }>(),
  { mono: false },
);
</script>

<template>
  <div class="plead">
    <span class="plead__key">{{ k }}</span>
    <span class="plead__rule" />
    <span
      class="plead__value"
      :class="{ 'plead__value--mono': mono }"
      :style="accent ? { color: accent } : undefined"
    >
      {{ v }}
    </span>
  </div>
</template>

<style scoped>
.plead {
  display: flex;
  align-items: baseline;
  gap: 8px;
}
.plead__key {
  font-size: 14px;
  color: var(--ink-soft);
  font-weight: 400;
  flex-shrink: 0;
  white-space: nowrap;
}
.plead__rule {
  flex: 1;
  min-width: 12px;
  border-bottom: 1px dotted var(--paper-line);
  transform: translateY(-3px);
}
.plead__value {
  font-family: var(--hand);
  font-size: 14px;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
}
.plead__value--mono {
  font-family: var(--mono);
  font-size: 13px;
}
</style>

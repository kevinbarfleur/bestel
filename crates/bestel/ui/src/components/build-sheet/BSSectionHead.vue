<script setup lang="ts">
withDefaults(
  defineProps<{ right?: string; index?: number }>(),
  { right: undefined, index: undefined },
);

function pad(n: number): string {
  return n < 10 ? `0${n}` : String(n);
}
</script>

<template>
  <div class="bs-sh">
    <span v-if="index !== undefined" class="bs-sh__num">{{ pad(index) }}.</span>
    <span class="bs-sh__title"><slot /></span>
    <span class="bs-sh__rule" />
    <span v-if="right" class="bs-sh__right">{{ right }}</span>
  </div>
</template>

<style scoped>
.bs-sh {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}
.bs-sh__num {
  font-family: 'JetBrains Mono', ui-monospace, monospace;
  font-size: 12px;
  color: var(--ink-faint);
  font-weight: 500;
  min-width: 22px;
  letter-spacing: 0.02em;
}
.bs-sh__title {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 700;
}
.bs-sh__rule {
  flex: 1;
  height: 1px;
  background: var(--paper-line);
  min-width: 8px;
}
.bs-sh__right {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 700;
}
</style>

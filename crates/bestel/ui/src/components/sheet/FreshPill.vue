<script setup lang="ts">
defineProps<{ dense?: boolean }>();
</script>

<template>
  <span class="fresh-pill" :class="{ 'fresh-pill--dense': dense }">
    <svg viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <path d="M2 6.5L5 9.5L10 3" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
    <span>fresh</span>
  </span>
</template>

<style scoped>
.fresh-pill {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 3px 10px;
  border-radius: 12px;
  background: rgba(84, 124, 74, 0.10);
  border: 1px solid var(--good);
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--good);
  font-weight: 700;
  line-height: 1;
}
.fresh-pill--dense {
  gap: 5px;
  padding: 1px 8px;
  font-size: 10px;
}
.fresh-pill svg {
  width: 11px;
  height: 11px;
}
.fresh-pill--dense svg {
  width: 9px;
  height: 9px;
}
</style>

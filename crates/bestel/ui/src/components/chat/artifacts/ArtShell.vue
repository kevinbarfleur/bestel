<script setup lang="ts">
// Almanach grammar: artifacts have no box, no border, no background.
// Distinction comes from the gutter label colour and typography.
// Kept as a wrapper for layout consistency only.
type Accent = 'ink' | 'amber' | 'cool' | 'arcane' | 'good' | 'bad' | 'lit' | 'chaos';
defineProps<{ accent?: Accent }>();
</script>

<template>
  <div class="art-shell">
    <slot />
  </div>
</template>

<style scoped>
.art-shell {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
</style>

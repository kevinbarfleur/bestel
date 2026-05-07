<script setup lang="ts">
defineProps<{
  selected: boolean;
  label: string;
  hint: string;
}>();
defineEmits<{ pick: [] }>();
</script>

<template>
  <button
    type="button"
    class="rail-item"
    :class="{ 'rail-item--selected': selected }"
    @click="$emit('pick')"
  >
    <span class="rail-item__label">{{ label }}</span>
    <span class="rail-item__hint">{{ hint }}</span>
  </button>
</template>

<style scoped>
.rail-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  width: 100%;
  padding: 10px 22px;
  border: 0;
  border-left: 3px solid transparent;
  background: transparent;
  cursor: pointer;
  text-align: left;
  font-family: var(--hand);
  color: var(--ink);
  transition: background 0.12s ease, border-color 0.12s ease;
}
.rail-item:hover {
  background: var(--paper-shade);
}
.rail-item--selected {
  border-left-color: var(--ink);
  background: var(--paper-shade);
}

.rail-item__label {
  font-size: var(--fs-body);
  font-weight: var(--fw-medium);
  color: var(--ink);
}
.rail-item--selected .rail-item__label {
  font-weight: var(--fw-semibold);
}

.rail-item__hint {
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}
</style>

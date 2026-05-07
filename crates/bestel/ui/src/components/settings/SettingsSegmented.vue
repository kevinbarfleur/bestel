<script setup lang="ts" generic="T extends string">
defineProps<{
  modelValue: T;
  options: { value: T; label: string }[];
}>();
defineEmits<{ 'update:modelValue': [value: T] }>();
</script>

<template>
  <div class="segmented" role="radiogroup">
    <button
      v-for="o in options"
      :key="o.value"
      type="button"
      class="segmented__btn"
      :class="{ 'segmented__btn--active': o.value === modelValue }"
      role="radio"
      :aria-checked="o.value === modelValue"
      @click="$emit('update:modelValue', o.value)"
    >
      {{ o.label }}
    </button>
  </div>
</template>

<style scoped>
.segmented {
  display: inline-flex;
  border: 1px solid var(--ink-soft);
  border-radius: 4px;
  overflow: hidden;
  align-self: flex-start;
}
.segmented__btn {
  padding: 8px 16px;
  border: 0;
  border-left: 1px solid var(--ink-soft);
  background: var(--paper);
  color: var(--ink);
  font-family: var(--hand);
  font-size: var(--fs-meta);
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease;
}
.segmented__btn:first-child {
  border-left: 0;
}
.segmented__btn:hover {
  background: var(--paper-shade);
}
.segmented__btn--active,
.segmented__btn--active:hover {
  background: var(--ink);
  color: var(--paper);
  font-weight: var(--fw-semibold);
}
</style>

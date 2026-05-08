<!--
  Search field with magnifier icon and right-aligned count. Sits at the
  top of every picker sidebar so the eye lands on it before scanning the
  list. v-model on `value`; `count` displays a result tally
  ("23" → "23 results"); ignored if undefined.
-->
<script setup lang="ts">
import PickerIcon from './PickerIcon.vue';

defineProps<{
  modelValue: string;
  placeholder?: string;
  count?: number | string;
}>();
defineEmits<{ 'update:modelValue': [value: string] }>();
</script>

<template>
  <div class="psearch">
    <PickerIcon name="search" :size="15" color="var(--ink-faint)" />
    <input
      :value="modelValue"
      type="text"
      :placeholder="placeholder"
      class="psearch__input"
      @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
    <span v-if="count !== undefined" class="psearch__count">{{ count }}</span>
  </div>
</template>

<style scoped>
.psearch {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--paper-line);
  background: var(--paper);
}
.psearch__input {
  flex: 1;
  min-width: 0;
  background: transparent;
  border: none;
  outline: none;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink);
}
.psearch__input::placeholder {
  color: var(--ink-faint);
}
.psearch__count {
  font-size: 14px;
  color: var(--ink-soft);
  font-variant-numeric: tabular-nums;
}
</style>

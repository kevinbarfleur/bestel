<script setup lang="ts">
import { onMounted, ref } from 'vue';

import RunicIcon from '../runic/RunicIcon.vue';

const props = withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    /** Optional count shown at the right edge (e.g. total filtered items). */
    count?: number;
    /** Auto-focus the input on mount. Default: true. */
    autofocus?: boolean;
  }>(),
  { placeholder: 'Search…', autofocus: true },
);

const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const inputRef = ref<HTMLInputElement | null>(null);

defineExpose({
  focus: () => inputRef.value?.focus(),
  blur: () => inputRef.value?.blur(),
});

onMounted(() => {
  if (props.autofocus) inputRef.value?.focus();
});
</script>

<template>
  <label class="picker-search">
    <span class="picker-search__icon" aria-hidden="true">
      <RunicIcon name="search" :size="15" />
    </span>
    <input
      ref="inputRef"
      type="text"
      class="picker-search__input"
      :value="modelValue"
      :placeholder="placeholder"
      autocomplete="off"
      spellcheck="false"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
    <span v-if="count != null" class="picker-search__count">{{ count }}</span>
  </label>
</template>

<style scoped>
.picker-search {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--paper-line);
  background: var(--paper);
}

.picker-search__icon {
  display: inline-flex;
  align-items: center;
  color: var(--ink-faint);
  flex: none;
}

.picker-search__input {
  flex: 1;
  min-width: 0;
  border: 0;
  outline: 0;
  background: transparent;
  font-family: var(--hand);
  font-size: var(--fs-body);
  color: var(--ink);
  font-weight: var(--fw-regular);
}
.picker-search__input::placeholder {
  font-family: var(--hand);
  font-size: var(--fs-body);
  color: var(--ink-faint);
  font-weight: var(--fw-regular);
}
.picker-search__input:focus {
  caret-color: var(--amber);
}

.picker-search__count {
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
  flex: none;
}
</style>

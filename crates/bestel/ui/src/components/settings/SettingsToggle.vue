<script setup lang="ts">
defineProps<{
  modelValue: boolean;
  caption: string;
  /** When true, the toggle is rendered visually but click does nothing
   *  (used for placeholder features that aren't wired yet). */
  disabled?: boolean;
}>();
defineEmits<{ 'update:modelValue': [value: boolean] }>();
</script>

<template>
  <label class="toggle-row" :class="{ 'toggle-row--disabled': disabled }">
    <span
      class="toggle-row__switch"
      :class="{ 'toggle-row__switch--on': modelValue }"
      role="switch"
      :aria-checked="modelValue"
      @click.prevent="!disabled && $emit('update:modelValue', !modelValue)"
    >
      <span class="toggle-row__thumb" />
    </span>
    <span class="toggle-row__caption">{{ caption }}</span>
  </label>
</template>

<style scoped>
.toggle-row {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  user-select: none;
}
.toggle-row--disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.toggle-row__switch {
  width: 36px;
  height: 20px;
  background: var(--paper-shade);
  border: 1px solid var(--ink-faint);
  border-radius: 10px;
  position: relative;
  flex: none;
  transition: background 0.12s ease, border-color 0.12s ease;
}
.toggle-row__switch--on {
  background: var(--ink);
  border-color: var(--ink);
}

.toggle-row__thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 14px;
  height: 14px;
  background: var(--ink-faint);
  border-radius: 50%;
  transition: left 0.12s ease, background 0.12s ease;
}
.toggle-row__switch--on .toggle-row__thumb {
  left: 18px;
  background: var(--paper);
}

.toggle-row__caption {
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}
</style>

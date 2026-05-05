<script setup lang="ts">
interface Props {
  modelValue: boolean;
  label?: string;
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  label: undefined,
  disabled: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const toggle = () => {
  if (!props.disabled) emit('update:modelValue', !props.modelValue);
};

const onKey = (e: KeyboardEvent) => {
  if (e.key === ' ' || e.key === 'Enter') {
    e.preventDefault();
    toggle();
  }
};
</script>

<template>
  <label
    class="runic-checkbox"
    :class="{
      'runic-checkbox--checked': modelValue,
      'runic-checkbox--disabled': disabled,
    }"
    @keydown="onKey"
    tabindex="0"
  >
    <span class="runic-checkbox__box" @click.prevent="toggle">
      <svg
        v-if="modelValue"
        class="runic-checkbox__check"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
      >
        <path stroke-linecap="round" stroke-linejoin="round" d="M3 8l3 3 7-7" />
      </svg>
    </span>
    <span v-if="label || $slots.default" class="runic-checkbox__label">
      <slot>{{ label }}</slot>
    </span>
  </label>
</template>

<style scoped>
.runic-checkbox {
  display: inline-flex;
  align-items: center;
  gap: 0.6rem;
  font-family: var(--hand);
  color: var(--ink);
  cursor: pointer;
  user-select: none;
  outline: none;
}

.runic-checkbox:focus-visible {
  outline: 1.5px solid var(--amber);
  outline-offset: 3px;
  border-radius: 2px;
}

.runic-checkbox--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.runic-checkbox__box {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  background: var(--paper);
  border: 1.4px solid var(--ink-soft);
  border-radius: 3px;
  color: var(--paper);
  transition: all 0.15s ease;
}

.runic-checkbox:hover .runic-checkbox__box {
  border-color: var(--amber);
}

.runic-checkbox--checked .runic-checkbox__box {
  background: var(--amber);
  border-color: var(--amber);
}

.runic-checkbox__check {
  width: 12px;
  height: 12px;
}

.runic-checkbox__label {
  font-size: 14px;
  letter-spacing: 0.01em;
  color: var(--ink);
}
</style>

<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  modelValue: boolean;
  labelOn?: string;
  labelOff?: string;
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  tier?: 'default' | 't0' | 't1' | 't2' | 't3';
}

const props = withDefaults(defineProps<Props>(), {
  labelOn: 'On',
  labelOff: 'Off',
  size: 'md',
  disabled: false,
  tier: 'default',
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const setValue = (v: boolean) => {
  if (!props.disabled && v !== props.modelValue) emit('update:modelValue', v);
};

const sliderStyle = computed(() => ({
  left: props.modelValue ? 'calc(50% + 1px)' : '2px',
  width: 'calc(50% - 3px)',
}));
</script>

<template>
  <div
    class="runic-toggle"
    :class="[
      `runic-toggle--${size}`,
      `runic-toggle--${tier}`,
      { 'runic-toggle--disabled': disabled, 'runic-toggle--on': modelValue },
    ]"
    role="switch"
    :aria-checked="modelValue"
    tabindex="0"
    @keydown.space.prevent="setValue(!modelValue)"
    @keydown.enter.prevent="setValue(!modelValue)"
  >
    <div class="runic-toggle__groove">
      <button
        type="button"
        class="runic-toggle__option"
        :class="{ 'runic-toggle__option--active': !modelValue }"
        :disabled="disabled"
        @click="setValue(false)"
      >
        {{ labelOff }}
      </button>
      <button
        type="button"
        class="runic-toggle__option"
        :class="{ 'runic-toggle__option--active': modelValue }"
        :disabled="disabled"
        @click="setValue(true)"
      >
        {{ labelOn }}
      </button>
      <div class="runic-toggle__slider" :style="sliderStyle"></div>
    </div>
  </div>
</template>

<style scoped>
.runic-toggle {
  display: inline-block;
  outline: none;
}
.runic-toggle:focus-visible {
  outline: 1px solid rgba(175, 96, 37, 0.5);
  outline-offset: 4px;
  border-radius: 6px;
}

.runic-toggle__groove {
  position: relative;
  display: flex;
  align-items: stretch;
  background: var(--paper-shade);
  border-radius: 4px;
  border: 1.4px solid var(--ink-faint);
  overflow: hidden;
}

.runic-toggle__groove::before { content: none; }

.runic-toggle__option {
  position: relative;
  flex: 1;
  background: none;
  border: none;
  cursor: pointer;
  font-family: var(--hand);
  font-weight: 600;
  letter-spacing: 0.02em;
  color: var(--ink-faint);
  padding: 0.5rem 1rem;
  transition: color 0.18s ease;
  z-index: 3;
}

.runic-toggle__option:disabled {
  cursor: not-allowed;
}

.runic-toggle__option--active {
  color: var(--paper);
}

.runic-toggle__slider {
  position: absolute;
  top: 2px;
  bottom: 2px;
  background: var(--ink);
  border-radius: 3px;
  border: 1px solid var(--ink);
  transition: left 0.28s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 2;
  pointer-events: none;
}

.runic-toggle__slider::before { content: none; }

.runic-toggle--sm .runic-toggle__option { padding: 0.35rem 0.75rem; font-size: 12px; }
.runic-toggle--md .runic-toggle__option { padding: 0.5rem 1rem; font-size: 13px; }
.runic-toggle--lg .runic-toggle__option { padding: 0.7rem 1.4rem; font-size: 14px; }

.runic-toggle--t0 .runic-toggle__slider { background: var(--amber); border-color: var(--amber); }
.runic-toggle--t1 .runic-toggle__slider { background: #7a6a8a; border-color: #7a6a8a; }
.runic-toggle--t2 .runic-toggle__slider { background: #5a7080; border-color: #5a7080; }
.runic-toggle--t3 .runic-toggle__slider { background: var(--ink-soft); border-color: var(--ink-soft); }

.runic-toggle--disabled {
  opacity: 0.5;
  pointer-events: none;
}
</style>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';

interface Option {
  value: string;
  label: string;
  color?: string;
}

interface Props {
  options?: Option[];
  modelValue: string | boolean;
  size?: 'sm' | 'md' | 'lg';
  toggle?: boolean;
  toggleColor?: string;
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  size: 'md',
  toggle: false,
  toggleColor: 'default',
  disabled: false,
  options: () => [],
});

const emit = defineEmits<{
  'update:modelValue': [value: string | boolean];
}>();

const grooveRef = ref<HTMLElement | null>(null);
const sliderPosition = ref({ left: 0, width: 0 });
const isMeasured = ref(false);

const effectiveOptions = computed(() => {
  if (props.toggle) {
    return [
      { value: 'off', label: '', color: 'off' },
      { value: 'on', label: '', color: props.toggleColor },
    ];
  }
  return props.options || [];
});

const internalValue = computed(() => {
  if (props.toggle) return props.modelValue ? 'on' : 'off';
  return props.modelValue as string;
});

const selectedIndex = computed(() =>
  effectiveOptions.value.findIndex((opt) => opt.value === internalValue.value),
);

const measureOptions = () => {
  if (!grooveRef.value) return;
  const buttons = grooveRef.value.querySelectorAll('.runic-radio__option');
  const index = selectedIndex.value;
  if (index === -1 || !buttons[index]) {
    isMeasured.value = false;
    return;
  }
  const button = buttons[index] as HTMLElement;
  sliderPosition.value = {
    left: button.offsetLeft,
    width: button.offsetWidth,
  };
  isMeasured.value = true;
};

const sliderStyle = computed(() => {
  const index = selectedIndex.value;
  const count = effectiveOptions.value.length;
  if (isMeasured.value && sliderPosition.value.width > 0) {
    return {
      left: `${sliderPosition.value.left}px`,
      width: `${sliderPosition.value.width}px`,
    };
  }
  return {
    left: index === -1 ? '0%' : `${(index / count) * 100}%`,
    width: `${100 / count}%`,
  };
});

watch(selectedIndex, () => {
  nextTick(measureOptions);
});

watch(
  () => effectiveOptions.value,
  () => {
    nextTick(measureOptions);
  },
  { deep: true },
);

onMounted(() => {
  nextTick(measureOptions);
  window.addEventListener('resize', measureOptions);
});

onUnmounted(() => {
  window.removeEventListener('resize', measureOptions);
});

const selectedColor = computed(() => {
  const option = effectiveOptions.value[selectedIndex.value];
  return option?.color || null;
});

const selectOption = (value: string) => {
  if (props.disabled) return;
  if (props.toggle) emit('update:modelValue', value === 'on');
  else emit('update:modelValue', value);
};

const handleToggleClick = () => {
  if (props.disabled) return;
  if (props.toggle) emit('update:modelValue', !props.modelValue);
};
</script>

<template>
  <div
    class="runic-radio"
    :class="[
      `runic-radio--${size}`,
      { 'runic-radio--toggle': toggle, 'runic-radio--disabled': disabled },
    ]"
    :role="toggle ? 'switch' : 'radiogroup'"
    :aria-checked="toggle ? (modelValue as boolean) : undefined"
    :aria-disabled="disabled"
  >
    <div
      ref="grooveRef"
      class="runic-radio__groove"
      :class="{
        'runic-radio__groove--clickable': toggle && !disabled,
        'runic-radio__groove--disabled': disabled,
      }"
      @click="!disabled && handleToggleClick()"
    >
      <button
        v-for="option in effectiveOptions"
        :key="option.value"
        type="button"
        :role="toggle ? 'presentation' : 'radio'"
        :aria-checked="!toggle ? internalValue === option.value : undefined"
        :tabindex="toggle ? -1 : 0"
        class="runic-radio__option"
        :class="[
          { 'runic-radio__option--selected': internalValue === option.value },
          option.color ? `runic-radio__option--${option.color}` : '',
        ]"
        @click.stop="!disabled && !toggle && selectOption(option.value)"
      >
        <span v-if="!toggle" class="runic-radio__label">{{ option.label }}</span>
      </button>

      <div
        class="runic-radio__slider"
        :class="selectedColor ? `runic-radio__slider--${selectedColor}` : ''"
        :style="sliderStyle"
        aria-hidden="true"
      >
        <span v-if="!toggle" class="runic-radio__slider-label">
          {{ effectiveOptions[selectedIndex]?.label }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.runic-radio {
  position: relative;
  display: inline-flex;
  font-family: var(--hand);
}

.runic-radio__groove {
  position: relative;
  display: flex;
  align-items: center;
  background: var(--paper-shade);
  border-radius: 4px;
  border: 1.4px solid var(--ink-faint);
  overflow: hidden;
}

.runic-radio__groove--clickable {
  cursor: pointer;
  transition: border-color 0.18s ease;
}
.runic-radio__groove--clickable:hover { border-color: var(--ink-soft); }

.runic-radio--disabled {
  opacity: 0.5;
  pointer-events: none !important;
  cursor: not-allowed;
}
.runic-radio--disabled * {
  pointer-events: none !important;
  cursor: not-allowed !important;
}
.runic-radio__groove--disabled {
  cursor: not-allowed !important;
  opacity: 0.6;
  pointer-events: none !important;
}

.runic-radio__groove::before { content: none; }

.runic-radio__option {
  position: relative;
  z-index: 1;
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all 0.3s ease;
  white-space: nowrap;
}

.runic-radio__label {
  position: relative;
  font-weight: 600;
  letter-spacing: 0.02em;
  color: var(--ink-faint);
  transition: color 0.18s ease;
}

.runic-radio__option:hover .runic-radio__label {
  color: var(--ink-soft);
}
.runic-radio__option--selected .runic-radio__label { opacity: 0; }

.runic-radio__slider {
  position: absolute;
  top: 2px;
  bottom: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--ink);
  border-radius: 3px;
  border: 1px solid var(--ink);
  transition: left 0.28s cubic-bezier(0.4, 0, 0.2, 1), background 0.18s ease, border-color 0.18s ease;
  z-index: 2;
  pointer-events: none;
}

.runic-radio__slider::before { content: none; }

.runic-radio__slider-label {
  font-weight: 600;
  letter-spacing: 0.02em;
  color: var(--paper);
  transition: color 0.18s ease;
}

.runic-radio__slider--default,
.runic-radio__slider:not([class*="--t"]):not([class*="--all"]) {
  background: var(--ink);
  border-color: var(--ink);
}

.runic-radio__slider--t0    { background: var(--amber); border-color: var(--amber); }
.runic-radio__slider--t1    { background: #7a6a8a; border-color: #7a6a8a; }
.runic-radio__slider--t2    { background: #5a7080; border-color: #5a7080; }
.runic-radio__slider--t3    { background: var(--ink-soft); border-color: var(--ink-soft); }

.runic-radio--sm .runic-radio__groove { min-height: 32px; }
.runic-radio--sm .runic-radio__option { padding: 0.375rem 0.625rem; }
.runic-radio--sm .runic-radio__label,
.runic-radio--sm .runic-radio__slider-label { font-size: 0.8125rem; }

@media (min-width: 640px) {
  .runic-radio--sm .runic-radio__groove { min-height: 38px; }
  .runic-radio--sm .runic-radio__option { padding: 0.5rem 1.25rem; }
  .runic-radio--sm .runic-radio__label,
  .runic-radio--sm .runic-radio__slider-label { font-size: 0.875rem; }
}

.runic-radio--md .runic-radio__groove { min-height: 38px; }
.runic-radio--md .runic-radio__option { padding: 0.5rem 0.75rem; }
.runic-radio--md .runic-radio__label,
.runic-radio--md .runic-radio__slider-label { font-size: 0.875rem; }

@media (min-width: 640px) {
  .runic-radio--md .runic-radio__groove { min-height: 46px; }
  .runic-radio--md .runic-radio__option { padding: 0.625rem 1.5rem; }
  .runic-radio--md .runic-radio__label,
  .runic-radio--md .runic-radio__slider-label { font-size: 0.9375rem; }
}

.runic-radio--lg .runic-radio__groove { min-height: 44px; }
.runic-radio--lg .runic-radio__option { padding: 0.5rem 1rem; }
.runic-radio--lg .runic-radio__label,
.runic-radio--lg .runic-radio__slider-label { font-size: 0.9375rem; }

@media (min-width: 640px) {
  .runic-radio--lg .runic-radio__groove { min-height: 54px; }
  .runic-radio--lg .runic-radio__option { padding: 0.75rem 1.75rem; }
  .runic-radio--lg .runic-radio__label,
  .runic-radio--lg .runic-radio__slider-label { font-size: 1.0625rem; }
}

.runic-radio--toggle .runic-radio__groove { min-height: 32px; }
.runic-radio--toggle .runic-radio__option { padding: 0.5rem 1rem; min-width: 28px; }
.runic-radio--toggle.runic-radio--sm .runic-radio__groove { min-height: 28px; }
.runic-radio--toggle.runic-radio--sm .runic-radio__option { padding: 0.375rem 0.75rem; min-width: 24px; }
.runic-radio--toggle.runic-radio--md .runic-radio__groove { min-height: 32px; }
.runic-radio--toggle.runic-radio--md .runic-radio__option { padding: 0.5rem 1rem; min-width: 28px; }
.runic-radio--toggle.runic-radio--lg .runic-radio__groove { min-height: 38px; }
.runic-radio--toggle.runic-radio--lg .runic-radio__option { padding: 0.5rem 1.25rem; min-width: 34px; }

.runic-radio--toggle .runic-radio__option { pointer-events: none; flex: 1; }

.runic-radio__slider--off {
  background: var(--ink-soft);
  border-color: var(--ink-soft);
}

.runic-radio--toggle .runic-radio__slider:not(.runic-radio__slider--off) {
  background: var(--amber);
  border-color: var(--amber);
}

.runic-radio__option:focus { outline: none; }
.runic-radio__option:focus-visible .runic-radio__label { color: var(--ink); }
</style>

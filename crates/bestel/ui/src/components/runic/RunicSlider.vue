<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  modelValue: number;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
  showValue?: boolean;
  valuePrefix?: string;
  valueSuffix?: string;
  size?: 'sm' | 'md' | 'lg';
}

const props = withDefaults(defineProps<Props>(), {
  min: 0,
  max: 100,
  step: 1,
  disabled: false,
  showValue: true,
  valuePrefix: '',
  valueSuffix: '',
  size: 'md',
});

const emit = defineEmits<{
  'update:modelValue': [value: number];
}>();

const displayValue = computed(() => `${props.valuePrefix}${props.modelValue}${props.valueSuffix}`);
const percentage = computed(() => ((props.modelValue - props.min) / (props.max - props.min)) * 100);

const handleInput = (event: Event) => {
  const target = event.target as HTMLInputElement;
  emit('update:modelValue', parseFloat(target.value));
};
</script>

<template>
  <div
    class="runic-slider"
    :class="[`runic-slider--${size}`, { 'runic-slider--disabled': disabled }]"
  >
    <div
      class="runic-slider__track-container"
      :style="{ '--thumb-position': `${percentage}%` }"
    >
      <span class="runic-slider__rune runic-slider__rune--left">◆</span>
      <span class="runic-slider__rune runic-slider__rune--right">◆</span>

      <div class="runic-slider__track">
        <div class="runic-slider__fill" :style="{ width: `${percentage}%` }"></div>
        <div class="runic-slider__notches">
          <span class="runic-slider__notch" style="left: 0%"></span>
          <span class="runic-slider__notch" style="left: 25%"></span>
          <span class="runic-slider__notch" style="left: 50%"></span>
          <span class="runic-slider__notch" style="left: 75%"></span>
          <span class="runic-slider__notch" style="left: 100%"></span>
        </div>
      </div>

      <input
        type="range"
        class="runic-slider__input"
        :value="modelValue"
        :min="min"
        :max="max"
        :step="step"
        :disabled="disabled"
        @input="handleInput"
      />
    </div>

    <div v-if="showValue" class="runic-slider__value">
      <span class="runic-slider__value-text">{{ displayValue }}</span>
    </div>
  </div>
</template>

<style scoped>
.runic-slider { display: flex; align-items: center; gap: 0.75rem; width: 100%; }
.runic-slider--disabled { opacity: 0.5; pointer-events: none; }

.runic-slider__track-container {
  position: relative;
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
}

.runic-slider__rune {
  position: absolute;
  font-size: 0.5rem;
  color: rgba(175, 96, 37, 0.4);
  z-index: 2;
  pointer-events: none;
  transition: all 0.3s ease;
}
.runic-slider__rune--left { left: -2px; top: 50%; transform: translateY(-50%); }
.runic-slider__rune--right { right: -2px; top: 50%; transform: translateY(-50%); }

.runic-slider:hover .runic-slider__rune,
.runic-slider:focus-within .runic-slider__rune {
  color: rgba(175, 96, 37, 0.7);
}

.runic-slider__track {
  position: relative;
  width: 100%;
  height: 8px;
  background: var(--paper-shade);
  border-radius: 4px;
  overflow: hidden;
  border: 1.4px solid var(--ink-faint);
}

.runic-slider__fill {
  position: absolute;
  top: 1px;
  left: 1px;
  bottom: 1px;
  background:
    repeating-linear-gradient(
      -45deg,
      transparent 0 4px,
      rgba(255, 255, 255, 0.15) 4px 5px
    ),
    var(--amber);
  border-radius: 2px;
  transition: width 0.15s ease;
  opacity: 0.85;
}

.runic-slider__notches {
  position: absolute;
  inset: 0;
  pointer-events: none;
}
.runic-slider__notch {
  position: absolute;
  top: 50%;
  width: 1px;
  height: 4px;
  background: var(--ink-faint);
  opacity: 0.45;
  transform: translate(-50%, -50%);
}

.runic-slider__input {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  margin: 0;
  padding: 0;
  opacity: 0;
  cursor: pointer;
  z-index: 3;
  -webkit-appearance: none;
  appearance: none;
}
.runic-slider__input::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 20px;
  height: 20px;
  cursor: pointer;
}
.runic-slider__input::-moz-range-thumb {
  width: 20px;
  height: 20px;
  border: none;
  background: transparent;
  cursor: pointer;
}

.runic-slider__track-container::after {
  content: '';
  position: absolute;
  top: 50%;
  left: calc(var(--thumb-position, 0%) - 10px);
  width: 20px;
  height: 20px;
  background: var(--paper);
  border: 2px solid var(--amber);
  border-radius: 50%;
  transform: translateY(-50%);
  box-shadow: 0 2px 6px rgba(40, 35, 30, 0.18);
  transition: border-color 0.15s ease;
  pointer-events: none;
  z-index: 4;
}

.runic-slider:hover .runic-slider__track-container::after,
.runic-slider:focus-within .runic-slider__track-container::after {
  border-color: var(--rune);
}

.runic-slider__track-container::before {
  content: '◆';
  position: absolute;
  top: 50%;
  left: calc(var(--thumb-position, 0%) - 10px);
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.5rem;
  color: var(--amber);
  transform: translateY(-50%);
  pointer-events: none;
  z-index: 5;
  transition: color 0.15s ease;
}

.runic-slider:hover .runic-slider__track-container::before,
.runic-slider:focus-within .runic-slider__track-container::before {
  color: var(--rune);
}

.runic-slider__value {
  flex-shrink: 0;
  min-width: 4rem;
  padding: 0.375rem 0.625rem;
  background: var(--paper-shade);
  border: 1.2px solid var(--ink-faint);
  border-radius: 3px;
  text-align: center;
}

.runic-slider__value-text {
  font-family: var(--hand-display);
  font-size: 14px;
  font-weight: 700;
  color: var(--ink);
}

.runic-slider--sm .runic-slider__track { height: 6px; }
.runic-slider--sm .runic-slider__track-container::after {
  width: 16px; height: 16px; left: calc(var(--thumb-position, 0%) - 8px);
}
.runic-slider--sm .runic-slider__track-container::before {
  width: 16px; height: 16px; left: calc(var(--thumb-position, 0%) - 8px); font-size: 0.4rem;
}
.runic-slider--sm .runic-slider__value { min-width: 3.5rem; padding: 0.25rem 0.5rem; }
.runic-slider--sm .runic-slider__value-text { font-size: 0.75rem; }
.runic-slider--sm .runic-slider__rune { font-size: 0.4rem; }

.runic-slider--lg .runic-slider__track { height: 10px; }
.runic-slider--lg .runic-slider__track-container::after {
  width: 24px; height: 24px; left: calc(var(--thumb-position, 0%) - 12px);
}
.runic-slider--lg .runic-slider__track-container::before {
  width: 24px; height: 24px; left: calc(var(--thumb-position, 0%) - 12px); font-size: 0.625rem;
}
.runic-slider--lg .runic-slider__value { min-width: 5rem; padding: 0.5rem 0.75rem; }
.runic-slider--lg .runic-slider__value-text { font-size: 1rem; }
.runic-slider--lg .runic-slider__rune { font-size: 0.625rem; }

@media (max-width: 640px) {
  .runic-slider__value { min-width: 3.5rem; padding: 0.25rem 0.5rem; }
  .runic-slider__value-text { font-size: 0.8125rem; }
}
</style>

<script setup lang="ts">
import { computed, ref } from 'vue';

interface Props {
  modelValue: string;
  placeholder?: string;
  icon?: 'search' | 'filter' | 'none';
  size?: 'sm' | 'md' | 'lg';
  clearable?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: '',
  icon: 'none',
  size: 'md',
  clearable: true,
});

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const inputRef = ref<HTMLInputElement | null>(null);

const updateValue = (event: Event) => {
  const target = event.target as HTMLInputElement;
  emit('update:modelValue', target.value);
};

const clearValue = () => {
  emit('update:modelValue', '');
  inputRef.value?.focus();
};

const showClear = computed(() => props.clearable && props.modelValue.length > 0);

const focus = () => {
  inputRef.value?.focus();
};

defineExpose({ focus });
</script>

<template>
  <div
    class="runic-input"
    :class="[
      `runic-input--${size}`,
      {
        'runic-input--has-icon': icon !== 'none',
        'runic-input--has-clear': showClear,
      },
    ]"
  >
    <div class="runic-input__groove">
      <span class="runic-input__rune runic-input__rune--tl">◆</span>
      <span class="runic-input__rune runic-input__rune--tr">◆</span>
      <span class="runic-input__rune runic-input__rune--bl">◆</span>
      <span class="runic-input__rune runic-input__rune--br">◆</span>

      <svg v-if="icon === 'search'" class="runic-input__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      <svg v-else-if="icon === 'filter'" class="runic-input__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
      </svg>

      <input
        ref="inputRef"
        type="text"
        class="runic-input__field"
        :value="modelValue"
        :placeholder="placeholder"
        @input="updateValue"
      />

      <button
        v-if="showClear"
        type="button"
        class="runic-input__clear"
        aria-label="Clear"
        @click="clearValue"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>

      <div class="runic-input__glow"></div>
    </div>
  </div>
</template>

<style scoped>
.runic-input {
  position: relative;
  display: inline-flex;
  width: 100%;
  font-family: var(--hand);
}

.runic-input__groove {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
  background: var(--paper);
  border: 1.4px solid var(--ink-faint);
  border-radius: 4px 7px 5px 6px / 6px 4px 7px 5px;
  overflow: hidden;
  transition: border-color 0.18s ease, background 0.18s ease;
}
.runic-input__groove:focus-within {
  border-color: var(--amber);
  background: var(--paper-shade);
}

.runic-input__rune {
  position: absolute;
  font-size: 0.55rem;
  color: rgba(175, 96, 37, 0.25);
  transition: color 0.2s ease;
  pointer-events: none;
  z-index: 2;
}
.runic-input__rune--tl { top: 3px; left: 5px; }
.runic-input__rune--tr { top: 3px; right: 5px; }
.runic-input__rune--bl { bottom: 3px; left: 5px; }
.runic-input__rune--br { bottom: 3px; right: 5px; }

.runic-input__groove:focus-within .runic-input__rune {
  color: var(--amber);
}

.runic-input__icon {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  margin-left: 0.85rem;
  color: var(--ink-faint);
  transition: color 0.18s ease;
}
.runic-input__groove:focus-within .runic-input__icon {
  color: var(--amber);
}

.runic-input__field {
  flex: 1;
  width: 100%;
  background: transparent;
  border: none;
  outline: none;
  color: var(--ink);
  font-family: var(--hand);
}
.runic-input__field::placeholder {
  color: var(--ink-faint);
}

.runic-input__clear {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  margin-right: 0.6rem;
  padding: 0;
  background: var(--paper-shade);
  border: 1px solid var(--ink-faint);
  border-radius: 50%;
  color: var(--ink-soft);
  cursor: pointer;
  transition: all 0.15s ease;
  z-index: 3;
}
.runic-input__clear:hover {
  background: rgba(175, 96, 37, 0.12);
  border-color: var(--amber);
  color: var(--amber);
}
.runic-input__clear:active { transform: scale(0.95); }
.runic-input__clear svg { width: 12px; height: 12px; }

.runic-input__glow {
  position: absolute;
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 0%;
  height: 1px;
  background: linear-gradient(
    90deg,
    transparent,
    var(--amber),
    transparent
  );
  transition: width 0.3s ease;
  opacity: 0.7;
}
.runic-input__groove:focus-within .runic-input__glow { width: 80%; }

.runic-input--sm .runic-input__groove { min-height: 38px; }
.runic-input--sm .runic-input__field { padding: 0.5rem 1.25rem; font-size: 1rem; }
.runic-input--sm.runic-input--has-icon .runic-input__field { padding-left: 0.75rem; }
.runic-input--sm.runic-input--has-clear .runic-input__field { padding-right: 0.5rem; }
.runic-input--sm .runic-input__icon { width: 16px; height: 16px; }
.runic-input--sm .runic-input__rune { font-size: 0.4rem; }
.runic-input--sm .runic-input__clear { width: 18px; height: 18px; margin-right: 0.5rem; }
.runic-input--sm .runic-input__clear svg { width: 10px; height: 10px; }

.runic-input--md .runic-input__groove { min-height: 46px; }
.runic-input--md .runic-input__field { padding: 0.625rem 1.5rem; font-size: 1.0625rem; }
.runic-input--md.runic-input--has-icon .runic-input__field { padding-left: 0.875rem; }
.runic-input--md.runic-input--has-clear .runic-input__field { padding-right: 0.5rem; }
.runic-input--md .runic-input__icon { width: 18px; height: 18px; }

.runic-input--lg .runic-input__groove { min-height: 54px; }
.runic-input--lg .runic-input__field { padding: 0.75rem 1.75rem; font-size: 1.1875rem; }
.runic-input--lg.runic-input--has-icon .runic-input__field { padding-left: 1rem; }
.runic-input--lg.runic-input--has-clear .runic-input__field { padding-right: 0.5rem; }
.runic-input--lg .runic-input__icon { width: 22px; height: 22px; margin-left: 1.25rem; }
.runic-input--lg .runic-input__rune { font-size: 0.5625rem; }
.runic-input--lg .runic-input__clear { width: 24px; height: 24px; margin-right: 1rem; }
.runic-input--lg .runic-input__clear svg { width: 14px; height: 14px; }

@media (max-width: 640px) {
  .runic-input--sm .runic-input__groove { min-height: 34px; }
  .runic-input--sm .runic-input__field { padding: 0.375rem 1rem; font-size: 0.9375rem; }
  .runic-input--md .runic-input__groove { min-height: 40px; }
  .runic-input--md .runic-input__field { padding: 0.5rem 1rem; font-size: 1rem; }
  .runic-input--md .runic-input__icon { width: 16px; height: 16px; margin-left: 0.75rem; }
  .runic-input--md.runic-input--has-icon .runic-input__field { padding-left: 0.625rem; }
  .runic-input--lg .runic-input__groove { min-height: 48px; }
  .runic-input--lg .runic-input__field { padding: 0.625rem 1.25rem; font-size: 1.0625rem; }
  .runic-input__rune { display: none; }
}
</style>

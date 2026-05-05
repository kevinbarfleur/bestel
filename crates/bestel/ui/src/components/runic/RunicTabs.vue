<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';

interface Tab {
  value: string;
  label: string;
  icon?: string;
}

interface Props {
  tabs: Tab[];
  modelValue: string;
  size?: 'sm' | 'md' | 'lg';
  fullWidth?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  size: 'md',
  fullWidth: true,
});

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const grooveRef = ref<HTMLElement | null>(null);
const sliderPosition = ref({ left: 0, width: 0 });
const isMeasured = ref(false);

const selectedIndex = computed(() => props.tabs.findIndex((t) => t.value === props.modelValue));

const measureTabs = () => {
  if (!grooveRef.value) return;
  const buttons = grooveRef.value.querySelectorAll('.runic-tabs__option');
  const i = selectedIndex.value;
  if (i === -1 || !buttons[i]) {
    isMeasured.value = false;
    return;
  }
  const btn = buttons[i] as HTMLElement;
  sliderPosition.value = { left: btn.offsetLeft, width: btn.offsetWidth };
  isMeasured.value = true;
};

const sliderStyle = computed(() => {
  const i = selectedIndex.value;
  const c = props.tabs.length;
  if (isMeasured.value && sliderPosition.value.width > 0) {
    return { left: `${sliderPosition.value.left}px`, width: `${sliderPosition.value.width}px` };
  }
  return {
    left: i === -1 ? '0%' : `${(i / c) * 100}%`,
    width: `${100 / c}%`,
  };
});

watch(selectedIndex, () => nextTick(measureTabs));
watch(() => props.tabs, () => nextTick(measureTabs), { deep: true });

onMounted(() => {
  nextTick(measureTabs);
  window.addEventListener('resize', measureTabs);
});

onUnmounted(() => {
  window.removeEventListener('resize', measureTabs);
});

const onSelect = (v: string) => emit('update:modelValue', v);
</script>

<template>
  <div
    class="runic-tabs"
    :class="[`runic-tabs--${size}`, { 'runic-tabs--full': fullWidth }]"
    role="tablist"
  >
    <div ref="grooveRef" class="runic-tabs__groove">
      <button
        v-for="tab in tabs"
        :key="tab.value"
        type="button"
        role="tab"
        :aria-selected="modelValue === tab.value"
        class="runic-tabs__option"
        :class="{ 'runic-tabs__option--selected': modelValue === tab.value }"
        @click="onSelect(tab.value)"
      >
        <span v-if="tab.icon" class="runic-tabs__icon">{{ tab.icon }}</span>
        <span class="runic-tabs__label">{{ tab.label }}</span>
      </button>

      <div class="runic-tabs__slider" :style="sliderStyle" aria-hidden="true">
        <span class="runic-tabs__slider-icon" v-if="tabs[selectedIndex]?.icon">
          {{ tabs[selectedIndex]?.icon }}
        </span>
        <span class="runic-tabs__slider-label">{{ tabs[selectedIndex]?.label }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.runic-tabs {
  position: relative;
  display: inline-flex;
  font-family: var(--hand);
}

.runic-tabs--full {
  display: flex;
  width: 100%;
}

.runic-tabs__groove {
  position: relative;
  display: flex;
  flex: 1;
  align-items: stretch;
  background: var(--paper-shade);
  border-radius: 4px;
  border: 1.4px solid var(--ink-faint);
  overflow: hidden;
}

.runic-tabs__groove::before { content: none; }

.runic-tabs__option {
  position: relative;
  z-index: 1;
  flex: 1 1 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all 0.3s ease;
  white-space: nowrap;
  padding: 0.625rem 1rem;
}

.runic-tabs__icon {
  font-size: 0.75rem;
  opacity: 0.55;
  transition: opacity 0.3s ease;
}

.runic-tabs__label {
  font-weight: 600;
  letter-spacing: 0.02em;
  color: var(--ink-faint);
  transition: color 0.18s ease;
}

.runic-tabs__option:hover .runic-tabs__label {
  color: var(--ink-soft);
}

.runic-tabs__option--selected .runic-tabs__label,
.runic-tabs__option--selected .runic-tabs__icon {
  opacity: 0;
}

.runic-tabs__slider {
  position: absolute;
  top: 2px;
  bottom: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  background: var(--ink);
  border-radius: 3px;
  border: 1px solid var(--ink);
  transition: left 0.28s cubic-bezier(0.4, 0, 0.2, 1), width 0.28s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 2;
  pointer-events: none;
}

.runic-tabs__slider::before { content: none; }

.runic-tabs__slider-icon { font-size: 0.75rem; color: var(--amber); }

.runic-tabs__slider-label {
  font-weight: 600;
  letter-spacing: 0.02em;
  color: var(--paper);
}

.runic-tabs--sm .runic-tabs__option { padding: 0.4rem 0.75rem; }
.runic-tabs--sm .runic-tabs__label,
.runic-tabs--sm .runic-tabs__slider-label { font-size: 0.75rem; }

.runic-tabs--md .runic-tabs__option { padding: 0.625rem 1rem; }
.runic-tabs--md .runic-tabs__label,
.runic-tabs--md .runic-tabs__slider-label { font-size: 0.875rem; }

.runic-tabs--lg .runic-tabs__option { padding: 0.875rem 1.5rem; }
.runic-tabs--lg .runic-tabs__label,
.runic-tabs--lg .runic-tabs__slider-label { font-size: 1rem; }
</style>

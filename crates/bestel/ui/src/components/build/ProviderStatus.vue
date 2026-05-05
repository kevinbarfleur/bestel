<script setup lang="ts">
import { computed } from 'vue';
import { storeToRefs } from 'pinia';

import { useSettingsStore } from '../../stores/settings';

const emit = defineEmits<{ openModelPicker: [] }>();

const settings = useSettingsStore();
const { activeModel, detection } = storeToRefs(settings);

const dotClass = computed(() => {
  if (!detection.value) return 'provider-status__dot--unknown';
  return detection.value.active_provider
    ? 'provider-status__dot--ok'
    : 'provider-status__dot--off';
});

const label = computed(() => {
  if (!activeModel.value) return 'No model';
  return activeModel.value.display_name;
});

const providerLabel = computed(() => detection.value?.active_provider ?? 'no provider detected');
</script>

<template>
  <button class="provider-status" type="button" @click="emit('openModelPicker')">
    <span class="provider-status__dot" :class="dotClass" />
    <span class="provider-status__label">{{ label }}</span>
    <span class="provider-status__sep">·</span>
    <span class="provider-status__provider">{{ providerLabel }}</span>
    <span class="provider-status__hint">Ctrl+P</span>
  </button>
</template>

<style scoped>
.provider-status {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.7rem;
  background: transparent;
  border: 1px solid rgba(50, 46, 42, 0.45);
  border-radius: 4px;
  color: var(--color-text-dim, #aa9);
  font-family: 'Cinzel', serif;
  font-size: 0.7rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  cursor: pointer;
  transition: border-color 0.2s ease, color 0.2s ease, background 0.2s ease;
}

.provider-status:hover {
  border-color: rgba(175, 96, 37, 0.5);
  color: var(--color-text-bright, #d8d2c5);
  background: rgba(20, 18, 16, 0.6);
}

.provider-status__dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

.provider-status__dot--ok { background: var(--color-success, #4a9f5a); box-shadow: 0 0 6px rgba(74, 159, 90, 0.4); }
.provider-status__dot--off { background: var(--color-error, #c45050); box-shadow: 0 0 6px rgba(196, 80, 80, 0.4); }
.provider-status__dot--unknown { background: var(--color-text-dim, #776); }

.provider-status__sep {
  color: var(--color-text-dim, #776);
  opacity: 0.5;
}

.provider-status__label {
  color: var(--color-text-bright, #d8d2c5);
}

.provider-status__provider {
  text-transform: lowercase;
  font-style: italic;
  font-family: 'Cormorant Garamond', serif;
  letter-spacing: 0;
  font-size: 0.85rem;
  color: var(--color-text-dim, #aa9);
}

.provider-status__hint {
  margin-left: auto;
  padding: 0.05rem 0.35rem;
  background: rgba(40, 38, 35, 0.7);
  color: var(--color-text-dim, #998);
  border-radius: 3px;
  font-size: 0.65rem;
}
</style>

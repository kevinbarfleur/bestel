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

const modelLabel = computed(() => activeModel.value?.display_name ?? 'no model');
const providerLabel = computed(() => detection.value?.active_provider ?? 'no provider');
</script>

<template>
  <button class="provider-status" type="button" @click="emit('openModelPicker')">
    <span class="provider-status__cap">model</span>
    <span class="provider-status__dots" />
    <span class="provider-status__dot" :class="dotClass" aria-hidden="true" />
    <span class="provider-status__model">{{ modelLabel }}</span>
    <span class="provider-status__sep">·</span>
    <span class="provider-status__provider">{{ providerLabel }}</span>
    <span class="provider-status__hint">Ctrl+P</span>
  </button>
</template>

<style scoped>
.provider-status {
  display: inline-flex;
  align-items: baseline;
  gap: 0.5rem;
  padding: 0.25rem 0.1rem;
  background: transparent;
  border: 0;
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink-soft);
  cursor: pointer;
  transition: color 0.15s ease;
  width: 100%;
  min-width: 0;
}
.provider-status:hover { color: var(--ink); }
.provider-status:hover .provider-status__model { color: var(--amber); }

.provider-status__cap {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  flex: 0 0 auto;
}

.provider-status__dots {
  flex: 1;
  height: 1px;
  border-bottom: 1px dotted var(--paper-line);
  align-self: center;
  min-width: 12px;
  transform: translateY(-3px);
}

.provider-status__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  align-self: center;
  flex: 0 0 auto;
}
.provider-status__dot--ok { background: var(--good); }
.provider-status__dot--off { background: var(--bad); }
.provider-status__dot--unknown { background: var(--ink-faint); }

.provider-status__model {
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
  transition: color 0.15s ease;
}

.provider-status__sep {
  color: var(--ink-faint);
}

.provider-status__provider {
  font-family: var(--hand);
  font-style: italic;
  font-size: 12px;
  color: var(--ink-soft);
  white-space: nowrap;
}

.provider-status__hint {
  margin-left: auto;
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-faint);
  flex: 0 0 auto;
}
</style>

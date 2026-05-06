<script setup lang="ts">
import { computed, watch } from 'vue';
import { storeToRefs } from 'pinia';

import { useSettingsStore } from '../../stores/settings';
import { useToastsStore } from '../../stores/toasts';
import RunicModal from '../runic/RunicModal.vue';
import type { ModelProfileDto, ProviderKindLabel } from '../../api/types';

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>();

const settings = useSettingsStore();
const { models, activeModel, providerAvailability } = storeToRefs(settings);
const toasts = useToastsStore();

const PROVIDER_LABELS: Record<ProviderKindLabel, string> = {
  anthropic: 'Anthropic API',
  codex_cli: 'Codex CLI',
  claude_cli: 'Claude Code CLI',
  ollama: 'Ollama (local)',
};

const grouped = computed(() => {
  const groups = new Map<ProviderKindLabel, ModelProfileDto[]>();
  for (const m of models.value) {
    const list = groups.get(m.provider) ?? [];
    list.push(m);
    groups.set(m.provider, list);
  }
  return groups;
});

const close = () => emit('update:modelValue', false);

const choose = async (id: string, displayName: string) => {
  const result = await settings.setActive(id);
  if (result) {
    toasts.push({ variant: 'success', title: 'Model switched', body: displayName });
    close();
  } else {
    toasts.push({ variant: 'error', title: 'Switch rejected', body: displayName });
  }
};

const isActive = (id: string) => activeModel.value?.id === id;

watch(
  () => props.modelValue,
  async (open) => {
    if (!open) return;
    await settings.refreshModels();
    await settings.refreshDetection();
  },
);
</script>

<template>
  <RunicModal
    :model-value="modelValue"
    title="Select a model"
    icon="◆"
    max-width="lg"
    @update:model-value="(v) => emit('update:modelValue', v)"
  >
    <div class="model-picker">
      <template v-for="[provider, items] in grouped" :key="provider">
        <section class="model-picker__group">
          <header class="model-picker__group-head">
            <span class="model-picker__bar" />
            <h3 class="model-picker__group-title">
              {{ PROVIDER_LABELS[provider] }}
            </h3>
            <span
              class="model-picker__avail"
              :class="{
                'model-picker__avail--ok': providerAvailability[provider],
                'model-picker__avail--off': !providerAvailability[provider],
              }"
            >
              {{ providerAvailability[provider] ? '✓ available' : '× unavailable' }}
            </span>
            <span class="model-picker__bar" />
          </header>

          <ul class="model-picker__list">
            <li
              v-for="m in items"
              :key="m.id"
              class="model-picker__row"
              :class="{
                'model-picker__row--active': isActive(m.id),
                'model-picker__row--unavailable': !providerAvailability[m.provider],
              }"
              @click="choose(m.id, m.display_name)"
            >
              <div class="model-picker__row-head">
                <span class="model-picker__rune">◆</span>
                <span class="model-picker__name">{{ m.display_name }}</span>
                <span class="model-picker__tag" :class="`model-picker__tag--${m.speed}`">
                  {{ m.speed }}
                </span>
                <span class="model-picker__tag" :class="`model-picker__tag--${m.cost}`">
                  {{ m.cost }}
                </span>
                <span
                  v-if="m.provider === 'ollama'"
                  class="model-picker__tag model-picker__tag--experimental"
                  title="Local models — quality varies. Persona, refusal, and PoE knowledge are weaker than Anthropic."
                >
                  experimental
                </span>
              </div>
              <p class="model-picker__desc">{{ m.description }}</p>
              <p v-if="m.cost_per_mtok" class="model-picker__price">
                ${{ m.cost_per_mtok[0].toFixed(2) }} / ${{ m.cost_per_mtok[1].toFixed(2) }} per Mtok
              </p>
            </li>
          </ul>
        </section>
      </template>
    </div>
  </RunicModal>
</template>

<style scoped>
.model-picker {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  max-height: 65vh;
  overflow-y: auto;
}

.model-picker__group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.model-picker__group-head {
  display: flex;
  align-items: center;
  gap: 0.55rem;
}

.model-picker__bar {
  flex: 1;
  height: 1px;
  background: var(--ink-ghost);
  opacity: 0.5;
}

.model-picker__group-title {
  margin: 0;
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  font-weight: 700;
  color: var(--ink);
}

.model-picker__avail {
  font-size: 10px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  font-family: var(--label);
}

.model-picker__avail--ok  { color: var(--good); }
.model-picker__avail--off { color: var(--bad); }

.model-picker__list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.model-picker__row {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  padding: 0.55rem 0.5rem;
  border: 0;
  border-bottom: 1px solid var(--paper-line);
  background: transparent;
  cursor: pointer;
  transition: background 0.15s ease;
  font-family: var(--hand);
  color: var(--ink);
}

.model-picker__row:hover { background: var(--paper-shade); }

.model-picker__row--active {
  background: var(--paper-shade);
}
.model-picker__row--active .model-picker__name {
  color: var(--amber);
}

.model-picker__row--unavailable {
  opacity: 0.55;
}

.model-picker__row-head {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.model-picker__rune {
  color: var(--amber);
  font-size: 0.7rem;
}

.model-picker__name {
  flex: 1;
  font-family: var(--hand-display);
  font-size: 14px;
  font-weight: 600;
  color: var(--ink);
}

.model-picker__tag {
  padding: 0.1rem 0.4rem;
  border-radius: 3px;
  font-size: 10px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  background: var(--paper-shade);
  border: 1px solid var(--ink-faint);
  color: var(--ink-soft);
  font-family: var(--label);
}

.model-picker__tag--fast    { color: var(--good); border-color: var(--good); }
.model-picker__tag--heavy   { color: var(--bad); border-color: var(--bad); }
.model-picker__tag--free    { color: var(--good); border-color: var(--good); }
.model-picker__tag--experimental {
  color: var(--amber);
  border-color: var(--amber);
  border-style: dashed;
}
.model-picker__tag--cheap   { color: var(--good); border-color: var(--good); }
.model-picker__tag--premium { color: var(--bad); border-color: var(--bad); }
.model-picker__tag--subscription { color: var(--amber); border-color: var(--amber); }

.model-picker__desc {
  margin: 0;
  font-family: var(--hand);
  font-size: 13px;
  color: var(--ink-soft);
  line-height: 1.45;
}

.model-picker__price {
  margin: 0;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 10.5px;
  color: var(--ink-faint);
}
</style>

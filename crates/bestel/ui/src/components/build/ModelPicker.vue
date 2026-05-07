<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import { useSettingsStore } from '../../stores/settings';
import { useToastsStore } from '../../stores/toasts';
import RunicModal from '../runic/RunicModal.vue';
import RunicButton from '../runic/RunicButton.vue';
import {
  ApiKeyField,
  PickerLayout,
  PickerListItem,
  PickerSearchInput,
  PickerStatusDot,
  PickerGroupLabel,
  PickerSectionHead,
} from '../pickers';
import { usePickerNav } from '../../composables/usePickerNav';
import type { ModelProfileDto, ProviderKindLabel } from '../../api/types';

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>();

const settings = useSettingsStore();
const { models, activeModel, providerAvailability, detection, apiKeys } =
  storeToRefs(settings);
const toasts = useToastsStore();

const PROVIDER_LABELS: Record<ProviderKindLabel, string> = {
  anthropic: 'Anthropic API',
  ollama: 'Ollama (local)',
};

const search = ref('');

const filteredModels = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return models.value;
  return models.value.filter((m) => {
    const haystack = [
      m.display_name,
      m.description,
      m.provider,
      m.model_id,
      m.cost,
      m.speed,
    ]
      .join(' ')
      .toLowerCase();
    return haystack.includes(q);
  });
});

const grouped = computed(() => {
  // Build groups in canonical order so the sidebar layout is stable across
  // searches (anthropic → ollama).
  const order: ProviderKindLabel[] = ['anthropic', 'ollama'];
  const groups = new Map<ProviderKindLabel, ModelProfileDto[]>();
  for (const m of filteredModels.value) {
    const list = groups.get(m.provider) ?? [];
    list.push(m);
    groups.set(m.provider, list);
  }
  // DeepSeek (api_key_env === 'DEEPSEEK_API_KEY') needs its own visual group
  // even though it's technically the Anthropic provider kind.
  const out: { label: string; key: string; items: ModelProfileDto[] }[] = [];
  for (const kind of order) {
    const items = groups.get(kind) ?? [];
    if (kind === 'anthropic') {
      const anthropic = items.filter((m) => m.api_key_env !== 'DEEPSEEK_API_KEY');
      const deepseek = items.filter((m) => m.api_key_env === 'DEEPSEEK_API_KEY');
      if (anthropic.length) out.push({ label: 'Anthropic API', key: 'anthropic', items: anthropic });
      if (deepseek.length) out.push({ label: 'DeepSeek API', key: 'deepseek', items: deepseek });
    } else if (items.length) {
      out.push({ label: PROVIDER_LABELS[kind], key: kind, items });
    }
  }
  return out;
});

const isProfileAvailable = (m: ModelProfileDto): boolean => {
  if (m.api_key_env) {
    const stored = apiKeys.value.find((k) => k.env_name === m.api_key_env);
    if (stored) return stored.set;
    const needle = m.api_key_env.toLowerCase().replace(/_api_key$/, '');
    const probe = detection.value?.probes.find((p) =>
      p.name.toLowerCase().includes(needle),
    );
    return probe?.installed ?? false;
  }
  return providerAvailability.value[m.provider];
};

const statusKind = (m: ModelProfileDto): 'on' | 'warn' | 'off' => {
  if (isProfileAvailable(m)) return 'on';
  return 'off';
};

const isActive = (m: ModelProfileDto) => activeModel.value?.id === m.id;

const close = () => emit('update:modelValue', false);

const choose = async (m: ModelProfileDto) => {
  if (!isProfileAvailable(m)) {
    toasts.push({
      variant: 'error',
      title: 'Model unavailable',
      body: m.api_key_env
        ? `Set ${m.api_key_env} first.`
        : 'Provider not detected.',
    });
    return;
  }
  const result = await settings.setActive(m.id);
  if (result) {
    toasts.push({
      variant: 'success',
      title: 'Model switched',
      body: m.display_name,
    });
    close();
  } else {
    toasts.push({
      variant: 'error',
      title: 'Switch rejected',
      body: m.display_name,
    });
  }
};

const { highlighted, selected, onKeydown } = usePickerNav<ModelProfileDto>(
  filteredModels,
  choose,
);

const detail = computed(() => selected.value);

watch(
  () => props.modelValue,
  async (open) => {
    if (!open) return;
    search.value = '';
    highlighted.value = 0;
    await Promise.all([
      settings.refreshModels(),
      settings.refreshDetection(),
      settings.refreshKeys(),
    ]);
  },
);

function priceLine(m: ModelProfileDto): string | null {
  if (!m.cost_per_mtok) return null;
  return `$${m.cost_per_mtok[0].toFixed(2)} in · $${m.cost_per_mtok[1].toFixed(2)} out / Mtok`;
}

function metaLine(m: ModelProfileDto): string {
  if (m.cost === 'free') return `${m.speed} · free, runs locally`;
  if (m.cost === 'subscription') return `${m.speed} · covered by your subscription`;
  if (m.cost_per_mtok) {
    return `${m.speed} · $${m.cost_per_mtok[0].toFixed(2)} / $${m.cost_per_mtok[1].toFixed(2)} per Mtok`;
  }
  return m.speed;
}

function providerSubLabel(m: ModelProfileDto): string {
  const groupLabel = m.api_key_env === 'DEEPSEEK_API_KEY' ? 'DeepSeek API' : PROVIDER_LABELS[m.provider];
  const tail = m.cost === 'free'
    ? 'free, runs locally'
    : m.cost === 'subscription'
      ? 'covered by your subscription'
      : m.cost_per_mtok
        ? `$${m.cost_per_mtok[0].toFixed(2)} / $${m.cost_per_mtok[1].toFixed(2)} per Mtok`
        : m.cost;
  return `via ${groupLabel} · ${m.speed} · ${tail}`;
}

function availabilityLabel(m: ModelProfileDto): string {
  if (isProfileAvailable(m)) return 'ready';
  if (m.api_key_env) return 'no key set';
  return 'unavailable';
}

function availabilityColor(m: ModelProfileDto): string {
  if (isProfileAvailable(m)) return 'var(--good)';
  return 'var(--bad)';
}

function actionLabel(m: ModelProfileDto): string {
  if (isActive(m)) return 'Already active';
  if (!isProfileAvailable(m) && m.api_key_env) return 'API key required';
  return 'Use this model';
}

function actionDisabled(m: ModelProfileDto): boolean {
  return isActive(m) || !isProfileAvailable(m);
}

function contextHint(m: ModelProfileDto): string {
  if (isActive(m)) return 'This model is already powering your current chat.';
  if (!isProfileAvailable(m) && m.api_key_env) return 'Save an API key above to enable this model.';
  return 'Switching applies to new messages only — your current chat stays.';
}

function detailGroupLabel(m: ModelProfileDto): string {
  return m.api_key_env === 'DEEPSEEK_API_KEY' ? 'DeepSeek API' : PROVIDER_LABELS[m.provider];
}

function indexInFiltered(m: ModelProfileDto): number {
  return filteredModels.value.findIndex((x) => x.id === m.id);
}
</script>

<template>
  <RunicModal
    :model-value="modelValue"
    title="Select a model"
    subtitle="Pick the LLM that powers Bestel. Each provider needs its own key (Anthropic, DeepSeek) or a running daemon (Ollama)."
    kbd="Ctrl+P"
    max-width="panes"
    @update:model-value="(v) => emit('update:modelValue', v)"
  >
    <PickerLayout>
      <template #sidebar>
        <PickerSearchInput
          v-model="search"
          placeholder="Search models, providers, tiers…"
          :count="filteredModels.length"
          @keydown="onKeydown"
        />

        <p v-if="!filteredModels.length" class="model-picker__empty">No models match.</p>

        <template v-for="g in grouped" :key="g.key">
          <PickerGroupLabel :count="g.items.length">{{ g.label }}</PickerGroupLabel>
          <PickerListItem
            v-for="m in g.items"
            :key="m.id"
            :active="isActive(m)"
            :highlighted="indexInFiltered(m) === highlighted"
            :disabled="!isProfileAvailable(m)"
            @click="choose(m)"
            @mouseenter="highlighted = indexInFiltered(m)"
          >
            <template #name>{{ m.display_name }}</template>
            <template #meta>{{ metaLine(m) }}</template>
            <template #right>
              <PickerStatusDot :kind="statusKind(m)" />
            </template>
          </PickerListItem>
        </template>
      </template>

      <template #main>
        <div v-if="detail" class="model-picker__detail">
          <header class="model-picker__title-row">
            <div class="model-picker__title-block">
              <h1 class="model-picker__title">{{ detail.display_name }}</h1>
              <p class="model-picker__subtitle">{{ providerSubLabel(detail) }}</p>
            </div>
            <span v-if="isActive(detail)" class="model-picker__active-chip">Currently active</span>
          </header>

          <!-- PRIORITY 1 — API key -->
          <section v-if="detail.api_key_env" class="model-picker__section">
            <PickerSectionHead accent>API key</PickerSectionHead>
            <ApiKeyField
              :env-name="detail.api_key_env"
              :provider-label="detailGroupLabel(detail)"
              :api-key-url="detail.api_key_url"
            />
          </section>

          <!-- PRIORITY 2 — Specs & pricing -->
          <section class="model-picker__section">
            <PickerSectionHead>Specs &amp; pricing</PickerSectionHead>
            <div class="model-picker__specs">
              <div class="leader-row">
                <span class="leader-row__k">speed</span>
                <span class="leader-row__dots" />
                <span class="leader-row__v">{{ detail.speed }}</span>
              </div>
              <div class="leader-row">
                <span class="leader-row__k">cost tier</span>
                <span class="leader-row__dots" />
                <span class="leader-row__v">{{ detail.cost }}</span>
              </div>
              <template v-if="detail.cost_per_mtok">
                <div class="leader-row">
                  <span class="leader-row__k">input</span>
                  <span class="leader-row__dots" />
                  <span class="leader-row__v model-picker__mono">${{ detail.cost_per_mtok[0].toFixed(2) }} / Mtok</span>
                </div>
                <div class="leader-row">
                  <span class="leader-row__k">output</span>
                  <span class="leader-row__dots" />
                  <span class="leader-row__v model-picker__mono">${{ detail.cost_per_mtok[1].toFixed(2) }} / Mtok</span>
                </div>
              </template>
              <div class="leader-row">
                <span class="leader-row__k">model id</span>
                <span class="leader-row__dots" />
                <span class="leader-row__v model-picker__mono">{{ detail.model_id || '—' }}</span>
              </div>
              <div class="leader-row">
                <span class="leader-row__k">availability</span>
                <span class="leader-row__dots" />
                <span class="leader-row__v" :style="{ color: availabilityColor(detail) }">
                  {{ availabilityLabel(detail) }}
                </span>
              </div>
            </div>
          </section>

          <!-- PRIORITY 3 — About -->
          <section class="model-picker__section">
            <PickerSectionHead>About</PickerSectionHead>
            <p class="model-picker__about">{{ detail.description }}</p>
          </section>
        </div>
        <p v-else class="model-picker__empty-detail">
          Pick a model on the left to see its details.
        </p>
      </template>

      <template v-if="detail" #actionBar>
        <span class="model-picker__hint">{{ contextHint(detail) }}</span>
        <RunicButton variant="secondary" no-runes @click="close">
          Cancel
        </RunicButton>
        <RunicButton
          variant="primary"
          no-runes
          icon="check"
          kbd="⏎"
          :disabled="actionDisabled(detail)"
          :disabled-reason="isActive(detail) ? 'Already active' : (!isProfileAvailable(detail) && detail.api_key_env ? 'API key required' : undefined)"
          @click="choose(detail)"
        >
          {{ actionLabel(detail) }}
        </RunicButton>
      </template>

      <template #footer>
        <span class="model-picker__hint-row">
          <span class="model-picker__kbd">↑↓</span>
          <span>navigate</span>
        </span>
        <span class="model-picker__hint-row">
          <span class="model-picker__kbd">⏎</span>
          <span>select</span>
        </span>
        <span class="model-picker__hint-row">
          <span class="model-picker__kbd">esc</span>
          <span>close</span>
        </span>
        <span style="flex: 1" />
        <span>{{ filteredModels.length }} model{{ filteredModels.length === 1 ? '' : 's' }}</span>
      </template>
    </PickerLayout>
  </RunicModal>
</template>

<style scoped>
.model-picker__empty,
.model-picker__empty-detail {
  margin: 1rem;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-faint);
  text-align: center;
}

.model-picker__detail {
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.model-picker__title-row {
  display: flex;
  align-items: flex-start;
  gap: 16px;
}

.model-picker__title-block {
  flex: 1;
  min-width: 0;
}

.model-picker__title {
  margin: 0;
  font-family: var(--hand);
  font-size: var(--fs-display);
  font-weight: var(--fw-bold);
  line-height: 1.05;
  color: var(--ink);
}

.model-picker__subtitle {
  margin: 6px 0 0;
  font-family: var(--hand);
  font-size: var(--fs-body);
  color: var(--ink-soft);
  font-weight: var(--fw-regular);
}

.model-picker__active-chip {
  font-family: var(--label);
  font-size: var(--fs-micro);
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--amber);
  font-weight: var(--fw-bold);
  padding: 4px 10px;
  border: 1.4px solid var(--amber);
  border-radius: 3px;
  background: var(--amber-glow);
  flex: none;
  white-space: nowrap;
}

.model-picker__section {
  display: flex;
  flex-direction: column;
}

.model-picker__specs {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 4px 32px;
}

.model-picker__about {
  margin: 0;
  font-family: var(--hand);
  font-size: var(--fs-body);
  line-height: 1.6;
  color: var(--ink);
  max-width: 640px;
}

.model-picker__mono {
  font-family: var(--mono);
  font-size: var(--fs-meta);
}

.model-picker__hint {
  flex: 1;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}

.model-picker__hint-row {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.model-picker__kbd {
  font-family: var(--label);
  font-size: var(--fs-micro);
  padding: 1px 5px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  color: var(--ink-soft);
}
</style>

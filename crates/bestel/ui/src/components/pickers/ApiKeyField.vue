<script setup lang="ts">
import { computed, ref, watch } from 'vue';

import { openLink } from '../../api/tauri';
import { useSettingsStore } from '../../stores/settings';
import { useToastsStore } from '../../stores/toasts';
import RunicButton from '../runic/RunicButton.vue';

const props = defineProps<{
  /** The env var name we are saving (must be on backend allowlist). */
  envName: string;
  /** Friendly provider label (e.g. "Anthropic API", "DeepSeek API"). */
  providerLabel?: string;
  /** External link to the provider's key creation page. */
  apiKeyUrl?: string | null;
}>();

const settings = useSettingsStore();
const toasts = useToastsStore();

const status = computed(() => settings.keyStatus(props.envName));

const draft = ref('');
const reveal = ref(false);
const saving = ref(false);
const editing = ref(false);

watch(
  () => props.envName,
  () => {
    draft.value = '';
    editing.value = false;
    reveal.value = false;
  },
);

const isFromEnvironment = computed(
  () => status.value?.set && status.value.masked_preview === '(from environment)',
);

const canSave = computed(() => draft.value.trim().length >= 10);

const providerName = computed(() => props.providerLabel ?? 'this provider');

async function save() {
  if (!canSave.value || saving.value) return;
  saving.value = true;
  const err = await settings.saveApiKey(props.envName, draft.value);
  saving.value = false;
  if (err) {
    toasts.push({ variant: 'error', title: 'Failed to save key', body: err });
  } else {
    toasts.push({
      variant: 'success',
      title: 'Key saved',
      body: `${props.envName} stored on disk`,
    });
    draft.value = '';
    editing.value = false;
    reveal.value = false;
  }
}

async function remove() {
  if (saving.value) return;
  saving.value = true;
  const err = await settings.deleteApiKey(props.envName);
  saving.value = false;
  if (err) {
    toasts.push({ variant: 'error', title: 'Failed to remove key', body: err });
  } else {
    toasts.push({ variant: 'info', title: 'Key removed', body: props.envName });
    draft.value = '';
    editing.value = false;
  }
}

function startEdit() {
  editing.value = true;
}

function cancelEdit() {
  editing.value = false;
  draft.value = '';
  reveal.value = false;
}

async function openKeyUrl() {
  if (!props.apiKeyUrl) return;
  try {
    await openLink(props.apiKeyUrl);
  } catch (e) {
    toasts.push({
      variant: 'error',
      title: 'Could not open link',
      body: e instanceof Error ? e.message : String(e),
    });
  }
}

function maskedDisplay(): string {
  const preview = status.value?.masked_preview;
  if (!preview) return '· stored ·';
  if (preview === '(from environment)') return 'set in shell environment';
  return preview;
}
</script>

<template>
  <div class="api-key-field">
    <!-- ENV-VAR mode: key picked up from shell, do not allow disk write -->
    <div v-if="isFromEnvironment" class="api-key-field__card api-key-field__card--env">
      <div class="api-key-field__row">
        <span class="api-key-field__label">{{ envName }}</span>
        <span class="api-key-field__hairline" />
        <span class="api-key-field__env-tag">set in shell</span>
      </div>
      <p class="api-key-field__note">
        This key was picked up from your shell environment. Bestel will not overwrite
        it from disk.
      </p>
    </div>

    <!-- SET on disk: green-border card with masked preview + actions -->
    <div
      v-else-if="status?.set && !editing"
      class="api-key-field__card api-key-field__card--set"
    >
      <div class="api-key-field__row">
        <span class="api-key-field__dot" aria-hidden="true" />
        <span class="api-key-field__masked">{{ maskedDisplay() }}</span>
        <span class="api-key-field__hairline" />
        <button type="button" class="link link--soft" @click="startEdit">replace</button>
        <button
          type="button"
          class="link link--danger"
          :disabled="saving"
          @click="remove"
        >
          remove
        </button>
      </div>
      <p class="api-key-field__note">
        Stored in <code>~/.bestel/runtime/keys.json</code> as plain text. Bestel
        never sends it anywhere except {{ providerName }}.
      </p>
    </div>

    <!-- UNSET / EDIT: input + Save key + disclaimer + Get-a-key link -->
    <div v-else class="api-key-field__card">
      <div class="api-key-field__row api-key-field__row--input">
        <input
          :type="reveal ? 'text' : 'password'"
          class="api-key-field__input"
          :placeholder="`Paste ${envName} here…`"
          v-model="draft"
          autocomplete="off"
          spellcheck="false"
          @keydown.enter.prevent="save"
          @keydown.esc.prevent="cancelEdit"
        />
        <button
          type="button"
          class="link link--soft api-key-field__reveal"
          @click="reveal = !reveal"
        >
          {{ reveal ? 'hide' : 'show' }}
        </button>
        <RunicButton
          variant="primary"
          size="sm"
          no-runes
          icon="save"
          :disabled="!canSave || saving"
          :disabled-reason="saving ? 'Saving…' : undefined"
          @click="save"
        >
          Save key
        </RunicButton>
        <button
          v-if="status?.set"
          type="button"
          class="link link--soft"
          @click="cancelEdit"
        >
          cancel
        </button>
      </div>
      <p class="api-key-field__note">
        Stored in <code>~/.bestel/runtime/keys.json</code> as plain text. Bestel
        never sends it anywhere except {{ providerName }}.
        <button
          v-if="apiKeyUrl"
          type="button"
          class="link"
          @click="openKeyUrl"
        >
          Get a key from {{ providerName }} ↗
        </button>
      </p>
    </div>
  </div>
</template>

<style scoped>
.api-key-field {
  display: flex;
  flex-direction: column;
}

.api-key-field__card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 14px;
  border: 1px solid var(--ink-soft);
  border-radius: 4px;
  background: var(--paper);
}
.api-key-field__card--set {
  border-color: var(--good);
  background: rgba(84, 124, 74, 0.06);
}
.api-key-field__card--env {
  border-color: var(--ink-faint);
  border-style: dashed;
  background: var(--paper-shade);
}

.api-key-field__row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.api-key-field__row--input {
  align-items: stretch;
}

.api-key-field__dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--good);
  flex: none;
}

.api-key-field__masked {
  font-family: var(--mono);
  font-size: var(--fs-body);
  color: var(--ink);
  letter-spacing: 0.04em;
}

.api-key-field__label {
  font-family: var(--mono);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}

.api-key-field__env-tag {
  font-family: var(--label);
  font-size: var(--fs-caps);
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: var(--fw-semibold);
}

.api-key-field__hairline {
  flex: 1;
}

.api-key-field__input {
  flex: 1;
  min-width: 0;
  border: 1px solid var(--ink-soft);
  border-radius: 4px;
  background: var(--paper);
  padding: 10px 12px;
  font-family: var(--mono);
  font-size: var(--fs-body);
  color: var(--ink);
  outline: 0;
  transition: border-color 0.15s ease;
}
.api-key-field__input:focus {
  border-color: var(--amber);
}
.api-key-field__input::placeholder {
  font-family: var(--mono);
  color: var(--ink-faint);
}

.api-key-field__reveal {
  align-self: center;
}

.api-key-field__note {
  margin: 4px 0 0;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
  line-height: 1.5;
}
.api-key-field__note code {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--ink);
  background: rgba(0, 0, 0, 0.05);
  padding: 1px 4px;
  border-radius: 2px;
}
.theme-dark .api-key-field__note code {
  background: rgba(255, 255, 255, 0.06);
}
</style>

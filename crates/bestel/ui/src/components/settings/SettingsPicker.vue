<script setup lang="ts">
import { computed, ref } from 'vue';
import { storeToRefs } from 'pinia';

import { useSettingsStore } from '../../stores/settings';
import { useChatHistoryStore } from '../../stores/chatHistory';
import { useToastsStore } from '../../stores/toasts';
import { useUiStore } from '../../stores/ui';
import RunicModal from '../runic/RunicModal.vue';
import RunicButton from '../runic/RunicButton.vue';
import RunicIcon from '../runic/RunicIcon.vue';

import SettingsField from './SettingsField.vue';
import SettingsRailItem from './SettingsRailItem.vue';
import SettingsThemeCard from './SettingsThemeCard.vue';
import SettingsSegmented from './SettingsSegmented.vue';
import SettingsToggle from './SettingsToggle.vue';
import SettingsRadioCard from './SettingsRadioCard.vue';
import SettingsModelRow from './SettingsModelRow.vue';
import SettingsPromptsPane from './SettingsPromptsPane.vue';
import SettingsDeveloperPane from './SettingsDeveloperPane.vue';

defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>();

const ui = useUiStore();

function openLibrary() {
  emit('update:modelValue', false);
  ui.openRegistryModal();
}

type Section =
  | 'appearance'
  | 'links'
  | 'model'
  | 'folders'
  | 'prompts'
  | 'developer'
  | 'shortcuts'
  | 'data';

const SECTIONS: Array<{ id: Section; label: string; hint: string }> = [
  { id: 'appearance', label: 'Appearance', hint: 'Theme & paper tone' },
  { id: 'links', label: 'Links', hint: 'Wiki & external behavior' },
  { id: 'model', label: 'Default model', hint: 'Boot selection' },
  { id: 'folders', label: 'Watcher folders', hint: 'Where Bestel finds PoB files' },
  { id: 'prompts', label: 'Prompts & docs', hint: 'Edit system prompt + per-model overrides' },
  { id: 'developer', label: 'Developer', hint: 'Test panel + battery runner' },
  { id: 'shortcuts', label: 'Shortcuts', hint: 'Keyboard reference' },
  { id: 'data', label: 'Data & privacy', hint: 'Storage, caches, keys' },
];

const SHORTCUTS: Array<[string, string[]]> = [
  ['Open model picker', ['Ctrl', 'P']],
  ['Open build picker', ['Ctrl', 'B']],
  ['Open chat picker', ['Ctrl', 'H']],
  ['Open settings', ['Ctrl', ',']],
  ['Reset chat', ['Ctrl', 'L']],
  ['Open debug view (dev)', ['Ctrl', 'Shift', 'D']],
  ['Send message', ['⏎']],
  ['Newline in composer', ['Shift', '⏎']],
  ['Close any modal', ['Esc']],
];

const settings = useSettingsStore();
const history = useChatHistoryStore();
const toasts = useToastsStore();

const {
  theme,
  density,
  linkBehavior,
  defaultModelId,
  watcherFolders,
  apiKeys,
  models,
  verifyEnabled,
} = storeToRefs(settings);

async function onVerifyToggle(next: boolean) {
  const err = await settings.setVerifyEnabled(next);
  if (err) {
    toasts.push({
      variant: 'error',
      title: 'Could not save verification setting',
      body: err,
    });
  }
}

const active = ref<Section>('appearance');

const reduceMotion = ref(false);
const pinnedWebviews = ref(true);
const confirmUnknown = ref(false);
const onUnavailable = ref<'fallback' | 'prompt' | 'fail'>('fallback');

const close = () => emit('update:modelValue', false);

function pickSection(id: Section) {
  active.value = id;
}

function placeholderToast(label: string) {
  toasts.push({
    variant: 'info',
    title: label,
    body: 'Coming in a future build.',
  });
}

async function clearChatHistory() {
  history.clear();
  toasts.push({
    variant: 'info',
    title: 'Chat history cleared',
    body: 'All saved conversations have been removed.',
  });
}

async function forgetAllKeys() {
  const stored = apiKeys.value.filter((k) => k.set);
  if (stored.length === 0) {
    toasts.push({
      variant: 'info',
      title: 'No keys stored',
      body: 'There were no API keys to forget.',
    });
    return;
  }
  let failed = 0;
  for (const k of stored) {
    const err = await settings.deleteApiKey(k.env_name);
    if (err) failed += 1;
  }
  if (failed > 0) {
    toasts.push({
      variant: 'error',
      title: 'Some keys could not be removed',
      body: `${stored.length - failed}/${stored.length} keys forgotten.`,
    });
  } else {
    toasts.push({
      variant: 'info',
      title: 'All API keys forgotten',
      body: `${stored.length} key${stored.length === 1 ? '' : 's'} removed from disk.`,
    });
  }
}

const storedKeyCount = computed(() => apiKeys.value.filter((k) => k.set).length);
const chatCount = computed(() => history.chats.length);
</script>

<template>
  <RunicModal
    :model-value="modelValue"
    title="Settings"
    subtitle="Application preferences. Changes save as you make them."
    kbd="Ctrl ,"
    max-width="panes"
    @update:model-value="(v) => emit('update:modelValue', v)"
  >
    <div class="settings">
      <!-- LEFT RAIL -->
      <aside class="settings__rail runic-scrollbar">
        <div class="settings__rail-head">Sections</div>
        <SettingsRailItem
          v-for="s in SECTIONS"
          :key="s.id"
          :selected="active === s.id"
          :label="s.label"
          :hint="s.hint"
          @pick="pickSection(s.id)"
        />
      </aside>

      <!-- DETAIL PANE -->
      <section class="settings__pane runic-scrollbar">
        <!-- ============== APPEARANCE ============== -->
        <div v-if="active === 'appearance'" class="pane">
          <header class="pane__header">
            <h1 class="pane__title">Appearance</h1>
            <p class="pane__sub">
              Bestel ships in two parchment tones. Pick whichever reads easier.
            </p>
          </header>

          <SettingsField label="Theme">
            <div class="theme-grid">
              <SettingsThemeCard
                value="light"
                :selected="theme === 'light'"
                label="Light"
                sub="Cool paper neutral"
                @pick="settings.setTheme('light')"
              />
              <SettingsThemeCard
                value="dark"
                :selected="theme === 'dark'"
                label="Dark"
                sub="Deep slate"
                @pick="settings.setTheme('dark')"
              />
            </div>
          </SettingsField>

          <SettingsField
            label="Density"
            hint="Spacing inside lists, modals and the chat."
          >
            <SettingsSegmented
              :model-value="density"
              :options="[
                { value: 'compact', label: 'Compact' },
                { value: 'standard', label: 'Standard' },
                { value: 'airy', label: 'Airy' },
              ]"
              @update:model-value="settings.setDensity"
            />
          </SettingsField>

          <SettingsField
            label="Reduce motion"
            hint="Disables decorative animations. Coming in a future build — currently follows system preference."
          >
            <SettingsToggle
              v-model="reduceMotion"
              caption="Off — follow system preference"
              disabled
            />
          </SettingsField>
        </div>

        <!-- ============== LINKS ============== -->
        <div v-else-if="active === 'links'" class="pane">
          <header class="pane__header">
            <h1 class="pane__title">Links</h1>
            <p class="pane__sub">
              What happens when you click a wiki entity, a citation, or any
              external URL in chat.
            </p>
          </header>

          <SettingsField label="Open behavior">
            <div class="settings__radio-stack">
              <SettingsRadioCard
                :selected="linkBehavior === 'modal'"
                icon="open"
                title="Open inside Bestel"
                body="Loads the page in a webview overlay above the chat. No tab switch, no browser jump — keep the conversation in view."
                badge="Default"
                @pick="settings.setLinkBehavior('modal')"
              />
              <SettingsRadioCard
                :selected="linkBehavior === 'browser'"
                icon="arrow"
                title="Open in default browser"
                body="Hands the URL to your OS. The page opens in Chrome / Firefox / Safari like any normal link."
                @pick="settings.setLinkBehavior('browser')"
              />
            </div>
          </SettingsField>

          <SettingsField
            label="Pinned webviews"
            hint="Pin rail under the topbar. Coming in a future build."
          >
            <SettingsToggle
              v-model="pinnedWebviews"
              caption="On — pin rail visible under the topbar when you have any"
              disabled
            />
          </SettingsField>

          <SettingsField
            label="Confirm before opening unknown domains"
            hint="Asks before loading anything outside the trusted source allowlist. Coming in a future build."
          >
            <SettingsToggle
              v-model="confirmUnknown"
              caption="Off"
              disabled
            />
          </SettingsField>
        </div>

        <!-- ============== DEFAULT MODEL ============== -->
        <div v-else-if="active === 'model'" class="pane">
          <header class="pane__header">
            <h1 class="pane__title">Default model</h1>
            <p class="pane__sub">
              Which model is selected when Bestel starts. You can switch any
              time with Ctrl+P.
            </p>
          </header>

          <SettingsField label="Boot model">
            <div class="settings__model-list">
              <SettingsModelRow
                v-for="m in models"
                :key="m.id"
                :active="defaultModelId === m.id"
                :name="m.display_name"
                :sub="`${m.provider.replace('_', ' ')} · ${m.cost}${m.cost_per_mtok ? ` · $${m.cost_per_mtok[0].toFixed(2)} / $${m.cost_per_mtok[1].toFixed(2)} per Mtok` : ''}`"
                @pick="settings.setDefaultModelId(m.id)"
              />
              <div v-if="models.length === 0" class="pane__empty">
                No models registered yet — open the model picker (Ctrl+P) to
                discover what's available on your machine.
              </div>
            </div>
            <button
              v-if="defaultModelId !== null"
              type="button"
              class="link link--soft settings__clear-default"
              @click="settings.setDefaultModelId(null)"
            >
              clear default — use last-active instead
            </button>
          </SettingsField>

          <SettingsField
            label="On model unavailable"
            hint="If the boot model can't load (no key, daemon offline). Coming in a future build."
          >
            <SettingsSegmented
              :model-value="onUnavailable"
              :options="[
                { value: 'fallback', label: 'Fall back to next available' },
                { value: 'prompt', label: 'Open picker' },
                { value: 'fail', label: 'Show error' },
              ]"
              @update:model-value="(v) => { onUnavailable = v; placeholderToast('On-unavailable behavior'); }"
            />
          </SettingsField>

          <SettingsField
            label="Claim verification"
            hint="When enabled, Bestel re-checks numerical and named claims against the local knowledge base before showing the answer. Adds ~3–5s latency and ~10–20% cost; trivial replies are skipped automatically."
          >
            <SettingsToggle
              :model-value="verifyEnabled"
              caption="Verify factual claims (Chain-of-Verification)"
              @update:model-value="onVerifyToggle"
            />
          </SettingsField>
        </div>

        <!-- ============== WATCHER FOLDERS ============== -->
        <div v-else-if="active === 'folders'" class="pane">
          <header class="pane__header">
            <h1 class="pane__title">Watcher folders</h1>
            <p class="pane__sub">
              Folders Bestel scans for Path of Building XML exports.
              Detected files show up in your library so you can save the ones
              you want to talk to Bestel about.
            </p>
          </header>

          <div class="settings__library-cta">
            <RunicButton
              variant="primary"
              icon="arrow-right"
              @click="openLibrary"
            >Manage builds in your library</RunicButton>
            <p class="settings__library-caption">
              Watcher folders feed your library — picking which detected builds
              you actually use happens there.
            </p>
          </div>

          <SettingsField
            label="Watched paths"
            hint="Recursive. Symlinks followed. Scans every 30s. Right now only the default Path of Building documents folder is monitored — explicit folder management is coming."
          >
            <div class="settings__folder-list">
              <div class="settings__folder-row">
                <div class="settings__folder-info">
                  <code class="settings__folder-path">~/Documents/Path of Building Community/Builds</code>
                  <span class="settings__folder-meta">Default · always watched</span>
                </div>
              </div>
              <div
                v-for="path in watcherFolders"
                :key="path"
                class="settings__folder-row"
              >
                <div class="settings__folder-info">
                  <code class="settings__folder-path">{{ path }}</code>
                  <span class="settings__folder-meta">Custom · saved</span>
                </div>
                <button
                  class="link link--danger"
                  type="button"
                  @click="settings.setWatcherFolders(watcherFolders.filter((p) => p !== path))"
                >Remove</button>
              </div>
              <RunicButton
                variant="secondary"
                no-runes
                size="sm"
                icon="plus"
                class="settings__folder-add"
                @click="placeholderToast('Add a folder')"
              >Add a folder…</RunicButton>
            </div>
          </SettingsField>
        </div>

        <!-- ============== PROMPTS & DOCS ============== -->
        <SettingsPromptsPane v-else-if="active === 'prompts'" />

        <!-- ============== DEVELOPER ============== -->
        <SettingsDeveloperPane v-else-if="active === 'developer'" />

        <!-- ============== SHORTCUTS ============== -->
        <div v-else-if="active === 'shortcuts'" class="pane">
          <header class="pane__header">
            <h1 class="pane__title">Shortcuts</h1>
            <p class="pane__sub">Reference only. These aren't customizable yet.</p>
          </header>

          <div class="settings__shortcuts">
            <div
              v-for="([label, keys], i) in SHORTCUTS"
              :key="i"
              class="settings__shortcut-row"
            >
              <span class="settings__shortcut-label">{{ label }}</span>
              <span class="settings__shortcut-keys">
                <span
                  v-for="k in keys"
                  :key="k"
                  class="settings__kbd"
                >{{ k }}</span>
              </span>
            </div>
          </div>
        </div>

        <!-- ============== DATA & PRIVACY ============== -->
        <div v-else-if="active === 'data'" class="pane">
          <header class="pane__header">
            <h1 class="pane__title">Data &amp; privacy</h1>
            <p class="pane__sub">
              Where Bestel keeps things on disk, and how to wipe them.
            </p>
          </header>

          <SettingsField label="Storage paths">
            <div class="settings__path-list">
              <div class="settings__path-row">
                <span class="settings__path-key">API keys</span>
                <code class="settings__path-val">~/.bestel/runtime/keys.json</code>
                <span class="settings__path-warn">plain text</span>
              </div>
              <div class="settings__path-row">
                <span class="settings__path-key">Chats</span>
                <code class="settings__path-val">localStorage / bestel.chats.v1</code>
              </div>
              <div class="settings__path-row">
                <span class="settings__path-key">Webview cache</span>
                <code class="settings__path-val">~/.bestel/cache/webview/</code>
              </div>
              <div class="settings__path-row">
                <span class="settings__path-key">Logs</span>
                <code class="settings__path-val">~/.bestel/logs/</code>
              </div>
            </div>
          </SettingsField>

          <SettingsField
            label="Wipe"
            hint="None of this is recoverable. Bestel does not require a restart."
          >
            <div class="settings__wipe-row">
              <RunicButton
                variant="secondary"
                no-runes
                size="sm"
                icon="trash"
                @click="placeholderToast('Clear webview cache')"
              >
                Clear webview cache
              </RunicButton>
              <RunicButton
                variant="secondary"
                no-runes
                size="sm"
                icon="trash"
                :disabled="chatCount === 0"
                :disabled-reason="chatCount === 0 ? 'No saved chats' : undefined"
                @click="clearChatHistory"
              >
                Clear chat history ({{ chatCount }})
              </RunicButton>
              <RunicButton
                variant="secondary"
                no-runes
                size="sm"
                icon="trash"
                :disabled="storedKeyCount === 0"
                :disabled-reason="storedKeyCount === 0 ? 'No keys stored' : undefined"
                @click="forgetAllKeys"
              >
                Forget all API keys ({{ storedKeyCount }})
              </RunicButton>
            </div>
          </SettingsField>
        </div>
      </section>
    </div>

    <template #footer>
      <span class="settings__saved">
        <span class="settings__saved-dot" />
        <span>All changes saved</span>
      </span>
      <span style="flex: 1" />
      <span class="settings__version">Bestel · v0.1</span>
    </template>
  </RunicModal>
</template>

<style scoped>
.settings {
  display: grid;
  grid-template-columns: 260px 1fr;
  height: 100%;
  min-height: 0;
  background: var(--paper);
  font-family: var(--hand);
  color: var(--ink);
}

.settings__rail {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-y: auto;
  border-right: 1px solid var(--paper-line);
  background: var(--paper);
  padding: 14px 0;
}
.settings__rail-head {
  padding: 6px 22px 10px;
  font-family: var(--label);
  font-size: var(--fs-caps);
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: var(--fw-semibold);
}

.settings__pane {
  min-height: 0;
  overflow-y: auto;
  padding: 26px 36px 32px;
}

.pane {
  display: flex;
  flex-direction: column;
  gap: 26px;
}
.pane__header {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.pane__title {
  margin: 0;
  font-size: var(--fs-h1);
  font-weight: var(--fw-bold);
  line-height: 1.1;
  color: var(--ink);
}
.pane__sub {
  margin: 0;
  max-width: 620px;
  font-size: var(--fs-h3);
  color: var(--ink-soft);
  line-height: 1.5;
}
.pane__empty {
  padding: 14px 0;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-faint);
}

.theme-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 18px;
}

.settings__radio-stack {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.settings__model-list {
  display: flex;
  flex-direction: column;
}

.settings__clear-default {
  margin-top: 14px;
  align-self: flex-start;
}

.settings__library-cta {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 18px;
  padding: 14px 16px;
  background: var(--paper-shade);
  border: 1px dashed var(--paper-line);
  border-radius: 5px;
}
.settings__library-caption {
  margin: 0;
  font-family: var(--hand);
  font-size: 13px;
  color: var(--ink-soft);
}

.settings__folder-list {
  display: flex;
  flex-direction: column;
}
.settings__folder-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 11px 4px;
  border-top: 1px dotted var(--paper-line);
}
.settings__folder-row:first-child { border-top: 0; }
.settings__folder-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.settings__folder-path {
  font-family: var(--mono);
  font-size: var(--fs-meta);
  color: var(--ink);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.settings__folder-meta {
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}
.settings__folder-add {
  margin-top: 14px;
  align-self: flex-start;
}

.settings__shortcuts {
  display: flex;
  flex-direction: column;
}
.settings__shortcut-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 4px;
  border-top: 1px dotted var(--paper-line);
}
.settings__shortcut-row:first-child { border-top: 0; }
.settings__shortcut-label {
  flex: 1;
  font-size: var(--fs-body);
  color: var(--ink);
}
.settings__shortcut-keys {
  display: inline-flex;
  gap: 4px;
}
.settings__kbd {
  font-family: var(--label);
  font-size: var(--fs-meta);
  padding: 2px 7px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  color: var(--ink-soft);
  background: var(--paper);
  letter-spacing: 0.04em;
}

.settings__path-list {
  display: flex;
  flex-direction: column;
}
.settings__path-row {
  display: flex;
  align-items: baseline;
  gap: 14px;
  padding: 9px 4px;
  border-top: 1px dotted var(--paper-line);
}
.settings__path-row:first-child { border-top: 0; }
.settings__path-key {
  width: 130px;
  flex: none;
  font-family: var(--hand);
  font-size: var(--fs-body);
  color: var(--ink-soft);
}
.settings__path-val {
  flex: 1;
  min-width: 0;
  font-family: var(--mono);
  font-size: var(--fs-meta);
  color: var(--ink);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.settings__path-warn {
  flex: none;
  font-family: var(--label);
  font-size: var(--fs-micro);
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--note);
  font-weight: var(--fw-bold);
  padding: 2px 7px;
  border: 1px solid var(--note);
  border-radius: 3px;
}

.settings__wipe-row {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.settings__saved {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}
.settings__saved-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--good);
  box-shadow: 0 0 0 3px rgba(46, 107, 58, 0.18);
}

.settings__version {
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-faint);
}
</style>

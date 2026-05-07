<script setup lang="ts">
import { computed, onMounted } from 'vue';

import RunicButton from '../runic/RunicButton.vue';
import SettingsField from './SettingsField.vue';

import { promptsOpenEditor } from '../../api/tauri';
import { usePromptsStore } from '../../stores/prompts';
import { useToastsStore } from '../../stores/toasts';

const store = usePromptsStore();
const toasts = useToastsStore();

const allFiles = computed(() => store.tree.groups.flatMap((g) => g.items));

onMounted(() => {
  void store.loadTree();
});

async function openEditor() {
  try {
    await promptsOpenEditor();
  } catch (e) {
    toasts.push({
      variant: 'error',
      title: 'Could not open prompt editor',
      body: String(e),
    });
  }
}

async function resetAll() {
  const ok = confirm(
    'Restore every shipped prompt to its bundled default? Your local edits to SYSTEM_PROMPT.md, CORE_KNOWLEDGE.md and local_addendum.md will be lost.',
  );
  if (!ok) return;
  await store.resetAll();
  toasts.push({
    variant: 'info',
    title: 'Prompts reset',
    body: 'All shipped prompts restored to bundled defaults.',
  });
}

function shortName(rel: string) {
  return rel.split('/').pop() ?? rel;
}
</script>

<template>
  <div class="pane">
    <header class="pane__header">
      <h1 class="pane__title">Prompts &amp; documentation</h1>
      <p class="pane__sub">
        Bestel reads its prompts from a folder of <code>.md</code> files. Edit them
        to add links, adjust the voice, or override a specific provider or model.
        Open-source friendly — fork the repo, change the files, ship a new tone.
      </p>
    </header>

    <SettingsField
      label="Prompt library"
      hint="Opens a separate window with a tree of every prompt + doc file. Edits autosave to ~/.bestel/prompts/."
    >
      <div class="prompts-cta">
        <div class="prompts-cta__copy">
          <div class="prompts-cta__title">
            <span>Open prompt editor</span>
            <code class="prompts-cta__path">~/.bestel/prompts/</code>
          </div>
          <p class="prompts-cta__body">
            Edit the system prompt, per-provider templates, per-model overrides,
            and the local addendum. A fresh window with a real text editor —
            line numbers, outline, save state.
          </p>
        </div>
        <RunicButton variant="primary" no-runes icon="open" @click="openEditor">
          Open editor
        </RunicButton>
      </div>
    </SettingsField>

    <SettingsField label="Files in your prompt folder">
      <div class="prompts-files">
        <div v-for="file in allFiles" :key="file.rel_path" class="prompts-files__row">
          <div class="prompts-files__main">
            <div class="prompts-files__name">
              {{ shortName(file.rel_path) }}
              <span v-if="file.modified_vs_bundled" class="prompts-files__dot" title="Modified vs bundled">•</span>
            </div>
            <div class="prompts-files__meta">{{ file.line_count }} lines</div>
          </div>
          <span
            v-if="file.kind === 'shipped'"
            class="prompts-files__badge"
          >shipped</span>
          <span v-else class="prompts-files__badge prompts-files__badge--custom">custom</span>
        </div>
        <p v-if="allFiles.length === 0" class="prompts-files__empty">
          Folder empty. Open the editor to seed defaults.
        </p>
      </div>
    </SettingsField>

    <SettingsField
      label="Restore"
      hint="Forgets every local edit and re-extracts the three bundled defaults."
    >
      <RunicButton variant="secondary" no-runes icon="trash" @click="resetAll">
        Reset all prompts to bundled defaults
      </RunicButton>
    </SettingsField>
  </div>
</template>

<style scoped>
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
  font-size: 24px;
  font-weight: 700;
  color: var(--ink);
}

.pane__sub {
  margin: 0;
  font-size: 15px;
  color: var(--ink-soft);
  max-width: 620px;
  line-height: 1.5;
}

.pane__sub code {
  font-family: var(--mono);
  font-size: 13px;
}

.prompts-cta {
  padding: 18px;
  border: 1px solid var(--paper-line);
  border-radius: 5px;
  background: var(--paper-shade);
  display: flex;
  align-items: center;
  gap: 16px;
}

.prompts-cta__copy {
  flex: 1;
  min-width: 0;
}

.prompts-cta__title {
  display: flex;
  align-items: baseline;
  gap: 10px;
  font-size: 16px;
  font-weight: 600;
  color: var(--ink);
}

.prompts-cta__path {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--ink-faint);
  font-weight: 400;
}

.prompts-cta__body {
  margin: 4px 0 0;
  font-size: 13.5px;
  color: var(--ink-soft);
  line-height: 1.5;
  max-width: 580px;
}

.prompts-files {
  display: flex;
  flex-direction: column;
}

.prompts-files__row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 14px;
  border-top: 1px dotted var(--paper-line);
}

.prompts-files__row:first-child {
  border-top: none;
}

.prompts-files__main {
  flex: 1;
  min-width: 0;
}

.prompts-files__name {
  display: flex;
  align-items: baseline;
  gap: 7px;
  font-family: var(--mono);
  font-size: 13.5px;
  color: var(--ink);
}

.prompts-files__dot {
  color: var(--amber);
  font-size: 14px;
  font-weight: 700;
  line-height: 1;
}

.prompts-files__meta {
  font-size: 12.5px;
  color: var(--ink-soft);
  margin-top: 2px;
}

.prompts-files__badge {
  font-family: var(--label);
  font-size: 10.5px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 600;
}

.prompts-files__badge--custom {
  color: var(--amber);
  font-weight: 700;
}

.prompts-files__empty {
  margin: 8px 0 0;
  font-size: 13px;
  font-style: italic;
  color: var(--ink-faint);
}
</style>

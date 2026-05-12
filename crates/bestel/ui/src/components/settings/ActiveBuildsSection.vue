<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { useRegistryStore } from '../../stores/registry';
import { useBuildStore } from '../../stores/build';
import { useUiStore } from '../../stores/ui';
import { useChatHistoryStore } from '../../stores/chatHistory';
import { useToastsStore } from '../../stores/toasts';
import { registryPreview } from '../../api/tauri';
import RegistryRow from '../registry/RegistryRow.vue';
import AddBuildModal from '../registry/AddBuildModal.vue';
import RunicButton from '../runic/RunicButton.vue';
import RunicIcon from '../runic/RunicIcon.vue';
import type { RegistryEntryDto } from '../../api/types';

const registry = useRegistryStore();
const builds = useBuildStore();
const ui = useUiStore();
const chatHistory = useChatHistoryStore();
const toasts = useToastsStore();
const compact = ref(false);
const addingOpen = ref(false);
const addPreview = ref<RegistryEntryDto | null>(null);
const addError = ref<string | null>(null);
const addBusy = ref(false);
const pathInput = ref('');

onMounted(() => {
  registry.load();
});

const entries = computed(() => registry.entries);
const hasEntries = computed(() => entries.value.length > 0);

/** Resolve a PoB path from the freeform text field or from the active
 *  build, then preview-parse it via `registry_preview` (no persistence).
 *  Persistence happens in `commitAdd` only — clicking Cancel discards the
 *  preview cleanly. A native file dialog plugin is wired up in a follow-up
 *  sprint; for now the user pastes a path or attaches via the BuildPicker. */
async function pickFile() {
  addError.value = null;
  const path = pathInput.value.trim() || builds.current?.source_file || '';
  if (!path) {
    addError.value = 'Paste the absolute path to a PoB XML file, or attach a PoB in the chat first.';
    return;
  }
  try {
    addBusy.value = true;
    addPreview.value = await registryPreview(path);
  } catch (e: unknown) {
    addError.value = e instanceof Error ? e.message : String(e);
  } finally {
    addBusy.value = false;
  }
}

function openAdd() {
  addPreview.value = null;
  addError.value = null;
  addingOpen.value = true;
}

function closeAdd() {
  addingOpen.value = false;
  addPreview.value = null;
  addError.value = null;
}

async function commitAdd(openInChat: boolean) {
  const preview = addPreview.value;
  if (!preview) {
    closeAdd();
    return;
  }
  try {
    addBusy.value = true;
    // Persist for real now — the preview only ran the parser, never wrote.
    const saved = await registry.add(preview.pob_path);
    closeAdd();
    if (openInChat) {
      chatHistory.startNew();
      await builds.setActive(saved.pob_path);
      void registry.touch(saved.id);
      ui.close();
    }
  } catch (e) {
    addError.value = e instanceof Error ? e.message : String(e);
  } finally {
    addBusy.value = false;
  }
}

async function onOpenEntry(entry: RegistryEntryDto) {
  // RegistryRow's @open action: start a fresh chat and attach this
  // build to it. The user lands back on the chat view ready to ask.
  try {
    chatHistory.startNew();
    await builds.setActive(entry.pob_path);
    void registry.touch(entry.id);
    ui.close();
  } catch (e) {
    toasts.push({
      variant: 'error',
      title: 'Could not open chat with this build',
      body: e instanceof Error ? e.message : String(e),
    });
  }
}

async function onMenuEntry(entry: RegistryEntryDto) {
  // Minimal context menu — confirm + remove. A richer menu (refresh,
  // rename) can land later via a popover; the destructive action is the
  // one that needs explicit confirmation.
  const ok = window.confirm(
    `Remove "${entry.display_name}" from your Active Builds registry?\n\nThe PoB file at ${entry.pob_path} is untouched. Any linked Build Sheet stays in your library.`,
  );
  if (!ok) return;
  try {
    await registry.remove(entry.id);
  } catch (e) {
    toasts.push({
      variant: 'error',
      title: 'Could not remove build',
      body: e instanceof Error ? e.message : String(e),
    });
  }
}

async function onAuthorSheet(entry: RegistryEntryDto) {
  // RegistryRow's @author-sheet action (rendered when linked_sheet_id is
  // null): open a new chat with the build attached, then send the
  // canonical "create build sheet" prompt so the agent enters Path B.
  try {
    chatHistory.startNew();
    await builds.setActive(entry.pob_path);
    void registry.touch(entry.id);
    ui.close();
    // The send below races with the chat view mounting — chat.send is
    // safe to call any time, the message queues correctly.
    const { useChatStore } = await import('../../stores/chat');
    const chat = useChatStore();
    await chat.send(
      "I'd like a full audit — please open the build sheet interview so we can capture this character's intent and defining items.",
    );
  } catch (e) {
    toasts.push({
      variant: 'error',
      title: 'Could not start sheet authoring',
      body: e instanceof Error ? e.message : String(e),
    });
  }
}

function onOpenSheet(_entry: RegistryEntryDto) {
  // RegistryRow's @open-sheet action: open the linked sheet detail
  // modal. The modal reads from the active sheet store, so we attach
  // the build first then open the modal — the build store's
  // rehydrate hook will populate the sheet store within a tick.
  ui.openSheetModal();
}
</script>

<template>
  <div class="active-builds">
    <header class="active-builds__header">
      <div class="active-builds__heading">
        <h2 class="active-builds__title">Active Builds</h2>
        <p class="active-builds__subtitle">
          <template v-if="hasEntries">
            {{ entries.length }} builds registered · last refreshed just now.
          </template>
          <template v-else>
            A persistent registry of builds you care about. Pick a PoB once, and Bestel can recall the
            build's identity, hash, and any associated sheet across chats and days.
          </template>
        </p>
      </div>
      <div class="active-builds__actions">
        <RunicButton variant="primary" icon="plus" kbd="⌘B" @click="openAdd">Add a build</RunicButton>
      </div>
    </header>

    <div v-if="hasEntries" class="active-builds__toolbar">
      <span class="active-builds__density-label">Density</span>
      <button
        type="button"
        class="active-builds__density-btn"
        :class="{ 'active-builds__density-btn--active': !compact }"
        @click="compact = false"
      >Comfortable</button>
      <button
        type="button"
        class="active-builds__density-btn"
        :class="{ 'active-builds__density-btn--active': compact }"
        @click="compact = true"
      >Compact</button>
      <span class="active-builds__spacer" />
      <span class="active-builds__sort">Sort: <strong>recently used</strong></span>
    </div>

    <div v-if="hasEntries" class="active-builds__list" :class="{ 'active-builds__list--compact': compact }">
      <RegistryRow
        v-for="e in entries"
        :key="e.id"
        :entry="e"
        :compact="compact"
        @menu="onMenuEntry(e)"
        @open="onOpenEntry(e)"
        @author-sheet="onAuthorSheet(e)"
        @open-sheet="onOpenSheet(e)"
      />
    </div>

    <div v-else class="active-builds__empty">
      <div class="active-builds__empty-icon">
        <RunicIcon name="plus" :size="24" />
      </div>
      <div class="active-builds__empty-title">No builds registered yet</div>
      <div class="active-builds__empty-text">
        Pick a Path of Building XML file to add it to your registry. Bestel will compute its signatures
        and keep it ready so future chats can pick the same character back up without re-importing.
      </div>
      <div class="active-builds__empty-actions">
        <RunicButton variant="primary" icon="plus" @click="pickFile">Browse for a PoB…</RunicButton>
      </div>
    </div>

    <AddBuildModal
      v-if="addingOpen"
      :preview="addPreview"
      :error="addError"
      :busy="addBusy"
      @close="closeAdd"
      @pick-file="pickFile"
      @save="commitAdd"
    />
  </div>
</template>

<style scoped>
.active-builds {
  display: flex;
  flex-direction: column;
  gap: 18px;
  height: 100%;
}
.active-builds__header {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--paper-line);
}
.active-builds__heading {
  flex: 1;
}
.active-builds__title {
  font-size: 24px;
  font-weight: 700;
  margin: 0;
}
.active-builds__subtitle {
  font-size: 14px;
  color: var(--ink-soft);
  margin: 4px 0 0;
  max-width: 640px;
}
.active-builds__toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 13px;
  color: var(--ink-soft);
}
.active-builds__density-label {
  font-family: var(--label);
  font-size: 12px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 600;
}
.active-builds__density-btn {
  padding: 3px 10px;
  border-radius: 3px;
  border: 1px solid var(--paper-line);
  background: var(--paper);
  font-size: 13px;
  font-family: var(--hand);
  color: var(--ink);
  cursor: pointer;
}
.active-builds__density-btn--active {
  border-color: var(--ink-soft);
  background: var(--paper-shade);
  font-weight: 600;
}
.active-builds__spacer {
  flex: 1;
}
.active-builds__sort strong {
  color: var(--ink);
  font-weight: 600;
}
.active-builds__list {
  flex: 1;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.active-builds__list--compact {
  gap: 6px;
}
.active-builds__empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  gap: 16px;
  padding: 32px;
}
.active-builds__empty-icon {
  width: 64px;
  height: 64px;
  border: 1.4px dashed var(--ink-faint);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--ink-faint);
}
.active-builds__empty-title {
  font-size: 22px;
  font-weight: 700;
}
.active-builds__empty-text {
  font-size: 15px;
  color: var(--ink-soft);
  max-width: 520px;
  line-height: 1.55;
}
.active-builds__empty-actions {
  display: flex;
  gap: 10px;
}
</style>

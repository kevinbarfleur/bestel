<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import { useBuildStore } from '../../stores/build';
import { useChatStore } from '../../stores/chat';
import { useRegistryStore } from '../../stores/registry';
import { useToastsStore } from '../../stores/toasts';
import { useUiStore } from '../../stores/ui';
import { registryPreview } from '../../api/tauri';
import type { PobBuildSummaryDto, RegistryEntryDto } from '../../api/types';
import RunicButton from '../runic/RunicButton.vue';
import RunicIcon from '../runic/RunicIcon.vue';
import FreshPill from '../sheet/FreshPill.vue';

/**
 * Dedicated dialog for everything build-attachment-related. Replaces the
 * old multi-tab BuildPicker for the registry-driven world: there's no
 * such thing as an "ad-hoc" attach anymore — every PoB you talk to with
 * Bestel goes through your library.
 *
 * Two sections:
 *   1. **Library** — builds the user has saved to their registry. Click a
 *      row to switch the active build (and bump it in the recently-used
 *      ordering). Each row carries a drift chip strip + actions.
 *   2. **Detected on disk** — Path of Building XML files the watcher
 *      surfaced from your scanned folders that are NOT yet in your
 *      library. Click + Add to register and immediately attach.
 *
 * A custom-path text field at the bottom covers the niche case where the
 * file lives outside your watcher folders.
 */
const ui = useUiStore();
const buildStore = useBuildStore();
const chat = useChatStore();
const registry = useRegistryStore();
const toasts = useToastsStore();

const { activeBuild: currentBuild } = storeToRefs(chat);
const { list: discovered, loadingList } = storeToRefs(buildStore);

const search = ref('');
const busy = ref(false);
const customPath = ref('');
const customError = ref<string | null>(null);
const previewedEntry = ref<RegistryEntryDto | null>(null);

onMounted(async () => {
  // Refresh both lists when the modal opens so the user sees the freshest
  // state (a recent registry change in another window, a new file landing
  // in a watcher folder, etc.).
  await Promise.all([registry.load(true), buildStore.refreshList()]);
});

watch(
  () => ui.registryModalOpen,
  async (open) => {
    if (!open) return;
    await Promise.all([registry.load(true), buildStore.refreshList()]);
  },
);

const filteredEntries = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return registry.entries;
  return registry.entries.filter((e) => {
    const hay = [
      e.display_name,
      e.summary.class,
      e.summary.ascendancy ?? '',
      e.summary.main_skill ?? '',
      e.pob_path,
      e.game,
      e.summary.defining_uniques.join(' '),
    ]
      .join(' ')
      .toLowerCase();
    return hay.includes(q);
  });
});

/** Watcher rows whose path is NOT already in the registry. The registry is
 *  authoritative for "I care about this build" — once a file is registered
 *  we stop offering it as a discovery candidate. */
const registeredPaths = computed(() => new Set(registry.entries.map((e) => e.pob_path)));
const discoveredNew = computed(() => {
  return discovered.value.filter((d) => !registeredPaths.value.has(d.path));
});

const filteredDiscovered = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return discoveredNew.value;
  return discoveredNew.value.filter((d) => {
    const hay = [
      d.header,
      d.file_name,
      d.class,
      d.ascendancy ?? '',
      d.main_skill_hint ?? '',
      d.game,
    ]
      .join(' ')
      .toLowerCase();
    return hay.includes(q);
  });
});

const activeEntryId = computed<number | null>(() => {
  const path = currentBuild.value?.source_file;
  if (!path) return null;
  const m = registry.entries.find((e) => e.pob_path === path);
  return m?.id ?? null;
});

function close() {
  ui.closeRegistryModal();
  customPath.value = '';
  customError.value = null;
  previewedEntry.value = null;
}

const FIRST_ATTACH_KEY = 'bestel.firstAttachBannerShown.v1';

/** True the first time the user successfully attaches any build. Used to
 *  fire the cross-chat-continuity nudge exactly once per install — the
 *  promise is invisible until we point at it.
 */
function consumeFirstAttachFlag(): boolean {
  try {
    if (localStorage.getItem(FIRST_ATTACH_KEY) === '1') return false;
    localStorage.setItem(FIRST_ATTACH_KEY, '1');
    return true;
  } catch {
    return false;
  }
}

async function attachEntry(entry: RegistryEntryDto) {
  // Non-destructive guard: if the active chat already has a different
  // build AND there are messages in it, ask the user whether they want
  // to replace the chat's build, open a new chat, or cancel. Silent
  // attach is only safe when the chat is empty (nothing to lose).
  const hasOtherBuild =
    chat.activeBuild !== null &&
    chat.activeBuild.source_file !== entry.pob_path;
  const hasMessages = chat.messages.length > 0;
  if (hasOtherBuild && hasMessages) {
    const choice = window.confirm(
      `This chat is currently using "${chat.activeBuild?.main_skill ?? chat.activeBuild?.class ?? 'another build'}".\n\n` +
        `OK — Replace it in this chat with "${entry.display_name}".\n` +
        'Cancel — Keep going. (Pick "Open in new chat" instead to use it side-by-side.)',
    );
    if (!choice) return;
  }

  busy.value = true;
  try {
    const dto = await chat.attachBuild(entry.pob_path);
    if (!dto) {
      toasts.push({ variant: 'error', title: "Couldn't load that PoB file", body: entry.pob_path });
      return;
    }
    void registry.touch(entry.id);
    const firstTime = consumeFirstAttachFlag();
    if (firstTime && !entry.linked_sheet_id) {
      toasts.push({
        variant: 'success',
        title: `Now using ${entry.display_name}`,
        body: 'This build follows you across chats. Start the interview to give it a Build Sheet.',
      });
    } else {
      toasts.push({ variant: 'success', title: `Now using ${entry.display_name}` });
    }
    close();
  } finally {
    busy.value = false;
  }
}

async function attachInNewChat(entry: RegistryEntryDto) {
  busy.value = true;
  try {
    // Reset the chat in-place so messages are wiped and a new chat
    // record is started on the next message. `chat.reset()` does the
    // mark-history-new dance for us.
    await chat.reset();
    const dto = await chat.attachBuild(entry.pob_path);
    if (!dto) {
      toasts.push({ variant: 'error', title: "Couldn't load that PoB file" });
      return;
    }
    void registry.touch(entry.id);
    close();
  } finally {
    busy.value = false;
  }
}

async function viewSheet(entry: RegistryEntryDto) {
  // The sheet store fetches from BuildContext, so we must make this
  // build the active one first. Then the user can read the existing
  // sheet — which is exactly what they're asking for. If they want a
  // different chat, the standard "Open in new chat" path is still there.
  busy.value = true;
  try {
    const dto = await chat.attachBuild(entry.pob_path);
    if (!dto) {
      toasts.push({ variant: 'error', title: "Couldn't open that Build Sheet", body: entry.pob_path });
      return;
    }
    void registry.touch(entry.id);
    close();
    ui.openSheetModal();
  } finally {
    busy.value = false;
  }
}

async function registerDiscovered(item: PobBuildSummaryDto) {
  busy.value = true;
  try {
    const entry = await registry.add(item.path);
    const dto = await chat.attachBuild(entry.pob_path);
    if (dto) {
      toasts.push({
        variant: 'success',
        title: 'Saved to library and using now',
        body: entry.display_name,
      });
    }
    close();
  } catch (e) {
    toasts.push({
      variant: 'error',
      title: "Couldn't save to library",
      body: e instanceof Error ? e.message : String(e),
    });
  } finally {
    busy.value = false;
  }
}

async function removeEntry(entry: RegistryEntryDto) {
  if (entry.pob_path === currentBuild.value?.source_file) {
    const ok = window.confirm(
      `"${entry.display_name}" is the currently attached build. Remove it from your library AND detach it from the chat?`,
    );
    if (!ok) return;
  } else {
    const ok = window.confirm(
      `Remove "${entry.display_name}" from your library?\n\nThe PoB file at ${entry.pob_path} is untouched. Any linked Build Sheet stays in your library.`,
    );
    if (!ok) return;
  }
  try {
    await registry.remove(entry.id);
    if (entry.pob_path === currentBuild.value?.source_file) {
      await chat.detachBuild();
    }
  } catch (e) {
    toasts.push({
      variant: 'error',
      title: 'Could not remove build',
      body: e instanceof Error ? e.message : String(e),
    });
  }
}

async function previewCustomPath() {
  customError.value = null;
  const p = customPath.value.trim();
  if (!p) {
    customError.value = 'Paste the absolute path to a Path of Building XML file.';
    return;
  }
  busy.value = true;
  try {
    previewedEntry.value = await registryPreview(p);
  } catch (e) {
    customError.value = e instanceof Error ? e.message : String(e);
    previewedEntry.value = null;
  } finally {
    busy.value = false;
  }
}

async function commitCustomPath() {
  if (!previewedEntry.value) return;
  busy.value = true;
  try {
    const entry = await registry.add(previewedEntry.value.pob_path);
    const dto = await chat.attachBuild(entry.pob_path);
    if (dto) {
      toasts.push({
        variant: 'success',
        title: 'Saved to library and using now',
        body: entry.display_name,
      });
    }
    close();
  } catch (e) {
    customError.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

function clearAttached() {
  void chat.detachBuild();
  close();
}
</script>

<template>
  <div v-if="ui.registryModalOpen" class="brm" role="dialog" aria-modal="true" aria-label="Build library">
    <div class="brm__backdrop" @click="close" />
    <div class="brm__panel">
      <header class="brm__header">
        <div class="brm__heading">
          <div class="brm__eyebrow">build library</div>
          <h2 class="brm__title">Your build library</h2>
          <p class="brm__subtitle">
            Bestel only talks about builds you've saved here. Pick one to use in
            this chat — the Build Sheet for that build (Bestel's notes on the
            character) follows you across every future chat.
          </p>
        </div>
        <button class="brm__close" type="button" aria-label="Close" @click="close">
          <RunicIcon name="close" :size="14" />
        </button>
      </header>

      <div class="brm__search">
        <RunicIcon name="search" :size="14" />
        <input
          v-model="search"
          type="search"
          placeholder="Search saved and detected builds…"
          class="brm__search-input"
        />
        <RunicButton
          v-if="currentBuild"
          variant="secondary"
          @click="clearAttached"
        >Detach from this chat</RunicButton>
      </div>

      <div class="brm__body">
        <!-- ─── LIBRARY (registry entries) ────────────────────────────── -->
        <section class="brm__section">
          <div class="brm__section-head">
            <span class="brm__section-label">saved builds</span>
            <span class="brm__section-rule" />
            <span class="brm__section-count">{{ registry.entries.length }}</span>
          </div>
          <div v-if="registry.entries.length === 0" class="brm__empty">
            Nothing here yet. Save a detected build below to get started.
          </div>
          <div v-else-if="filteredEntries.length === 0" class="brm__empty">
            No saved build matches your search.
          </div>
          <div v-else class="brm__rows">
            <article
              v-for="e in filteredEntries"
              :key="`r-${e.id}`"
              class="brm__row"
              :class="{ 'brm__row--active': e.id === activeEntryId }"
            >
              <span class="brm__game">{{ e.game }}</span>
              <div class="brm__row-meta">
                <div class="brm__row-title">
                  {{ e.display_name }}
                  <span v-if="e.id === activeEntryId" class="brm__active-pill">using in this chat</span>
                </div>
                <div class="brm__row-sub">
                  {{ e.summary.class }} · {{ e.summary.ascendancy ?? '—' }}
                  <template v-if="e.summary.level"> · lvl {{ e.summary.level }}</template>
                  <template v-if="e.summary.main_skill"> · {{ e.summary.main_skill }}</template>
                </div>
                <div v-if="e.summary.defining_uniques.length" class="brm__row-uniques">
                  {{ e.summary.defining_uniques.slice(0, 4).join(' · ') }}
                </div>
              </div>
              <div class="brm__row-status">
                <span v-if="registry.missingIds.has(e.id)" class="brm__row-missing">
                  file missing
                </span>
                <template v-else>
                  <FreshPill v-if="e.linked_sheet_id" dense />
                  <span v-else class="brm__row-nosheet">no Build Sheet yet</span>
                </template>
              </div>
              <div class="brm__row-actions">
                <button
                  type="button"
                  class="brm__btn brm__btn--primary"
                  :disabled="busy || e.id === activeEntryId || registry.missingIds.has(e.id)"
                  :title="registry.missingIds.has(e.id)
                    ? `The PoB file at ${e.pob_path} can't be found. Re-export it from Path of Building, or remove this entry.`
                    : undefined"
                  @click="attachEntry(e)"
                >
                  {{ e.id === activeEntryId ? 'In use' : 'Use in this chat' }}
                </button>
                <button
                  v-if="!registry.missingIds.has(e.id)"
                  type="button"
                  class="brm__btn brm__btn--ghost"
                  :disabled="busy"
                  title="Open in a new chat"
                  @click="attachInNewChat(e)"
                >
                  Open in new chat
                </button>
                <button
                  v-if="e.linked_sheet_id && !registry.missingIds.has(e.id)"
                  type="button"
                  class="brm__btn brm__btn--ghost"
                  :disabled="busy"
                  title="View this build's Build Sheet"
                  @click="viewSheet(e)"
                >
                  View sheet
                </button>
                <button
                  type="button"
                  class="brm__btn brm__btn--icon"
                  :disabled="busy"
                  aria-label="Remove from library"
                  @click="removeEntry(e)"
                >
                  <RunicIcon name="trash" :size="13" />
                </button>
              </div>
            </article>
          </div>
        </section>

        <!-- ─── DETECTED on disk (not yet in registry) ────────────────── -->
        <section class="brm__section">
          <div class="brm__section-head">
            <span class="brm__section-label">detected in your watcher folders</span>
            <span class="brm__section-rule" />
            <span v-if="loadingList" class="brm__section-count">scanning…</span>
            <span v-else class="brm__section-count">{{ discoveredNew.length }}</span>
          </div>
          <div v-if="loadingList && discoveredNew.length === 0" class="brm__empty">
            Scanning your watcher folders for PoB files…
          </div>
          <div v-else-if="discoveredNew.length === 0" class="brm__empty">
            Bestel didn't find any new PoB files. Add a watcher folder in Settings,
            or paste a path below.
          </div>
          <div v-else-if="filteredDiscovered.length === 0" class="brm__empty">
            No detected build matches your search.
          </div>
          <div v-else class="brm__rows">
            <article
              v-for="d in filteredDiscovered"
              :key="`d-${d.path}`"
              class="brm__row brm__row--discovered"
            >
              <span class="brm__game">{{ d.game === 'poe1' ? 'PoE1' : 'PoE2' }}</span>
              <div class="brm__row-meta">
                <div class="brm__row-title">{{ d.file_name }}</div>
                <div class="brm__row-sub">
                  {{ d.class }}<template v-if="d.ascendancy"> · {{ d.ascendancy }}</template>
                  <template v-if="d.level"> · lvl {{ d.level }}</template>
                  <template v-if="d.main_skill_hint"> · {{ d.main_skill_hint }}</template>
                </div>
                <div class="brm__row-path">{{ d.path }}</div>
              </div>
              <div class="brm__row-actions">
                <button
                  type="button"
                  class="brm__btn brm__btn--primary"
                  :disabled="busy"
                  @click="registerDiscovered(d)"
                >
                  Save &amp; use in this chat
                </button>
              </div>
            </article>
          </div>
        </section>

        <!-- ─── CUSTOM path (paste an XML outside your watcher folders) ─── -->
        <section class="brm__section">
          <div class="brm__section-head">
            <span class="brm__section-label">add a file by path</span>
            <span class="brm__section-rule" />
          </div>
          <div class="brm__custom">
            <input
              v-model="customPath"
              type="text"
              spellcheck="false"
              placeholder="C:\Users\…\PathOfBuilding\Builds\My Character.xml"
              class="brm__custom-input"
              @keydown.enter="previewCustomPath"
            />
            <button
              type="button"
              class="brm__btn brm__btn--primary"
              :disabled="busy || !customPath.trim()"
              @click="previewCustomPath"
            >Preview</button>
          </div>
          <div v-if="customError" class="brm__error">{{ customError }}</div>
          <div v-if="previewedEntry" class="brm__preview">
            <div class="brm__row brm__row--preview">
              <span class="brm__game">{{ previewedEntry.game }}</span>
              <div class="brm__row-meta">
                <div class="brm__row-title">{{ previewedEntry.display_name }}</div>
                <div class="brm__row-sub">
                  {{ previewedEntry.summary.class }} ·
                  {{ previewedEntry.summary.ascendancy ?? '—' }}
                  <template v-if="previewedEntry.summary.level">
                    · lvl {{ previewedEntry.summary.level }}
                  </template>
                </div>
                <div v-if="previewedEntry.summary.defining_uniques.length" class="brm__row-uniques">
                  {{ previewedEntry.summary.defining_uniques.slice(0, 4).join(' · ') }}
                </div>
              </div>
              <div class="brm__row-actions">
                <button
                  type="button"
                  class="brm__btn brm__btn--primary"
                  :disabled="busy"
                  @click="commitCustomPath"
                >Save to library &amp; use</button>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<style scoped>
.brm {
  position: fixed;
  inset: 0;
  z-index: 110;
  font-family: var(--hand);
  color: var(--ink);
  display: flex;
  align-items: center;
  justify-content: center;
}
.brm__backdrop {
  position: absolute;
  inset: 0;
  background: rgba(60, 55, 50, 0.35);
}
.theme-dark .brm__backdrop {
  background: rgba(8, 6, 4, 0.55);
}
.brm__panel {
  position: relative;
  width: min(940px, calc(100% - 48px));
  max-height: calc(100% - 64px);
  background: var(--paper);
  border-radius: 6px;
  display: grid;
  grid-template-rows: auto auto 1fr;
  overflow: hidden;
  box-shadow: 0 14px 30px rgba(60, 40, 20, 0.18);
}
.theme-dark .brm__panel {
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.5);
}
.brm__header {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 18px 24px 14px;
  border-bottom: 1px solid var(--paper-line);
}
.brm__heading { flex: 1; }
.brm__eyebrow {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 600;
}
.brm__title {
  font-size: 22px;
  font-weight: 700;
  margin: 4px 0 0;
}
.brm__subtitle {
  font-size: 13.5px;
  color: var(--ink-soft);
  margin: 4px 0 0;
  max-width: 620px;
}
.brm__close {
  width: 30px;
  height: 30px;
  padding: 0;
  background: transparent;
  color: var(--ink-soft);
  border: 1px solid transparent;
  border-radius: 4px;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.brm__close:hover { background: var(--paper-shade); }
.brm__search {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 24px;
  border-bottom: 1px solid var(--paper-line);
  background: var(--paper);
}
.brm__search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink);
}
.brm__search-input::placeholder { color: var(--ink-faint); }
.brm__body {
  overflow: auto;
  padding: 16px 24px 24px;
  display: flex;
  flex-direction: column;
  gap: 22px;
}
.brm__section { display: flex; flex-direction: column; gap: 10px; }
.brm__section-head {
  display: flex;
  align-items: baseline;
  gap: 8px;
}
.brm__section-label {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 600;
}
.brm__section-rule { flex: 1; height: 1px; background: var(--paper-line); }
.brm__section-count {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  color: var(--ink-faint);
}
.brm__empty {
  padding: 12px 16px;
  font-size: 13.5px;
  color: var(--ink-faint);
  background: var(--paper-shade);
  border-radius: 4px;
  font-style: italic;
}
.brm__rows { display: flex; flex-direction: column; gap: 8px; }
.brm__row {
  display: grid;
  grid-template-columns: auto 1fr auto auto;
  align-items: center;
  gap: 14px;
  padding: 10px 14px;
  border: 1px solid var(--paper-line);
  border-radius: 5px;
  background: var(--paper);
}
.brm__row--active {
  border-color: var(--amber);
  background: var(--amber-glow);
}
.brm__row--discovered {
  grid-template-columns: auto 1fr auto;
}
.brm__row--preview {
  grid-template-columns: auto 1fr auto;
  border-color: var(--ink-soft);
  background: var(--paper-shade);
}
.brm__game {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: var(--ink-faint);
  padding: 2px 6px;
  border: 1px solid var(--paper-line);
  border-radius: 3px;
  flex: none;
}
.brm__row-meta { min-width: 0; }
.brm__row-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: flex;
  align-items: center;
  gap: 8px;
}
.brm__active-pill {
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--amber);
  padding: 1px 6px;
  border: 1px solid var(--amber);
  border-radius: 10px;
  font-weight: 700;
}
.brm__row-sub {
  font-size: 13px;
  color: var(--ink-soft);
  margin-top: 2px;
}
.brm__row-uniques {
  font-size: 12.5px;
  color: var(--ink-soft);
  margin-top: 2px;
}
.brm__row-path {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: var(--ink-faint);
  margin-top: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.brm__row-status {
  display: flex;
  align-items: center;
  justify-content: flex-end;
}
.brm__row-nosheet {
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink-faint);
}
.brm__row-missing {
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  font-weight: 700;
  color: var(--bad);
  padding: 2px 7px;
  border: 1px solid var(--bad);
  border-radius: 3px;
  background: rgba(167, 73, 67, 0.08);
  white-space: nowrap;
}
.brm__row-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}
.brm__btn {
  font-family: var(--hand);
  font-size: 12.5px;
  cursor: pointer;
  border-radius: 4px;
  padding: 5px 11px;
  border: 1px solid var(--ink-soft);
  background: var(--paper);
  color: var(--ink);
}
.brm__btn:hover:not(:disabled) {
  border-color: var(--amber);
  background: var(--amber-glow);
}
.brm__btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.brm__btn--primary {
  background: var(--ink);
  color: var(--paper);
  border-color: var(--ink);
  font-weight: 600;
}
.brm__btn--primary:hover:not(:disabled) {
  background: var(--amber);
  border-color: var(--amber);
}
.brm__btn--ghost {
  border: 1px dashed var(--ink-faint);
  background: transparent;
  font-size: 12px;
  color: var(--ink-soft);
}
.brm__btn--icon {
  width: 28px;
  height: 28px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid transparent;
  background: transparent;
  color: var(--ink-soft);
}
.brm__btn--icon:hover:not(:disabled) {
  background: rgba(164, 72, 72, 0.1);
  color: var(--bad);
  border-color: var(--bad);
}
.brm__custom {
  display: flex;
  gap: 8px;
  align-items: center;
}
.brm__custom-input {
  flex: 1;
  padding: 7px 12px;
  border: 1px solid var(--paper-line);
  border-radius: 4px;
  background: var(--paper);
  color: var(--ink);
  font-family: 'JetBrains Mono', monospace;
  font-size: 12.5px;
}
.brm__custom-input:focus {
  outline: none;
  border-color: var(--ink-soft);
}
.brm__error {
  font-size: 13px;
  color: var(--bad);
  background: rgba(164, 72, 72, 0.08);
  border: 1px solid var(--bad);
  border-radius: 4px;
  padding: 8px 12px;
}
.brm__preview { margin-top: 4px; }
</style>

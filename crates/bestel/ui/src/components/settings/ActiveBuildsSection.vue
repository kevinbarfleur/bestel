<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { useRegistryStore } from '../../stores/registry';
import { useBuildStore } from '../../stores/build';
import RegistryRow from '../registry/RegistryRow.vue';
import AddBuildModal from '../registry/AddBuildModal.vue';
import RunicButton from '../runic/RunicButton.vue';
import RunicIcon from '../runic/RunicIcon.vue';
import type { RegistryEntryDto } from '../../api/types';

const registry = useRegistryStore();
const builds = useBuildStore();
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

/** Add a build by path. For Sprint v3 v1 the path is sourced two ways:
 *  - if the user already has an active PoB attached, fall back to that path
 *  - else read from the freeform `pathInput` text field in the modal
 *  A native file dialog plugin is wired up in a follow-up sprint. */
async function pickFile() {
  addError.value = null;
  const path = pathInput.value.trim() || builds.current?.source_file || '';
  if (!path) {
    addError.value = 'Paste the absolute path to a PoB XML file, or attach a PoB in the chat first.';
    return;
  }
  try {
    addBusy.value = true;
    const entry = await registry.add(path);
    addPreview.value = entry;
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

function commitAdd(_openInChat: boolean) {
  // The entry is already persisted via the registry.add() call inside
  // pickFile — the modal only confirms the user wants to keep it.
  // `_openInChat` is plumbed for a future Sprint 6 hook that attaches
  // the build to the current chat right after save; deferred for now.
  closeAdd();
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
        @menu="() => {}"
        @open="() => {}"
        @author-sheet="() => {}"
        @open-sheet="() => {}"
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

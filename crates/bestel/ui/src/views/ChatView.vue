<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useRouter } from 'vue-router';

import { useStreaming } from '../composables/useStreaming';
import { useBuildWatcher } from '../composables/useBuildWatcher';
import { useShortcuts } from '../composables/useShortcuts';
import { useChatStore } from '../stores/chat';
import { useChatHistoryStore } from '../stores/chatHistory';
import { useSettingsStore } from '../stores/settings';
import { useUiStore } from '../stores/ui';

import BuildPanel from '../components/build/BuildPanel.vue';
import BSInviteSidebarCard from '../components/build-sheet/BSInviteSidebarCard.vue';
import BSInterviewSidebarCard from '../components/build-sheet/BSInterviewSidebarCard.vue';
import BSLinkedSheetCard from '../components/build-sheet/BSLinkedSheetCard.vue';
import BSSheetFullModal from '../components/build-sheet/BSSheetFullModal.vue';
import ChatStream from '../components/chat/ChatStream.vue';
import ChatComposer from '../components/chat/ChatComposer.vue';
import ChatEmptyHero from '../components/chat/ChatEmptyHero.vue';
import PanelView from '../components/panel/PanelView.vue';
import DriftDrawer from '../components/sheet/DriftDrawer.vue';
import BuildRegistryModal from '../components/registry/BuildRegistryModal.vue';
import { useSheetStore } from '../stores/sheet';

const chat = useChatStore();
const chatHistory = useChatHistoryStore();
const settings = useSettingsStore();
const sheetStore = useSheetStore();
const ui = useUiStore();
const router = useRouter();

const { activeBuild: current } = storeToRefs(chat);
const { panelArtifact, driftDrawerAxis } = storeToRefs(ui);

const driftDrawerAuthoredAt = computed(() => {
  const s = sheetStore.activeSheet;
  if (!s) return '';
  // Best-effort YYYY-MM-DD slice; falls back to the full ISO string.
  return s.authoredAt.slice(0, 10);
});

const REAUTHOR_MESSAGE =
  'Please refresh the build sheet — re-run the interview from scratch so I can update the signatures that drifted since the previous authoring.';

async function onDrawerReauthor() {
  ui.closeDriftDrawer();
  await chat.send(REAUTHOR_MESSAGE);
}

function onDrawerKeep() {
  ui.closeDriftDrawer();
}

const sidebarCollapsed = computed(() => current.value === null);
const panelOpen = computed(() => panelArtifact.value !== null);
/** Hero on a blank canvas: only when the chat has no build AND no
 *  messages. The composer would otherwise float over an empty page
 *  with no signposting on first launch. */
const showEmptyHero = computed(
  () => current.value === null && chat.messages.length === 0,
);

useStreaming();
useBuildWatcher();

useShortcuts({
  onBuildPicker: () => ui.openRegistryModal(),
  onModelPicker: () => ui.openModel(),
  onSettings: () => ui.openSettings(),
  onEscape: () => {
    if (ui.linkViewerUrl) ui.closeLinkViewer();
    else if (ui.sheetModalOpen) ui.closeSheetModal();
    else if (ui.picker) ui.close();
    else if (panelArtifact.value) ui.closePanel();
    else if (chat.isStreaming) void chat.cancel();
  },
  onResetChat: () => {
    void chat.reset();
  },
  onOpenDebug: () => {
    void router.push('/debug');
  },
});

onMounted(async () => {
  await Promise.all([
    settings.refreshModels(),
    settings.refreshDetection(),
    settings.refreshKeys(),
    settings.refreshRuntimeSettings(),
  ]);
  // Restore the last active chat if any. `loadFromSaved` re-attaches
  // the saved build to the chat (best-effort; missing file is silent),
  // so no manual reconciliation is needed here anymore.
  const saved = chatHistory.findActive();
  if (saved && saved.messages.length > 0) {
    await chat.loadFromSaved(saved);
  }
});
</script>

<template>
  <div
    class="chat-view sketch-bg"
    :class="{
      'chat-view--collapsed': sidebarCollapsed,
      'chat-view--panel-open': panelOpen,
    }"
  >
    <div class="chat-view__sidebar">
      <div class="chat-view__sidebar-stack">
        <BuildPanel @open-picker="ui.openRegistryModal()">
          <template #after-header>
            <BSInviteSidebarCard />
            <BSInterviewSidebarCard />
            <BSLinkedSheetCard @view-full="ui.openSheetModal()" />
          </template>
        </BuildPanel>
      </div>
    </div>

    <div class="chat-view__main">
      <ChatEmptyHero v-if="showEmptyHero" />
      <ChatStream v-else class="chat-view__stream" />
      <ChatComposer />
    </div>

    <div v-if="panelOpen" class="chat-view__panel">
      <PanelView />
    </div>

    <BSSheetFullModal />

    <!-- Sprint v3 — dedicated build library modal. Replaces the legacy
         BuildPicker as the sole entry point for build attachment. -->
    <BuildRegistryModal />

    <!-- Sprint v3 — slide-from-right drift drawer. Mounts globally so a
         chip click anywhere in the sidebar / sheet detail can open it. -->
    <Transition name="drift-drawer">
      <div v-if="driftDrawerAxis" class="chat-view__drift-drawer">
        <DriftDrawer
          :kind="driftDrawerAxis"
          :authored-at="driftDrawerAuthoredAt"
          @close="ui.closeDriftDrawer()"
          @reauthor="onDrawerReauthor"
          @keep="onDrawerKeep"
        />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.chat-view__drift-drawer {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  z-index: 90;
  display: flex;
}
.drift-drawer-enter-active,
.drift-drawer-leave-active {
  transition: transform 0.22s ease-out;
}
.drift-drawer-enter-from,
.drift-drawer-leave-to {
  transform: translateX(100%);
}

.chat-view {
  --sidebar-w: 360px;
  --panel-w: 0px;
  flex: 1;
  display: flex;
  min-height: 0;
  min-width: 0;
  /* Hard-clamp to the viewport so child flex math NEVER overflows the
   * window. The outer flex chain (`app-shell` → `app-body` → `chat-view`)
   * has historically allowed children to push the chat past the window
   * edge on certain content widths; a viewport-anchored max-width
   * sidesteps every flex-sizing quirk. */
  max-width: 100vw;
  overflow: hidden;
  position: relative;
}
.chat-view--collapsed {
  --sidebar-w: 0px;
}
.chat-view--panel-open {
  --panel-w: 380px;
}

.chat-view__sidebar {
  width: var(--sidebar-w);
  flex: none;
  display: flex;
  min-height: 0;
  overflow: hidden;
  transition: width 0.28s ease;
}

.chat-view__sidebar-stack {
  width: 360px;
  flex: 0 0 360px;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-y: auto;
}

/* BuildPanel ships with its own 360px declaration; honor it inside the
 * stack and let BSLinkedSheetCard inherit the same rail. */
.chat-view__sidebar :deep(.bp) {
  width: 100%;
  flex: 0 0 auto;
}

.chat-view__main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  /* `contain: inline-size` — hardware-isolation. The browser treats this
   * box's inline-size (width) as independent from its content's intrinsic
   * width. So no matter what a descendant requests as `min-content`, the
   * chat-view__main stays sized by the flex chain alone. Combined with
   * `overflow: hidden` (visual clip) and `min-width: 0` (allow flex
   * shrink), this is the strongest possible containment short of
   * iframing. Anything wider than this box is clipped, full stop. */
  contain: inline-size;
  overflow: hidden;
  background: var(--paper);
}

.chat-view__stream {
  flex: 1;
  min-height: 0;
}

.chat-view__panel {
  width: var(--panel-w);
  flex: none;
  display: flex;
  min-height: 0;
  overflow: hidden;
  border-left: 1px solid var(--paper-line);
  background: var(--paper-shade);
  transition: width 0.28s ease;
}
</style>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useRouter } from 'vue-router';

import { useStreaming } from '../composables/useStreaming';
import { useBuildWatcher } from '../composables/useBuildWatcher';
import { useShortcuts } from '../composables/useShortcuts';
import { useChatStore } from '../stores/chat';
import { useChatHistoryStore } from '../stores/chatHistory';
import { useBuildStore } from '../stores/build';
import { useSettingsStore } from '../stores/settings';
import { useUiStore } from '../stores/ui';

import BuildPanel from '../components/build/BuildPanel.vue';
import ChatStream from '../components/chat/ChatStream.vue';
import ChatComposer from '../components/chat/ChatComposer.vue';

const chat = useChatStore();
const chatHistory = useChatHistoryStore();
const buildStore = useBuildStore();
const settings = useSettingsStore();
const ui = useUiStore();
const router = useRouter();

const { current } = storeToRefs(buildStore);

const sidebarCollapsed = computed(() => current.value === null);

useStreaming();
useBuildWatcher();

useShortcuts({
  onBuildPicker: () => ui.openBuild(),
  onModelPicker: () => ui.openModel(),
  onEscape: () => {
    if (ui.picker) ui.close();
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
    buildStore.refreshActive(),
    settings.refreshModels(),
    settings.refreshDetection(),
  ]);
  // Restore the last active chat if any. The associated build is loaded
  // best-effort; if the file is gone we silently keep the current one.
  const saved = chatHistory.findActive();
  if (saved && saved.messages.length > 0) {
    await chat.loadFromSaved(saved);
    if (saved.attached_build_path && buildStore.current?.source_file !== saved.attached_build_path) {
      await buildStore.setActive(saved.attached_build_path);
    }
  }
});
</script>

<template>
  <div
    class="chat-view sketch-bg"
    :class="{ 'chat-view--collapsed': sidebarCollapsed }"
  >
    <div class="chat-view__sidebar">
      <BuildPanel @open-picker="ui.openBuild()" />
    </div>

    <div class="chat-view__main">
      <ChatStream class="chat-view__stream" />
      <ChatComposer />
    </div>
  </div>
</template>

<style scoped>
.chat-view {
  --sidebar-w: 360px;
  flex: 1;
  display: flex;
  min-height: 0;
  position: relative;
}
.chat-view--collapsed {
  --sidebar-w: 0px;
}

.chat-view__sidebar {
  width: var(--sidebar-w);
  flex: none;
  display: flex;
  min-height: 0;
  overflow: hidden;
  transition: width 0.28s ease;
}

.chat-view__sidebar :deep(.bp) {
  width: 360px;
  flex: 0 0 360px;
}

.chat-view__main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  background: var(--paper);
}

.chat-view__stream {
  flex: 1;
  min-height: 0;
}
</style>

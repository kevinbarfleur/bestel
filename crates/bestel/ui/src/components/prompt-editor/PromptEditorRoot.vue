<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';

import PromptEditorTitleBar from './PromptEditorTitleBar.vue';
import PromptEditorToolbar from './PromptEditorToolbar.vue';
import PromptFileTree from './PromptFileTree.vue';
import PromptEditorBody from './PromptEditorBody.vue';
import PromptOutlinePanel from './PromptOutlinePanel.vue';
import PromptStatusBar from './PromptStatusBar.vue';

import { usePromptsStore } from '../../stores/prompts';

const store = usePromptsStore();
const treeRef = ref<InstanceType<typeof PromptFileTree> | null>(null);
const bodyRef = ref<InstanceType<typeof PromptEditorBody> | null>(null);
const cursor = ref<{ line: number; col: number }>({ line: 1, col: 1 });

async function initialOpen() {
  await store.loadTree();
  void store.watchExternalChanges();
  if (store.activeFile == null) {
    const first = store.tree.groups[0]?.items[0];
    if (first) {
      await store.openFile(first.rel_path);
    }
  }
}

function onKeydown(e: KeyboardEvent) {
  const mod = e.ctrlKey || e.metaKey;
  if (!mod) return;
  if (e.key === 's' || e.key === 'S') {
    e.preventDefault();
    void store.saveActive();
    return;
  }
  if (e.key === 'p' || e.key === 'P') {
    e.preventDefault();
    treeRef.value?.focusFilter();
    return;
  }
  if (e.key === 'w' || e.key === 'W') {
    e.preventDefault();
    void getCurrentWindow().close();
    return;
  }
  if (e.key === 'f' || e.key === 'F') {
    e.preventDefault();
    bodyRef.value?.openSearch();
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown);
  void initialOpen();
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown);
  store.stopWatching();
});

function onCursor(c: { line: number; col: number }) {
  cursor.value = c;
}

function onJumpHeading(line: number) {
  bodyRef.value?.jumpToLine(line);
}
</script>

<template>
  <div class="pe-root">
    <PromptEditorTitleBar />
    <PromptEditorToolbar />
    <div class="pe-grid">
      <PromptFileTree ref="treeRef" />
      <PromptEditorBody ref="bodyRef" @cursor="onCursor" />
      <PromptOutlinePanel :active-line="cursor.line" @jump="onJumpHeading" />
    </div>
    <PromptStatusBar :line="cursor.line" :col="cursor.col" />
  </div>
</template>

<style scoped>
.pe-root {
  width: 100vw;
  height: 100vh;
  background: var(--paper);
  color: var(--ink);
  display: grid;
  grid-template-rows: auto auto 1fr auto;
  overflow: hidden;
}

.pe-grid {
  display: grid;
  grid-template-columns: 286px 1fr 246px;
  min-height: 0;
  border-top: 1px solid var(--paper-line);
}

@media (max-width: 1100px) {
  .pe-grid {
    grid-template-columns: 240px 1fr 220px;
  }
}

@media (max-width: 920px) {
  .pe-grid {
    grid-template-columns: 220px 1fr;
  }

  .pe-grid > :nth-child(3) {
    display: none;
  }
}
</style>

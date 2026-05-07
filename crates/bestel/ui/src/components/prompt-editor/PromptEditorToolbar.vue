<script setup lang="ts">
import { computed } from 'vue';

import { usePromptsStore } from '../../stores/prompts';

const store = usePromptsStore();

const activeName = computed(() => {
  const path = store.activeFile;
  if (path == null) return '';
  return path.split('/').pop() ?? path;
});

const activeDir = computed(() => {
  const path = store.activeFile;
  if (path == null) return '~/.bestel/prompts/';
  const parts = path.split('/');
  if (parts.length <= 1) return '~/.bestel/prompts/';
  return `~/.bestel/prompts/${parts.slice(0, -1).join('/')}/`;
});

const isShipped = computed(() => {
  const path = store.activeFile;
  return path != null && store.kinds[path] === 'shipped';
});

const savedAgo = computed(() => {
  const path = store.activeFile;
  if (path == null) return '';
  const ts = store.lastSavedAt[path];
  if (ts == null) return '';
  const sec = Math.max(0, Math.floor((Date.now() - ts) / 1000));
  if (sec < 1) return 'just now';
  if (sec < 60) return `${sec}s ago`;
  return `${Math.floor(sec / 60)}m ago`;
});

const showPulse = computed(
  () => store.activeFile != null && !store.activeIsDirty && savedAgo.value !== '',
);

async function reset() {
  if (!isShipped.value) return;
  const path = store.activeFile;
  const ok = confirm(
    `Reset ${path} to the bundled default? Your local edits to this file will be lost.`,
  );
  if (!ok) return;
  await store.resetCurrent();
}

async function reveal() {
  await store.reveal(store.activeFile);
}
</script>

<template>
  <div class="pe-toolbar">
    <div class="pe-toolbar__breadcrumb">
      <span class="pe-toolbar__label">file</span>
      <span class="pe-toolbar__path">{{ activeDir }}</span>
      <span class="pe-toolbar__name">{{ activeName }}</span>
      <span v-if="isShipped" class="pe-toolbar__chip">base</span>
      <span v-if="store.activeIsDirty" class="pe-toolbar__dirty" title="Unsaved changes">•</span>
    </div>

    <span v-if="showPulse" class="pe-toolbar__saved">
      <span class="pe-toolbar__pulse" />
      <span>Saved {{ savedAgo }}</span>
    </span>

    <span v-if="showPulse" class="pe-toolbar__divider" />

    <button type="button" class="pe-toolbar__btn" @click="reveal" title="Open in OS file explorer">
      <span>Reveal in finder</span>
    </button>
    <button
      v-if="isShipped"
      type="button"
      class="pe-toolbar__btn pe-toolbar__btn--danger"
      @click="reset"
      title="Restore the bundled default"
    >
      <span>Reset to bundled</span>
    </button>
  </div>
</template>

<style scoped>
.pe-toolbar {
  padding: 10px 18px;
  display: flex;
  align-items: center;
  gap: 12px;
  background: var(--paper);
  border-bottom: 1px solid var(--paper-line);
}

.pe-toolbar__breadcrumb {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.pe-toolbar__label {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 600;
}

.pe-toolbar__path {
  font-family: var(--mono);
  font-size: 13px;
  color: var(--ink-soft);
}

.pe-toolbar__name {
  font-family: var(--mono);
  font-size: 13px;
  color: var(--ink);
  font-weight: 600;
}

.pe-toolbar__chip {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--amber);
  font-weight: 700;
  padding: 2px 7px;
  border: 1px solid var(--amber);
  border-radius: 3px;
  margin-left: 4px;
}

.pe-toolbar__dirty {
  color: var(--amber);
  font-size: 16px;
  font-weight: 700;
  line-height: 1;
}

.pe-toolbar__saved {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--ink-soft);
}

.pe-toolbar__pulse {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--good);
  box-shadow: 0 0 0 3px rgba(84, 124, 74, 0.18);
  flex: none;
}

.pe-toolbar__divider {
  width: 1px;
  height: 18px;
  background: var(--paper-line);
}

.pe-toolbar__btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 11px;
  background: transparent;
  border: 1px solid var(--paper-line);
  border-radius: 4px;
  font-family: var(--hand);
  font-size: 13.5px;
  font-weight: 500;
  color: var(--ink-soft);
  cursor: pointer;
  line-height: 1.2;
}

.pe-toolbar__btn:hover {
  background: var(--paper-shade);
  border-color: var(--ink-faint);
  color: var(--ink);
}

.pe-toolbar__btn--danger {
  color: var(--bad);
}

.pe-toolbar__btn--danger:hover {
  border-color: var(--bad);
  background: rgba(162, 58, 58, 0.06);
  color: var(--bad);
}
</style>

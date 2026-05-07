<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';

const isMaximized = ref(false);
let unlisten: (() => void) | null = null;

onMounted(async () => {
  const w = getCurrentWindow();
  isMaximized.value = await w.isMaximized();
  unlisten = await w.onResized(async () => {
    isMaximized.value = await w.isMaximized();
  });
});

onUnmounted(() => {
  unlisten?.();
});

const minimize = () => getCurrentWindow().minimize();
const toggleMaximize = () => getCurrentWindow().toggleMaximize();
const close = () => getCurrentWindow().close();
</script>

<template>
  <div class="pe-titlebar">
    <div class="pe-titlebar__drag" data-tauri-drag-region>
      <span class="pe-titlebar__rune">◆</span>
      <span class="pe-titlebar__title">Prompts &amp; documentation</span>
      <span class="pe-titlebar__brand">— bestel</span>
    </div>
    <div class="pe-titlebar__controls">
      <button
        type="button"
        class="pe-titlebar__btn"
        aria-label="Minimize"
        title="Minimize"
        @click="minimize"
      >
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <line x1="1" y1="5" x2="9" y2="5" stroke="currentColor" stroke-width="1.5" />
        </svg>
      </button>
      <button
        type="button"
        class="pe-titlebar__btn"
        aria-label="Maximize"
        title="Maximize"
        @click="toggleMaximize"
      >
        <svg v-if="!isMaximized" width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <rect x="1" y="1" width="8" height="8" fill="none" stroke="currentColor" stroke-width="1.5" />
        </svg>
        <svg v-else width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <rect x="1" y="3" width="6" height="6" fill="none" stroke="currentColor" stroke-width="1.5" />
          <path d="M3 3 V1 H9 V7 H7" fill="none" stroke="currentColor" stroke-width="1.5" />
        </svg>
      </button>
      <button
        type="button"
        class="pe-titlebar__btn pe-titlebar__btn--close"
        aria-label="Close"
        title="Close (Ctrl+W)"
        @click="close"
      >
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <line x1="1" y1="1" x2="9" y2="9" stroke="currentColor" stroke-width="1.5" />
          <line x1="9" y1="1" x2="1" y2="9" stroke="currentColor" stroke-width="1.5" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.pe-titlebar {
  height: 32px;
  display: flex;
  align-items: stretch;
  background: var(--paper-shade);
  border-bottom: 1px solid var(--paper-line);
  user-select: none;
}

.pe-titlebar__drag {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  cursor: grab;
}

.pe-titlebar__drag:active {
  cursor: grabbing;
}

.pe-titlebar__rune {
  color: var(--amber);
  opacity: 0.6;
  font-size: 11px;
  pointer-events: none;
}

.pe-titlebar__title {
  font-family: var(--hand-display);
  font-size: 14px;
  font-weight: 600;
  color: var(--ink-soft);
  letter-spacing: 0.01em;
  pointer-events: none;
}

.pe-titlebar__brand {
  font-family: var(--hand-display);
  font-size: 14px;
  font-weight: 400;
  color: var(--ink-faint);
  pointer-events: none;
}

.pe-titlebar__controls {
  display: flex;
  align-items: stretch;
  flex-shrink: 0;
}

.pe-titlebar__btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--ink-faint);
  cursor: pointer;
  padding: 0;
  transition: background 150ms ease, color 150ms ease;
}

.pe-titlebar__btn:hover {
  background: var(--paper-line);
  color: var(--ink);
}

.pe-titlebar__btn--close:hover {
  background: #e81123;
  color: #fff;
}
</style>

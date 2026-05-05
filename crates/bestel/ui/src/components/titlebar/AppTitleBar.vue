<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { windowClose, windowMinimize, windowToggleMaximize } from '../../api/tauri';

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

const onMinimize = () => windowMinimize();
const onMaximize = () => windowToggleMaximize();
const onClose = () => windowClose();
</script>

<template>
  <div class="titlebar-frame">
    <div class="titlebar">
      <div class="titlebar__left">
        <span class="titlebar__rune titlebar__rune--corner">◆</span>
      </div>
      <div class="titlebar__drag" data-tauri-drag-region>
        <span class="titlebar__rune">◆</span>
        <span class="titlebar__title">Bestel</span>
        <span class="titlebar__rune">◆</span>
      </div>
      <div class="titlebar__controls">
        <button
          class="titlebar__btn"
          type="button"
          aria-label="Minimize"
          title="Minimize"
          @click="onMinimize"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <line x1="1" y1="5" x2="9" y2="5" stroke="currentColor" stroke-width="1.5" />
          </svg>
        </button>
        <button
          class="titlebar__btn"
          type="button"
          aria-label="Maximize"
          title="Maximize"
          @click="onMaximize"
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
          class="titlebar__btn titlebar__btn--close"
          type="button"
          aria-label="Close"
          title="Close"
          @click="onClose"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <line x1="1" y1="1" x2="9" y2="9" stroke="currentColor" stroke-width="1.5" />
            <line x1="9" y1="1" x2="1" y2="9" stroke="currentColor" stroke-width="1.5" />
          </svg>
        </button>
      </div>
    </div>
    <div class="titlebar-frame__accent"></div>
  </div>
</template>

<style scoped>
.titlebar-frame {
  flex-shrink: 0;
  user-select: none;
}

.titlebar {
  display: flex;
  align-items: stretch;
  height: 32px;
  background: linear-gradient(
    180deg,
    rgba(18, 17, 15, 0.98) 0%,
    rgba(12, 11, 10, 0.99) 100%
  );
  border-bottom: 1px solid rgba(40, 38, 35, 0.6);
  box-shadow:
    inset 0 1px 0 rgba(80, 70, 55, 0.1),
    0 2px 6px rgba(0, 0, 0, 0.4);
}

.titlebar__left {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  flex-shrink: 0;
}

.titlebar__drag {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  cursor: grab;
}
.titlebar__drag:active {
  cursor: grabbing;
}

.titlebar__rune {
  font-size: 0.4rem;
  color: rgba(175, 96, 37, 0.5);
  text-shadow: 0 0 6px rgba(175, 96, 37, 0.2);
  pointer-events: none;
}
.titlebar__rune--corner {
  font-size: 0.45rem;
  color: rgba(175, 96, 37, 0.35);
}

.titlebar__title {
  font-family: 'Cinzel', serif;
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: rgba(180, 165, 140, 0.7);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.6);
  pointer-events: none;
}

.titlebar__controls {
  display: flex;
  align-items: stretch;
  flex-shrink: 0;
}

.titlebar__btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 100%;
  border: none;
  background: transparent;
  color: rgba(140, 130, 120, 0.55);
  cursor: pointer;
  transition: background var(--transition-fast), color var(--transition-fast);
  padding: 0;
}
.titlebar__btn:hover {
  background: rgba(80, 70, 55, 0.25);
  color: rgba(220, 205, 180, 0.95);
}
.titlebar__btn:active {
  background: rgba(80, 70, 55, 0.4);
}
.titlebar__btn--close:hover {
  background: rgba(232, 17, 35, 0.85);
  color: #fff;
}

.titlebar-frame__accent {
  height: 1px;
  background: linear-gradient(
    to right,
    rgba(80, 70, 55, 0.2),
    rgba(175, 96, 37, 0.45) 30%,
    rgba(175, 96, 37, 0.6) 50%,
    rgba(175, 96, 37, 0.45) 70%,
    rgba(80, 70, 55, 0.2)
  );
  box-shadow: 0 1px 4px rgba(175, 96, 37, 0.18);
}
</style>

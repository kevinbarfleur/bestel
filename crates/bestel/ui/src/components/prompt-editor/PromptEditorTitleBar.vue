<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';

async function close() {
  await getCurrentWindow().close();
}
</script>

<template>
  <div class="pe-titlebar" data-tauri-drag-region>
    <div class="pe-titlebar__dots">
      <span class="pe-titlebar__dot pe-titlebar__dot--red" />
      <span class="pe-titlebar__dot pe-titlebar__dot--amber" />
      <span class="pe-titlebar__dot pe-titlebar__dot--green" />
    </div>
    <span class="pe-titlebar__spacer" />
    <span class="pe-titlebar__title">
      <span class="pe-titlebar__rune">◆</span>
      <span>Prompts &amp; documentation</span>
      <span class="pe-titlebar__brand">— bestel</span>
    </span>
    <span class="pe-titlebar__spacer" />
    <button
      type="button"
      class="pe-titlebar__close"
      aria-label="Close window"
      title="Close (Ctrl+W)"
      @click="close"
    >
      <svg viewBox="0 0 12 12" width="12" height="12" aria-hidden="true">
        <path d="M2 2 L10 10 M10 2 L2 10" stroke="currentColor" stroke-width="1.4" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.pe-titlebar {
  height: 38px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 14px;
  background: var(--paper-shade);
  border-bottom: 1px solid var(--paper-line);
  position: relative;
  user-select: none;
}

.pe-titlebar__dots {
  display: flex;
  gap: 8px;
  flex: none;
}

.pe-titlebar__dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  box-shadow: inset 0 0 0 0.5px rgba(0, 0, 0, 0.18);
  flex: none;
}

.pe-titlebar__dot--red {
  background: #e8695c;
}

.pe-titlebar__dot--amber {
  background: #e0b73e;
}

.pe-titlebar__dot--green {
  background: #73c563;
}

.pe-titlebar__spacer {
  flex: 1;
}

.pe-titlebar__title {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  font-family: var(--hand-display);
  font-size: 14px;
  font-weight: 600;
  color: var(--ink-soft);
  letter-spacing: 0.01em;
  display: flex;
  align-items: center;
  gap: 8px;
  pointer-events: none;
}

.pe-titlebar__rune {
  color: var(--amber);
  opacity: 0.7;
}

.pe-titlebar__brand {
  color: var(--ink-faint);
  font-weight: 400;
}

.pe-titlebar__close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 22px;
  border: none;
  background: transparent;
  color: var(--ink-soft);
  cursor: pointer;
  border-radius: 3px;
  flex: none;
}

.pe-titlebar__close:hover {
  background: rgba(232, 105, 92, 0.15);
  color: #d23a2c;
}
</style>

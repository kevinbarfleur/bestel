<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { getCurrentWindow } from '@tauri-apps/api/window';

import { windowClose, windowMinimize, windowToggleMaximize } from '../../api/tauri';
import { useBuildStore } from '../../stores/build';
import { useChatHistoryStore } from '../../stores/chatHistory';
import { useSettingsStore } from '../../stores/settings';
import { useUiStore } from '../../stores/ui';

const buildStore = useBuildStore();
const chatHistory = useChatHistoryStore();
const settings = useSettingsStore();
const ui = useUiStore();

const { current: currentBuild } = storeToRefs(buildStore);
const { activeModel, theme } = storeToRefs(settings);
const { activeId: activeChatId } = storeToRefs(chatHistory);

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

const buildName = computed(() => {
  const c = currentBuild.value;
  if (!c) return null;
  const cls = c.class ?? '';
  const asc = c.ascendancy ?? '';
  return asc ? `${cls} · ${asc}` : cls || 'Build';
});

const buildLevel = computed(() => {
  const c = currentBuild.value;
  if (!c?.level) return null;
  return `lvl ${c.level}`;
});

const modelLabel = computed(() => activeModel.value?.display_name ?? 'model');
const themeLabel = computed(() => (theme.value === 'dark' ? 'dark' : 'light'));

const chatLabel = computed(() => {
  if (!activeChatId.value) return 'New chat';
  const c = chatHistory.findActive();
  if (!c) return 'New chat';
  const t = c.title;
  return t.length > 28 ? `${t.slice(0, 27)}…` : t;
});

const onToggleTheme = () => settings.toggleTheme();
</script>

<template>
  <header class="topbar">
    <!-- Logo -->
    <div class="topbar__logo">
      <div class="topbar__brand">
        <span class="topbar__brand-mark">B</span>
      </div>
      <span class="topbar__brand-name">Bestel</span>
    </div>

    <span class="topbar__hairline" aria-hidden="true" />

    <!-- Build picker — pure typography, opens BuildPicker -->
    <button
      type="button"
      class="topbar__picker"
      :title="currentBuild?.file_name ?? 'Load build'"
      @click="ui.openBuild()"
    >
      <span class="topbar__picker-label">build</span>
      <template v-if="buildName">
        <span class="topbar__picker-value">{{ buildName }}</span>
        <span v-if="buildLevel" class="topbar__picker-meta">· {{ buildLevel }}</span>
      </template>
      <span v-else class="topbar__picker-value topbar__picker-value--empty">Load build</span>
      <span class="topbar__picker-caret">▾</span>
    </button>

    <!-- Drag region -->
    <div class="topbar__drag" data-tauri-drag-region />

    <!-- Chat history picker -->
    <button
      type="button"
      class="topbar__picker"
      title="Browse saved chats"
      @click="ui.openChat()"
    >
      <span class="topbar__picker-label">chat</span>
      <span class="topbar__picker-value topbar__picker-value--sm">{{ chatLabel }}</span>
      <span class="topbar__picker-caret">▾</span>
    </button>

    <!-- Model picker -->
    <button
      type="button"
      class="topbar__picker"
      :title="`Model: ${modelLabel}`"
      @click="ui.openModel()"
    >
      <span class="topbar__picker-label">model</span>
      <span class="topbar__picker-value topbar__picker-value--sm">{{ modelLabel }}</span>
      <span class="topbar__picker-caret">▾</span>
    </button>

    <!-- Theme toggle -->
    <button
      type="button"
      class="topbar__picker topbar__theme"
      :title="`Theme: ${themeLabel}`"
      @click="onToggleTheme"
    >
      <span class="topbar__picker-label">theme</span>
      <span class="topbar__picker-value topbar__picker-value--sm">{{ themeLabel }}</span>
    </button>

    <!-- Window controls -->
    <div class="topbar__controls">
      <button class="topbar__btn" type="button" aria-label="Minimize" title="Minimize" @click="onMinimize">
        <svg width="10" height="10" viewBox="0 0 10 10"><line x1="1" y1="5" x2="9" y2="5" stroke="currentColor" stroke-width="1.5" /></svg>
      </button>
      <button class="topbar__btn" type="button" aria-label="Maximize" title="Maximize" @click="onMaximize">
        <svg v-if="!isMaximized" width="10" height="10" viewBox="0 0 10 10"><rect x="1" y="1" width="8" height="8" fill="none" stroke="currentColor" stroke-width="1.5" /></svg>
        <svg v-else width="10" height="10" viewBox="0 0 10 10"><rect x="1" y="3" width="6" height="6" fill="none" stroke="currentColor" stroke-width="1.5" /><rect x="3" y="1" width="6" height="6" fill="none" stroke="currentColor" stroke-width="1.5" /></svg>
      </button>
      <button class="topbar__btn topbar__btn--close" type="button" aria-label="Close" title="Close" @click="onClose">
        <svg width="10" height="10" viewBox="0 0 10 10"><line x1="1.5" y1="1.5" x2="8.5" y2="8.5" stroke="currentColor" stroke-width="1.5" /><line x1="8.5" y1="1.5" x2="1.5" y2="8.5" stroke="currentColor" stroke-width="1.5" /></svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.topbar {
  height: 46px;
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 0 0 0 18px;
  border-bottom: 1px solid var(--paper-line);
  background: var(--paper);
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}

/* Logo */
.topbar__logo {
  display: flex;
  align-items: center;
  gap: 8px;
  -webkit-app-region: no-drag;
  flex-shrink: 0;
}
.topbar__brand {
  width: 18px;
  height: 18px;
  transform: rotate(45deg);
  border: 1.2px solid var(--amber);
  display: flex;
  align-items: center;
  justify-content: center;
}
.topbar__brand-mark {
  transform: rotate(-45deg);
  font-family: var(--hand-display);
  font-size: 11px;
  font-weight: 700;
  color: var(--amber);
}
.topbar__brand-name {
  font-family: var(--hand-display);
  font-size: 16px;
  font-weight: 700;
  letter-spacing: 0.04em;
  color: var(--ink);
}

/* Hairline divider between logo and pickers */
.topbar__hairline {
  width: 1px;
  height: 18px;
  background: var(--paper-line);
  flex-shrink: 0;
}

/* Pure-typography picker (build / model / theme) */
.topbar__picker {
  display: flex;
  align-items: baseline;
  gap: 6px;
  padding: 0;
  background: transparent;
  border: none;
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
}
.topbar__picker:hover .topbar__picker-value { color: var(--amber); }
.topbar__picker:focus { outline: none; }
.topbar__picker:focus-visible .topbar__picker-value { color: var(--amber); }

.topbar__picker-label {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 500;
}
.topbar__picker-value {
  font-family: var(--hand-display);
  font-size: 14px;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
  transition: color 0.15s ease;
}
.topbar__picker-value--sm {
  font-size: 13px;
  font-weight: 500;
}
.topbar__picker-value--empty {
  color: var(--ink-faint);
}
.topbar__picker-meta {
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink-faint);
  white-space: nowrap;
}
.topbar__picker-caret {
  font-size: 10px;
  color: var(--ink-faint);
  margin-left: 2px;
}

/* Drag region (window move) */
.topbar__drag {
  flex: 1;
  height: 100%;
  -webkit-app-region: drag;
  app-region: drag;
  min-width: 24px;
}

/* Window controls */
.topbar__controls {
  display: flex;
  margin-left: 4px;
  height: 100%;
  -webkit-app-region: no-drag;
}
.topbar__btn {
  width: 38px;
  height: 100%;
  border: 0;
  background: transparent;
  color: var(--ink-soft);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s ease, color 0.15s ease;
}
.topbar__btn:hover {
  background: var(--paper-shade);
  color: var(--ink);
}
.topbar__btn--close:hover {
  background: var(--bad);
  color: var(--paper);
}
</style>

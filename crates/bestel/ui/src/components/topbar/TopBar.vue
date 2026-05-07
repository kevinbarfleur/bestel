<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { getCurrentWindow } from '@tauri-apps/api/window';

import { windowClose, windowMinimize, windowToggleMaximize } from '../../api/tauri';
import { useBuildStore } from '../../stores/build';
import { useChatHistoryStore } from '../../stores/chatHistory';
import { useSettingsStore } from '../../stores/settings';
import { useUiStore } from '../../stores/ui';
import RunicIcon from '../runic/RunicIcon.vue';

const buildStore = useBuildStore();
const chatHistory = useChatHistoryStore();
const settings = useSettingsStore();
const ui = useUiStore();

const { current: currentBuild } = storeToRefs(buildStore);
const { activeModel } = storeToRefs(settings);
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
  return c.class ?? 'Build';
});

const buildSub = computed(() => {
  const c = currentBuild.value;
  if (!c) return null;
  const parts: string[] = [];
  if (c.ascendancy) parts.push(c.ascendancy);
  if (c.level) parts.push(`${c.level}`);
  return parts.length ? parts.join(' · ') : null;
});

const modelLabel = computed(() => activeModel.value?.display_name ?? 'pick a model');
const isDev = import.meta.env.DEV;

const chatLabel = computed(() => {
  if (!activeChatId.value) return 'New chat';
  const c = chatHistory.findActive();
  if (!c) return 'New chat';
  const t = c.title;
  return t.length > 32 ? `${t.slice(0, 31)}…` : t;
});

</script>

<template>
  <header class="topbar">
    <!-- Logo -->
    <div class="topbar__logo">
      <div class="topbar__brand" aria-hidden="true">
        <span class="topbar__brand-mark">B</span>
      </div>
      <span class="topbar__brand-name">Bestel</span>
    </div>

    <span class="topbar__hairline" aria-hidden="true" />

    <!-- BUILD pill — amber bordered, opens BuildPicker -->
    <button
      type="button"
      class="topbar__pill topbar__pill--amber"
      :class="{ 'topbar__pill--empty': !currentBuild }"
      :title="currentBuild?.file_name ?? 'Load build'"
      @click="ui.openBuild()"
    >
      <span class="topbar__pill-label">build</span>
      <template v-if="buildName">
        <span class="topbar__pill-value">{{ buildName }}</span>
        <span v-if="buildSub" class="topbar__pill-sub">· {{ buildSub }}</span>
      </template>
      <span v-else class="topbar__pill-value topbar__pill-value--empty">Load build</span>
      <span class="topbar__pill-caret">▾</span>
    </button>

    <!-- Drag region — window move -->
    <div class="topbar__drag" data-tauri-drag-region />

    <!-- CHAT pill -->
    <button
      type="button"
      class="topbar__pill"
      title="Browse saved chats"
      @click="ui.openChat()"
    >
      <span class="topbar__pill-label">chat</span>
      <span class="topbar__pill-value">{{ chatLabel }}</span>
      <span class="topbar__pill-caret">▾</span>
    </button>

    <!-- MODEL pill -->
    <button
      type="button"
      class="topbar__pill"
      :title="`Model: ${modelLabel}`"
      @click="ui.openModel()"
    >
      <span class="topbar__pill-label">model</span>
      <span class="topbar__pill-value">{{ modelLabel }}</span>
      <span class="topbar__pill-caret">▾</span>
    </button>

    <!-- Settings — opens the SettingsPicker (theme is now managed there) -->
    <button
      type="button"
      class="topbar__theme-chip"
      title="Settings"
      aria-label="Open settings"
      @click="ui.openSettings()"
    >
      <RunicIcon name="gear" :size="16" />
    </button>

    <!-- Debug entry (dev mode only) -->
    <router-link
      v-if="isDev"
      to="/debug"
      class="topbar__pill topbar__pill--ghost"
      title="Debug · chat history (dev only)"
    >
      <span class="topbar__pill-label">debug</span>
      <span class="topbar__pill-value">runs</span>
    </router-link>

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
  height: 52px;
  display: flex;
  align-items: center;
  gap: 14px;
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
  gap: 10px;
  -webkit-app-region: no-drag;
  flex-shrink: 0;
}
.topbar__brand {
  width: 20px;
  height: 20px;
  transform: rotate(45deg);
  border: 1.4px solid var(--amber);
  display: flex;
  align-items: center;
  justify-content: center;
}
.topbar__brand-mark {
  transform: rotate(-45deg);
  font-family: var(--hand);
  font-size: 12px;
  font-weight: var(--fw-bold);
  color: var(--amber);
}
.topbar__brand-name {
  font-family: var(--hand);
  font-size: 20px;
  font-weight: var(--fw-bold);
  letter-spacing: 0.02em;
  color: var(--ink);
}

/* Hairline divider between logo and pills */
.topbar__hairline {
  width: 1px;
  height: 20px;
  background: var(--paper-line);
  flex-shrink: 0;
}

/* Pill — bordered button with small caps label + value + caret */
.topbar__pill {
  display: inline-flex;
  align-items: baseline;
  gap: 8px;
  padding: 5px 12px;
  background: transparent;
  border: 1px solid var(--ink-faint);
  border-radius: 16px;
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  font-family: var(--hand);
  -webkit-app-region: no-drag;
  transition: border-color 0.15s ease, background 0.15s ease;
}
.topbar__pill:hover {
  border-color: var(--ink-soft);
  background: var(--paper-shade);
}
.topbar__pill:focus { outline: none; }
.topbar__pill:focus-visible {
  border-color: var(--amber);
}

.topbar__pill--amber {
  border-color: var(--amber);
  background: var(--amber-glow);
}
.topbar__pill--amber:hover {
  background: var(--amber-bg);
  filter: brightness(0.97);
}
.theme-dark .topbar__pill--amber:hover {
  filter: brightness(1.1);
}
.topbar__pill--amber.topbar__pill--empty {
  background: transparent;
  border-style: dashed;
}

.topbar__pill--ghost {
  border-style: dashed;
  border-color: var(--ink-ghost);
  text-decoration: none;
}

.topbar__pill-label {
  font-family: var(--label);
  font-size: var(--fs-micro);
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: var(--fw-semibold);
}
.topbar__pill--amber .topbar__pill-label {
  color: var(--amber);
}

.topbar__pill-value {
  font-family: var(--hand);
  font-size: var(--fs-body);
  font-weight: var(--fw-medium);
  color: var(--ink);
  white-space: nowrap;
}
.topbar__pill-value--empty {
  color: var(--ink-faint);
}

.topbar__pill-sub {
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
  white-space: nowrap;
}

.topbar__pill-caret {
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

/* Theme chip — ghost icon button, hover paper-shade. */
.topbar__theme-chip {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  cursor: pointer;
  color: var(--ink-soft);
  flex-shrink: 0;
  -webkit-app-region: no-drag;
  transition: background 0.15s ease, color 0.15s ease;
}
.topbar__theme-chip:hover {
  color: var(--ink);
  background: var(--paper-shade);
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

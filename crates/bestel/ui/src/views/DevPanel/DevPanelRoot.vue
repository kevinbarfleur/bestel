<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

import RunsTab from './tabs/RunsTab.vue';
import ScenariosTab from './tabs/ScenariosTab.vue';
import RealPromptsTab from './tabs/RealPromptsTab.vue';
import LiveTestTab from './tabs/LiveTestTab.vue';

type Tab = 'runs' | 'scenarios' | 'real-prompts' | 'live-test';

const TABS: Array<{ id: Tab; label: string; description: string }> = [
  { id: 'runs', label: 'Runs', description: 'Browse past chat runs' },
  { id: 'scenarios', label: 'Scenarios', description: 'TOML scenarios with assertions' },
  { id: 'real-prompts', label: 'Real prompts', description: 'Mined Reddit/forum prompts' },
  { id: 'live-test', label: 'Live test', description: 'Free chat with rendered + raw capture' },
];

const active = ref<Tab>('runs');

const closeWindow = async () => {
  try {
    await invoke('window_close');
  } catch (e) {
    console.error('window_close failed', e);
  }
};

const tabHint = computed(
  () => TABS.find((t) => t.id === active.value)?.description ?? ''
);
</script>

<template>
  <div class="dev-panel-root">
    <header class="dev-header" data-tauri-drag-region>
      <div class="dev-header__title" data-tauri-drag-region>
        <span class="dev-header__brand">Bestel</span>
        <span class="dev-header__sep">·</span>
        <span class="dev-header__label">Dev panel</span>
      </div>
      <div class="dev-header__hint">{{ tabHint }}</div>
      <button class="dev-header__close" @click="closeWindow" title="Close">×</button>
    </header>

    <nav class="dev-tabs">
      <button
        v-for="tab in TABS"
        :key="tab.id"
        class="dev-tab"
        :class="{ 'dev-tab--active': active === tab.id }"
        @click="active = tab.id"
      >
        {{ tab.label }}
      </button>
    </nav>

    <main class="dev-content">
      <RunsTab v-if="active === 'runs'" />
      <div v-else-if="active === 'scenarios'" class="dev-pane">
        <ScenariosTab />
      </div>
      <div v-else-if="active === 'real-prompts'" class="dev-pane">
        <RealPromptsTab />
      </div>
      <div v-else-if="active === 'live-test'" class="dev-pane dev-pane--full">
        <LiveTestTab />
      </div>
    </main>
  </div>
</template>

<style scoped>
.dev-panel-root {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #0a0a0c;
  color: var(--c-fg, #e8e6df);
  font-family: var(--font-body, 'EB Garamond', serif);
}

.dev-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 16px;
  background: #141417;
  border-bottom: 1px solid #2a2a30;
  user-select: none;
  -webkit-app-region: drag;
}

.dev-header__title {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-family: var(--font-display, 'Cinzel', serif);
  font-size: 14px;
  letter-spacing: 0.06em;
}

.dev-header__brand {
  color: #c9a86a;
  font-weight: 600;
}

.dev-header__sep {
  color: #50505a;
}

.dev-header__label {
  color: #a8a8b0;
  text-transform: uppercase;
  font-size: 11px;
  letter-spacing: 0.16em;
}

.dev-header__hint {
  flex: 1;
  font-size: 12px;
  color: #75757f;
  font-style: italic;
}

.dev-header__close {
  -webkit-app-region: no-drag;
  background: transparent;
  border: none;
  color: #888;
  font-size: 18px;
  width: 28px;
  height: 28px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}

.dev-header__close:hover {
  background: #c14a4a;
  color: white;
}

.dev-tabs {
  display: flex;
  gap: 0;
  padding: 0 16px;
  background: #141417;
  border-bottom: 1px solid #2a2a30;
}

.dev-tab {
  background: transparent;
  border: none;
  color: #75757f;
  font-family: inherit;
  font-size: 13px;
  padding: 10px 16px;
  cursor: pointer;
  position: relative;
  transition: color 0.12s ease;
}

.dev-tab:hover {
  color: #c9c9d0;
}

.dev-tab--active {
  color: #e8e6df;
}

.dev-tab--active::after {
  content: '';
  position: absolute;
  left: 12px;
  right: 12px;
  bottom: -1px;
  height: 2px;
  background: #c9a86a;
}

.dev-content {
  flex: 1;
  overflow: auto;
  padding: 24px;
}

.dev-pane {
  max-width: 1100px;
  margin: 0 auto;
}

.dev-pane--full {
  max-width: none;
  margin: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.dev-placeholder {
  color: #50505a;
  font-style: italic;
  text-align: center;
  padding: 48px 0;
}
</style>

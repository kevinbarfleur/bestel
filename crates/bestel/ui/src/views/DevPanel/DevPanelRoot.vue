<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

import RunsTab from './tabs/RunsTab.vue';
import ScenariosTab from './tabs/ScenariosTab.vue';
import RealPromptsTab from './tabs/RealPromptsTab.vue';
import LiveTestTab from './tabs/LiveTestTab.vue';
import { listDebugRuns } from '../../api/tauri';

type Tab = 'runs' | 'scenarios' | 'real-prompts' | 'live-test';

interface TabSpec {
  id: Tab;
  label: string;
  description: string;
  count: number | null;
}

const tabs = ref<TabSpec[]>([
  { id: 'runs', label: 'Runs', description: 'Browse past chat runs', count: null },
  { id: 'scenarios', label: 'Scenarios', description: 'TOML scenarios with assertions', count: null },
  { id: 'real-prompts', label: 'Real prompts', description: 'Mined Reddit/forum prompts', count: null },
  { id: 'live-test', label: 'Live test', description: 'Free chat with rendered + raw capture', count: null },
]);

const active = ref<Tab>('runs');

const tabHint = computed(
  () => tabs.value.find((t) => t.id === active.value)?.description ?? ''
);

async function ipc(name: string) {
  try {
    await invoke(name);
  } catch (e) {
    console.error(`${name} failed`, e);
  }
}

const closeWindow = () => ipc('window_close');
const minimizeWindow = () => ipc('window_minimize');
const toggleMaximize = () => ipc('toggle_maximize');

async function refreshCounts() {
  try {
    const runs = await listDebugRuns();
    setCount('runs', runs.length);
  } catch {
    /* leave null */
  }
  try {
    const scenarios = await invoke<unknown[]>('dev_load_scenarios');
    setCount('scenarios', scenarios.length);
  } catch {
    /* leave null */
  }
  try {
    const prompts = await invoke<unknown[]>('dev_load_real_prompts');
    setCount('real-prompts', prompts.length);
  } catch {
    /* leave null */
  }
}

function setCount(id: Tab, count: number) {
  const t = tabs.value.find((x) => x.id === id);
  if (t) t.count = count;
}

onMounted(refreshCounts);
</script>

<template>
  <div class="dp">
    <header class="dp__title" data-tauri-drag-region>
      <div class="dp__lights" data-tauri-drag-region>
        <button class="dp__light dp__light--close" type="button" aria-label="Close" @click.stop="closeWindow" />
        <button class="dp__light dp__light--min" type="button" aria-label="Minimize" @click.stop="minimizeWindow" />
        <button class="dp__light dp__light--max" type="button" aria-label="Maximize" @click.stop="toggleMaximize" />
      </div>
      <div class="dp__brand" data-tauri-drag-region>
        <span class="dp__rune">◆</span>
        <span class="dp__name">Bestel</span>
        <span class="dp__sep">·</span>
        <span class="dp__label">Dev panel</span>
        <span class="dp__sep">·</span>
        <span class="dp__hint">{{ tabHint }}</span>
      </div>
      <button class="dp__close" type="button" aria-label="Close window" @click="closeWindow">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <path d="M6 6 L18 18 M6 18 L18 6" />
        </svg>
      </button>
    </header>

    <nav class="dp__tabs">
      <button
        v-for="t in tabs"
        :key="t.id"
        type="button"
        class="dp__tab"
        :class="{ 'dp__tab--active': active === t.id }"
        @click="active = t.id"
      >
        <span class="dp__tab-label">{{ t.label }}</span>
        <span v-if="t.count !== null" class="dp__tab-count">{{ t.count }}</span>
      </button>
    </nav>

    <main class="dp__content">
      <RunsTab v-if="active === 'runs'" />
      <ScenariosTab v-else-if="active === 'scenarios'" />
      <RealPromptsTab v-else-if="active === 'real-prompts'" />
      <LiveTestTab v-else-if="active === 'live-test'" />
    </main>

    <footer class="dp__status">
      <span class="dp__status-pip">
        <span class="dp__pip" />
        <span>Daemon connected</span>
      </span>
      <span class="dp__divider" />
      <span class="dp__status-mono">~/.bestel/runs/</span>
      <span class="dp__divider" />
      <span class="dp__status-text">debug build</span>
      <span class="dp__spacer" />
      <span class="dp__status-kbd">
        <kbd>Ctrl</kbd><kbd>R</kbd><span>refresh</span>
      </span>
      <span class="dp__status-kbd">
        <kbd>Ctrl</kbd><kbd>W</kbd><span>close window</span>
      </span>
    </footer>
  </div>
</template>

<style scoped>
.dp {
  display: grid;
  grid-template-rows: auto auto 1fr auto;
  height: 100vh;
  background: var(--paper);
  color: var(--ink);
  font-family: var(--hand);
  overflow: hidden;
}

.dp__title {
  display: flex;
  align-items: center;
  gap: 14px;
  height: 40px;
  padding: 0 14px;
  background: var(--paper-shade);
  border-bottom: 1px solid var(--paper-line);
  position: relative;
  user-select: none;
}

.dp__lights {
  display: flex;
  gap: 8px;
  -webkit-app-region: no-drag;
}

.dp__light {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: none;
  padding: 0;
  cursor: pointer;
  box-shadow: inset 0 0 0 0.5px rgba(0, 0, 0, 0.18);
  transition: filter 100ms ease;
}
.dp__light:hover {
  filter: brightness(1.08);
}
.dp__light--close { background: #e8695c; }
.dp__light--min   { background: #e0b73e; }
.dp__light--max   { background: #73c563; }

.dp__brand {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
  color: var(--ink-soft);
  font-weight: 500;
  white-space: nowrap;
  pointer-events: none;
}
.dp__rune {
  color: var(--amber);
  opacity: 0.75;
}
.dp__name {
  font-weight: 700;
  color: var(--ink);
  letter-spacing: 0.02em;
}
.dp__sep {
  color: var(--ink-faint);
}
.dp__label {
  font-family: var(--label);
  font-size: 12.5px;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  font-weight: 600;
}
.dp__hint {
  font-style: italic;
  color: var(--ink-faint);
}

.dp__close {
  margin-left: auto;
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  color: var(--ink-soft);
  cursor: pointer;
  -webkit-app-region: no-drag;
}
.dp__close:hover {
  background: var(--paper);
  border-color: var(--paper-line);
  color: var(--ink);
}

.dp__tabs {
  display: flex;
  padding: 0 18px;
  background: var(--paper);
  border-bottom: 1px solid var(--paper-line);
  gap: 4px;
}

.dp__tab {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 14px 16px 12px;
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--ink-soft);
  font-family: var(--hand);
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: color 100ms ease, border-color 100ms ease;
}
.dp__tab:hover {
  color: var(--ink);
}
.dp__tab--active {
  border-bottom-color: var(--amber);
  color: var(--ink);
  font-weight: 700;
}

.dp__tab-count {
  font-family: var(--label);
  font-size: 11.5px;
  letter-spacing: 0.14em;
  color: var(--ink-faint);
  font-weight: 600;
  padding: 1px 6px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  line-height: 1.2;
  font-variant-numeric: tabular-nums;
}
.dp__tab--active .dp__tab-count {
  color: var(--amber);
  border-color: var(--amber);
}

.dp__content {
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: var(--paper);
}

.dp__content > * {
  flex: 1;
  min-height: 0;
}

.dp__status {
  padding: 9px 18px;
  border-top: 1px solid var(--paper-line);
  background: var(--paper-shade);
  display: flex;
  align-items: center;
  gap: 14px;
  font-size: 13.5px;
  color: var(--ink-soft);
}

.dp__status-pip {
  display: inline-flex;
  align-items: center;
  gap: 7px;
}
.dp__pip {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--good);
  box-shadow: 0 0 0 3px rgba(46, 107, 58, 0.18);
}
.dp__divider {
  width: 1px;
  height: 14px;
  background: var(--paper-line);
}
.dp__status-mono {
  font-family: var(--mono);
  font-size: 12.5px;
}
.dp__status-text {
  font-style: italic;
  color: var(--ink-faint);
}
.dp__spacer {
  flex: 1;
}
.dp__status-kbd {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}
.dp__status-kbd kbd {
  font-family: var(--label);
  font-size: 11px;
  padding: 1px 5px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  color: var(--ink-soft);
  line-height: 1.2;
  background: var(--paper);
  font-variant-numeric: tabular-nums;
}
</style>

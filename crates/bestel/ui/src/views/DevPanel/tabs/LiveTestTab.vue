<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from 'vue';
import {
  chatCancel,
  chatReset,
  chatStart,
  clearActiveBuild,
  getActiveBuild,
  getActiveModel,
  listModels,
  onLlmDelta,
  setActiveBuild,
  setActiveModel,
} from '../../../api/tauri';
import type {
  LlmDeltaEvent,
  ModelProfileDto,
  PobBuildDto,
} from '../../../api/types';
import { renderMarkdown } from '../../../api/markdown';

type RightPane = 'rendered' | 'raw' | 'stats';

const models = ref<ModelProfileDto[]>([]);
const activeModelId = ref<string>('');
const activeBuild = ref<PobBuildDto | null>(null);
const fixturePath = ref<string>('');

const KNOWN_FIXTURES = [
  { id: 'poe1_inquisitor', label: 'poe1_inquisitor.xml' },
  { id: 'poe2_druid', label: 'poe2_druid.xml' },
];

const prompt = ref('');
const isRunning = ref(false);
const rightPane = ref<RightPane>('rendered');

interface ToolEntry {
  id: string;
  name: string;
  detail: string | null;
  status: 'running' | 'done' | 'failed';
  summary: string | null;
  output: string;
}

const finalText = ref('');
const toolEntries = ref<ToolEntry[]>([]);
const rawEvents = ref<LlmDeltaEvent[]>([]);
const usage = ref<{
  input_tokens: number;
  cached_input_tokens: number;
  output_tokens: number;
  cost_usd: number | null;
} | null>(null);
const errorMsg = ref<string | null>(null);

let unlisten: (() => void) | null = null;
let currentSession: number | null = null;

onMounted(async () => {
  try {
    models.value = await listModels();
    const active = await getActiveModel();
    activeModelId.value = active?.id ?? '';
    activeBuild.value = await getActiveBuild();
  } catch (e) {
    errorMsg.value = `init: ${e}`;
  }
  unlisten = await onLlmDelta(handleDelta);
});

onUnmounted(() => {
  if (unlisten) unlisten();
});

function handleDelta(ev: LlmDeltaEvent) {
  // Filter to the session we started.
  if (currentSession === null || ev.session_id !== currentSession) {
    return;
  }
  rawEvents.value = [...rawEvents.value, ev];

  switch (ev.kind) {
    case 'text':
      finalText.value += ev.text;
      break;
    case 'tool_begin':
      toolEntries.value = [
        ...toolEntries.value,
        {
          id: ev.id,
          name: ev.name,
          detail: ev.detail,
          status: 'running',
          summary: null,
          output: '',
        },
      ];
      break;
    case 'tool_output': {
      const idx = toolEntries.value.findIndex((t) => t.id === ev.id);
      if (idx >= 0) {
        const next = [...toolEntries.value];
        next[idx] = { ...next[idx], output: next[idx].output + ev.chunk };
        toolEntries.value = next;
      }
      break;
    }
    case 'tool_end': {
      const idx = toolEntries.value.findIndex((t) => t.id === ev.id);
      if (idx >= 0) {
        const next = [...toolEntries.value];
        next[idx] = {
          ...next[idx],
          status: ev.status,
          summary: ev.summary,
        };
        toolEntries.value = next;
      }
      break;
    }
    case 'usage':
      usage.value = {
        input_tokens: ev.input_tokens,
        cached_input_tokens: ev.cached_input_tokens,
        output_tokens: ev.output_tokens,
        cost_usd: ev.cost_usd,
      };
      break;
    case 'error':
      errorMsg.value = ev.message;
      isRunning.value = false;
      break;
    case 'cancelled':
    case 'completed':
    case 'message_end':
      isRunning.value = false;
      break;
    default:
      break;
  }
}

async function onModelChange(id: string) {
  try {
    activeModelId.value = id;
    await setActiveModel(id);
  } catch (e) {
    errorMsg.value = `set model: ${e}`;
  }
}

async function loadFixture(name: string) {
  const path = `tests/fixtures/pob/${name}.xml`;
  await loadBuildPath(path);
}

async function loadBuildPath(path: string) {
  if (!path) return;
  try {
    const built = await setActiveBuild(path);
    activeBuild.value = built;
    fixturePath.value = path;
  } catch (e) {
    errorMsg.value = `load build: ${e}`;
  }
}

async function clearBuild() {
  try {
    await clearActiveBuild();
    activeBuild.value = null;
    fixturePath.value = '';
  } catch (e) {
    errorMsg.value = `clear build: ${e}`;
  }
}

async function run() {
  if (isRunning.value || !prompt.value.trim()) return;
  errorMsg.value = null;
  finalText.value = '';
  toolEntries.value = [];
  rawEvents.value = [];
  usage.value = null;

  try {
    await chatReset();
    isRunning.value = true;
    currentSession = await chatStart(prompt.value, []);
  } catch (e) {
    isRunning.value = false;
    errorMsg.value = `chat_start: ${e}`;
  }
}

async function cancel() {
  try {
    await chatCancel();
  } catch (e) {
    errorMsg.value = `cancel: ${e}`;
  }
}

const renderedHtml = computed(() => {
  const game: 'poe1' | 'poe2' = activeBuild.value?.game === 'poe2' ? 'poe2' : 'poe1';
  return renderMarkdown(finalText.value, game, new Set());
});

const fmtTokens = (n: number) =>
  n >= 1000 ? `${(n / 1000).toFixed(1)}k` : `${n}`;
</script>

<template>
  <div class="lt-tab">
    <header class="lt-bar">
      <div class="lt-field">
        <label>Model</label>
        <select :value="activeModelId" @change="onModelChange(($event.target as HTMLSelectElement).value)">
          <option v-for="m in models" :key="m.id" :value="m.id">
            {{ m.display_name || m.id }}
          </option>
        </select>
      </div>

      <div class="lt-field">
        <label>PoB fixture</label>
        <select @change="loadFixture(($event.target as HTMLSelectElement).value)">
          <option value="" disabled selected>Pick a fixture…</option>
          <option v-for="f in KNOWN_FIXTURES" :key="f.id" :value="f.id">{{ f.label }}</option>
        </select>
        <input
          v-model="fixturePath"
          class="lt-fixture-path"
          type="text"
          placeholder="…or path to .xml"
          @keydown.enter="loadBuildPath(fixturePath)"
        />
        <button class="lt-btn" @click="loadBuildPath(fixturePath)">Load</button>
        <button class="lt-btn lt-btn--ghost" @click="clearBuild" :disabled="!activeBuild">
          Clear
        </button>
      </div>

      <div v-if="activeBuild" class="lt-build-pill">
        active: {{ activeBuild.class }} · {{ activeBuild.game }}
        <span v-if="activeBuild.level">· lvl {{ activeBuild.level }}</span>
      </div>
    </header>

    <main class="lt-main">
      <section class="lt-prompt">
        <textarea
          v-model="prompt"
          class="lt-prompt__input"
          placeholder="Type a prompt to test…"
          rows="6"
          :disabled="isRunning"
        />
        <div class="lt-prompt__actions">
          <button v-if="!isRunning" class="lt-btn lt-btn--primary" @click="run" :disabled="!prompt.trim()">
            ▶ Run
          </button>
          <button v-else class="lt-btn lt-btn--danger" @click="cancel">⨯ Cancel</button>
          <span v-if="isRunning" class="lt-running">running…</span>
          <span v-if="errorMsg" class="lt-err">{{ errorMsg }}</span>
        </div>
      </section>

      <section class="lt-right">
        <nav class="lt-right-tabs">
          <button
            class="lt-rt"
            :class="{ 'lt-rt--active': rightPane === 'rendered' }"
            @click="rightPane = 'rendered'"
          >
            Rendered
          </button>
          <button
            class="lt-rt"
            :class="{ 'lt-rt--active': rightPane === 'raw' }"
            @click="rightPane = 'raw'"
          >
            Raw events ({{ rawEvents.length }})
          </button>
          <button
            class="lt-rt"
            :class="{ 'lt-rt--active': rightPane === 'stats' }"
            @click="rightPane = 'stats'"
          >
            Stats
          </button>
        </nav>

        <div class="lt-right__body">
          <div v-if="rightPane === 'rendered'" class="lt-rendered">
            <div v-if="toolEntries.length" class="lt-tools">
              <div v-for="t in toolEntries" :key="t.id" class="lt-tool">
                <span class="lt-tool__name">{{ t.name }}</span>
                <span class="lt-tool__status" :data-status="t.status">{{ t.status }}</span>
                <span v-if="t.summary" class="lt-tool__summary">{{ t.summary }}</span>
              </div>
            </div>
            <div v-if="finalText" class="lt-md" v-html="renderedHtml" />
            <p v-else-if="!isRunning" class="lt-empty">No response yet.</p>
          </div>

          <pre v-else-if="rightPane === 'raw'" class="lt-raw">{{ JSON.stringify(rawEvents, null, 2) }}</pre>

          <div v-else-if="rightPane === 'stats'" class="lt-stats">
            <table v-if="usage">
              <tbody>
                <tr>
                  <th>Input tokens</th>
                  <td>{{ fmtTokens(usage.input_tokens + usage.cached_input_tokens) }}</td>
                </tr>
                <tr>
                  <th>Cached</th>
                  <td>{{ fmtTokens(usage.cached_input_tokens) }}</td>
                </tr>
                <tr>
                  <th>Output tokens</th>
                  <td>{{ fmtTokens(usage.output_tokens) }}</td>
                </tr>
                <tr v-if="usage.cost_usd !== null">
                  <th>Cost</th>
                  <td>${{ usage.cost_usd?.toFixed(4) }}</td>
                </tr>
                <tr>
                  <th>Tool calls</th>
                  <td>{{ toolEntries.length }}</td>
                </tr>
                <tr>
                  <th>Stream events</th>
                  <td>{{ rawEvents.length }}</td>
                </tr>
              </tbody>
            </table>
            <p v-else class="lt-empty">Run something to see stats.</p>
          </div>
        </div>
      </section>
    </main>
  </div>
</template>

<style scoped>
.lt-tab {
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100%;
}
.lt-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  padding: 12px;
  background: #141417;
  border: 1px solid #2a2a30;
  border-radius: 6px;
}
.lt-field {
  display: flex;
  align-items: center;
  gap: 6px;
}
.lt-field label {
  font-size: 11px;
  color: #75757f;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
.lt-field select,
.lt-fixture-path {
  background: #0a0a0c;
  border: 1px solid #2a2a30;
  color: #e8e6df;
  padding: 4px 8px;
  border-radius: 3px;
  font: inherit;
  font-size: 12px;
}
.lt-fixture-path { width: 220px; }
.lt-btn {
  background: #1a1a1f;
  border: 1px solid #2a2a30;
  color: #c9c9d0;
  padding: 4px 10px;
  border-radius: 3px;
  cursor: pointer;
  font: inherit;
  font-size: 12px;
}
.lt-btn:hover:not(:disabled) {
  background: #22222a;
}
.lt-btn--primary {
  background: #2a3f2a;
  border-color: #4a6a4a;
  color: #b4d4b4;
}
.lt-btn--primary:hover:not(:disabled) { background: #355035; }
.lt-btn--danger {
  background: #3f2a2a;
  border-color: #6a4a4a;
  color: #d4b4b4;
}
.lt-btn--ghost { background: transparent; }
.lt-build-pill {
  margin-left: auto;
  font-size: 11px;
  color: #c9a86a;
  background: #0a0a0c;
  padding: 4px 10px;
  border-radius: 3px;
}
.lt-main {
  display: grid;
  grid-template-columns: 360px 1fr;
  gap: 12px;
  flex: 1;
  min-height: 0;
}
.lt-prompt {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.lt-prompt__input {
  background: #141417;
  border: 1px solid #2a2a30;
  color: #e8e6df;
  padding: 10px;
  border-radius: 6px;
  font: inherit;
  font-size: 13px;
  resize: vertical;
  min-height: 120px;
}
.lt-prompt__input:focus {
  outline: 1px solid #c9a86a;
  border-color: #c9a86a;
}
.lt-prompt__actions {
  display: flex;
  align-items: center;
  gap: 12px;
}
.lt-running {
  font-size: 11px;
  color: #c9a86a;
  font-style: italic;
}
.lt-err {
  font-size: 11px;
  color: #d84a4a;
}
.lt-right {
  display: flex;
  flex-direction: column;
  background: #141417;
  border: 1px solid #2a2a30;
  border-radius: 6px;
  overflow: hidden;
  min-height: 0;
}
.lt-right-tabs {
  display: flex;
  background: #0a0a0c;
  border-bottom: 1px solid #2a2a30;
}
.lt-rt {
  background: transparent;
  border: none;
  color: #75757f;
  padding: 8px 14px;
  font: inherit;
  font-size: 12px;
  cursor: pointer;
}
.lt-rt:hover { color: #c9c9d0; }
.lt-rt--active {
  color: #e8e6df;
  background: #141417;
  position: relative;
}
.lt-rt--active::after {
  content: '';
  position: absolute;
  left: 12px; right: 12px; bottom: -1px;
  height: 2px;
  background: #c9a86a;
}
.lt-right__body {
  flex: 1;
  overflow: auto;
  padding: 16px;
}
.lt-tools {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 12px;
  padding-bottom: 12px;
  border-bottom: 1px solid #2a2a30;
}
.lt-tool {
  display: flex;
  gap: 8px;
  align-items: center;
  font-size: 11px;
}
.lt-tool__name { color: #c9a86a; font-family: var(--font-mono, monospace); }
.lt-tool__status { color: #75757f; text-transform: lowercase; }
.lt-tool__status[data-status="done"] { color: #75c075; }
.lt-tool__status[data-status="failed"] { color: #d84a4a; }
.lt-tool__summary { color: #a8a8b0; font-style: italic; }
.lt-md {
  color: #e8e6df;
  font-size: 14px;
  line-height: 1.6;
  white-space: normal;
}
.lt-md :deep(p) { margin: 0 0 8px 0; }
.lt-md :deep(code) {
  background: #0a0a0c;
  color: #c9a86a;
  padding: 1px 4px;
  border-radius: 2px;
  font-size: 12px;
}
.lt-md :deep(pre) {
  background: #0a0a0c;
  border: 1px solid #2a2a30;
  padding: 8px;
  border-radius: 4px;
  overflow-x: auto;
}
.lt-md :deep(a) { color: #6fa8d8; }
.lt-empty {
  color: #50505a;
  font-style: italic;
  text-align: center;
  padding: 32px 0;
}
.lt-raw {
  font-family: var(--font-mono, monospace);
  font-size: 11px;
  color: #c9c9d0;
  white-space: pre-wrap;
  margin: 0;
}
.lt-stats table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}
.lt-stats th {
  text-align: left;
  color: #75757f;
  font-weight: 400;
  padding: 6px 12px;
  border-bottom: 1px solid #2a2a30;
  width: 40%;
}
.lt-stats td {
  padding: 6px 12px;
  border-bottom: 1px solid #2a2a30;
  font-family: var(--font-mono, monospace);
  color: #c9a86a;
  font-variant-numeric: tabular-nums;
}
</style>

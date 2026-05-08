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
  if (currentSession === null || ev.session_id !== currentSession) return;
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
        next[idx] = { ...next[idx], status: ev.status, summary: ev.summary };
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

const promptTokens = computed(() => Math.ceil(prompt.value.length / 4));
const hasResponse = computed(
  () => finalText.value.length > 0 || toolEntries.value.length > 0
);
const stateLabel = computed(() => {
  if (errorMsg.value) return 'error';
  if (isRunning.value) return 'streaming…';
  if (hasResponse.value) return 'completed';
  return 'idle';
});
</script>

<template>
  <div class="lt">
    <header class="lt__bar">
      <span class="lt__label">Model</span>
      <div class="lt__select">
        <select :value="activeModelId" @change="onModelChange(($event.target as HTMLSelectElement).value)">
          <option v-for="m in models" :key="m.id" :value="m.id">
            {{ m.display_name || m.id }}
          </option>
        </select>
      </div>

      <span class="lt__divider" />

      <span class="lt__label">PoB fixture</span>
      <div class="lt__select">
        <select @change="loadFixture(($event.target as HTMLSelectElement).value)">
          <option value="" disabled selected>Pick a fixture…</option>
          <option v-for="f in KNOWN_FIXTURES" :key="f.id" :value="f.id">{{ f.label }}</option>
        </select>
      </div>
      <input
        v-model="fixturePath"
        class="lt__input-mono"
        type="text"
        placeholder="…or path to .xml"
        @keydown.enter="loadBuildPath(fixturePath)"
      />
      <button class="lt__btn" @click="loadBuildPath(fixturePath)">Load</button>
      <button class="lt__btn" @click="clearBuild" :disabled="!activeBuild">Clear</button>

      <span class="lt__spacer" />

      <span v-if="activeBuild" class="lt__active-pill">
        <span class="lt__active-tag">Active</span>
        <span class="lt__active-text">
          {{ activeBuild.class }} · {{ activeBuild.game }}
          <span v-if="activeBuild.level"> · lvl {{ activeBuild.level }}</span>
        </span>
      </span>
    </header>

    <main class="lt__body">
      <section class="lt__composer">
        <div class="lt__sec-head">
          <span class="lt__caps">Prompt</span>
          <span class="lt__spacer" />
          <span class="lt__token-count">{{ promptTokens }} / 8 000 tokens</span>
        </div>
        <textarea
          v-model="prompt"
          class="lt__textarea"
          placeholder="Type a prompt to test…"
          :disabled="isRunning"
        />
        <div class="lt__actions">
          <button v-if="!isRunning" type="button" class="lt__run" @click="run" :disabled="!prompt.trim()">
            <span class="lt__run-arrow">▶</span>
            <span>Run</span>
          </button>
          <button v-else type="button" class="lt__cancel" @click="cancel">
            <span>×</span><span>Cancel</span>
          </button>
          <span class="lt__spacer" />
          <span class="lt__send-hint">
            <kbd>Ctrl</kbd><kbd>⏎</kbd><span>send</span>
          </span>
        </div>

        <div v-if="errorMsg" class="lt__err">{{ errorMsg }}</div>
      </section>

      <section class="lt__viewer">
        <nav class="lt__viewer-tabs">
          <button
            type="button"
            class="lt__vt"
            :class="{ 'lt__vt--active': rightPane === 'rendered' }"
            @click="rightPane = 'rendered'"
          >Rendered</button>
          <button
            type="button"
            class="lt__vt"
            :class="{ 'lt__vt--active': rightPane === 'raw' }"
            @click="rightPane = 'raw'"
          >
            Raw events
            <span class="lt__vt-count">({{ rawEvents.length }})</span>
          </button>
          <button
            type="button"
            class="lt__vt"
            :class="{ 'lt__vt--active': rightPane === 'stats' }"
            @click="rightPane = 'stats'"
          >Stats</button>
          <span class="lt__spacer" />
          <span class="lt__viewer-state">
            {{ hasResponse ? 'Response captured' : 'No response yet' }} · {{ stateLabel }}
          </span>
        </nav>

        <div class="lt__viewer-body">
          <div v-if="rightPane === 'rendered'" class="lt__pane">
            <div v-if="toolEntries.length" class="lt__tools">
              <div v-for="t in toolEntries" :key="t.id" class="lt__tool">
                <span class="lt__tool-name">{{ t.name }}</span>
                <span class="lt__tool-status" :data-status="t.status">{{ t.status }}</span>
                <span v-if="t.summary" class="lt__tool-summary">{{ t.summary }}</span>
              </div>
            </div>
            <div v-if="finalText" class="lt__md" v-html="renderedHtml" />
            <div v-else-if="!isRunning" class="lt__empty">
              <div class="lt__empty-circle">
                <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                  <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
                  <circle cx="12" cy="12" r="3" />
                </svg>
              </div>
              <div class="lt__empty-title">No response yet</div>
              <div class="lt__empty-hint">
                Pick a model, optionally load a Path of Building fixture, type a prompt and hit
                <em>Run</em>. Both the rendered markdown and the raw
                <code>LlmDelta</code> stream will be captured side by side.
              </div>
            </div>
          </div>

          <pre v-else-if="rightPane === 'raw'" class="lt__raw">{{ JSON.stringify(rawEvents, null, 2) }}</pre>

          <div v-else-if="rightPane === 'stats'" class="lt__stats">
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
            <div v-else class="lt__empty">
              <div class="lt__empty-title">No usage yet</div>
              <div class="lt__empty-hint">Run something to see token counts and cost.</div>
            </div>
          </div>
        </div>
      </section>
    </main>
  </div>
</template>

<style scoped>
.lt {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--paper);
  color: var(--ink);
  font-family: var(--hand);
  min-height: 0;
}

.lt__bar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
  padding: 14px 22px;
  border-bottom: 1px solid var(--paper-line);
  background: var(--paper);
}

.lt__label {
  font-family: var(--label);
  font-size: 11.5px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 600;
}

.lt__select {
  display: inline-flex;
  align-items: center;
  padding: 0;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 4px;
  position: relative;
}
.lt__select:focus-within {
  border-color: var(--amber);
}
.lt__select select {
  padding: 7px 10px;
  background: transparent;
  border: none;
  outline: none;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink);
  font-weight: 600;
  cursor: pointer;
}
.lt__select select:disabled {
  color: var(--ink-faint);
}

.lt__input-mono {
  padding: 7px 10px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 4px;
  font-family: var(--mono);
  font-size: 13px;
  color: var(--ink);
  min-width: 220px;
  outline: none;
}
.lt__input-mono:focus {
  border-color: var(--amber);
}

.lt__btn {
  padding: 7px 14px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 4px;
  color: var(--ink);
  font-family: var(--hand);
  font-size: 13.5px;
  font-weight: 500;
  cursor: pointer;
  transition: border-color 100ms ease;
}
.lt__btn:hover:not(:disabled) {
  border-color: var(--amber);
}
.lt__btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.lt__divider {
  width: 1px;
  height: 22px;
  background: var(--paper-line);
}

.lt__spacer {
  flex: 1;
}

.lt__active-pill {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border: 1px solid var(--amber);
  background: var(--amber-glow);
  border-radius: 4px;
  font-size: 13.5px;
}
.lt__active-tag {
  font-family: var(--label);
  font-size: 10.5px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--amber);
  font-weight: 700;
}
.lt__active-text {
  color: var(--ink);
  font-weight: 600;
}

.lt__body {
  flex: 1;
  display: grid;
  grid-template-columns: 420px 1fr;
  min-height: 0;
}

.lt__composer {
  border-right: 1px solid var(--paper-line);
  padding: 20px 22px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-height: 0;
}

.lt__sec-head {
  display: flex;
  align-items: center;
}

.lt__caps {
  font-family: var(--label);
  font-size: 11.5px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 600;
}

.lt__token-count {
  font-size: 12.5px;
  color: var(--ink-faint);
  font-variant-numeric: tabular-nums;
}

.lt__textarea {
  flex: 1;
  min-height: 220px;
  padding: 14px 16px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 5px;
  font-family: var(--hand);
  font-size: 15.5px;
  color: var(--ink);
  line-height: 1.5;
  resize: none;
  outline: none;
}
.lt__textarea:focus {
  border-color: var(--amber);
}
.lt__textarea::placeholder {
  color: var(--ink-faint);
}

.lt__actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.lt__run {
  display: inline-flex;
  align-items: center;
  gap: 9px;
  padding: 11px 22px;
  background: var(--good);
  color: var(--paper);
  border: 1px solid var(--good);
  border-radius: 4px;
  font-family: var(--hand);
  font-size: 16px;
  font-weight: 700;
  letter-spacing: 0.01em;
  cursor: pointer;
  transition: filter 100ms ease;
}
.lt__run:hover:not(:disabled) {
  filter: brightness(1.08);
}
.lt__run:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.lt__run-arrow {
  font-size: 12px;
}

.lt__cancel {
  display: inline-flex;
  align-items: center;
  gap: 9px;
  padding: 11px 22px;
  background: var(--paper);
  color: var(--bad);
  border: 1px solid var(--bad);
  border-radius: 4px;
  font-family: var(--hand);
  font-size: 16px;
  font-weight: 700;
  cursor: pointer;
}

.lt__send-hint {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--ink-soft);
}
.lt__send-hint kbd {
  font-family: var(--label);
  font-size: 11px;
  padding: 1px 5px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  color: var(--ink-soft);
  background: var(--paper);
  line-height: 1.2;
}

.lt__err {
  margin: 0;
  padding: 8px 12px;
  border: 1px solid var(--bad);
  background: rgba(162, 58, 58, 0.06);
  border-radius: 4px;
  color: var(--bad);
  font-size: 13px;
}

.lt__viewer {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.lt__viewer-tabs {
  display: flex;
  align-items: stretch;
  background: var(--paper);
  border-bottom: 1px solid var(--paper-line);
  padding: 0 22px;
  gap: 4px;
}

.lt__vt {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 12px 14px 10px;
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--ink-soft);
  font-family: var(--hand);
  font-size: 14.5px;
  font-weight: 500;
  cursor: pointer;
  transition: color 100ms ease, border-color 100ms ease;
}
.lt__vt:hover {
  color: var(--ink);
}
.lt__vt--active {
  border-bottom-color: var(--amber);
  color: var(--ink);
  font-weight: 700;
}
.lt__vt-count {
  font-family: var(--mono);
  font-size: 11.5px;
  color: var(--ink-faint);
  font-weight: 500;
}

.lt__viewer-state {
  align-self: center;
  font-size: 13px;
  color: var(--ink-soft);
}

.lt__viewer-body {
  flex: 1;
  overflow: auto;
  min-height: 0;
  padding: 24px 28px;
  background: var(--paper);
}

.lt__pane {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.lt__tools {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-bottom: 12px;
  border-bottom: 1px dashed var(--paper-line);
}

.lt__tool {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
}
.lt__tool-name {
  font-family: var(--mono);
  color: var(--ink);
  font-weight: 600;
}
.lt__tool-status {
  font-family: var(--label);
  font-size: 10.5px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  font-weight: 700;
  padding: 1px 6px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  color: var(--ink-faint);
}
.lt__tool-status[data-status="done"]    { color: var(--good);  border-color: var(--good); }
.lt__tool-status[data-status="failed"]  { color: var(--bad);   border-color: var(--bad); }
.lt__tool-status[data-status="running"] { color: var(--amber); border-color: var(--amber); }
.lt__tool-summary {
  font-style: italic;
  color: var(--ink-soft);
}

.lt__md {
  font-size: 15px;
  line-height: 1.6;
  color: var(--ink);
}
.lt__md :deep(p) { margin: 0 0 10px 0; }
.lt__md :deep(code) {
  font-family: var(--mono);
  background: var(--paper-shade);
  padding: 1px 5px;
  border-radius: 3px;
  font-size: 13px;
  color: var(--ink);
}
.lt__md :deep(pre) {
  background: var(--paper-shade);
  border: 1px solid var(--paper-line);
  padding: 10px 12px;
  border-radius: 4px;
  overflow-x: auto;
  font-family: var(--mono);
  font-size: 13px;
}
.lt__md :deep(a) {
  color: var(--amber);
  text-decoration: underline;
  text-decoration-style: dotted;
  text-underline-offset: 3px;
}

.lt__empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  text-align: center;
  color: var(--ink-faint);
}
.lt__empty-circle {
  width: 72px;
  height: 72px;
  border: 1.5px dashed var(--ink-faint);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--ink-faint);
}
.lt__empty-title {
  font-size: 18px;
  color: var(--ink-soft);
  font-weight: 600;
}
.lt__empty-hint {
  font-size: 14.5px;
  line-height: 1.55;
  color: var(--ink-soft);
  max-width: 360px;
}
.lt__empty-hint em {
  font-style: normal;
  font-weight: 700;
  color: var(--ink);
}
.lt__empty-hint code {
  font-family: var(--mono);
  background: var(--paper-shade);
  padding: 1px 5px;
  border-radius: 3px;
  color: var(--ink);
}

.lt__raw {
  font-family: var(--mono);
  font-size: 12.5px;
  color: var(--ink);
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
}

.lt__stats table {
  width: 100%;
  border-collapse: collapse;
  font-size: 14px;
}
.lt__stats th {
  text-align: left;
  font-family: var(--label);
  font-size: 11.5px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 600;
  padding: 8px 14px;
  border-bottom: 1px dotted var(--paper-line);
  width: 40%;
}
.lt__stats td {
  padding: 8px 14px;
  border-bottom: 1px dotted var(--paper-line);
  font-family: var(--mono);
  font-size: 14px;
  color: var(--ink);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
</style>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

import {
  deleteAllDebugRuns,
  deleteDebugRun,
  getDebugRun,
  lintDebugRun,
  listDebugRuns,
} from '../../../api/tauri';
import type { DebugRunDto, DebugRunStats, LintFinding, LintReport } from '../../../api/types';
import type { ChatMessageVm, Segment } from '../../../stores/chat';
import ChatMessage from '../../../components/chat/ChatMessage.vue';
import { useToastsStore } from '../../../stores/toasts';

const toasts = useToastsStore();

const runs = ref<DebugRunDto[]>([]);
const selected = ref<DebugRunDto | null>(null);
const loading = ref(false);
const filter = ref('');
// Lint reports cached per run id. Populated lazily from `lint_debug_run`
// after the run list refreshes; the sidebar reads it for the FAIL/WARN
// badges and the main pane reads it for the per-finding list.
const lintReports = ref<Map<string, LintReport>>(new Map());
// Which model ids the user has explicitly toggled OFF. Default empty =
// everything visible, including any model that appears in the future.
const hiddenModels = ref<Set<string>>(new Set());

interface ModelTab {
  id: string;
  display: string;
  count: number;
  enabled: boolean;
}

const modelTabs = computed<ModelTab[]>(() => {
  const counts = new Map<string, { display: string; count: number }>();
  for (const r of runs.value) {
    const cur = counts.get(r.model_id);
    if (cur) cur.count += 1;
    else counts.set(r.model_id, { display: r.model_display_name, count: 1 });
  }
  const list: ModelTab[] = [];
  for (const [id, v] of counts) {
    list.push({
      id,
      display: v.display,
      count: v.count,
      enabled: !hiddenModels.value.has(id),
    });
  }
  list.sort((a, b) => a.id.localeCompare(b.id));
  return list;
});

const filteredRuns = computed(() => {
  const q = filter.value.trim().toLowerCase();
  return runs.value.filter((r) => {
    if (hiddenModels.value.has(r.model_id)) return false;
    if (!q) return true;
    return (
      r.user_text.toLowerCase().includes(q) ||
      r.model_display_name.toLowerCase().includes(q) ||
      r.model_id.toLowerCase().includes(q) ||
      r.source.toLowerCase().includes(q)
    );
  });
});

function toggleModel(id: string) {
  const next = new Set(hiddenModels.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  hiddenModels.value = next;
}

function showAllModels() {
  hiddenModels.value = new Set();
}

function showOnlyModel(id: string) {
  const next = new Set<string>();
  for (const t of modelTabs.value) {
    if (t.id !== id) next.add(t.id);
  }
  hiddenModels.value = next;
}

const visibleModelCount = computed(
  () => modelTabs.value.filter((t) => t.enabled).length,
);

async function refresh() {
  loading.value = true;
  try {
    runs.value = await listDebugRuns();
    if (!selected.value && runs.value.length > 0) {
      await select(runs.value[0]);
    } else if (selected.value && !runs.value.find((r) => r.id === selected.value!.id)) {
      selected.value = null;
    }
    void refreshLints();
  } catch (e) {
    toasts.push({ variant: 'error', title: 'Debug runs', body: String(e) });
  } finally {
    loading.value = false;
  }
}

async function refreshLints() {
  // Compute lint reports for every run that we have not yet cached. Runs
  // are processed in parallel but with a small concurrency window so the
  // dev panel does not freeze on a fresh import of hundreds of runs.
  const pending = runs.value.filter((r) => !lintReports.value.has(r.id));
  if (pending.length === 0) return;
  const next = new Map(lintReports.value);
  const concurrency = 6;
  for (let i = 0; i < pending.length; i += concurrency) {
    const batch = pending.slice(i, i + concurrency);
    const results = await Promise.all(
      batch.map((r) => lintDebugRun(r.id).catch(() => null)),
    );
    for (let j = 0; j < batch.length; j += 1) {
      const r = batch[j];
      const report = results[j];
      if (report) next.set(r.id, report);
    }
  }
  lintReports.value = next;
}

function findingsForRun(id: string): LintFinding[] {
  return lintReports.value.get(id)?.findings ?? [];
}

function failCountForRun(id: string): number {
  return findingsForRun(id).filter((f) => f.severity === 'fail').length;
}

function warnCountForRun(id: string): number {
  return findingsForRun(id).filter((f) => f.severity === 'warn').length;
}

async function select(run: DebugRunDto) {
  try {
    const full = await getDebugRun(run.id);
    selected.value = full ?? run;
  } catch (e) {
    toasts.push({ variant: 'error', title: 'Open run', body: String(e) });
  }
}

async function removeOne(run: DebugRunDto, ev: Event) {
  ev.stopPropagation();
  try {
    await deleteDebugRun(run.id);
    if (selected.value?.id === run.id) selected.value = null;
    await refresh();
  } catch (e) {
    toasts.push({ variant: 'error', title: 'Delete failed', body: String(e) });
  }
}

async function removeAll() {
  if (!confirm('Delete every debug run? This cannot be undone.')) return;
  try {
    const n = await deleteAllDebugRuns();
    selected.value = null;
    await refresh();
    toasts.push({ variant: 'info', title: 'Cleared', body: `${n} run(s) deleted.` });
  } catch (e) {
    toasts.push({ variant: 'error', title: 'Clear failed', body: String(e) });
  }
}

const exporting = ref(false);
async function exportAll() {
  if (exporting.value || runs.value.length === 0) return;
  exporting.value = true;
  try {
    const path = await invoke<string>('dev_export_all_runs');
    toasts.push({
      variant: 'info',
      title: `Exported ${runs.value.length} run(s)`,
      body: path,
    });
  } catch (e) {
    toasts.push({ variant: 'error', title: 'Export failed', body: String(e) });
  } finally {
    exporting.value = false;
  }
}

onMounted(refresh);

function fmtTime(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

function fmtElapsed(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function hasCacheStats(stats: DebugRunStats): boolean {
  return (
    (stats.input_tokens ?? 0) +
      (stats.cached_input_tokens ?? 0) +
      (stats.cache_creation_tokens ?? 0) +
      (stats.output_tokens ?? 0) >
    0
  );
}

function cacheHitPct(stats: DebugRunStats): string {
  const fresh = stats.input_tokens ?? 0;
  const cached = stats.cached_input_tokens ?? 0;
  const total = fresh + cached;
  if (total === 0) return '—';
  return `${Math.round((cached / total) * 100)}%`;
}

function truncate(s: string, max: number): string {
  if (s.length <= max) return s;
  return s.slice(0, max - 1) + '…';
}

function segmentsFromRun(run: DebugRunDto): Segment[] {
  const out: Segment[] = [];
  let i = 0;
  for (const s of run.assistant_segments) {
    if (s.kind === 'text') {
      out.push({ kind: 'text', id: `dbg_t_${i++}`, text: s.text });
    } else if (s.kind === 'reasoning') {
      out.push({ kind: 'reasoning', id: `dbg_r_${i++}`, text: s.text, open: true });
    } else if (s.kind === 'tool') {
      out.push({
        kind: 'tool',
        id: s.id,
        name: s.name,
        detail: s.detail,
        status: s.status,
        summary: s.summary,
        output: s.outputs.join(''),
      });
    }
  }
  return out;
}

interface ToolDetail {
  id: string;
  name: string;
  detail: string | null;
  outputs: string[];
  status: 'running' | 'done' | 'failed';
  summary: string | null;
}

const toolSegments = computed<ToolDetail[]>(() => {
  const r = selected.value;
  if (!r) return [];
  const out: ToolDetail[] = [];
  for (const s of r.assistant_segments) {
    if (s.kind === 'tool') {
      out.push({
        id: s.id,
        name: s.name,
        detail: s.detail,
        outputs: s.outputs,
        status: s.status,
        summary: s.summary,
      });
    }
  }
  return out;
});

const messagesForSelected = computed<ChatMessageVm[]>(() => {
  const r = selected.value;
  if (!r) return [];
  const userMsg: ChatMessageVm = {
    id: `dbg_user_${r.id}`,
    role: 'user',
    segments: [{ kind: 'text', id: `dbg_user_seg_${r.id}`, text: r.user_text }],
    attachments: r.user_attachments.map((a) => ({
      name: a.name,
      mime: a.mime,
      data_base64: '',
    })),
    status: 'complete',
    sessionId: null,
    errorMessage: null,
  };
  const asstMsg: ChatMessageVm = {
    id: `dbg_asst_${r.id}`,
    role: 'assistant',
    segments: segmentsFromRun(r),
    attachments: [],
    status: r.error ? 'error' : 'complete',
    sessionId: null,
    errorMessage: r.error,
  };
  return [userMsg, asstMsg];
});
</script>

<template>
  <div class="debug">
    <aside class="debug__side">
      <header class="debug__side-head">
        <span class="debug__caps">debug · runs</span>
        <span class="debug__count" v-if="runs.length">{{ filteredRuns.length }} / {{ runs.length }}</span>
      </header>
      <div v-if="modelTabs.length > 0" class="debug__models">
        <div class="debug__models-head">
          <span class="debug__caps">models</span>
          <button
            v-if="visibleModelCount < modelTabs.length"
            class="debug__pill-all"
            type="button"
            @click="showAllModels"
            title="Show every model"
          >show all</button>
        </div>
        <div class="debug__pills">
          <button
            v-for="t in modelTabs"
            :key="t.id"
            type="button"
            class="debug__pill"
            :class="{ 'debug__pill--off': !t.enabled }"
            :title="t.enabled ? `${t.display}  ·  shift-click for solo` : `Hidden — click to show`"
            @click="(e) => (e.shiftKey ? showOnlyModel(t.id) : toggleModel(t.id))"
          >
            <span class="debug__pill-name">{{ t.display }}</span>
            <span class="debug__pill-count">{{ t.count }}</span>
          </button>
        </div>
      </div>
      <div class="debug__filter">
        <input
          v-model="filter"
          class="debug__input"
          type="text"
          placeholder="filter by model, prompt, source…"
        />
        <button class="debug__refresh" @click="refresh" :disabled="loading">↻</button>
      </div>
      <ul class="debug__list">
        <li
          v-for="r in filteredRuns"
          :key="r.id"
          class="debug__item"
          :class="{ 'debug__item--active': selected?.id === r.id }"
          @click="select(r)"
        >
          <div class="debug__row">
            <span class="debug__model">{{ r.model_display_name }}</span>
            <span class="debug__src" :class="`debug__src--${r.source}`">{{ r.source }}</span>
          </div>
          <p class="debug__prompt">{{ truncate(r.user_text, 120) }}</p>
          <div class="debug__meta">
            <span>{{ fmtTime(r.started_at) }}</span>
            <span class="debug__dot">·</span>
            <span>{{ fmtElapsed(r.stats.elapsed_ms) }}</span>
            <span class="debug__dot">·</span>
            <span>{{ r.stats.tool_calls }}🔧</span>
            <span class="debug__dot">·</span>
            <span>{{ r.stats.text_bytes }}b</span>
            <span
              v-if="failCountForRun(r.id) > 0"
              class="debug__lint-pill debug__lint-pill--fail"
              :title="`${failCountForRun(r.id)} lint FAIL`"
            >{{ failCountForRun(r.id) }}F</span>
            <span
              v-if="warnCountForRun(r.id) > 0"
              class="debug__lint-pill debug__lint-pill--warn"
              :title="`${warnCountForRun(r.id)} lint WARN`"
            >{{ warnCountForRun(r.id) }}W</span>
            <span v-if="r.error" class="debug__bad">err</span>
            <button class="debug__del" @click="removeOne(r, $event)" title="delete">×</button>
          </div>
        </li>
        <li v-if="runs.length === 0" class="debug__empty">
          No debug runs yet. Start a chat or run <code>cargo run --example ollama_smoke</code>.
        </li>
      </ul>
      <footer class="debug__side-foot">
        <button
          class="debug__btn"
          @click="exportAll"
          :disabled="runs.length === 0 || exporting"
          :title="`Write all ${runs.length} run(s) to a single JSON under ~/.bestel/exports/`"
        >
          {{ exporting ? 'exporting…' : 'export all' }}
        </button>
        <button
          class="debug__btn debug__btn--danger"
          @click="removeAll"
          :disabled="runs.length === 0"
        >
          clear all
        </button>
      </footer>
    </aside>

    <section class="debug__main">
      <template v-if="selected">
        <header class="debug__head">
          <div class="debug__head-row">
            <span class="debug__caps">model</span>
            <span class="debug__head-val">{{ selected.model_display_name }}</span>
            <span class="debug__caps">id</span>
            <span class="debug__head-mono">{{ selected.model_id }}</span>
            <span class="debug__caps">provider</span>
            <span class="debug__head-mono">{{ selected.provider }}</span>
            <span class="debug__caps">source</span>
            <span class="debug__head-mono">{{ selected.source }}</span>
          </div>
          <div class="debug__head-row debug__head-row--stats">
            <span class="debug__caps">started</span>
            <span class="debug__head-mono">{{ fmtTime(selected.started_at) }}</span>
            <span class="debug__caps">elapsed</span>
            <span class="debug__head-mono">{{ fmtElapsed(selected.stats.elapsed_ms) }}</span>
            <span class="debug__caps">text</span>
            <span class="debug__head-mono">{{ selected.stats.text_bytes }}b</span>
            <span class="debug__caps">reasoning</span>
            <span class="debug__head-mono">{{ selected.stats.reasoning_bytes }}b</span>
            <span class="debug__caps">tools</span>
            <span class="debug__head-mono">
              {{ selected.stats.tool_calls }} ({{ selected.stats.tool_done }}✓ {{ selected.stats.tool_failed }}✗)
            </span>
            <span v-if="selected.error" class="debug__bad">error: {{ selected.error }}</span>
          </div>
          <div
            v-if="hasCacheStats(selected.stats)"
            class="debug__head-row debug__head-row--stats"
          >
            <span class="debug__caps">input</span>
            <span class="debug__head-mono">{{ selected.stats.input_tokens ?? 0 }}t</span>
            <span class="debug__caps">cache read</span>
            <span class="debug__head-mono">{{ selected.stats.cached_input_tokens ?? 0 }}t</span>
            <span class="debug__caps">cache write</span>
            <span class="debug__head-mono">{{ selected.stats.cache_creation_tokens ?? 0 }}t</span>
            <span class="debug__caps">output</span>
            <span class="debug__head-mono">{{ selected.stats.output_tokens ?? 0 }}t</span>
            <span class="debug__caps">hit</span>
            <span class="debug__head-mono">{{ cacheHitPct(selected.stats) }}</span>
            <span v-if="selected.stats.cost_usd != null" class="debug__caps">cost</span>
            <span v-if="selected.stats.cost_usd != null" class="debug__head-mono">
              ${{ selected.stats.cost_usd.toFixed(4) }}
            </span>
          </div>
        </header>

        <section
          v-if="findingsForRun(selected.id).length > 0"
          class="debug__lints"
        >
          <header class="debug__lints-head">
            <span class="debug__caps">lint findings</span>
            <span class="debug__lints-counts">
              <span
                v-if="failCountForRun(selected.id) > 0"
                class="debug__lint-pill debug__lint-pill--fail"
              >{{ failCountForRun(selected.id) }} FAIL</span>
              <span
                v-if="warnCountForRun(selected.id) > 0"
                class="debug__lint-pill debug__lint-pill--warn"
              >{{ warnCountForRun(selected.id) }} WARN</span>
            </span>
          </header>
          <ul class="debug__lints-list">
            <li
              v-for="(f, i) in findingsForRun(selected.id)"
              :key="`${selected.id}_${i}`"
              class="debug__lint-row"
              :class="`debug__lint-row--${f.severity}`"
            >
              <span class="debug__lint-tag">{{ f.severity.toUpperCase() }}</span>
              <code class="debug__lint-id">{{ f.id }}</code>
              <span class="debug__lint-msg">{{ f.message }}</span>
              <pre v-if="f.evidence" class="debug__lint-ev">{{ f.evidence }}</pre>
            </li>
          </ul>
        </section>

        <div class="debug__chat">
          <template v-for="m in messagesForSelected" :key="m.id">
            <ChatMessage :message="m" />
          </template>
        </div>

        <details class="debug__raw" v-if="toolSegments.length > 0">
          <summary>tool call details ({{ toolSegments.length }})</summary>
          <div class="debug__tools">
            <div
              v-for="seg in toolSegments"
              :key="seg.id"
              class="debug__tool"
            >
              <h4 class="debug__tool-h">
                <code>{{ seg.name }}</code>
                <span
                  class="debug__caps debug__tool-status"
                  :class="`debug__tool-status--${seg.status}`"
                >
                  {{ seg.status }}
                </span>
              </h4>
              <p v-if="seg.detail" class="debug__tool-meta">
                <span class="debug__caps">detail</span>
                <code>{{ seg.detail }}</code>
              </p>
              <p v-if="seg.summary" class="debug__tool-meta">
                <span class="debug__caps">summary</span>
                <code>{{ seg.summary }}</code>
              </p>
              <pre v-if="seg.outputs.length" class="debug__tool-out">{{ seg.outputs.join('') }}</pre>
            </div>
          </div>
        </details>
      </template>
      <div v-else class="debug__placeholder">
        <span class="debug__caps">debug</span>
        <p>Select a run from the sidebar to view its full transcript.</p>
      </div>
    </section>
  </div>
</template>

<style scoped>
.debug {
  display: flex;
  flex: 1;
  min-height: 0;
  background: var(--paper);
  color: var(--ink);
  font-family: var(--hand);
}

.debug__side {
  width: 360px;
  flex: none;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--paper-line);
  background: var(--paper-shade);
  min-height: 0;
}

.debug__side-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  padding: 14px 16px 8px;
  border-bottom: 1px solid var(--paper-line);
}

.debug__caps {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-soft);
}

.debug__count {
  font-family: var(--mono, 'JetBrains Mono', monospace);
  font-size: 11px;
  color: var(--ink-soft);
}

.debug__models {
  padding: 8px 12px;
  border-bottom: 1px dashed var(--paper-line);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.debug__models-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
}

.debug__pill-all {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  background: transparent;
  border: 0;
  color: var(--amber);
  cursor: pointer;
  padding: 0;
}

.debug__pill-all:hover {
  text-decoration: underline;
}

.debug__pills {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.debug__pill {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  font-family: var(--hand);
  font-size: 11px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 4px;
  color: var(--ink);
  cursor: pointer;
  transition: border-color 0.12s ease, opacity 0.12s ease;
}

.debug__pill:hover {
  border-color: var(--amber);
}

.debug__pill--off {
  opacity: 0.45;
  border-style: dashed;
}

.debug__pill--off .debug__pill-name {
  text-decoration: line-through;
}

.debug__pill-name {
  font-family: var(--hand-display);
  font-size: 12px;
  font-weight: 600;
}

.debug__pill-count {
  font-family: var(--mono, 'JetBrains Mono', monospace);
  font-size: 10px;
  color: var(--ink-faint);
  padding: 1px 5px;
  background: var(--paper-shade);
  border-radius: 3px;
}

.debug__filter {
  display: flex;
  gap: 6px;
  padding: 8px 12px;
  border-bottom: 1px dashed var(--paper-line);
}

.debug__input {
  flex: 1;
  padding: 6px 8px;
  font-family: var(--hand);
  font-size: 13px;
  background: var(--paper);
  color: var(--ink);
  border: 1px solid var(--paper-line);
  border-radius: 4px;
}

.debug__input:focus {
  outline: none;
  border-color: var(--amber);
}

.debug__refresh {
  padding: 4px 10px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  cursor: pointer;
  border-radius: 4px;
  color: var(--ink);
}

.debug__refresh:hover {
  border-color: var(--amber);
}

.debug__list {
  flex: 1;
  overflow-y: auto;
  list-style: none;
  margin: 0;
  padding: 0;
}

.debug__item {
  padding: 10px 14px;
  border-bottom: 1px solid var(--paper-line);
  cursor: pointer;
  transition: background 0.12s ease;
}

.debug__item:hover {
  background: var(--paper);
}

.debug__item--active {
  background: var(--paper);
  border-left: 2px solid var(--amber);
  padding-left: 12px;
}

.debug__row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 4px;
}

.debug__model {
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 600;
  color: var(--ink);
}

.debug__src {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--ink-faint);
  padding: 1px 5px;
  border: 1px dashed var(--paper-line);
  border-radius: 3px;
}

.debug__src--smoke {
  color: var(--el-cold-deep, #4a6b8a);
  border-color: var(--el-cold-deep, #4a6b8a);
}

.debug__src--ui {
  color: var(--amber);
  border-color: var(--amber);
}

.debug__prompt {
  margin: 0 0 6px;
  font-style: italic;
  font-size: 13px;
  color: var(--ink);
  line-height: 1.4;
}

.debug__meta {
  display: flex;
  align-items: center;
  gap: 5px;
  font-family: var(--mono, 'JetBrains Mono', monospace);
  font-size: 10px;
  color: var(--ink-faint);
}

.debug__dot {
  opacity: 0.5;
}

.debug__bad {
  color: var(--bad);
  font-weight: 600;
}

.debug__del {
  margin-left: auto;
  background: transparent;
  border: 0;
  color: var(--ink-faint);
  cursor: pointer;
  padding: 2px 6px;
  font-size: 14px;
}

.debug__del:hover {
  color: var(--bad);
}

.debug__empty {
  padding: 32px 16px;
  font-style: italic;
  color: var(--ink-faint);
  font-size: 12px;
  line-height: 1.5;
}

.debug__empty code {
  background: var(--paper);
  padding: 1px 4px;
  font-family: var(--mono, 'JetBrains Mono', monospace);
  font-size: 11px;
}

.debug__side-foot {
  display: flex;
  gap: 8px;
  padding: 10px 12px;
  border-top: 1px solid var(--paper-line);
}

.debug__btn {
  padding: 6px 12px;
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  cursor: pointer;
  border-radius: 4px;
  color: var(--ink);
}

.debug__btn:hover:not(:disabled) {
  border-color: var(--amber);
}

.debug__btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.debug__btn--danger:hover:not(:disabled) {
  border-color: var(--bad);
  color: var(--bad);
}

.debug__main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow-y: auto;
  background: var(--paper);
}

.debug__head {
  padding: 14px 24px;
  border-bottom: 1px solid var(--paper-line);
  background: var(--paper-shade);
}

.debug__head-row {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 6px 14px;
  font-size: 12px;
}

.debug__head-row + .debug__head-row {
  margin-top: 6px;
}

.debug__head-val {
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 600;
}

.debug__head-mono {
  font-family: var(--mono, 'JetBrains Mono', monospace);
  font-size: 11px;
  color: var(--ink-soft);
}

.debug__chat {
  padding: 24px 32px;
  display: flex;
  flex-direction: column;
  gap: 18px;
  flex: 1;
  min-height: 0;
}

.debug__placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 10px;
  color: var(--ink-faint);
  font-style: italic;
}

.debug__raw {
  margin: 0 24px 24px;
  border: 1px dashed var(--paper-line);
  padding: 10px 14px;
}

.debug__raw summary {
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--ink-soft);
  cursor: pointer;
}

.debug__tools {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.debug__tool {
  border-left: 2px solid var(--paper-line);
  padding-left: 12px;
}

.debug__tool-h {
  margin: 0 0 4px;
  font-size: 13px;
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.debug__tool-h code {
  font-family: var(--mono, 'JetBrains Mono', monospace);
  background: var(--paper-shade);
  padding: 1px 6px;
  border-radius: 3px;
}

.debug__tool-status--done {
  color: var(--good);
}

.debug__tool-status--failed {
  color: var(--bad);
}

.debug__tool-status--running {
  color: var(--amber);
}

.debug__tool-meta {
  margin: 4px 0;
  display: flex;
  gap: 8px;
  font-size: 11px;
  color: var(--ink-soft);
}

.debug__tool-meta code {
  font-family: var(--mono, 'JetBrains Mono', monospace);
}

.debug__tool-out {
  margin: 6px 0 0;
  font-family: var(--mono, 'JetBrains Mono', monospace);
  font-size: 11px;
  background: var(--paper-shade);
  padding: 8px 10px;
  border-radius: 3px;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 240px;
  overflow-y: auto;
}

/* Sprint A — lint findings surfacing in the dev panel. */
.debug__lint-pill {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  padding: 1px 6px;
  border-radius: 3px;
  border: 1px solid transparent;
  margin-left: 4px;
  font-weight: 600;
}

.debug__lint-pill--fail {
  color: var(--paper);
  background: #b6432e;
  border-color: #8a2f1d;
}

.debug__lint-pill--warn {
  color: #6b4d10;
  background: #f1cd6a;
  border-color: #c9a541;
}

.debug__lints {
  border-top: 1px dashed var(--paper-line);
  border-bottom: 1px solid var(--paper-line);
  background: var(--paper-shade);
  padding: 10px 16px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.debug__lints-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
}

.debug__lints-counts {
  display: flex;
  gap: 4px;
}

.debug__lints-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.debug__lint-row {
  display: grid;
  grid-template-columns: auto auto 1fr;
  align-items: baseline;
  gap: 8px;
  padding: 4px 6px;
  border-left: 3px solid transparent;
  font-size: 12px;
  line-height: 1.4;
}

.debug__lint-row--fail {
  border-left-color: #b6432e;
  background: rgba(182, 67, 46, 0.06);
}

.debug__lint-row--warn {
  border-left-color: #c9a541;
  background: rgba(201, 165, 65, 0.06);
}

.debug__lint-tag {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.1em;
  font-weight: 600;
}

.debug__lint-row--fail .debug__lint-tag { color: #8a2f1d; }
.debug__lint-row--warn .debug__lint-tag { color: #6b4d10; }

.debug__lint-id {
  font-family: var(--mono, 'JetBrains Mono', monospace);
  font-size: 11px;
  color: var(--ink-soft);
}

.debug__lint-msg {
  color: var(--ink);
}

.debug__lint-ev {
  grid-column: 1 / -1;
  margin: 4px 0 0;
  font-family: var(--mono, 'JetBrains Mono', monospace);
  font-size: 10px;
  background: var(--paper);
  border: 1px dashed var(--paper-line);
  padding: 4px 6px;
  border-radius: 2px;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 120px;
  overflow-y: auto;
  color: var(--ink-soft);
}
</style>

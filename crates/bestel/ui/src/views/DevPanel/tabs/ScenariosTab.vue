<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface Expectation {
  must_match: string[];
  must_not_match: string[];
  must_cite_domain: string[];
  must_call_tool: string | null;
  forbid_tool: string[];
  min_final_text_len: number;
  min_tool_calls: number;
}

interface Scenario {
  name: string;
  provider: string;
  build_fixture: string | null;
  prompt: string;
  timeout_secs: number;
  cost: 'low' | 'high';
  expectations: Expectation[];
  source_path: string;
}

const scenarios = ref<Scenario[]>([]);
const loading = ref(false);
const errorMsg = ref<string | null>(null);
const filter = ref('');

const filtered = computed(() => {
  const q = filter.value.trim().toLowerCase();
  if (!q) return scenarios.value;
  return scenarios.value.filter(
    (s) =>
      s.name.toLowerCase().includes(q) ||
      s.prompt.toLowerCase().includes(q) ||
      (s.build_fixture ?? '').toLowerCase().includes(q)
  );
});

async function refresh() {
  loading.value = true;
  errorMsg.value = null;
  try {
    scenarios.value = await invoke<Scenario[]>('dev_load_scenarios');
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    loading.value = false;
  }
}

function runScenario(s: Scenario) {
  console.log('[scenarios] run requested', s.name);
}

function relPath(s: Scenario): string {
  return s.source_path.replace(/^.*[\\/]tests[\\/]/, 'tests/');
}

onMounted(refresh);
</script>

<template>
  <div class="sc-tab">
    <header class="sc-tab__head">
      <div class="sc-search">
        <svg class="sc-search__icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <circle cx="11" cy="11" r="7" />
          <path d="m21 21-4.35-4.35" />
        </svg>
        <input
          v-model="filter"
          type="text"
          placeholder="Filter scenarios… (name, prompt, fixture)"
        />
      </div>
      <button class="sc-tab__btn" @click="refresh" :disabled="loading">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <path d="M21 12a9 9 0 1 1-3-6.7L21 8" /><path d="M21 3v5h-5" />
        </svg>
        <span>{{ loading ? '…' : 'Refresh' }}</span>
      </button>
      <span class="sc-tab__count">{{ filtered.length }} / {{ scenarios.length }}</span>
    </header>

    <p v-if="errorMsg" class="sc-tab__err">{{ errorMsg }}</p>

    <ul class="sc-list">
      <li v-for="s in filtered" :key="s.name" class="sc-card">
        <div class="sc-card__main">
          <div class="sc-card__row">
            <span class="sc-card__name">{{ s.name }}</span>
            <span class="sc-tag">{{ s.provider }}</span>
            <span class="sc-tag sc-tag--amber">{{ s.cost }}</span>
            <span v-if="s.build_fixture" class="sc-tag sc-tag--mono">build: {{ s.build_fixture }}</span>
            <span class="sc-tag sc-tag--mono">{{ s.timeout_secs }}s</span>
          </div>
          <p class="sc-card__prompt">{{ s.prompt }}</p>
          <div class="sc-card__meta">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
              <path d="M5 12h14M13 5l7 7-7 7" />
            </svg>
            <span>{{ s.expectations.length }} expectation{{ s.expectations.length === 1 ? '' : 's' }}</span>
          </div>

          <details v-if="s.expectations.length" class="sc-card__exp">
            <summary>show expectations</summary>
            <ol>
              <li v-for="(exp, i) in s.expectations" :key="i" class="sc-exp">
                <ul class="sc-exp__list">
                  <li v-for="r in exp.must_match" :key="'mm:' + r">
                    <span class="sc-exp-tag sc-exp-tag--must">must match</span>
                    <code>{{ r }}</code>
                  </li>
                  <li v-for="r in exp.must_not_match" :key="'nn:' + r">
                    <span class="sc-exp-tag sc-exp-tag--not">must NOT match</span>
                    <code>{{ r }}</code>
                  </li>
                  <li v-for="d in exp.must_cite_domain" :key="'cd:' + d">
                    <span class="sc-exp-tag sc-exp-tag--cite">cite domain</span>
                    <code>{{ d }}</code>
                  </li>
                  <li v-if="exp.must_call_tool">
                    <span class="sc-exp-tag sc-exp-tag--tool">must call tool</span>
                    <code>{{ exp.must_call_tool }}</code>
                  </li>
                  <li v-for="t in exp.forbid_tool" :key="'ft:' + t">
                    <span class="sc-exp-tag sc-exp-tag--forbid">forbid tool</span>
                    <code>{{ t }}</code>
                  </li>
                  <li v-if="exp.min_final_text_len">
                    <span class="sc-exp-tag">min length</span>
                    <code>{{ exp.min_final_text_len }} chars</code>
                  </li>
                  <li v-if="exp.min_tool_calls">
                    <span class="sc-exp-tag">min tool calls</span>
                    <code>{{ exp.min_tool_calls }}</code>
                  </li>
                </ul>
              </li>
            </ol>
          </details>
        </div>

        <div class="sc-card__side">
          <button type="button" class="sc-run" @click="runScenario(s)">
            <span class="sc-run__arrow">▶</span>
            <span>Run</span>
          </button>
          <span class="sc-card__path">{{ relPath(s) }}</span>
        </div>
      </li>
      <li v-if="!loading && scenarios.length === 0" class="sc-empty">
        No scenarios found under <code>tests/scenarios/</code>.
      </li>
    </ul>
  </div>
</template>

<style scoped>
.sc-tab {
  height: 100%;
  overflow: auto;
  padding: 20px 28px 28px;
  background: var(--paper);
  color: var(--ink);
  font-family: var(--hand);
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.sc-tab__head {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.sc-search {
  flex: 1;
  min-width: 240px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 4px;
}
.sc-search:focus-within {
  border-color: var(--amber);
}
.sc-search__icon {
  color: var(--ink-faint);
  flex: none;
}
.sc-search input {
  flex: 1;
  min-width: 0;
  background: transparent;
  border: none;
  outline: none;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink);
}
.sc-search input::placeholder {
  color: var(--ink-faint);
}

.sc-tab__btn {
  display: inline-flex;
  align-items: center;
  gap: 7px;
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
.sc-tab__btn:hover:not(:disabled) {
  border-color: var(--amber);
}
.sc-tab__btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.sc-tab__count {
  font-family: var(--mono);
  font-size: 13px;
  color: var(--ink-soft);
  font-variant-numeric: tabular-nums;
}

.sc-tab__err {
  margin: 0;
  padding: 10px 14px;
  border: 1px solid var(--bad);
  background: rgba(162, 58, 58, 0.06);
  border-radius: 4px;
  color: var(--bad);
  font-size: 13px;
}

.sc-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.sc-card {
  display: flex;
  gap: 22px;
  padding: 18px 22px;
  border: 1px solid var(--paper-line);
  background: var(--paper);
  border-radius: 5px;
}

.sc-card__main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.sc-card__row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.sc-card__name {
  font-family: var(--mono);
  font-size: 14px;
  font-weight: 700;
  color: var(--ink);
  letter-spacing: 0.02em;
}

.sc-tag {
  font-family: var(--label);
  font-size: 11.5px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  font-weight: 600;
  padding: 2px 8px;
  border: 1px solid var(--ink-faint);
  color: var(--ink-soft);
  background: var(--paper);
  border-radius: 3px;
  line-height: 1.3;
}
.sc-tag--amber {
  color: var(--amber);
  border-color: var(--amber);
  background: var(--amber-glow);
}
.sc-tag--mono {
  font-family: var(--mono);
  font-size: 12px;
  letter-spacing: 0.02em;
  text-transform: none;
  font-weight: 500;
}

.sc-card__prompt {
  margin: 0;
  font-size: 15px;
  line-height: 1.55;
  color: var(--ink-soft);
  white-space: pre-wrap;
}

.sc-card__meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13.5px;
  color: var(--ink-faint);
}

.sc-card__exp {
  margin-top: 4px;
}
.sc-card__exp summary {
  cursor: pointer;
  font-family: var(--label);
  font-size: 11.5px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 600;
  padding: 4px 0;
}
.sc-card__exp summary:hover {
  color: var(--amber);
}
.sc-card__exp ol {
  margin: 8px 0 0 0;
  padding-left: 18px;
}

.sc-exp__list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.sc-exp__list li {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--ink);
}

.sc-exp-tag {
  font-family: var(--label);
  font-size: 10.5px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 700;
  padding: 1px 7px;
  border: 1px solid var(--ink-faint);
  border-radius: 2px;
  flex: none;
}
.sc-exp-tag--must   { color: var(--good);  border-color: var(--good); }
.sc-exp-tag--not    { color: var(--bad);   border-color: var(--bad);  }
.sc-exp-tag--cite   { color: var(--amber); border-color: var(--amber);}
.sc-exp-tag--tool   { color: var(--note);  border-color: var(--note); }
.sc-exp-tag--forbid { color: var(--bad);   border-color: var(--bad);  }

.sc-exp__list code {
  font-family: var(--mono);
  font-size: 12px;
  background: var(--paper-shade);
  color: var(--ink);
  padding: 1px 6px;
  border-radius: 3px;
}

.sc-card__side {
  flex: none;
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: flex-end;
}

.sc-run {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: var(--paper);
  color: var(--good);
  border: 1px solid var(--good);
  border-radius: 4px;
  font-family: var(--hand);
  font-size: 14.5px;
  font-weight: 600;
  cursor: pointer;
  transition: background 100ms ease;
}
.sc-run:hover {
  background: rgba(46, 107, 58, 0.06);
}
.sc-run__arrow {
  font-size: 11px;
}

.sc-card__path {
  font-family: var(--mono);
  font-size: 12.5px;
  color: var(--ink-faint);
}

.sc-empty {
  padding: 32px 16px;
  text-align: center;
  font-style: italic;
  color: var(--ink-faint);
  font-size: 14px;
}
.sc-empty code {
  font-family: var(--mono);
  background: var(--paper-shade);
  padding: 1px 6px;
  border-radius: 3px;
  color: var(--ink);
}
</style>

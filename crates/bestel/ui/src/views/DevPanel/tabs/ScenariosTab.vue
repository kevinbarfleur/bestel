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
  // Wired in Step 4 (Live test tab pre-fill + run).
  console.log('[scenarios] run requested', s.name);
}

onMounted(refresh);
</script>

<template>
  <div class="scenarios-tab">
    <header class="scenarios-tab__head">
      <input
        v-model="filter"
        class="scenarios-tab__filter"
        type="text"
        placeholder="Filter scenarios… (name, prompt, fixture)"
      />
      <button class="scenarios-tab__btn" @click="refresh" :disabled="loading">
        {{ loading ? '…' : '↻ refresh' }}
      </button>
      <span class="scenarios-tab__count">{{ filtered.length }} / {{ scenarios.length }}</span>
    </header>

    <p v-if="errorMsg" class="scenarios-tab__err">{{ errorMsg }}</p>

    <ul class="scenarios-tab__list">
      <li v-for="s in filtered" :key="s.name" class="scenario-card">
        <header class="scenario-card__head">
          <h3 class="scenario-card__name">{{ s.name }}</h3>
          <div class="scenario-card__pills">
            <span class="pill" :data-tone="s.provider">{{ s.provider }}</span>
            <span class="pill" :data-tone="s.cost">{{ s.cost }}</span>
            <span v-if="s.build_fixture" class="pill" data-tone="fixture">
              build: {{ s.build_fixture }}
            </span>
            <span class="pill" data-tone="timeout">{{ s.timeout_secs }}s</span>
          </div>
          <button class="scenario-card__run" @click="runScenario(s)">▶ Run</button>
        </header>

        <p class="scenario-card__prompt">{{ s.prompt }}</p>

        <details v-if="s.expectations.length" class="scenario-card__exp">
          <summary>{{ s.expectations.length }} expectation(s)</summary>
          <ol>
            <li v-for="(exp, i) in s.expectations" :key="i" class="exp-row">
              <ul class="exp-row__list">
                <li v-for="r in exp.must_match" :key="'mm:' + r">
                  <span class="exp-tag exp-tag--must">must match</span>
                  <code>{{ r }}</code>
                </li>
                <li v-for="r in exp.must_not_match" :key="'nn:' + r">
                  <span class="exp-tag exp-tag--not">must NOT match</span>
                  <code>{{ r }}</code>
                </li>
                <li v-for="d in exp.must_cite_domain" :key="'cd:' + d">
                  <span class="exp-tag exp-tag--cite">cite domain</span>
                  <code>{{ d }}</code>
                </li>
                <li v-if="exp.must_call_tool">
                  <span class="exp-tag exp-tag--tool">must call tool</span>
                  <code>{{ exp.must_call_tool }}</code>
                </li>
                <li v-for="t in exp.forbid_tool" :key="'ft:' + t">
                  <span class="exp-tag exp-tag--forbid">forbid tool</span>
                  <code>{{ t }}</code>
                </li>
                <li v-if="exp.min_final_text_len">
                  <span class="exp-tag">min length</span>
                  {{ exp.min_final_text_len }} chars
                </li>
                <li v-if="exp.min_tool_calls">
                  <span class="exp-tag">min tool calls</span>
                  {{ exp.min_tool_calls }}
                </li>
              </ul>
            </li>
          </ol>
        </details>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.scenarios-tab {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.scenarios-tab__head {
  display: flex;
  align-items: center;
  gap: 12px;
}
.scenarios-tab__filter {
  flex: 1;
  background: #141417;
  border: 1px solid #2a2a30;
  color: #e8e6df;
  padding: 8px 12px;
  border-radius: 4px;
  font: inherit;
  font-size: 13px;
}
.scenarios-tab__filter:focus {
  outline: 1px solid #c9a86a;
  border-color: #c9a86a;
}
.scenarios-tab__btn {
  background: #1a1a1f;
  border: 1px solid #2a2a30;
  color: #c9c9d0;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  font: inherit;
  font-size: 12px;
}
.scenarios-tab__btn:hover:not(:disabled) {
  background: #22222a;
}
.scenarios-tab__count {
  font-size: 11px;
  color: #75757f;
  font-variant-numeric: tabular-nums;
}
.scenarios-tab__err {
  color: #d84a4a;
  background: #2a1518;
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 12px;
}
.scenarios-tab__list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.scenario-card {
  background: #141417;
  border: 1px solid #2a2a30;
  border-radius: 6px;
  padding: 16px;
}
.scenario-card__head {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}
.scenario-card__name {
  margin: 0;
  font-family: var(--font-display, 'Cinzel', serif);
  font-size: 14px;
  color: #e8e6df;
  letter-spacing: 0.04em;
}
.scenario-card__pills {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.pill {
  background: #1f1f25;
  color: #a8a8b0;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 11px;
  letter-spacing: 0.04em;
  text-transform: lowercase;
}
.pill[data-tone="anthropic"] { color: #d8a86f; }
.pill[data-tone="ollama"] { color: #6fa8d8; }
.pill[data-tone="any"] { color: #888; }
.pill[data-tone="high"] { color: #d8786f; }
.pill[data-tone="low"] { color: #75c075; }
.pill[data-tone="fixture"] { color: #c9a86a; }
.scenario-card__run {
  margin-left: auto;
  background: #2a3f2a;
  border: 1px solid #4a6a4a;
  color: #b4d4b4;
  padding: 4px 12px;
  border-radius: 4px;
  font: inherit;
  font-size: 12px;
  cursor: pointer;
}
.scenario-card__run:hover {
  background: #355035;
}
.scenario-card__prompt {
  color: #c9c9d0;
  margin: 0 0 8px 0;
  white-space: pre-wrap;
  font-size: 13px;
  line-height: 1.5;
}
.scenario-card__exp summary {
  cursor: pointer;
  color: #75757f;
  font-size: 11px;
  padding: 4px 0;
}
.scenario-card__exp ol {
  margin: 8px 0 0 0;
  padding-left: 18px;
}
.exp-row__list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.exp-row__list li {
  display: flex;
  gap: 6px;
  align-items: center;
  font-size: 11px;
  color: #c9c9d0;
}
.exp-tag {
  background: #1f1f25;
  color: #75757f;
  padding: 1px 6px;
  border-radius: 2px;
  font-size: 10px;
  letter-spacing: 0.02em;
  font-family: var(--font-mono, 'JetBrains Mono', monospace);
}
.exp-tag--must { color: #75c075; }
.exp-tag--not { color: #d8786f; }
.exp-tag--cite { color: #d8a86f; }
.exp-tag--tool { color: #6fa8d8; }
.exp-tag--forbid { color: #c14a4a; }
code {
  background: #0a0a0c;
  color: #c9a86a;
  padding: 1px 4px;
  border-radius: 2px;
  font-size: 11px;
}
</style>

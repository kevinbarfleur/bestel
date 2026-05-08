<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface RealPrompt {
  id: string;
  category: string;
  text: string;
  intent: string;
  expected: string;
  source: string | null;
}

const prompts = ref<RealPrompt[]>([]);
const loading = ref(false);
const errorMsg = ref<string | null>(null);
const filter = ref('');
const activeCategory = ref<string>('all');

const categories = computed(() => {
  const counts = new Map<string, number>();
  for (const p of prompts.value) {
    counts.set(p.category, (counts.get(p.category) ?? 0) + 1);
  }
  const out = Array.from(counts.entries()).map(([id, count]) => ({ id, count }));
  out.sort((a, b) => a.id.localeCompare(b.id));
  out.unshift({ id: 'all', count: prompts.value.length });
  return out;
});

const filtered = computed(() => {
  const q = filter.value.trim().toLowerCase();
  return prompts.value.filter((p) => {
    if (activeCategory.value !== 'all' && p.category !== activeCategory.value) return false;
    if (!q) return true;
    return (
      p.text.toLowerCase().includes(q) ||
      p.intent.toLowerCase().includes(q) ||
      p.expected.toLowerCase().includes(q) ||
      p.id.toLowerCase().includes(q)
    );
  });
});

async function refresh() {
  loading.value = true;
  errorMsg.value = null;
  try {
    prompts.value = await invoke<RealPrompt[]>('dev_load_real_prompts');
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    loading.value = false;
  }
}

function runWith(p: RealPrompt) {
  console.log('[real-prompts] run-with requested', p.id);
}

onMounted(refresh);
</script>

<template>
  <div class="rp-tab">
    <header class="rp-tab__head">
      <div class="rp-search">
        <svg class="rp-search__icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <circle cx="11" cy="11" r="7" /><path d="m21 21-4.35-4.35" />
        </svg>
        <input
          v-model="filter"
          type="text"
          placeholder="Search prompts (text, intent, expected, id)…"
        />
      </div>
      <button class="rp-tab__btn" @click="refresh" :disabled="loading">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <path d="M21 12a9 9 0 1 1-3-6.7L21 8" /><path d="M21 3v5h-5" />
        </svg>
        <span>{{ loading ? '…' : 'Refresh' }}</span>
      </button>
    </header>

    <nav class="rp-cats">
      <button
        v-for="c in categories"
        :key="c.id"
        type="button"
        class="rp-cat"
        :class="{ 'rp-cat--active': activeCategory === c.id }"
        @click="activeCategory = c.id"
      >
        <span class="rp-cat__label">{{ c.id }}</span>
        <span class="rp-cat__count">{{ c.count }}</span>
      </button>
    </nav>

    <p v-if="errorMsg" class="rp-tab__err">{{ errorMsg }}</p>

    <ul class="rp-list">
      <li v-for="p in filtered" :key="p.id" class="rp-card">
        <div class="rp-card__main">
          <div class="rp-card__row">
            <span class="rp-card__id">{{ p.id }}</span>
            <span class="rp-tag">{{ p.category }}</span>
            <a v-if="p.source" :href="p.source" class="rp-card__src" target="_blank" rel="noopener">
              source ↗
            </a>
          </div>
          <p class="rp-card__text">"{{ p.text }}"</p>
          <div v-if="p.intent" class="rp-card__line">
            <span class="rp-line-label rp-line-label--intent">Intent</span>
            <span class="rp-line-value">{{ p.intent }}</span>
          </div>
          <div v-if="p.expected" class="rp-card__line">
            <span class="rp-line-label rp-line-label--expected">Expected</span>
            <span class="rp-line-value">{{ p.expected }}</span>
          </div>
        </div>

        <div class="rp-card__side">
          <button type="button" class="rp-run" @click="runWith(p)">
            <span class="rp-run__arrow">▶</span>
            <span>Run with…</span>
          </button>
        </div>
      </li>
      <li v-if="!loading && prompts.length === 0" class="rp-empty">
        No prompts found.
      </li>
    </ul>
  </div>
</template>

<style scoped>
.rp-tab {
  height: 100%;
  overflow: auto;
  padding: 20px 28px 28px;
  background: var(--paper);
  color: var(--ink);
  font-family: var(--hand);
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.rp-tab__head {
  display: flex;
  align-items: center;
  gap: 12px;
}

.rp-search {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 4px;
}
.rp-search:focus-within {
  border-color: var(--amber);
}
.rp-search__icon {
  color: var(--ink-faint);
  flex: none;
}
.rp-search input {
  flex: 1;
  min-width: 0;
  background: transparent;
  border: none;
  outline: none;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink);
}
.rp-search input::placeholder {
  color: var(--ink-faint);
}

.rp-tab__btn {
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
.rp-tab__btn:hover:not(:disabled) {
  border-color: var(--amber);
}
.rp-tab__btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.rp-cats {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 4px;
}
.rp-cat {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 4px 10px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 3px;
  cursor: pointer;
  font-family: var(--mono);
  font-size: 12.5px;
  color: var(--ink-soft);
  font-weight: 500;
  transition: border-color 100ms ease, background 100ms ease;
}
.rp-cat:hover {
  border-color: var(--amber);
  color: var(--ink);
}
.rp-cat--active {
  border-color: var(--ink);
  background: var(--paper-shade);
  color: var(--ink);
  font-weight: 700;
}
.rp-cat__label {
  font-family: var(--mono);
}
.rp-cat__count {
  font-family: var(--label);
  font-size: 11.5px;
  color: var(--ink-faint);
  font-weight: 600;
}
.rp-cat--active .rp-cat__count {
  color: var(--amber);
}

.rp-tab__err {
  margin: 0;
  padding: 10px 14px;
  border: 1px solid var(--bad);
  background: rgba(162, 58, 58, 0.06);
  border-radius: 4px;
  color: var(--bad);
  font-size: 13px;
}

.rp-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.rp-card {
  display: flex;
  gap: 18px;
  padding: 16px 20px;
  border: 1px solid var(--paper-line);
  background: var(--paper);
  border-radius: 5px;
}

.rp-card__main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.rp-card__row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.rp-card__id {
  font-family: var(--mono);
  font-size: 13.5px;
  font-weight: 700;
  color: var(--ink);
}

.rp-tag {
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

.rp-card__src {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--ink-faint);
  text-decoration: none;
}
.rp-card__src:hover {
  color: var(--amber);
  text-decoration: underline;
}

.rp-card__text {
  margin: 0;
  font-size: 17px;
  font-weight: 500;
  color: var(--ink);
  line-height: 1.45;
  white-space: pre-wrap;
}

.rp-card__line {
  display: flex;
  align-items: baseline;
  gap: 10px;
}

.rp-line-label {
  font-family: var(--label);
  font-size: 11.5px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  font-weight: 700;
  flex: none;
  padding-top: 2px;
}
.rp-line-label--intent   { color: var(--amber); }
.rp-line-label--expected { color: var(--good); }

.rp-line-value {
  font-size: 14.5px;
  color: var(--ink-soft);
  line-height: 1.5;
}

.rp-card__side {
  flex: none;
}

.rp-run {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: var(--paper);
  color: var(--good);
  border: 1px solid var(--good);
  border-radius: 4px;
  font-family: var(--hand);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: background 100ms ease;
}
.rp-run:hover {
  background: rgba(46, 107, 58, 0.06);
}
.rp-run__arrow {
  font-size: 11px;
}

.rp-empty {
  padding: 32px 16px;
  text-align: center;
  font-style: italic;
  color: var(--ink-faint);
  font-size: 14px;
}
</style>

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
  // Wired in Step 4 — switches to Live test tab with prompt pre-filled.
  console.log('[real-prompts] run-with requested', p.id);
}

onMounted(refresh);
</script>

<template>
  <div class="rp-tab">
    <header class="rp-tab__head">
      <input
        v-model="filter"
        class="rp-tab__filter"
        type="text"
        placeholder="Search prompts (text, intent, expected, id)…"
      />
      <button class="rp-tab__btn" @click="refresh" :disabled="loading">
        {{ loading ? '…' : '↻ refresh' }}
      </button>
    </header>

    <nav class="rp-cats">
      <button
        v-for="c in categories"
        :key="c.id"
        class="rp-cat"
        :class="{ 'rp-cat--active': activeCategory === c.id }"
        @click="activeCategory = c.id"
      >
        {{ c.id }} <span class="rp-cat__count">{{ c.count }}</span>
      </button>
    </nav>

    <p v-if="errorMsg" class="rp-tab__err">{{ errorMsg }}</p>

    <ul class="rp-list">
      <li v-for="p in filtered" :key="p.id" class="rp-card">
        <header class="rp-card__head">
          <code class="rp-card__id">{{ p.id }}</code>
          <span class="pill" data-tone="cat">{{ p.category }}</span>
          <a v-if="p.source" :href="p.source" class="rp-card__src" target="_blank" rel="noopener">
            source ↗
          </a>
          <button class="rp-card__run" @click="runWith(p)">▶ Run with…</button>
        </header>

        <p class="rp-card__text">{{ p.text }}</p>

        <div v-if="p.intent || p.expected" class="rp-card__notes">
          <p v-if="p.intent"><strong>Intent.</strong> {{ p.intent }}</p>
          <p v-if="p.expected"><strong>Expected.</strong> {{ p.expected }}</p>
        </div>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.rp-tab {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.rp-tab__head {
  display: flex;
  align-items: center;
  gap: 12px;
}
.rp-tab__filter {
  flex: 1;
  background: #141417;
  border: 1px solid #2a2a30;
  color: #e8e6df;
  padding: 8px 12px;
  border-radius: 4px;
  font: inherit;
  font-size: 13px;
}
.rp-tab__filter:focus {
  outline: 1px solid #c9a86a;
  border-color: #c9a86a;
}
.rp-tab__btn {
  background: #1a1a1f;
  border: 1px solid #2a2a30;
  color: #c9c9d0;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  font: inherit;
  font-size: 12px;
}
.rp-tab__err {
  color: #d84a4a;
  background: #2a1518;
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 12px;
}
.rp-cats {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}
.rp-cat {
  background: #141417;
  border: 1px solid #2a2a30;
  color: #75757f;
  padding: 4px 10px;
  border-radius: 4px;
  cursor: pointer;
  font: inherit;
  font-size: 11px;
  text-transform: lowercase;
  letter-spacing: 0.04em;
}
.rp-cat:hover { color: #c9c9d0; }
.rp-cat--active {
  background: #2a2a30;
  color: #e8e6df;
  border-color: #c9a86a;
}
.rp-cat__count {
  margin-left: 4px;
  color: #50505a;
  font-variant-numeric: tabular-nums;
}
.rp-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.rp-card {
  background: #141417;
  border: 1px solid #2a2a30;
  border-radius: 6px;
  padding: 16px;
}
.rp-card__head {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}
.rp-card__id {
  color: #c9a86a;
  font-size: 11px;
  background: #0a0a0c;
  padding: 2px 6px;
  border-radius: 2px;
}
.pill {
  background: #1f1f25;
  color: #a8a8b0;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 11px;
  text-transform: lowercase;
}
.pill[data-tone="cat"] { color: #d8a86f; }
.rp-card__src {
  color: #6fa8d8;
  font-size: 11px;
  text-decoration: none;
}
.rp-card__src:hover { text-decoration: underline; }
.rp-card__run {
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
.rp-card__run:hover { background: #355035; }
.rp-card__text {
  color: #e8e6df;
  margin: 0 0 8px 0;
  white-space: pre-wrap;
  font-size: 13px;
  line-height: 1.5;
}
.rp-card__notes {
  font-size: 12px;
  color: #a8a8b0;
  border-top: 1px solid #2a2a30;
  padding-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.rp-card__notes p {
  margin: 0;
}
.rp-card__notes strong {
  color: #c9a86a;
  font-weight: 500;
}
</style>

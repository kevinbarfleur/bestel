<script setup lang="ts">
import { openExternal } from '../../../api/tauri';

interface Source {
  site: string;
  page: string;
  url?: string;
  quote?: string;
}

withDefaults(defineProps<{ sources?: Source[] }>(), { sources: () => [] });

const open = (url?: string) => {
  if (!url) return;
  void openExternal(url);
};
</script>

<template>
  <div class="art-src">
    <div v-for="(s, i) in sources" :key="i" class="art-src__row">
      <span class="art-src__num">[{{ i + 1 }}]</span>
      <div class="art-src__body">
        <div class="art-src__page">
          <a v-if="s.url" class="link" @click.prevent="open(s.url)">{{ s.page }}</a>
          <span v-else>{{ s.page }}</span>
          <span class="art-src__site"> · {{ s.site }}</span>
        </div>
        <div v-if="s.quote" class="art-src__quote">"{{ s.quote }}"</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.art-src {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.art-src__row {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--paper-line);
}
.art-src__row:last-child {
  border-bottom: none;
  padding-bottom: 0;
}
.art-src__num {
  flex: none;
  min-width: 18px;
  font-family: var(--hand-display);
  font-size: 11px;
  font-weight: 700;
  color: var(--amber);
}
.art-src__body {
  flex: 1;
  min-width: 0;
}
.art-src__page {
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink);
}
.art-src__site {
  color: var(--ink-faint);
}
.art-src__quote {
  font-family: var(--script);
  font-size: 11px;
  color: var(--ink-soft);
  font-style: italic;
  line-height: 1.4;
  margin-top: 2px;
}
</style>

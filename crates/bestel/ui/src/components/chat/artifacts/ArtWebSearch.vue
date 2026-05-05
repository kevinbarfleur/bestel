<script setup lang="ts">
interface Query {
  q: string;
  sites?: string[];
}

withDefaults(
  defineProps<{
    queries?: Query[];
    title?: string;
    meta?: string;
  }>(),
  {
    queries: () => [],
    title: 'web search',
    meta: '',
  },
);
</script>

<template>
  <div class="art-search">
    <div v-if="title || meta" class="art-search__head">
      <a class="link art-search__title">{{ title }}</a>
      <span v-if="meta" class="aside">{{ meta }}</span>
    </div>
    <div class="art-search__queries">
      <span v-for="(row, i) in queries" :key="i" class="aside">"{{ row.q }}"</span>
    </div>
  </div>
</template>

<style scoped>
.art-search {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.art-search__head {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex-wrap: nowrap;
  white-space: nowrap;
}
.art-search__title {
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 600;
}
.art-search__queries {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-left: 14px;
  border-left: 1px dotted var(--paper-line);
}
</style>

<script setup lang="ts">
interface DiffRow {
  stat: string;
  before: string;
  after: string;
  tone: 'good' | 'bad';
  meta?: string;
}

withDefaults(
  defineProps<{
    title?: string;
    meta?: string;
    rows?: DiffRow[];
  }>(),
  {
    title: 'proposed change',
    meta: '',
    rows: () => [],
  },
);
</script>

<template>
  <div class="art-diff">
    <div class="art-diff__head">
      <span class="art-diff__title">{{ title }}</span>
      <span v-if="meta" class="aside">{{ meta }}</span>
    </div>
    <div class="art-diff__rows">
      <div v-for="(r, i) in rows" :key="i" class="art-diff__row">
        <span :class="`art-diff__sign art-diff__sign--${r.tone}`">{{ r.tone === 'good' ? '+' : '−' }}</span>
        <span class="art-diff__stat">{{ r.stat }}</span>
        <span v-if="r.meta" class="aside art-diff__meta">{{ r.meta }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.art-diff {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.art-diff__head {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex-wrap: nowrap;
  white-space: nowrap;
}
.art-diff__title {
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 600;
  color: var(--ink);
}
.art-diff__rows {
  padding-left: 14px;
  border-left: 1px dotted var(--paper-line);
  display: flex;
  flex-direction: column;
  gap: 3px;
  font-family: var(--hand);
  font-size: 12px;
  line-height: 1.5;
}
.art-diff__row {
  display: flex;
  align-items: baseline;
  gap: 6px;
}
.art-diff__sign {
  font-family: var(--hand-display);
  font-weight: 700;
  font-size: 13px;
  flex: none;
}
.art-diff__sign--good { color: var(--good); }
.art-diff__sign--bad  { color: var(--bad); }
.art-diff__stat { color: var(--ink); }
.art-diff__meta { white-space: nowrap; }
</style>

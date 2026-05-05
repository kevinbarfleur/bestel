<script setup lang="ts">
interface Listing {
  name: string;
  price: string;
  meta?: string;
  tone?: 'good' | 'bad' | 'ink';
}

withDefaults(
  defineProps<{
    title?: string;
    meta?: string;
    listings?: Listing[];
  }>(),
  {
    title: 'official trade',
    meta: '',
    listings: () => [],
  },
);
</script>

<template>
  <div class="art-trade">
    <div class="art-trade__head">
      <a class="link art-trade__title">{{ title }}</a>
      <span v-if="meta" class="aside">{{ meta }}</span>
    </div>
    <div class="art-trade__rows">
      <div v-for="(l, i) in listings" :key="i" class="leader">
        <span class="leader__k leader__k--name">{{ l.name }}</span>
        <span class="leader__dots" />
        <span class="leader__v">
          <a class="link">{{ l.price }}</a>
          <span v-if="l.meta" class="leader__meta" :class="`leader__meta--${l.tone ?? 'ink'}`"> · {{ l.meta }}</span>
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.art-trade {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.art-trade__head {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex-wrap: nowrap;
  white-space: nowrap;
}
.art-trade__title {
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 600;
}
.art-trade__rows {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.leader {
  display: flex;
  align-items: baseline;
  gap: 6px;
}
.leader__k {
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.10em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 500;
  flex-shrink: 0;
  white-space: nowrap;
}
.leader__k--name {
  text-transform: lowercase;
  letter-spacing: 0.06em;
}
.leader__dots {
  flex: 1;
  min-width: 12px;
  border-bottom: 1px dotted var(--paper-line);
  transform: translateY(-3px);
}
.leader__v {
  font-family: var(--hand-display);
  font-size: 12px;
  font-weight: 600;
  color: var(--ink);
  flex: 0 0 auto;
  white-space: nowrap;
}
.leader__meta {
  font-family: var(--hand);
  font-size: 11px;
}
.leader__meta--good { color: var(--good); }
.leader__meta--bad  { color: var(--bad); }
.leader__meta--ink  { color: var(--ink-faint); }
</style>

<script setup lang="ts">
import { computed } from 'vue';

import { PickerSectionHead } from '../pickers';

interface ItemMod {
  kind: 'implicit' | 'enchant' | 'explicit' | 'crafted' | 'fractured';
  text: string;
}
interface ComparisonDelta {
  stat: string;
  delta: string;
  tone?: 'good' | 'bad' | 'note';
}
interface Comparison {
  replaces: string;
  deltas: ComparisonDelta[];
}
export interface ItemCardPayload {
  name: string;
  base?: string;
  rarity?: 'normal' | 'magic' | 'rare' | 'unique' | 'relic' | string;
  ilvl?: number | string;
  slot?: string;
  mods?: ItemMod[];
  comparison?: Comparison;
}

const props = defineProps<{ payload: unknown }>();

const data = computed<ItemCardPayload>(() => {
  // Tolerant: payload comes from the LLM and may not match the schema.
  const p = (props.payload ?? {}) as Partial<ItemCardPayload>;
  return {
    name: p.name ?? 'Unknown item',
    base: p.base,
    rarity: p.rarity,
    ilvl: p.ilvl,
    slot: p.slot,
    mods: Array.isArray(p.mods) ? p.mods : [],
    comparison: p.comparison,
  };
});

const subline = computed(() => {
  const parts: string[] = [];
  if (data.value.slot) parts.push(data.value.slot);
  if (data.value.base) parts.push(data.value.base);
  if (data.value.rarity) parts.push(data.value.rarity);
  if (data.value.ilvl) parts.push(`ilvl ${data.value.ilvl}`);
  return parts.join(' · ');
});

const rarityClass = computed(() => {
  const r = (data.value.rarity ?? '').toLowerCase();
  if (r.includes('unique')) return 'rarity-unique';
  if (r.includes('rare')) return 'rarity-rare';
  if (r.includes('magic')) return 'rarity-magic';
  if (r.includes('relic')) return 'rarity-relic';
  return '';
});

function modToneClass(kind: string): string {
  return `mod-${kind}`;
}

function deltaToneStyle(tone?: string): string {
  switch (tone) {
    case 'good': return 'color: var(--good); font-weight: var(--fw-semibold);';
    case 'bad': return 'color: var(--bad); font-weight: var(--fw-semibold);';
    case 'note': return 'color: var(--note); font-weight: var(--fw-semibold);';
    default: return 'color: var(--ink);';
  }
}
</script>

<template>
  <div class="panel-item">
    <header class="panel-item__head">
      <h3 class="panel-item__name" :class="rarityClass">{{ data.name }}</h3>
      <p v-if="subline" class="panel-item__sub">{{ subline }}</p>
    </header>

    <section v-if="data.mods && data.mods.length" class="panel-item__section">
      <PickerSectionHead>Modifiers</PickerSectionHead>
      <ul class="panel-item__mods">
        <li
          v-for="(m, i) in data.mods"
          :key="i"
          :class="modToneClass(m.kind)"
        >
          <span class="panel-item__mod-kind">{{ m.kind }}</span>
          <span class="panel-item__mod-text">{{ m.text }}</span>
        </li>
      </ul>
    </section>

    <section v-if="data.comparison" class="panel-item__section">
      <PickerSectionHead>
        Replaces
        <template #right>{{ data.comparison.replaces }}</template>
      </PickerSectionHead>
      <ul class="panel-item__deltas">
        <li v-for="(d, i) in data.comparison.deltas" :key="i" class="leader-row">
          <span class="leader-row__k">{{ d.stat }}</span>
          <span class="leader-row__dots" />
          <span class="leader-row__v" :style="deltaToneStyle(d.tone)">{{ d.delta }}</span>
        </li>
      </ul>
    </section>

    <p v-if="!data.mods?.length && !data.comparison" class="panel-item__empty">
      No structured detail provided.
    </p>
  </div>
</template>

<style scoped>
.panel-item {
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.panel-item__head {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.panel-item__name {
  margin: 0;
  font-family: var(--hand);
  font-size: var(--fs-h2);
  font-weight: var(--fw-bold);
  line-height: 1.2;
  color: var(--ink);
}

/* Rarity colors — use existing PoB tokens. */
.panel-item__name.rarity-unique { color: var(--pob-rarity-unique); }
.panel-item__name.rarity-rare   { color: var(--pob-rarity-rare); }
.panel-item__name.rarity-magic  { color: var(--pob-rarity-magic); }
.panel-item__name.rarity-relic  { color: var(--pob-rarity-relic); }

.panel-item__sub {
  margin: 0;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}

.panel-item__section {
  display: flex;
  flex-direction: column;
}

.panel-item__mods {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.panel-item__mods li {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding: 4px 0;
  font-family: var(--hand);
  font-size: var(--fs-body);
  line-height: 1.45;
  color: var(--ink);
}
.panel-item__mod-kind {
  font-family: var(--label);
  font-size: var(--fs-micro);
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink-faint);
  flex: none;
  width: 64px;
}
.panel-item__mod-text { flex: 1; }

/* Mod-kind color hints — implicit/enchant slightly different. */
.panel-item__mods li.mod-implicit .panel-item__mod-text { color: var(--ink-soft); }
.panel-item__mods li.mod-enchant  .panel-item__mod-text { color: var(--el-cold-deep); }
.panel-item__mods li.mod-crafted  .panel-item__mod-text { color: var(--el-cold-deep); font-style: italic; }
.panel-item__mods li.mod-fractured .panel-item__mod-text { color: var(--el-lit-deep); }

.panel-item__deltas {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.panel-item__empty {
  margin: 0;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-faint);
}
</style>

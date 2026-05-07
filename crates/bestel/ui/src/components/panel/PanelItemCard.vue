<script setup lang="ts">
import { computed } from 'vue';

import { useToastsStore } from '../../stores/toasts';
import { PickerSectionHead } from '../pickers';
import RunicIcon from '../runic/RunicIcon.vue';

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

const toasts = useToastsStore();

const data = computed<ItemCardPayload>(() => {
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

function modKindLabel(kind: string): string {
  // Implicit/explicit are conventional and don't need a small-caps tag —
  // their color carries the meaning. The other three benefit from a label.
  if (kind === 'crafted' || kind === 'enchant' || kind === 'fractured') return kind;
  return '';
}

function deltaToneStyle(tone?: string): string {
  switch (tone) {
    case 'good': return 'color: var(--good); font-weight: 600;';
    case 'bad': return 'color: var(--bad); font-weight: 600;';
    case 'note': return 'color: var(--note); font-weight: 600;';
    default: return 'color: var(--ink);';
  }
}

function onOpenInBuilder() {
  toasts.push({
    variant: 'info',
    title: 'Builder integration coming.',
  });
}

function onFindOnTrade() {
  toasts.push({
    variant: 'info',
    title: 'Trade integration coming in Phase 2.',
  });
}
</script>

<template>
  <div class="panel-item">
    <p v-if="subline" class="panel-item__sub">{{ subline }}</p>

    <section v-if="data.mods && data.mods.length" class="panel-item__mods-card">
      <div
        v-for="(m, i) in data.mods"
        :key="i"
        class="panel-item__mod"
        :class="`mod-${m.kind}`"
      >
        <span v-if="modKindLabel(m.kind)" class="panel-item__mod-kind">{{ modKindLabel(m.kind) }}</span>
        <span class="panel-item__mod-text">{{ m.text }}</span>
      </div>
    </section>

    <section v-if="data.comparison" class="panel-item__section">
      <PickerSectionHead>Comparison</PickerSectionHead>
      <p class="panel-item__replaces">
        Replaces <strong>{{ data.comparison.replaces }}</strong>
      </p>
      <ul class="panel-item__deltas">
        <li v-for="(d, i) in data.comparison.deltas" :key="i" class="leader-row">
          <span class="leader-row__k">{{ d.stat }}</span>
          <span class="leader-row__dots" />
          <span class="leader-row__v" :style="deltaToneStyle(d.tone)">{{ d.delta }}</span>
        </li>
      </ul>
    </section>

    <div class="panel-item__cta">
      <button type="button" class="panel-item__primary-btn" @click="onOpenInBuilder">
        <RunicIcon name="open" :size="14" />
        <span>Open in builder</span>
      </button>
      <button type="button" class="panel-item__link-btn" @click="onFindOnTrade">
        Find a similar craft on trade…
      </button>
    </div>

    <p v-if="!data.mods?.length && !data.comparison" class="panel-item__empty">
      No structured detail provided.
    </p>
  </div>
</template>

<style scoped>
.panel-item {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.panel-item__sub {
  margin: 0;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink-soft);
}

/* v9 mods card — single bordered surface, paper bg. Color carries the mod
 * type semantics; small-caps prefix only on crafted / enchant / fractured. */
.panel-item__mods-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 14px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 4px;
}

.panel-item__mod {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-family: var(--hand);
  font-size: 14.5px;
  line-height: 1.35;
  color: var(--ink);
}

.panel-item__mod-kind {
  flex: none;
  padding-top: 2px;
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  font-weight: 700;
}

.panel-item__mod.mod-implicit { color: var(--ink-soft); }
.panel-item__mod.mod-explicit { color: var(--ink); }
.panel-item__mod.mod-crafted,
.panel-item__mod.mod-crafted .panel-item__mod-kind { color: var(--accent, #4d7da8); }
.panel-item__mod.mod-enchant,
.panel-item__mod.mod-enchant .panel-item__mod-kind { color: var(--accent-enchant, #6b4d8a); }
.panel-item__mod.mod-fractured,
.panel-item__mod.mod-fractured .panel-item__mod-kind { color: var(--note, #b88a2a); }

.panel-item__section {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.panel-item__replaces {
  margin: 0 0 6px;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink-soft);
}
.panel-item__replaces strong {
  color: var(--ink);
  font-weight: 600;
}

.panel-item__deltas {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

/* CTA stack — primary solid-ink button + link-style trade hint. */
.panel-item__cta {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 4px;
}

.panel-item__primary-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 10px 16px;
  background: var(--ink);
  color: var(--paper);
  border: 1px solid var(--ink);
  border-radius: 4px;
  font-family: var(--hand);
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s ease, opacity 0.15s ease;
}
.panel-item__primary-btn:hover {
  opacity: 0.92;
}

.panel-item__link-btn {
  align-self: center;
  padding: 4px 0;
  background: transparent;
  border: 0;
  font-family: var(--hand);
  font-size: 14px;
  font-weight: 500;
  color: var(--amber);
  text-decoration: underline dotted var(--amber-soft, var(--amber));
  text-underline-offset: 3px;
  cursor: pointer;
  transition: color 0.15s ease;
}
.panel-item__link-btn:hover {
  color: var(--ink);
}

.panel-item__empty {
  margin: 0;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink-faint);
}
</style>

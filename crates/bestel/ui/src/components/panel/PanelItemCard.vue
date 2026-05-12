<script setup lang="ts">
import { computed, ref } from 'vue';

import { useToastsStore } from '../../stores/toasts';
import { useChatStore } from '../../stores/chat';
import { buildItemTradeUrl, openExternal } from '../../api/tauri';
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

const toasts = useToastsStore();
const chat = useChatStore();
const tradeBusy = ref(false);

/** Map an item's slot label to the trade-site category filter. The trade
 *  API uses fine-grained slot names ("armour.helmet" / "weapon.bow") so a
 *  user with no resolved mods still lands on the right pane. Best-effort:
 *  unmatched slots return null and the search runs without a category. */
function inferTradeCategory(slot: string | undefined): string | null {
  if (!slot) return null;
  const s = slot.toLowerCase();
  if (s.includes('helmet')) return 'armour.helmet';
  if (s.includes('body')) return 'armour.chest';
  if (s.includes('glove')) return 'armour.gloves';
  if (s.includes('boot')) return 'armour.boots';
  if (s.includes('shield')) return 'armour.shield';
  if (s.includes('quiver')) return 'armour.quiver';
  if (s.includes('amulet')) return 'accessory.amulet';
  if (s.includes('belt')) return 'accessory.belt';
  if (s.includes('ring')) return 'accessory.ring';
  if (s.includes('flask')) return 'flask';
  if (s.includes('jewel')) return 'jewel';
  if (s.includes('bow')) return 'weapon.bow';
  if (s.includes('claw')) return 'weapon.claw';
  if (s.includes('dagger')) return 'weapon.dagger';
  if (s.includes('staff')) return 'weapon.staff';
  if (s.includes('wand')) return 'weapon.wand';
  if (s.includes('mace')) return 'weapon.onemace';
  if (s.includes('sword')) return 'weapon.onesword';
  if (s.includes('axe')) return 'weapon.oneaxe';
  return null;
}

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

async function onFindOnTrade() {
  if (tradeBusy.value) return;
  const mods = (data.value.mods ?? []).filter((m) => m.text.trim().length > 0);
  if (mods.length === 0) {
    toasts.push({
      variant: 'info',
      title: 'No mods to search for.',
      body: 'This item card has no explicit mods to build a trade query from.',
    });
    return;
  }
  const game = (chat.activeBuild?.game ?? 'poe1') as string;
  const rarity = typeof data.value.rarity === 'string' ? data.value.rarity.toLowerCase() : undefined;
  const category = inferTradeCategory(data.value.slot ?? undefined);
  tradeBusy.value = true;
  try {
    const result = await buildItemTradeUrl({
      game,
      mods: mods.map((m) => ({ kind: m.kind, text: m.text })),
      rarity: rarity && ['normal', 'magic', 'rare', 'unique'].includes(rarity) ? rarity : null,
      category,
    });
    // Open in the OS default browser so the user's pathofexile.com session
    // is reused and the listings show real prices + contact info.
    await openExternal(result.url);
    const summary: string[] = [`${result.total} listings in ${result.league}.`];
    if (result.unresolved_mods.length > 0) {
      summary.push(`${result.unresolved_mods.length} mod(s) couldn't be mapped — refine on the trade page.`);
    }
    toasts.push({
      variant: 'success',
      title: 'Trade search opened',
      body: summary.join(' '),
    });
  } catch (e) {
    toasts.push({
      variant: 'error',
      title: 'Trade search failed',
      body: e instanceof Error ? e.message : String(e),
    });
  } finally {
    tradeBusy.value = false;
  }
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
      <button
        type="button"
        class="panel-item__primary-btn"
        :disabled="tradeBusy"
        @click="onFindOnTrade"
      >
        <span v-if="tradeBusy">Building trade query…</span>
        <span v-else>Find a similar craft on trade…</span>
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

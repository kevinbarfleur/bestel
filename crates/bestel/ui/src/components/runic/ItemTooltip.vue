<script setup lang="ts">
import { computed } from 'vue';

import { parsePobItem } from '../../api/pob-item';
import type { ModSource, Rarity } from '../../api/pob-item';

const props = defineProps<{
  raw: string;
  slotName?: string | null;
}>();

const parsed = computed(() => parsePobItem(props.raw));

const RARITY_COLOR: Record<Rarity, string> = {
  normal: 'var(--ink)',
  magic: 'var(--el-cold)',
  rare: '#c9a227',
  unique: 'var(--amber)',
  gem: 'var(--good)',
  currency: 'var(--ink-soft)',
  relic: '#82ad6a',
};
const RARITY_BORDER: Record<Rarity, string> = {
  normal: 'var(--ink-soft)',
  magic: 'var(--el-cold)',
  rare: '#c9a227',
  unique: 'var(--amber)',
  gem: 'var(--good)',
  currency: 'var(--ink-soft)',
  relic: '#82ad6a',
};
const RARITY_BG: Record<Rarity, string> = {
  normal: 'transparent',
  magic: 'rgba(58, 110, 138, 0.05)',
  rare: 'rgba(201, 162, 39, 0.05)',
  unique: 'rgba(175, 96, 37, 0.05)',
  gem: 'rgba(84, 124, 74, 0.05)',
  currency: 'transparent',
  relic: 'rgba(130, 173, 106, 0.05)',
};

const SOURCE_COLOR: Record<ModSource, string> = {
  implicit: 'var(--ink-soft)',
  explicit: 'var(--el-cold-deep)',
  crafted: 'var(--el-cold)',
  enchant: 'var(--el-chaos)',
  fractured: 'var(--el-lit)',
  eater: 'var(--el-chaos-deep)',
  exarch: 'var(--el-fire)',
  unveiled: 'var(--amber)',
  corrupted: 'var(--bad)',
};

const SOURCE_LABEL: Record<ModSource, string> = {
  implicit: '',
  explicit: '',
  crafted: 'crafted',
  enchant: 'enchant',
  fractured: 'fractured',
  eater: 'eater',
  exarch: 'exarch',
  unveiled: 'unveiled',
  corrupted: 'corrupted',
};

const rarityColor = computed(() => RARITY_COLOR[parsed.value.rarity]);
const rarityBorder = computed(() => RARITY_BORDER[parsed.value.rarity]);
const rarityBg = computed(() => RARITY_BG[parsed.value.rarity]);

const propsRow = computed(() => {
  const out: { label: string; value: string }[] = [];
  const p = parsed.value;
  if (p.quality != null) out.push({ label: 'Quality', value: `+${p.quality}%` });
  if (p.sockets) out.push({ label: 'Sockets', value: p.sockets });
  if (p.levelReq != null) out.push({ label: 'Req. Level', value: String(p.levelReq) });
  if (p.itemLevel != null) out.push({ label: 'Item Level', value: String(p.itemLevel) });
  return out.concat(p.properties);
});
</script>

<template>
  <div
    class="item-tt"
    :class="`item-tt--${parsed.rarity}`"
    :style="{ borderTopColor: rarityBorder, background: rarityBg }"
  >
    <!-- Rarity label -->
    <div class="item-tt__rarity-band" :style="{ background: rarityBorder }">
      <span :style="{ color: 'var(--paper)' }">{{ parsed.rarity }}</span>
    </div>

    <!-- Name + base -->
    <div class="item-tt__head">
      <div v-if="parsed.name && parsed.name !== parsed.base" class="item-tt__name" :style="{ color: rarityColor }">
        {{ parsed.name }}
      </div>
      <div v-if="parsed.base" class="item-tt__base" :style="{ color: parsed.name ? 'var(--ink-soft)' : rarityColor }">
        {{ parsed.base }}
      </div>
      <div v-if="slotName" class="item-tt__slot">{{ slotName }}</div>
    </div>

    <!-- Properties -->
    <div v-if="propsRow.length" class="item-tt__sep" />
    <div v-if="propsRow.length" class="item-tt__props">
      <div v-for="p in propsRow" :key="p.label" class="item-tt__prop">
        <span class="item-tt__prop-k">{{ p.label }}</span>
        <span class="item-tt__prop-dots" />
        <span class="item-tt__prop-v">{{ p.value }}</span>
      </div>
    </div>

    <!-- Implicits -->
    <div v-if="parsed.implicits.length" class="item-tt__sep" />
    <div v-if="parsed.implicits.length" class="item-tt__mods item-tt__mods--implicit">
      <div v-for="(m, i) in parsed.implicits" :key="`i-${i}`" class="item-tt__mod">
        {{ m.text }}
      </div>
    </div>

    <!-- Explicits -->
    <div v-if="parsed.explicits.length" class="item-tt__sep" />
    <div v-if="parsed.explicits.length" class="item-tt__mods">
      <div
        v-for="(m, i) in parsed.explicits"
        :key="`e-${i}`"
        class="item-tt__mod"
        :style="{ color: SOURCE_COLOR[m.source] }"
      >
        <span v-if="SOURCE_LABEL[m.source]" class="item-tt__mod-tag">{{ SOURCE_LABEL[m.source] }}</span>
        {{ m.text }}
      </div>
    </div>

    <!-- Flags -->
    <div v-if="parsed.flags.length" class="item-tt__sep" />
    <div v-if="parsed.flags.length" class="item-tt__flags">
      <span
        v-for="f in parsed.flags"
        :key="f"
        class="item-tt__flag"
        :class="{ 'item-tt__flag--corrupted': f === 'Corrupted' }"
      >{{ f }}</span>
    </div>
  </div>
</template>

<style scoped>
.item-tt {
  font-family: var(--hand);
  border-top: 2px solid var(--ink-soft);
  margin: -8px -12px;
  padding: 0 0 8px;
  min-width: 240px;
  max-width: 360px;
}

.item-tt__rarity-band {
  padding: 4px 12px 4px;
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  font-weight: 600;
  color: var(--paper);
  display: flex;
  align-items: center;
  gap: 6px;
}

.item-tt__head {
  padding: 8px 12px 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  text-align: center;
}
.item-tt__name {
  font-family: var(--hand-display);
  font-size: 16px;
  font-weight: 700;
  line-height: 1.15;
  letter-spacing: 0.01em;
}
.item-tt__base {
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 600;
  line-height: 1.2;
}
.item-tt__slot {
  margin-top: 4px;
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
}

.item-tt__sep {
  height: 1px;
  background: var(--paper-line);
  margin: 4px 12px;
}

.item-tt__props {
  padding: 0 12px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.item-tt__prop {
  display: flex;
  align-items: baseline;
  gap: 6px;
  font-size: 12px;
}
.item-tt__prop-k {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.10em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 500;
}
.item-tt__prop-dots {
  flex: 1;
  min-width: 12px;
  border-bottom: 1px dotted var(--paper-line);
  transform: translateY(-3px);
}
.item-tt__prop-v {
  font-family: var(--hand-display);
  font-weight: 600;
  color: var(--ink);
}

.item-tt__mods {
  padding: 0 12px;
  display: flex;
  flex-direction: column;
  gap: 3px;
  font-size: 12px;
  line-height: 1.4;
}
.item-tt__mods--implicit { color: var(--ink-soft); }
.item-tt__mod-tag {
  display: inline-block;
  font-family: var(--label);
  font-size: 8px;
  letter-spacing: 0.10em;
  text-transform: uppercase;
  margin-right: 4px;
  opacity: 0.7;
  font-weight: 600;
}

.item-tt__flags {
  padding: 0 12px;
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}
.item-tt__flag {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-soft);
  padding: 2px 6px;
  border: 1px dashed var(--ink-faint);
  border-radius: 3px;
}
.item-tt__flag--corrupted {
  color: var(--bad);
  border-color: var(--bad);
}
</style>

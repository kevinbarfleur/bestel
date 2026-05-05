<script setup lang="ts">
import { computed } from 'vue';

import type { ItemSummaryDto } from '../../api/types';
import SlotIcon from './SlotIcon.vue';

const props = defineProps<{ item: ItemSummaryDto }>();

const rarity = computed(() => (props.item.rarity ?? 'normal').toLowerCase());

const displayName = computed(() => props.item.name ?? props.item.base ?? 'Sans nom');

const slotLabel = computed(() => props.item.slot ?? 'inconnu');

const showBaseLine = computed(() => Boolean(props.item.name && props.item.base));
</script>

<template>
  <article class="item-card" :class="`item-card--${rarity}`">
    <div class="item-card__head">
      <SlotIcon :slot="item.slot" class="item-card__slot-icon" />
      <span class="item-card__slot-label">{{ slotLabel }}</span>
    </div>
    <div class="item-card__body">
      <p class="item-card__name">{{ displayName }}</p>
      <p v-if="showBaseLine" class="item-card__base">{{ item.base }}</p>
    </div>
  </article>
</template>

<style scoped>
.item-card {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 0.45rem 0.55rem;
  background: rgba(15, 14, 12, 0.55);
  border: 1px solid rgba(50, 46, 42, 0.5);
  border-radius: 4px;
  transition: border-color 0.18s ease;
}

.item-card:hover { border-color: rgba(175, 96, 37, 0.45); }

.item-card__head {
  display: flex;
  align-items: center;
  gap: 0.35rem;
}

.item-card__slot-icon { color: var(--color-text-dim, #7f7f7f); }

.item-card__slot-label {
  font-family: 'Cinzel', serif;
  font-size: 0.6rem;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--color-text-dim, #7f7f7f);
}

.item-card__name {
  margin: 0;
  font-family: 'Cinzel', serif;
  font-size: 0.82rem;
  letter-spacing: 0.04em;
  line-height: 1.3;
  color: var(--pob-rarity-normal);
}

.item-card__base {
  margin: 0;
  font-family: 'Crimson Text', serif;
  font-size: 0.78rem;
  font-style: italic;
  color: var(--color-text-dim, #7f7f7f);
}

.item-card--normal .item-card__name { color: var(--pob-rarity-normal); }
.item-card--magic .item-card__name { color: var(--pob-rarity-magic); }
.item-card--rare .item-card__name { color: var(--pob-rarity-rare); }
.item-card--unique .item-card__name { color: var(--pob-rarity-unique); }
.item-card--relic .item-card__name { color: var(--pob-rarity-relic); }
.item-card--gem .item-card__name { color: var(--pob-rarity-gem); }

.item-card--unique { border-color: rgba(175, 96, 37, 0.45); background: rgba(40, 22, 12, 0.55); }
.item-card--rare { border-color: rgba(255, 255, 119, 0.32); background: rgba(40, 38, 12, 0.45); }
.item-card--magic { border-color: rgba(136, 136, 255, 0.32); background: rgba(20, 22, 38, 0.6); }
.item-card--relic { border-color: rgba(130, 173, 106, 0.32); }
</style>

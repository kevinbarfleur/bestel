<script setup lang="ts">
import { computed } from 'vue';

import type { ResistanceDto } from '../../api/types';

const props = defineProps<{ resistances: ResistanceDto[] }>();

const CAP = 75;

interface ResVm {
  name: ResistanceDto['name'];
  label: string;
  value: number | null;
  status: 'over' | 'cap' | 'under' | 'missing';
  fill: number;
}

const items = computed<ResVm[]>(() =>
  props.resistances.map((r) => {
    const v = r.value;
    const fill = Math.min(100, Math.max(0, v ?? 0));
    let status: ResVm['status'] = 'missing';
    if (v == null) status = 'missing';
    else if (v >= CAP + 0.01) status = 'over';
    else if (Math.abs(v - CAP) < 0.5) status = 'cap';
    else status = 'under';
    return {
      name: r.name,
      label: { fire: 'Feu', cold: 'Froid', lightning: 'Foudre', chaos: 'Chaos' }[r.name],
      value: v,
      status,
      fill,
    };
  }),
);

const fmt = (v: number | null): string => {
  if (v == null) return '—';
  return `${v >= 0 ? '' : ''}${v.toFixed(0)}%`;
};
</script>

<template>
  <div class="res-grid">
    <div
      v-for="r in items"
      :key="r.name"
      class="res-cell"
      :class="[`res-cell--${r.name}`, `res-cell--${r.status}`]"
      :data-tooltip-text="`${r.label} : ${fmt(r.value)}` + (r.status === 'under' ? ' (sous le cap 75%)' : r.status === 'over' ? ' (overcapped)' : ' (au cap)')"
      :data-tooltip-title="r.label"
    >
      <div class="res-cell__head">
        <span class="res-cell__name">{{ r.label }}</span>
        <span class="res-cell__value">{{ fmt(r.value) }}</span>
      </div>
      <div class="res-cell__bar">
        <span class="res-cell__bar-fill" :style="{ width: r.fill + '%' }" />
        <span class="res-cell__bar-cap" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.res-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.4rem;
}

.res-cell {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  padding: 0.4rem 0.55rem;
  background: rgba(15, 14, 12, 0.55);
  border: 1px solid rgba(50, 46, 42, 0.45);
  border-radius: 4px;
}

.res-cell--fire { border-color: rgba(201, 41, 41, 0.4); background: var(--pob-fire-bg); }
.res-cell--cold { border-color: rgba(54, 100, 146, 0.5); background: var(--pob-cold-bg); }
.res-cell--lightning { border-color: rgba(255, 215, 0, 0.32); background: var(--pob-lightning-bg); }
.res-cell--chaos { border-color: rgba(208, 32, 144, 0.4); background: var(--pob-chaos-bg); }

.res-cell__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.4rem;
}

.res-cell__name {
  font-family: 'Cinzel', serif;
  font-size: 0.7rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--color-text-dim, #aa9);
}

.res-cell--fire .res-cell__name { color: var(--pob-fire); }
.res-cell--cold .res-cell__name { color: var(--pob-cold); }
.res-cell--lightning .res-cell__name { color: var(--pob-lightning); }
.res-cell--chaos .res-cell__name { color: var(--pob-chaos); }

.res-cell__value {
  font-family: 'Crimson Text', serif;
  font-size: 0.95rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  color: var(--color-text-primary, #e8e6e3);
  letter-spacing: 0.02em;
}

.res-cell--under .res-cell__value { color: var(--color-error, #c45050); }
.res-cell--cap .res-cell__value { color: var(--color-gold, #c9a227); }
.res-cell--over .res-cell__value { color: var(--color-success, #4a9f5a); }

.res-cell__bar {
  position: relative;
  height: 5px;
  background: rgba(0, 0, 0, 0.55);
  border-radius: 2px;
  overflow: hidden;
  border: 1px solid rgba(60, 55, 50, 0.5);
}

.res-cell__bar-fill {
  position: absolute;
  inset: 0 auto 0 0;
  background: var(--pob-fire);
  transition: width 0.4s cubic-bezier(0.03, 0.98, 0.52, 0.99);
}

.res-cell--fire .res-cell__bar-fill { background: var(--pob-fire); }
.res-cell--cold .res-cell__bar-fill { background: var(--pob-cold); }
.res-cell--lightning .res-cell__bar-fill { background: var(--pob-lightning); }
.res-cell--chaos .res-cell__bar-fill { background: var(--pob-chaos); }

.res-cell--under .res-cell__bar-fill { background: var(--color-error, #c45050); }

.res-cell__bar-cap {
  position: absolute;
  top: -1px;
  bottom: -1px;
  left: 75%;
  width: 1.5px;
  background: rgba(255, 255, 255, 0.55);
  box-shadow: 0 0 4px rgba(255, 255, 255, 0.5);
}
</style>

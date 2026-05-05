<script setup lang="ts">
import { computed } from 'vue';

import type { PobBuildDto } from '../../api/types';

const props = defineProps<{ build: PobBuildDto }>();

interface Row {
  key: string;
  label: string;
  value: number | null;
  className: string;
  format: 'int' | 'dps';
}

const rows = computed<Row[]>(() => {
  const list: Row[] = [
    { key: 'life', label: 'Vie', value: props.build.life, className: 'vit--life', format: 'int' },
    { key: 'mana', label: 'Mana', value: props.build.mana, className: 'vit--mana', format: 'int' },
  ];
  if (props.build.energy_shield != null) {
    list.push({
      key: 'es',
      label: 'Bouclier',
      value: props.build.energy_shield,
      className: 'vit--es',
      format: 'int',
    });
  }
  if (props.build.ehp != null) {
    list.push({
      key: 'ehp',
      label: 'EHP',
      value: props.build.ehp,
      className: 'vit--ehp',
      format: 'int',
    });
  }
  if (props.build.dps != null) {
    list.push({
      key: 'dps',
      label: 'Combined DPS',
      value: props.build.dps,
      className: 'vit--dps',
      format: 'dps',
    });
  }
  return list;
});

const fmt = (v: number | null, kind: 'int' | 'dps'): string => {
  if (v == null) return '—';
  if (kind === 'dps') {
    if (v >= 1_000_000) return `${(v / 1_000_000).toFixed(2)} M`;
    if (v >= 1000) return `${(v / 1000).toFixed(1)} k`;
    return Math.round(v).toString();
  }
  if (Math.abs(v) >= 1000) return Math.round(v).toLocaleString('fr-FR');
  return v.toFixed(0);
};
</script>

<template>
  <table class="vitals">
    <tbody>
      <tr v-for="r in rows" :key="r.key">
        <th scope="row">
          <span class="vitals__dot" :class="r.className" />
          {{ r.label }}
        </th>
        <td :class="r.className">{{ fmt(r.value, r.format) }}</td>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
.vitals {
  width: 100%;
  border-collapse: collapse;
  font-family: 'Crimson Text', serif;
  font-size: 0.92rem;
}

.vitals tbody tr {
  border-bottom: 1px dashed rgba(60, 55, 50, 0.4);
}

.vitals tbody tr:last-child {
  border-bottom: 0;
}

.vitals th {
  text-align: left;
  padding: 0.32rem 0.4rem 0.32rem 0;
  font-weight: 400;
  color: var(--color-text-dim, #aa9);
  letter-spacing: 0.04em;
}

.vitals td {
  text-align: right;
  padding: 0.32rem 0 0.32rem 0.4rem;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
  letter-spacing: 0.02em;
}

.vitals__dot {
  display: inline-block;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  margin-right: 0.45rem;
  vertical-align: middle;
  box-shadow: 0 0 6px currentColor;
}

.vit--life { color: var(--pob-life, #ff6464); }
.vit--mana { color: var(--pob-mana, #6e8bff); }
.vit--es { color: var(--pob-energy-shield, #88ffee); }
.vit--ehp { color: var(--color-text-primary, #e8e6e3); }
.vit--dps { color: var(--color-gold, #c9a227); }

.vit--life.vitals__dot { background: var(--pob-life); }
.vit--mana.vitals__dot { background: var(--pob-mana); }
.vit--es.vitals__dot { background: var(--pob-energy-shield); }
.vit--ehp.vitals__dot { background: var(--color-text-primary); }
.vit--dps.vitals__dot { background: var(--color-gold); }
</style>

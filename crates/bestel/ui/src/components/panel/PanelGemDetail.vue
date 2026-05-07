<script setup lang="ts">
import { computed } from 'vue';

import { PickerSectionHead } from '../pickers';

interface ScalingRow {
  stat: string;
  value: string | number;
}
interface SupportRow {
  name: string;
  role?: string;
}

export interface GemDetailPayload {
  name: string;
  level?: number | string;
  quality?: number | string;
  tags?: string[];
  scaling?: ScalingRow[];
  recommended_supports?: SupportRow[];
}

const props = defineProps<{ payload: unknown }>();

const data = computed<GemDetailPayload>(() => {
  const p = (props.payload ?? {}) as Partial<GemDetailPayload>;
  return {
    name: p.name ?? 'Unknown gem',
    level: p.level,
    quality: p.quality,
    tags: Array.isArray(p.tags) ? p.tags : [],
    scaling: Array.isArray(p.scaling) ? p.scaling : [],
    recommended_supports: Array.isArray(p.recommended_supports)
      ? p.recommended_supports
      : [],
  };
});

const subline = computed(() => {
  const parts: string[] = [];
  if (data.value.level) parts.push(`level ${data.value.level}`);
  if (data.value.quality) parts.push(`${data.value.quality}% quality`);
  return parts.join(' · ');
});
</script>

<template>
  <div class="panel-gem">
    <header class="panel-gem__head">
      <h3 class="panel-gem__name">{{ data.name }}</h3>
      <p v-if="subline" class="panel-gem__sub">{{ subline }}</p>
      <div v-if="data.tags && data.tags.length" class="panel-gem__tags">
        <span v-for="t in data.tags" :key="t" class="panel-gem__tag">{{ t }}</span>
      </div>
    </header>

    <section v-if="data.scaling && data.scaling.length" class="panel-gem__section">
      <PickerSectionHead>Scaling</PickerSectionHead>
      <ul class="panel-gem__rows">
        <li v-for="(s, i) in data.scaling" :key="i" class="leader-row">
          <span class="leader-row__k">{{ s.stat }}</span>
          <span class="leader-row__dots" />
          <span class="leader-row__v">{{ s.value }}</span>
        </li>
      </ul>
    </section>

    <section
      v-if="data.recommended_supports && data.recommended_supports.length"
      class="panel-gem__section"
    >
      <PickerSectionHead>Recommended supports</PickerSectionHead>
      <ul class="panel-gem__supports">
        <li v-for="(sup, i) in data.recommended_supports" :key="i">
          <span class="panel-gem__support-name">{{ sup.name }}</span>
          <span v-if="sup.role" class="panel-gem__support-role">{{ sup.role }}</span>
        </li>
      </ul>
    </section>

    <p
      v-if="!data.scaling?.length && !data.recommended_supports?.length"
      class="panel-gem__empty"
    >
      No structured detail provided.
    </p>
  </div>
</template>

<style scoped>
.panel-gem {
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.panel-gem__head {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.panel-gem__name {
  margin: 0;
  font-family: var(--hand);
  font-size: var(--fs-h2);
  font-weight: var(--fw-bold);
  line-height: 1.2;
  color: var(--ink);
}

.panel-gem__sub {
  margin: 0;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}

.panel-gem__tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 4px;
}
.panel-gem__tag {
  font-family: var(--label);
  font-size: var(--fs-micro);
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink-soft);
  background: var(--paper);
  border: 1px solid var(--paper-line);
  padding: 2px 8px;
  border-radius: 3px;
  font-weight: var(--fw-semibold);
}

.panel-gem__section {
  display: flex;
  flex-direction: column;
}

.panel-gem__rows {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.panel-gem__supports {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.panel-gem__supports li {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 6px 10px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 4px;
}
.panel-gem__support-name {
  font-family: var(--hand);
  font-size: var(--fs-body);
  font-weight: var(--fw-semibold);
  color: var(--ink);
}
.panel-gem__support-role {
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}

.panel-gem__empty {
  margin: 0;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-faint);
}
</style>

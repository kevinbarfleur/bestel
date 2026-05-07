<script setup lang="ts">
import { computed } from 'vue';
import { useBuildStore } from '../../stores/build';
import { renderMarkdown } from '../../api/markdown';

interface MechanicSection {
  heading: string;
  body_md: string;
}
export interface MechanicPayload {
  summary?: string;
  sections?: MechanicSection[];
}

const props = defineProps<{ payload: unknown }>();

const buildStore = useBuildStore();

const data = computed<MechanicPayload>(() => {
  const p = (props.payload ?? {}) as Partial<MechanicPayload>;
  return {
    summary: p.summary,
    sections: Array.isArray(p.sections) ? p.sections : [],
  };
});

const game = computed(() => buildStore.current?.game ?? 'poe1');

function renderBody(md: string): string {
  return renderMarkdown(md, game.value);
}
</script>

<template>
  <div class="panel-mech">
    <p v-if="data.summary" class="panel-mech__summary">{{ data.summary }}</p>

    <section
      v-for="(sec, i) in data.sections"
      :key="i"
      class="panel-mech__section"
    >
      <h4 class="panel-mech__heading">{{ sec.heading }}</h4>
      <div class="markdown-body" v-html="renderBody(sec.body_md ?? '')" />
    </section>

    <p
      v-if="!data.summary && (!data.sections || data.sections.length === 0)"
      class="panel-mech__empty"
    >
      No structured detail provided.
    </p>
  </div>
</template>

<style scoped>
.panel-mech {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.panel-mech__summary {
  margin: 0;
  padding: 12px 14px;
  background: var(--paper);
  border-left: 3px solid var(--paper-line);
  border-radius: 0 4px 4px 0;
  font-family: var(--hand);
  font-size: var(--fs-body);
  line-height: 1.55;
  color: var(--ink);
}

.panel-mech__section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.panel-mech__heading {
  margin: 0;
  font-family: var(--label);
  font-size: var(--fs-caps);
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: var(--fw-semibold);
  padding-bottom: 6px;
  border-bottom: 1px solid var(--paper-line);
}

.panel-mech__empty {
  margin: 0;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-faint);
}
</style>

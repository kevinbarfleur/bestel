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

    <div v-if="data.sections && data.sections.length" class="panel-mech__sections">
      <section
        v-for="(sec, i) in data.sections"
        :key="i"
        class="panel-mech__section"
        :class="{ 'panel-mech__section--last': i === data.sections.length - 1 }"
      >
        <h4 class="panel-mech__heading">{{ sec.heading }}</h4>
        <div class="markdown-body panel-mech__body" v-html="renderBody(sec.body_md ?? '')" />
      </section>
    </div>

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

/* v9 summary — full bordered card, not a left-border quote. Sets the
 * tone for the rest of the panel. */
.panel-mech__summary {
  margin: 0;
  padding: 12px 14px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 4px;
  font-family: var(--hand);
  font-size: 15px;
  line-height: 1.55;
  color: var(--ink);
}

.panel-mech__sections {
  display: flex;
  flex-direction: column;
}

/* v9 sections — dotted dividers between siblings, no border on the last
 * one. Vertical rhythm comes from padding+margin not gap so the dotted
 * line sits between sections at a consistent height. */
.panel-mech__section {
  display: flex;
  flex-direction: column;
  padding-bottom: 14px;
  margin-bottom: 14px;
  border-bottom: 1px dotted var(--paper-line);
}
.panel-mech__section--last {
  padding-bottom: 0;
  margin-bottom: 0;
  border-bottom: 0;
}

.panel-mech__heading {
  margin: 0 0 6px;
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 600;
}

.panel-mech__body {
  font-size: 14.5px;
  line-height: 1.55;
  color: var(--ink);
}

.panel-mech__empty {
  margin: 0;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink-faint);
}
</style>

<script setup lang="ts">
import { computed } from 'vue';

import { renderMarkdown } from '../../api/markdown';
import { INTERVIEW_SECTION_ORDER } from '../../stores/sheet';
import BSSectionHead from './BSSectionHead.vue';

/**
 * Reusable read-only renderer for a Build Sheet. Used by both
 * `BSSheetFullModal` (sidebar `View full` button) and the `BuildPicker`
 * sheets pane. Doesn't touch any store — props provide everything so
 * the same view works for an *active* sheet (with `useSheetStore`) and
 * for a *picker-fetched* sheet that may not be the currently linked one.
 */

interface BuildSheetSection {
  id: string;
  label: string;
  body: string;
  drafted?: boolean;
  confirmed?: boolean;
}
interface DefiningItem {
  name: string;
  role: string;
  purpose?: string | null;
}
interface IntentEntry { constraint: string; }
interface KnownGap { label: string; note?: string | null; }
interface BuildSheetPayload {
  sections?: BuildSheetSection[];
  defining_items?: DefiningItem[];
  intent?: IntentEntry[];
  known_gaps?: KnownGap[];
}

const props = defineProps<{
  /** Parsed sheet payload — same shape Rust serializes from `BuildSheet`.
   *  When null/undefined the view renders an empty-state line. */
  payload: unknown;
  /** Resolved game flavor for the markdown wiki-link router. */
  game: 'poe1' | 'poe2';
}>();

const typedPayload = computed<BuildSheetPayload | null>(
  () => (props.payload as BuildSheetPayload | undefined) ?? null,
);

const orderedSections = computed<BuildSheetSection[]>(() => {
  const list = typedPayload.value?.sections ?? [];
  return [...list].sort(
    (a, b) =>
      INTERVIEW_SECTION_ORDER.indexOf(a.id as typeof INTERVIEW_SECTION_ORDER[number]) -
      INTERVIEW_SECTION_ORDER.indexOf(b.id as typeof INTERVIEW_SECTION_ORDER[number]),
  );
});

const definingItems = computed<DefiningItem[]>(() => typedPayload.value?.defining_items ?? []);
const intentList = computed<IntentEntry[]>(() => typedPayload.value?.intent ?? []);
const knownGaps = computed<KnownGap[]>(() => typedPayload.value?.known_gaps ?? []);

function bodyHtml(body: string): string {
  return renderMarkdown(body, props.game);
}

const definingSectionIndex = computed(() => orderedSections.value.length + 1);
const intentSectionIndex = computed(
  () => orderedSections.value.length + (definingItems.value.length ? 1 : 0) + 1,
);
const gapsSectionIndex = computed(
  () =>
    orderedSections.value.length
    + (definingItems.value.length ? 1 : 0)
    + (intentList.value.length ? 1 : 0)
    + 1,
);
</script>

<template>
  <div v-if="typedPayload" class="bs-full">
    <!-- Sections -->
    <div
      v-for="(section, i) in orderedSections"
      :key="section.id"
      class="bs-full__section"
    >
      <BSSectionHead :index="i + 1">{{ section.label || section.id }}</BSSectionHead>
      <div
        class="bs-full__body"
        v-html="bodyHtml(section.body)"
      />
    </div>

    <!-- Defining items -->
    <div v-if="definingItems.length" class="bs-full__section">
      <BSSectionHead :index="definingSectionIndex">Defining items</BSSectionHead>
      <table class="bs-full__items">
        <tbody>
          <tr v-for="it in definingItems" :key="it.name">
            <td class="bs-full__role" :class="`bs-full__role--${it.role}`">{{ it.role }}</td>
            <td class="bs-full__name">{{ it.name }}</td>
            <td v-if="it.purpose" class="bs-full__purpose">{{ it.purpose }}</td>
            <td v-else class="bs-full__purpose bs-full__purpose--empty">—</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Intent -->
    <div v-if="intentList.length" class="bs-full__section">
      <BSSectionHead :index="intentSectionIndex">Intent &amp; constraints</BSSectionHead>
      <ul class="bs-full__bullets">
        <li v-for="i in intentList" :key="i.constraint">{{ i.constraint }}</li>
      </ul>
    </div>

    <!-- Known gaps -->
    <div v-if="knownGaps.length" class="bs-full__section">
      <BSSectionHead :index="gapsSectionIndex">Known gaps</BSSectionHead>
      <ul class="bs-full__bullets">
        <li v-for="g in knownGaps" :key="g.label">
          <strong>{{ g.label }}</strong>
          <span v-if="g.note" class="bs-full__gap-note"> · {{ g.note }}</span>
        </li>
      </ul>
    </div>
  </div>
  <div v-else class="bs-full__empty">
    No sheet payload to display.
  </div>
</template>

<style scoped>
.bs-full {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 4px 0 8px;
}
.bs-full__empty {
  padding: 32px 8px;
  text-align: center;
  color: var(--ink-faint);
  font-family: var(--hand);
  font-style: italic;
}

.bs-full__section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.bs-full__body {
  margin: 4px 0 0 32px;
  font-family: var(--hand);
  font-size: 15px;
  color: var(--ink);
  line-height: 1.55;
  overflow-wrap: anywhere;
}
.bs-full__body :deep(p) { margin: 0 0 8px 0; }
.bs-full__body :deep(p:last-child) { margin-bottom: 0; }
.bs-full__body :deep(ul),
.bs-full__body :deep(ol) {
  margin: 4px 0 8px;
  padding-left: 20px;
}
.bs-full__body :deep(li) { margin-bottom: 2px; }
.bs-full__body :deep(strong) { color: var(--ink); font-weight: 600; }
.bs-full__body :deep(em) { color: var(--ink-soft); }
.bs-full__body :deep(code) {
  font-family: ui-monospace, monospace;
  font-size: 0.9em;
  background: var(--paper-shade);
  padding: 1px 5px;
  border-radius: 3px;
}

/* ─── Defining items table ───────────────────────────────────────────── */
.bs-full__items {
  margin: 4px 0 0 32px;
  border-collapse: collapse;
  width: calc(100% - 32px);
}
.bs-full__items td {
  padding: 6px 10px 6px 0;
  vertical-align: baseline;
  font-family: var(--hand);
  font-size: 14px;
  border-bottom: 1px dotted var(--paper-line);
}
.bs-full__items tr:last-child td { border-bottom: 0; }
.bs-full__role {
  width: 88px;
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 700;
  white-space: nowrap;
}
.bs-full__role--engine { color: var(--amber); }
.bs-full__role--defining { color: var(--ink); }
.bs-full__role--amplifier { color: var(--ink-soft); }
.bs-full__role--enabler { color: var(--ink-soft); }
.bs-full__role--filler { color: var(--ink-faint); }
.bs-full__name {
  width: 200px;
  color: var(--ink);
  font-weight: 600;
}
.bs-full__purpose {
  color: var(--ink-soft);
  font-style: italic;
}
.bs-full__purpose--empty {
  color: var(--ink-faint);
  font-style: normal;
}

/* ─── Intent / Known gaps bullets ─────────────────────────────────── */
.bs-full__bullets {
  margin: 4px 0 0 32px;
  padding-left: 20px;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink);
  line-height: 1.55;
}
.bs-full__bullets li { margin-bottom: 4px; }
.bs-full__gap-note {
  color: var(--ink-soft);
  font-style: italic;
}
</style>

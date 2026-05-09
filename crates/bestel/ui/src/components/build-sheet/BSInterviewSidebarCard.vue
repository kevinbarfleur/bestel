<script setup lang="ts">
import { computed } from 'vue';
import { storeToRefs } from 'pinia';

import { useSheetStore } from '../../stores/sheet';
import { INTERVIEW_SECTION_ORDER } from '../../stores/sheet';

/**
 * Sidebar progress card surfaced while a Build Sheet interview is being
 * filled. One row per agent question (Sprint UX-2.8 — flat per-question
 * checklist replacing the prior per-section status). Each row ticks off
 * the moment the user picks at least one chip OR types Other text in the
 * panel — instant feedback so the user knows when every question has
 * been addressed and Submit becomes meaningful.
 *
 * Visibility: live whenever `activeInterview` exists AND has not been
 * submitted. Once the user submits, this card disappears and
 * `BSLinkedSheetCard` (the finalized state) takes over.
 */

const sheet = useSheetStore();
const { activeInterview } = storeToRefs(sheet);

type Status = 'done' | 'queued';

interface Row {
  questionId: string;
  title: string;
  status: Status;
}

const visible = computed(
  () => activeInterview.value !== null && activeInterview.value.submitted === false,
);

/** Flat, canonical-order list of questions. We iterate
 *  `INTERVIEW_SECTION_ORDER` so the sidebar order matches the panel's
 *  visual order, then collect every question for that section. A row is
 *  `done` as soon as the answer has at least one selected chip OR a
 *  non-empty Other text. */
const rows = computed<Row[]>(() => {
  const iv = activeInterview.value;
  if (!iv) return [];
  const out: Row[] = [];
  for (const sectionId of INTERVIEW_SECTION_ORDER) {
    const qs = iv.questions.filter((q) => q.sectionId === sectionId);
    for (const q of qs) {
      const a = iv.answers.get(q.questionId);
      const answered = !!a && (a.selected.length > 0 || a.otherText.trim().length > 0);
      out.push({
        questionId: q.questionId,
        title: q.title,
        status: answered ? 'done' : 'queued',
      });
    }
  }
  return out;
});

const total = computed(() => rows.value.length);
const done = computed(() => rows.value.filter((r) => r.status === 'done').length);

function onSkip() {
  sheet.cancelInterview();
}
</script>

<template>
  <div v-if="visible" class="bs-iv-side">
    <div class="bs-iv-side__head">
      <span class="bs-iv-side__badge">
        <span class="bs-iv-side__glyph">◆</span>
        Drafting · {{ done }}/{{ total }}
      </span>
    </div>
    <div v-if="rows.length" class="bs-iv-side__list">
      <div
        v-for="row in rows"
        :key="row.questionId"
        class="bs-iv-side__row"
        :class="`bs-iv-side__row--${row.status}`"
      >
        <span class="bs-iv-side__dot" :class="`bs-iv-side__dot--${row.status}`">
          <span v-if="row.status === 'done'" class="bs-iv-side__check">✓</span>
        </span>
        <span class="bs-iv-side__qtitle" :title="row.title">{{ row.title }}</span>
      </div>
    </div>
    <div v-else class="bs-iv-side__empty">
      No leverage questions on this turn — review the section bodies and submit.
    </div>
    <div class="bs-iv-side__foot">
      <button type="button" class="bs-iv-side__skip" @click="onSkip">
        Skip &amp; save draft
      </button>
    </div>
  </div>
</template>

<style scoped>
.bs-iv-side {
  flex: 0 0 auto;
  width: 100%;
  padding: 14px 16px;
  border: 1px solid var(--amber);
  border-radius: 5px;
  background: var(--amber-glow);
  display: flex;
  flex-direction: column;
  gap: 12px;
  box-sizing: border-box;
}

.bs-iv-side__head {
  display: flex;
  align-items: center;
  gap: 8px;
}
.bs-iv-side__badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 2px 8px;
  border: 1px solid var(--amber);
  border-radius: 3px;
  background: rgba(175, 96, 37, 0.18);
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--amber);
  font-weight: 700;
  white-space: nowrap;
}
.bs-iv-side__glyph {
  font-family: var(--hand);
  font-size: 12px;
  line-height: 1;
  letter-spacing: 0;
}

/* ─── Per-question rows ─────────────────────────────────────────────── */
.bs-iv-side__list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.bs-iv-side__row {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  color: var(--ink);
  min-width: 0;
}
.bs-iv-side__row--queued { color: var(--ink-soft); }
.bs-iv-side__row--done .bs-iv-side__qtitle {
  color: var(--ink-soft);
  text-decoration: line-through;
  text-decoration-color: var(--paper-line);
  text-decoration-thickness: 1px;
  text-underline-offset: 0;
}

/* ─── Status dot — solid green when done, dashed grey when queued ─── */
.bs-iv-side__dot {
  width: 13px;
  height: 13px;
  border-radius: 50%;
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
}
.bs-iv-side__dot--done {
  background: var(--good);
  color: var(--paper);
}
.bs-iv-side__dot--queued {
  border: 1.2px dashed var(--ink-faint);
  background: transparent;
}
.bs-iv-side__check {
  font-size: 8px;
  font-weight: 700;
  line-height: 1;
}

.bs-iv-side__qtitle {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--hand);
  line-height: 1.4;
}

.bs-iv-side__empty {
  font-family: var(--hand);
  font-size: 13px;
  color: var(--ink-soft);
  line-height: 1.5;
  font-style: italic;
}

/* ─── Footer ──────────────────────────────────────────────────────────── */
.bs-iv-side__foot {
  padding-top: 4px;
  border-top: 1px dotted var(--paper-line);
  margin-top: 2px;
}
.bs-iv-side__skip {
  background: transparent;
  border: 0;
  padding: 0;
  font-family: var(--hand);
  font-size: 13px;
  color: var(--ink-soft);
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 3px;
}
.bs-iv-side__skip:hover { color: var(--amber); }
</style>

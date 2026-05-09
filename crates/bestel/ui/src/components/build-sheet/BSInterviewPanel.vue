<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import { renderMarkdown } from '../../api/markdown';
import { useBuildStore } from '../../stores/build';
import { useChatStore } from '../../stores/chat';
import { useSheetStore } from '../../stores/sheet';
import type { InterviewQuestion, InterviewSection } from '../../stores/sheet';
import { INTERVIEW_SECTION_ORDER } from '../../stores/sheet';
import BSSectionHead from './BSSectionHead.vue';

/**
 * One-shot Build Sheet interview panel (Sprint UX-2).
 *
 * Two render modes share this component:
 *
 * 1. **Active** (`!submitted`) — interactive form: chips, Other textareas,
 *    section editors, footer with Submit/Cancel. The agent emitted the
 *    payload via `sheet_open_interview`; the user fills the entire form
 *    in one round, then clicks Submit.
 *
 * 2. **Summary** (`submitted`) — compact read-only summary that stays in
 *    the chat after submission. Each question's answer is rendered inline
 *    next to its title; section bodies are rendered as plain prose; no
 *    chips, no buttons.
 *
 * Submission round-trips back to the agent as a structured `[INTERVIEW
 * SUBMISSION]` message via `chatStore.send`, then the agent calls
 * `sheet_finalize_request` and answers the original question.
 */

const chat = useChatStore();
const sheet = useSheetStore();
const buildStore = useBuildStore();
const { activeInterview, interviewReady } = storeToRefs(sheet);

/** PoE game (poe1 / poe2) for the markdown renderer's wiki-link router. */
const renderGame = computed(() => buildStore.current?.game ?? 'poe1');

const orderedSections = computed<InterviewSection[]>(() => {
  if (!activeInterview.value) return [];
  return [...activeInterview.value.sections].sort(
    (a, b) =>
      INTERVIEW_SECTION_ORDER.indexOf(a.id) -
      INTERVIEW_SECTION_ORDER.indexOf(b.id),
  );
});

function questionsFor(sectionId: InterviewQuestion['sectionId']) {
  if (!activeInterview.value) return [];
  return activeInterview.value.questions.filter((q) => q.sectionId === sectionId);
}

/** Sections the agent always pre-drafts from PoB analysis (vs. axes that
 *  rely on user purpose). These get the paper-shade box + ✓ confirmed
 *  badge treatment from the design's `FormSection kind='confirm'`. */
function isConfirmStyle(sectionId: string): boolean {
  return sectionId === 'identity' || sectionId === 'archetype';
}

/** Per-section ephemeral edit toggle. Outside the store because it's
 *  pure UI state — toggling Edit doesn't mutate persisted answers until
 *  the user types something + clicks Done. */
const editingSection = ref<Set<string>>(new Set());
const editBuffer = ref<Map<string, string>>(new Map());

function startEditingSection(sectionId: string, currentBody: string) {
  editingSection.value.add(sectionId);
  if (!editBuffer.value.has(sectionId)) {
    editBuffer.value.set(sectionId, currentBody);
  }
  editingSection.value = new Set(editingSection.value);
}

function commitSectionEdit(sectionId: string) {
  const body = editBuffer.value.get(sectionId) ?? '';
  sheet.setSectionEdit(sectionId, body.trim());
  editingSection.value.delete(sectionId);
  editingSection.value = new Set(editingSection.value);
}

function cancelSectionEdit(sectionId: string) {
  editingSection.value.delete(sectionId);
  editingSection.value = new Set(editingSection.value);
}

function bodyFor(sectionId: string, draftBody: string): string {
  return activeInterview.value?.sectionEdits.get(sectionId) ?? draftBody;
}

function isEdited(sectionId: string): boolean {
  return activeInterview.value?.sectionEdits.has(sectionId) ?? false;
}

/** Render the section body through the project's markdown pipeline so
 *  the agent can use **bold**, lists, italics, inline code, etc. for
 *  readability. */
function bodyHtml(sectionId: string, draftBody: string): string {
  return renderMarkdown(bodyFor(sectionId, draftBody), renderGame.value);
}

/** Toggle a chip selection. Honors `multi` — single-select replaces;
 *  multi appends/removes. Always coexists with Other text. */
function toggleChip(question: InterviewQuestion, idx: number) {
  const iv = activeInterview.value;
  if (!iv) return;
  const ans = iv.answers.get(question.questionId) ?? { selected: [], otherText: '' };
  let next: number[];
  if (question.multi) {
    next = ans.selected.includes(idx)
      ? ans.selected.filter((i) => i !== idx)
      : [...ans.selected, idx];
  } else {
    next = ans.selected.includes(idx) ? [] : [idx];
  }
  sheet.setAnswer(question.questionId, next, ans.otherText);
}

const otherExpanded = ref<Set<string>>(new Set());

function toggleOther(question: InterviewQuestion) {
  if (otherExpanded.value.has(question.questionId)) {
    otherExpanded.value.delete(question.questionId);
  } else {
    otherExpanded.value.add(question.questionId);
  }
  otherExpanded.value = new Set(otherExpanded.value);
}

function isOtherActive(question: InterviewQuestion): boolean {
  if (otherExpanded.value.has(question.questionId)) return true;
  const ans = activeInterview.value?.answers.get(question.questionId);
  return Boolean(ans && ans.otherText.trim().length > 0);
}

function otherTextFor(question: InterviewQuestion): string {
  return activeInterview.value?.answers.get(question.questionId)?.otherText ?? '';
}

function setOtherText(question: InterviewQuestion, text: string) {
  const iv = activeInterview.value;
  if (!iv) return;
  const ans = iv.answers.get(question.questionId) ?? { selected: [], otherText: '' };
  sheet.setAnswer(question.questionId, ans.selected, text);
}

function selectedFor(question: InterviewQuestion): number[] {
  return activeInterview.value?.answers.get(question.questionId)?.selected ?? [];
}

const notes = computed({
  get: () => activeInterview.value?.notes ?? '',
  set: (v: string) => sheet.setNotes(v),
});

const submitted = computed(() => activeInterview.value?.submitted === true);

/** Live progress counter for the footer + parity with the sidebar card.
 *  A question counts as "answered" when at least one chip is selected OR
 *  the Other text is non-empty. */
const totalQuestions = computed(() => activeInterview.value?.questions.length ?? 0);
const answeredCount = computed(() => {
  const iv = activeInterview.value;
  if (!iv) return 0;
  let n = 0;
  for (const q of iv.questions) {
    const a = iv.answers.get(q.questionId);
    if (!a) continue;
    if (a.selected.length > 0 || a.otherText.trim().length > 0) n += 1;
  }
  return n;
});

/** Render a question's answer as a plain string for the summary mode.
 *  Selected option labels joined by ` · `, optionally followed by the
 *  Other text. Returns `(skipped)` when nothing is selected. */
function formatAnswer(question: InterviewQuestion): string {
  const ans = activeInterview.value?.answers.get(question.questionId);
  if (!ans) return '(skipped)';
  const selectedLabels = ans.selected
    .map((i) => question.options[i])
    .filter((s): s is string => Boolean(s));
  const other = ans.otherText.trim();
  const parts: string[] = [];
  if (selectedLabels.length > 0) parts.push(selectedLabels.join(' · '));
  if (other) parts.push(`other: ${other}`);
  return parts.length > 0 ? parts.join(' | ') : '(skipped)';
}

async function onSubmit() {
  if (!interviewReady.value || !activeInterview.value) return;
  const message = sheet.submitInterview();
  await chat.send(message);
}

function onCancel() {
  sheet.cancelInterview();
}

/** Mirror the live sheet store state onto the chat-history segment so
 *  closing the app mid-interview persists the user's selections. */
watch(
  activeInterview,
  (iv) => {
    if (!iv) return;
    const answers: Record<string, { selected: number[]; otherText: string }> = {};
    for (const [k, v] of iv.answers.entries()) {
      answers[k] = { selected: [...v.selected], otherText: v.otherText };
    }
    const sectionEdits: Record<string, string> = {};
    for (const [k, v] of iv.sectionEdits.entries()) {
      sectionEdits[k] = v;
    }
    chat.updateSheetInterviewState({
      answers,
      sectionEdits,
      notes: iv.notes,
      submitted: iv.submitted,
    });
  },
  { deep: true },
);
</script>

<template>
  <!-- ─── ACTIVE MODE — interactive interview form ─────────────────────── -->
  <div v-if="activeInterview && !submitted" class="bs-iv">
    <!-- Header -->
    <div class="bs-iv__head">
      <span class="bs-iv__tag">◆ build sheet · interview</span>
      <span class="bs-iv__rule" />
      <span class="bs-iv__head-meta">
        {{ orderedSections.length + 1 }} sections · saved on send
      </span>
    </div>
    <p class="bs-iv__lede">
      Confirm or correct each section, then answer the questions below. One round, then I finalize the sheet
      and answer your original question.
    </p>

    <!-- Sections -->
    <div
      v-for="(section, i) in orderedSections"
      :key="section.id"
      class="bs-iv__section"
    >
      <BSSectionHead :index="i + 1">{{ section.title }}</BSSectionHead>
      <div v-if="isConfirmStyle(section.id)" class="bs-iv__sec-hint">
        I read this from the PoB. Confirm or edit.
      </div>

      <!-- Read-only body + Edit affordance — confirm-style for identity/archetype -->
      <template v-if="!editingSection.has(section.id) && isConfirmStyle(section.id)">
        <div class="bs-iv__confirm">
          <div
            class="bs-iv__confirm-body"
            v-html="bodyHtml(section.id, section.draftBody)"
          />
          <div class="bs-iv__confirm-meta">
            <span
              v-if="!isEdited(section.id)"
              class="bs-iv__confirm-pill bs-iv__confirm-pill--ok"
            >✓ confirmed</span>
            <span v-else class="bs-iv__confirm-pill bs-iv__confirm-pill--edited">
              ✎ edited
            </span>
            <button
              type="button"
              class="bs-iv__edit-link"
              @click="startEditingSection(section.id, bodyFor(section.id, section.draftBody))"
            >Edit</button>
          </div>
        </div>
      </template>

      <!-- Read-only body + Edit affordance — plain markdown for the rest -->
      <template v-else-if="!editingSection.has(section.id)">
        <div
          class="bs-iv__body"
          v-html="bodyHtml(section.id, section.draftBody)"
        />
        <button
          type="button"
          class="bs-iv__edit-link"
          @click="startEditingSection(section.id, bodyFor(section.id, section.draftBody))"
        >+ Edit this body</button>
      </template>

      <!-- Inline edit textarea -->
      <template v-else>
        <textarea
          class="bs-iv__edit-area"
          :value="editBuffer.get(section.id) ?? ''"
          rows="4"
          @input="(e) => editBuffer.set(section.id, (e.target as HTMLTextAreaElement).value)"
        />
        <div class="bs-iv__edit-actions">
          <button
            type="button"
            class="bs-iv__btn bs-iv__btn--primary"
            @click="commitSectionEdit(section.id)"
          >✓ Done</button>
          <button
            type="button"
            class="bs-iv__btn bs-iv__btn--ghost"
            @click="cancelSectionEdit(section.id)"
          >Cancel</button>
        </div>
      </template>

      <!-- Per-section questions -->
      <div
        v-for="q in questionsFor(section.id)"
        :key="q.questionId"
        class="bs-iv__q"
      >
        <div class="bs-iv__q-title">{{ q.title }}</div>
        <div v-if="q.subtitle" class="bs-iv__q-sub">{{ q.subtitle }}</div>
        <div class="bs-iv__chips">
          <button
            v-for="(opt, idx) in q.options"
            :key="idx"
            type="button"
            class="bs-iv__chip"
            :class="{ 'bs-iv__chip--sel': selectedFor(q).includes(idx) }"
            @click="toggleChip(q, idx)"
          >
            <span
              v-if="q.multi"
              class="bs-iv__check"
              :class="{ 'bs-iv__check--sel': selectedFor(q).includes(idx) }"
            >{{ selectedFor(q).includes(idx) ? '✓' : '' }}</span>
            {{ opt }}
          </button>
          <button
            v-if="q.hasOther"
            type="button"
            class="bs-iv__chip bs-iv__chip--other"
            :class="{ 'bs-iv__chip--sel': isOtherActive(q) }"
            @click="toggleOther(q)"
          >
            <span
              v-if="q.multi"
              class="bs-iv__check"
              :class="{ 'bs-iv__check--sel': isOtherActive(q) }"
            >{{ isOtherActive(q) ? '✓' : '' }}</span>
            Other…
          </button>
        </div>
        <div
          v-if="otherExpanded.has(q.questionId) || otherTextFor(q).length > 0"
          class="bs-iv__other-wrap"
        >
          <label class="bs-iv__other-label">Custom answer</label>
          <textarea
            class="bs-iv__other-area"
            :value="otherTextFor(q)"
            placeholder="Type your own answer…"
            rows="3"
            @input="(e) => setOtherText(q, (e.target as HTMLTextAreaElement).value)"
          />
        </div>
        <div v-if="q.multi" class="bs-iv__hint">
          Pick one or more
          <span class="bs-iv__hint-strong">· {{ selectedFor(q).length }} selected</span>
        </div>
      </div>
    </div>

    <!-- Notes -->
    <div class="bs-iv__section">
      <BSSectionHead :index="orderedSections.length + 1">Notes</BSSectionHead>
      <p v-if="activeInterview.notesPrompt" class="bs-iv__q-sub bs-iv__q-sub--notes">
        {{ activeInterview.notesPrompt }}
      </p>
      <textarea
        class="bs-iv__notes"
        :value="notes"
        rows="4"
        placeholder="Anything else about your goals, gear constraints, or play style?"
        @input="(e) => (notes = (e.target as HTMLTextAreaElement).value)"
      />
    </div>

    <!-- Footer -->
    <div class="bs-iv__footer">
      <button type="button" class="bs-iv__btn bs-iv__btn--ghost" @click="onCancel">Cancel</button>
      <span class="bs-iv__spacer" />
      <span v-if="!interviewReady" class="bs-iv__hint">
        Answer every question before submitting.
      </span>
      <span class="bs-iv__counter">
        {{ answeredCount }} of {{ totalQuestions }} reviewed
      </span>
      <button
        type="button"
        class="bs-iv__btn bs-iv__btn--primary"
        :disabled="!interviewReady"
        @click="onSubmit"
      >✓ Save Build Sheet &amp; answer my question</button>
    </div>
  </div>

  <!-- ─── SUMMARY MODE — submitted, read-only digest ────────────────────── -->
  <div v-else-if="activeInterview && submitted" class="bs-iv bs-iv--summary">
    <!-- Header -->
    <div class="bs-iv__head">
      <span class="bs-iv__tag bs-iv__tag--ok">✓ build sheet · submitted</span>
      <span class="bs-iv__rule" />
      <span class="bs-iv__head-meta">
        {{ answeredCount }} of {{ totalQuestions }} answered
      </span>
    </div>

    <p class="bs-iv__lede bs-iv__lede--summary">
      Your answers, recorded. Bestel is finalizing the sheet and will answer your question next.
    </p>

    <!-- Sections (read-only) -->
    <div
      v-for="(section, i) in orderedSections"
      :key="section.id"
      class="bs-iv__section"
    >
      <BSSectionHead :index="i + 1">{{ section.title }}</BSSectionHead>
      <div
        class="bs-iv__body"
        v-html="bodyHtml(section.id, section.draftBody)"
      />

      <!-- Per-question answers, inline -->
      <div
        v-for="q in questionsFor(section.id)"
        :key="q.questionId"
        class="bs-iv__qsum"
      >
        <span class="bs-iv__qsum-title">{{ q.title }}</span>
        <span class="bs-iv__qsum-rule" />
        <span
          class="bs-iv__qsum-ans"
          :class="{ 'bs-iv__qsum-ans--skip': formatAnswer(q) === '(skipped)' }"
        >{{ formatAnswer(q) }}</span>
      </div>
    </div>

    <!-- Notes -->
    <div class="bs-iv__section">
      <BSSectionHead :index="orderedSections.length + 1">Notes</BSSectionHead>
      <div v-if="activeInterview.notes.trim().length > 0" class="bs-iv__notes-summary">
        {{ activeInterview.notes }}
      </div>
      <div v-else class="bs-iv__notes-summary bs-iv__notes-summary--empty">
        (no notes)
      </div>
    </div>
  </div>
</template>

<style scoped>
.bs-iv {
  margin-top: 12px;
  border: 1px solid var(--amber);
  border-radius: 6px;
  background: var(--paper);
  padding: 16px 18px 18px;
  overflow: hidden;
  box-sizing: border-box;
  width: 100%;
  min-width: 0;
  max-width: 100%;
}
.bs-iv * {
  min-width: 0;
}

/* Summary mode swaps the amber border for the muted "good" green so the
 * user sees at-a-glance that the panel is now a record, not a form. */
.bs-iv--summary {
  border-color: var(--good);
  background: var(--paper);
}

/* ─── Header ─────────────────────────────────────────────────────────── */
.bs-iv__head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--paper-line);
}
.bs-iv__tag {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--amber);
  font-weight: 700;
  white-space: nowrap;
}
.bs-iv__tag--ok {
  color: var(--good);
}
.bs-iv__rule {
  flex: 1;
  height: 1px;
  background: var(--paper-line);
}
.bs-iv__head-meta {
  font-size: 13px;
  color: var(--ink-faint);
  white-space: nowrap;
}

/* ─── Lede paragraph ─────────────────────────────────────────────────── */
.bs-iv__lede {
  margin: 12px 0 18px;
  color: var(--ink-soft);
  font-size: 14px;
  line-height: 1.5;
}
.bs-iv__lede--summary {
  color: var(--ink-soft);
}

/* ─── Sections ───────────────────────────────────────────────────────── */
.bs-iv__section {
  margin-top: 22px;
  max-width: 100%;
  min-width: 0;
}
.bs-iv__sec-hint {
  margin: 4px 0 8px 32px;
  font-size: 13px;
  color: var(--ink-soft);
  line-height: 1.5;
}

/* ─── Confirm-style box (identity, archetype) ────────────────────────── */
.bs-iv__confirm {
  margin: 6px 0 0 32px;
  padding: 10px 12px;
  border: 1px solid var(--paper-line);
  border-radius: 4px;
  background: var(--paper-shade);
  display: flex;
  align-items: flex-start;
  gap: 12px;
  flex-wrap: wrap;
}
.bs-iv__confirm-body {
  flex: 1 1 0;
  min-width: 220px;
  font-size: 14.5px;
  color: var(--ink);
  line-height: 1.55;
  overflow-wrap: anywhere;
}
.bs-iv__confirm-body :deep(p) { margin: 0 0 6px 0; }
.bs-iv__confirm-body :deep(p:last-child) { margin-bottom: 0; }
.bs-iv__confirm-body :deep(strong) { font-weight: 600; }
.bs-iv__confirm-body :deep(em) { color: var(--ink-soft); }
.bs-iv__confirm-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: none;
}
.bs-iv__confirm-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 9px;
  border-radius: 3px;
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  font-weight: 700;
}
.bs-iv__confirm-pill--ok {
  border: 1px solid var(--good);
  color: var(--good);
}
.bs-iv__confirm-pill--edited {
  border: 1px solid var(--amber);
  color: var(--amber);
}

/* ─── Plain markdown body (damage, defense, items, intent) ────────────── */
.bs-iv__body {
  margin: 0 0 0 32px;
  font-size: 15px;
  color: var(--ink);
  line-height: 1.5;
  overflow-wrap: anywhere;
  word-break: normal;
}
.bs-iv__body :deep(p) { margin: 0 0 8px 0; }
.bs-iv__body :deep(p:last-child) { margin-bottom: 0; }
.bs-iv__body :deep(ul),
.bs-iv__body :deep(ol) {
  margin: 4px 0 8px 0;
  padding-left: 20px;
}
.bs-iv__body :deep(li) { margin-bottom: 2px; }
.bs-iv__body :deep(strong) { color: var(--ink); font-weight: 600; }
.bs-iv__body :deep(em) { color: var(--ink-soft); }
.bs-iv__body :deep(code) {
  font-family: ui-monospace, monospace;
  font-size: 0.9em;
  background: var(--paper-shade);
  padding: 1px 5px;
  border-radius: 3px;
}
.bs-iv__edit-link {
  margin: 6px 0 0 32px;
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
.bs-iv__edit-link:hover { color: var(--amber); }
.bs-iv__confirm .bs-iv__edit-link { margin: 0; }

.bs-iv__edit-area,
.bs-iv__other-area,
.bs-iv__notes {
  width: 100%;
  margin-top: 8px;
  padding: 10px 12px;
  border: 1px solid var(--paper-line);
  border-radius: 4px;
  background: var(--paper-shade);
  color: var(--ink);
  font-family: var(--hand);
  font-size: 14px;
  line-height: 1.5;
  resize: vertical;
  box-sizing: border-box;
}
.bs-iv__edit-area:focus,
.bs-iv__other-area:focus,
.bs-iv__notes:focus {
  outline: none;
  border-color: var(--amber);
  background: var(--paper);
}
.bs-iv__edit-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 8px 0 0 32px;
}

/* ─── Per-section question card ───────────────────────────────────────── */
.bs-iv__q {
  margin: 16px 0 0 32px;
  padding: 12px 14px;
  border: 1px solid var(--paper-line);
  border-radius: 5px;
  background: var(--paper);
  box-sizing: border-box;
  max-width: calc(100% - 32px);
  min-width: 0;
  overflow: hidden;
}
.bs-iv__q-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--ink);
  line-height: 1.35;
}
.bs-iv__q-sub {
  margin-top: 4px;
  font-size: 13px;
  color: var(--ink-soft);
  line-height: 1.5;
}
.bs-iv__q-sub--notes {
  margin-left: 32px;
}
.bs-iv__chips {
  margin-top: 10px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  width: 100%;
  max-width: 100%;
  min-width: 0;
}
.bs-iv__chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  border: 1px solid var(--ink-soft);
  border-radius: 18px;
  background: var(--paper);
  color: var(--ink);
  font-family: var(--hand);
  font-size: 14px;
  font-weight: 500;
  line-height: 1.2;
  cursor: pointer;
  white-space: normal;
  text-align: left;
  max-width: 100%;
  overflow-wrap: anywhere;
}
.bs-iv__chip:hover:not(:disabled):not(.bs-iv__chip--sel) {
  border-color: var(--amber);
}
.bs-iv__chip:disabled {
  opacity: 0.6;
  cursor: default;
}
.bs-iv__chip--sel {
  background: var(--ink);
  color: var(--paper);
  border-color: var(--ink);
}
.bs-iv__chip--other {
  border-style: dashed;
  border-color: var(--ink-faint);
  background: var(--paper-shade);
  color: var(--ink-faint);
  font-style: italic;
}
.bs-iv__chip--other.bs-iv__chip--sel {
  background: var(--amber);
  color: var(--paper);
  border-color: var(--amber);
  border-style: solid;
  font-style: normal;
}
.bs-iv__check {
  width: 12px;
  height: 12px;
  border-radius: 2px;
  border: 1.4px solid var(--ink-soft);
  background: transparent;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--ink);
  font-size: 9px;
  font-weight: 800;
}
.bs-iv__check--sel {
  border-color: var(--paper);
  background: var(--paper);
}
.bs-iv__other-wrap {
  margin-top: 10px;
  border: 1px dashed var(--paper-line);
  border-radius: 4px;
  padding: 10px 12px;
  background: var(--paper-shade);
}
.bs-iv__other-label {
  display: block;
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 700;
  margin-bottom: 4px;
}
.bs-iv__hint {
  font-size: 13px;
  color: var(--ink-faint);
  font-family: var(--label);
  letter-spacing: 0.04em;
}
.bs-iv__hint-strong { color: var(--ink-soft); }

/* ─── Notes textarea (active mode) ────────────────────────────────────── */
.bs-iv .bs-iv__notes {
  margin-left: 32px;
  width: calc(100% - 32px);
}

/* ─── Footer ──────────────────────────────────────────────────────────── */
.bs-iv__footer {
  margin-top: 24px;
  padding-top: 14px;
  border-top: 1px dotted var(--paper-line);
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}
.bs-iv__counter {
  font-family: var(--label);
  font-size: 12px;
  letter-spacing: 0.06em;
  color: var(--ink-soft);
  font-weight: 600;
  white-space: nowrap;
}
.bs-iv__spacer { flex: 1; }
.bs-iv__btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  font-family: var(--hand);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border-radius: 4px;
}
.bs-iv__btn--primary {
  background: var(--ink);
  color: var(--paper);
  border: 1px solid var(--ink);
}
.bs-iv__btn--primary:hover:not(:disabled) {
  background: var(--ink-soft);
}
.bs-iv__btn--primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.bs-iv__btn--ghost {
  background: transparent;
  color: var(--ink-soft);
  border: 1px solid var(--paper-line);
}
.bs-iv__btn--ghost:hover {
  border-color: var(--ink-soft);
  color: var(--ink);
}

/* ─── Summary mode — per-question line + notes block ──────────────────── */
.bs-iv__qsum {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin: 8px 0 0 32px;
  padding-top: 6px;
  border-top: 1px dotted var(--paper-line);
  font-size: 13.5px;
  line-height: 1.5;
  flex-wrap: wrap;
}
.bs-iv__qsum-title {
  color: var(--ink);
  font-weight: 600;
  flex: 0 0 auto;
  max-width: 60%;
}
.bs-iv__qsum-rule {
  flex: 1 0 24px;
  height: 1px;
  background: var(--paper-line);
  align-self: center;
}
.bs-iv__qsum-ans {
  color: var(--ink-soft);
  flex: 1 1 60%;
  text-align: right;
  overflow-wrap: anywhere;
}
.bs-iv__qsum-ans--skip {
  color: var(--ink-faint);
  font-style: italic;
}
.bs-iv__notes-summary {
  margin-left: 32px;
  margin-top: 6px;
  padding: 10px 12px;
  border: 1px solid var(--paper-line);
  border-radius: 4px;
  background: var(--paper-shade);
  color: var(--ink);
  font-family: var(--hand);
  font-size: 14px;
  line-height: 1.55;
  font-style: italic;
  white-space: pre-wrap;
}
.bs-iv__notes-summary--empty {
  color: var(--ink-faint);
  font-style: italic;
}
</style>

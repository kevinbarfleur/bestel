<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import { renderMarkdown } from '../../api/markdown';
import { interceptAnchorClick } from '../../api/tauri';
import { useChatStore } from '../../stores/chat';
import type { SheetInterviewSegment } from '../../stores/chat';
import { useSheetStore } from '../../stores/sheet';
import type {
  ActiveInterview,
  InterviewAnswer,
  InterviewQuestion,
  InterviewSection,
} from '../../stores/sheet';
import { INTERVIEW_SECTION_ORDER } from '../../stores/sheet';
import BSSectionHead from './BSSectionHead.vue';

/**
 * Optional `segment` prop — when provided, the panel falls back to it
 * when the live sheet-store interview is null. This keeps the submitted
 * digest visible in chat history forever, including past the
 * `sheet_finalize_request` call (which nulls `activeInterview` so the
 * NEXT chat starts clean).
 */
const props = defineProps<{
  segment?: SheetInterviewSegment;
}>();

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
const { activeInterview, interviewReady } = storeToRefs(sheet);

/** PoE game (poe1 / poe2) for the markdown renderer's wiki-link router. */
const renderGame = computed(() => chat.activeBuild?.game ?? 'poe1');

/** Snapshot the segment's payload+state into an `ActiveInterview`-shaped
 * object. Used in history mode when the live store has been cleared
 * by `sheet_finalize_request` but the chat-history segment still carries
 * the full submission. */
function rehydratedFromSegment(): ActiveInterview | null {
  const seg = props.segment;
  if (!seg || !seg.payload) return null;
  const sections: InterviewSection[] = seg.payload.sections.map((s) => ({
    id: s.id as InterviewSection['id'],
    title: s.title,
    draftBody: s.draft_body,
  }));
  const questions: InterviewQuestion[] = seg.payload.questions.map((q) => ({
    questionId: q.question_id,
    sectionId: q.section_id as InterviewQuestion['sectionId'],
    title: q.title,
    subtitle: q.subtitle ?? null,
    options: q.options,
    multi: q.multi ?? false,
    hasOther: q.has_other ?? true,
  }));
  const answers = new Map<string, InterviewAnswer>();
  const state = seg.state;
  for (const q of questions) {
    const a = state?.answers?.[q.questionId];
    answers.set(q.questionId, {
      selected: a?.selected ? [...a.selected] : [],
      otherText: a?.otherText ?? '',
    });
  }
  const sectionEdits = new Map<string, string>();
  if (state?.sectionEdits) {
    for (const [k, v] of Object.entries(state.sectionEdits)) {
      sectionEdits.set(k, v);
    }
  }
  return {
    sections,
    questions,
    notesPrompt: seg.payload.notes_prompt,
    answers,
    sectionEdits,
    notes: state?.notes ?? '',
    submitted: state?.submitted ?? true,
  };
}

/** Live store wins; segment is the history fallback. When both are null,
 * the panel renders nothing (the v-if in ChatMessage.vue ensures the
 * component only mounts when there's something to show). */
const displayInterview = computed<ActiveInterview | null>(
  () => activeInterview.value ?? rehydratedFromSegment(),
);

/** True when we're showing a frozen submission from chat history (live
 * store is null but the segment carries the submitted payload). The
 * template uses this to skip the interactive-form branch entirely and
 * always render the read-only summary, regardless of `submitted` flag
 * on the segment state. */
const isHistoryOnly = computed(
  () => activeInterview.value === null && rehydratedFromSegment() !== null,
);

const orderedSections = computed<InterviewSection[]>(() => {
  const iv = displayInterview.value;
  if (!iv) return [];
  return [...iv.sections].sort(
    (a, b) =>
      INTERVIEW_SECTION_ORDER.indexOf(a.id) -
      INTERVIEW_SECTION_ORDER.indexOf(b.id),
  );
});

function questionsFor(sectionId: InterviewQuestion['sectionId']) {
  const iv = displayInterview.value;
  if (!iv) return [];
  return iv.questions.filter((q) => q.sectionId === sectionId);
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
  return displayInterview.value?.sectionEdits.get(sectionId) ?? draftBody;
}

function isEdited(sectionId: string): boolean {
  return displayInterview.value?.sectionEdits.has(sectionId) ?? false;
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

/** Submitted when the live store says so OR when we're showing a frozen
 *  history-only segment (which is always in submitted shape since the
 *  agent only reaches `sheet_finalize_request` after a real submission).
 */
const submitted = computed(
  () => displayInterview.value?.submitted === true || isHistoryOnly.value,
);

/** Live progress counter for the footer + parity with the sidebar card.
 *  A question counts as "answered" when at least one chip is selected OR
 *  the Other text is non-empty. */
const totalQuestions = computed(() => displayInterview.value?.questions.length ?? 0);
const answeredCount = computed(() => {
  const iv = displayInterview.value;
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
  const ans = displayInterview.value?.answers.get(question.questionId);
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

/** Show / hide the "Voir tout" overlay with the full submission digest.
 *  Local UI state — never persisted; tied to the lifetime of this
 *  component instance. Reopened on every visit. */
const showFullModal = ref(false);

/** Identity section's draftBody trimmed to one short line for the
 *  compact summary header. Strips leading `**...**` markdown emphasis
 *  and chops at the first ` | ` separator (typical pattern: "Inquisitor
 *  Penance Brand crit caster | Hybrid armour+life | 4040 life, 2060
 *  ES, 75% res"). Returns null when the identity section is absent or
 *  empty so the template can skip the line entirely. */
const identityHeadline = computed<string | null>(() => {
  const iv = displayInterview.value;
  if (!iv) return null;
  const id = iv.sections.find((s) => s.id === 'identity');
  if (!id) return null;
  const raw = bodyFor(id.id, id.draftBody);
  const cleaned = raw.replace(/\*\*/g, '').trim();
  if (!cleaned) return null;
  return cleaned.split(/\s*\|\s*/)[0].trim();
});

interface CompactAnswerRow {
  questionId: string;
  label: string;
  value: string;
}

/** Top-of-summary answer rows: every question with a non-skipped answer,
 *  formatted as `<short label> → <first-clause answer>`. The template
 *  caps the visible slice at 3 and renders a `+ N more` line for the
 *  rest, so a verbose interview doesn't blow past the ~80px target
 *  height in the compact card. */
const answeredQuestions = computed<CompactAnswerRow[]>(() => {
  const iv = displayInterview.value;
  if (!iv) return [];
  const rows: CompactAnswerRow[] = [];
  for (const q of iv.questions) {
    const formatted = formatAnswer(q);
    if (formatted === '(skipped)') continue;
    rows.push({
      questionId: q.questionId,
      label: q.title.replace(/[—–\-:].*$/, '').trim(),
      value: formatted.replace(/\s*\|\s.*$/, '').trim(),
    });
  }
  return rows;
});

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
            @click="interceptAnchorClick"
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
          @click="interceptAnchorClick"
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

  <!-- ─── SUMMARY MODE — submitted, compact digest ────────────────────────
       Renders both when the live store has just been submitted (between
       submit click and `sheet_finalize_request` completion) AND when only
       the chat-history segment carries the interview (post-finalize, the
       live store was nulled by `loadActiveSheet`). The `isHistoryOnly`
       branch keeps the panel visible forever in chat history. Compact
       layout (~80px) — full breakdown moved to the `Voir tout` modal. -->
  <div v-else-if="displayInterview && submitted" class="bs-iv bs-iv--summary">
    <!-- Header -->
    <div class="bs-iv__head">
      <span class="bs-iv__tag bs-iv__tag--ok">✓ build sheet · submitted</span>
      <span class="bs-iv__rule" />
      <span class="bs-iv__head-meta">
        {{ answeredCount }} of {{ totalQuestions }} answered
      </span>
    </div>

    <p v-if="identityHeadline" class="bs-iv__compact-identity">
      {{ identityHeadline }}
    </p>

    <div v-if="answeredQuestions.length" class="bs-iv__compact-answers">
      <div
        v-for="row in answeredQuestions.slice(0, 3)"
        :key="row.questionId"
        class="bs-iv__compact-row"
      >
        <span class="bs-iv__compact-k">{{ row.label }}</span>
        <span class="bs-iv__compact-arr">→</span>
        <span class="bs-iv__compact-v">{{ row.value }}</span>
      </div>
      <p
        v-if="answeredQuestions.length > 3"
        class="bs-iv__compact-more"
      >
        + {{ answeredQuestions.length - 3 }} more
      </p>
    </div>

    <div class="bs-iv__compact-actions">
      <button
        type="button"
        class="bs-iv__btn bs-iv__btn--ghost"
        @click="showFullModal = true"
      >Voir tout</button>
    </div>
  </div>

  <!-- ─── FULL DETAILS MODAL — "Voir tout" overlay ──────────────────────── -->
  <Teleport v-if="showFullModal" to="body">
    <div class="bs-iv-modal" role="dialog" aria-modal="true" @click.self="showFullModal = false">
      <div class="bs-iv-modal__box">
        <div class="bs-iv-modal__head">
          <span class="bs-iv__tag bs-iv__tag--ok">✓ build sheet · submitted</span>
          <button
            type="button"
            class="bs-iv-modal__close"
            aria-label="Close"
            @click="showFullModal = false"
          >✕</button>
        </div>

        <div class="bs-iv-modal__scroll">
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
              @click="interceptAnchorClick"
            />
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
            <div v-if="(displayInterview?.notes ?? '').trim().length > 0" class="bs-iv__notes-summary">
              {{ displayInterview?.notes }}
            </div>
            <div v-else class="bs-iv__notes-summary bs-iv__notes-summary--empty">
              (no notes)
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
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

/* ─── Compact summary card (history mode) ─────────────────────────── */
.bs-iv__compact-identity {
  margin: 8px 0 6px 0;
  font-family: var(--hand);
  font-size: 13px;
  color: var(--ink);
  line-height: 1.4;
}
.bs-iv__compact-answers {
  display: flex;
  flex-direction: column;
  gap: 3px;
  margin-top: 4px;
}
.bs-iv__compact-row {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-family: var(--hand);
  font-size: 11px;
  color: var(--ink-soft);
  line-height: 1.4;
}
.bs-iv__compact-k {
  color: var(--ink-faint);
  flex-shrink: 0;
  white-space: nowrap;
  max-width: 40%;
  overflow: hidden;
  text-overflow: ellipsis;
}
.bs-iv__compact-arr {
  color: var(--ink-faint);
  flex-shrink: 0;
}
.bs-iv__compact-v {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.bs-iv__compact-more {
  margin: 4px 0 0 0;
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.10em;
  text-transform: uppercase;
  color: var(--ink-faint);
}
.bs-iv__compact-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 10px;
  padding-top: 8px;
  border-top: 1px dotted var(--paper-line);
}

/* ─── Modal overlay ───────────────────────────────────────────────── */
.bs-iv-modal {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 24px;
  animation: bs-iv-modal-fade 0.12s ease-out;
}
@keyframes bs-iv-modal-fade {
  from { opacity: 0; }
  to { opacity: 1; }
}
.bs-iv-modal__box {
  background: var(--paper);
  border: 1px solid var(--good);
  border-radius: 8px;
  max-width: 720px;
  width: 100%;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 12px 48px rgba(0, 0, 0, 0.45);
}
.bs-iv-modal__head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 18px 12px;
  border-bottom: 1px solid var(--paper-line);
}
.bs-iv-modal__head .bs-iv__tag {
  flex: 1;
}
.bs-iv-modal__close {
  background: transparent;
  border: none;
  color: var(--ink-soft);
  font-size: 16px;
  line-height: 1;
  padding: 4px 8px;
  cursor: pointer;
  border-radius: 4px;
}
.bs-iv-modal__close:hover {
  background: var(--paper-tint, rgba(0, 0, 0, 0.05));
  color: var(--ink);
}
.bs-iv-modal__scroll {
  overflow-y: auto;
  padding: 14px 18px 18px;
}
</style>

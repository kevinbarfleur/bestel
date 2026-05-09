// Pinia store for the Build Sheets feature.
//
// Live state during an interview:
//   - draftSections    — agent-drafted prose per section, keyed by section_id
//   - pendingAsk       — current sheet_ask question (null when none in flight)
//   - phase            — `none | invite | interview | finalized | stale`
//   - activeSheet      — null until `sheet_finalize_request` succeeds and we
//                         load the validated payload back from disk
//
// The store is event-driven: every `sheet_draft_update` or `sheet_ask_user`
// delta from the agent flows in via `applyDraftUpdate` / `applyAskUser`. The
// 7-state design (empty / invite / match / interview / edit / finalized /
// stale) maps onto a smaller set of phase transitions here — UI components
// derive the rendered state from this phase plus the build store and the
// presence of an active sheet.

import { defineStore } from 'pinia';
import { computed, ref } from 'vue';

export type SheetPhase =
  | 'none'        // no PoB or no sheet activity yet
  | 'invite'      // PoB attached, agent has not started drafting
  | 'interview'   // drafting is live (sections / ask cards arriving)
  | 'edit'        // user is editing one drafted section
  | 'finalized'   // sheet validated, loaded from disk
  | 'stale';      // sheet validated but PoB drift detected

export interface SheetDraftSection {
  sectionId: string;
  title: string;
  body: string;
  confirmed: boolean;
  /** Wall-clock of the most recent update — used for sort stability when
   * the agent re-drafts an existing section. */
  updatedAt: number;
}

export interface PendingAsk {
  questionId: string;
  title: string;
  subtitle: string | null;
  options: string[];
  multi: boolean;
  hasOther: boolean;
  selected: number[];
}

/** Mirror of `crates/bestel-core/src/sheets/types.rs::BuildSheet` once a
 * sheet is validated and loaded back. */
export interface ActiveSheet {
  sheetId: string;
  fingerprint: string;
  name: string;
  pobHash: string;
  authoredAt: string;
  updatedAt: string;
  schemaVersion: number;
  payload: unknown;
}

/** Sprint UX-2 — the one-shot interview emitted by `sheet_open_interview`.
 * Sections and questions arrive together; the user fills the entire form
 * in one round; submission round-trips back as a structured user message
 * via `submitInterview()` -> `chatStore.send(...)`. */
export interface InterviewSection {
  id: 'identity' | 'archetype' | 'damage' | 'defense' | 'items' | 'intent';
  title: string;
  draftBody: string;
}

export interface InterviewQuestion {
  questionId: string;
  sectionId: 'identity' | 'archetype' | 'damage' | 'defense' | 'items' | 'intent';
  title: string;
  subtitle: string | null;
  options: string[];
  multi: boolean;
  hasOther: boolean;
}

export interface InterviewAnswer {
  selected: number[];
  otherText: string;
}

export interface ActiveInterview {
  sections: InterviewSection[];
  questions: InterviewQuestion[];
  notesPrompt: string;
  /** keyed by `questionId`. */
  answers: Map<string, InterviewAnswer>;
  /** keyed by `sectionId`. Empty when the user kept the agent's draft. */
  sectionEdits: Map<string, string>;
  notes: string;
  submitted: boolean;
}

/** Section order the interview panel and the sidebar progress card render
 *  in, and the submission message preserves. Exported at module scope so
 *  components don't need to round-trip through the Pinia store. */
export const INTERVIEW_SECTION_ORDER: InterviewSection['id'][] = [
  'identity',
  'archetype',
  'damage',
  'defense',
  'items',
  'intent',
];

export const useSheetStore = defineStore('sheet', () => {
  const phase = ref<SheetPhase>('none');
  const draftSections = ref<Map<string, SheetDraftSection>>(new Map());
  const pendingAsk = ref<PendingAsk | null>(null);
  const activeSheet = ref<ActiveSheet | null>(null);
  const activeInterview = ref<ActiveInterview | null>(null);

  /** True whenever there's any drafting evidence (section OR question
   * waiting), regardless of whether the user has entered the page yet. */
  const isInterviewing = computed(
    () => draftSections.value.size > 0 || pendingAsk.value !== null,
  );

  /** Sections sorted by first-seen order — when a section is re-drafted,
   * its existing position is preserved. */
  const sectionsList = computed<SheetDraftSection[]>(() =>
    Array.from(draftSections.value.values()).sort((a, b) => a.updatedAt - b.updatedAt),
  );

  /** Streaming entry point: agent called `sheet_propose_section`. New
   * sections are appended; re-drafts overwrite the body in place but keep
   * the section's position. */
  function applyDraftUpdate(payload: {
    sectionId: string;
    title: string;
    body: string;
    confirmed: boolean;
  }) {
    const existing = draftSections.value.get(payload.sectionId);
    const updatedAt = existing?.updatedAt ?? Date.now();
    draftSections.value.set(payload.sectionId, {
      sectionId: payload.sectionId,
      title: payload.title,
      body: payload.body,
      confirmed: payload.confirmed,
      updatedAt,
    });
    if (phase.value === 'none' || phase.value === 'invite') {
      phase.value = 'interview';
    }
  }

  /** Streaming entry point: agent called `sheet_ask`. Replaces any prior
   * pending ask (rare; the system prompt forbids two in flight). */
  function applyAskUser(payload: {
    questionId: string;
    title: string;
    subtitle: string | null;
    options: string[];
    multi: boolean;
    hasOther: boolean;
  }) {
    pendingAsk.value = {
      questionId: payload.questionId,
      title: payload.title,
      subtitle: payload.subtitle,
      options: payload.options,
      multi: payload.multi,
      hasOther: payload.hasOther,
      selected: [],
    };
    if (phase.value === 'none' || phase.value === 'invite') {
      phase.value = 'interview';
    }
  }

  /** User clicked Confirm on a section — flips the local copy. The actual
   * confirmation flows back to the agent as a regular user message; this
   * just keeps the UI in sync optimistically. */
  function confirmSection(sectionId: string) {
    const seg = draftSections.value.get(sectionId);
    if (seg) seg.confirmed = true;
  }

  /** User edited a section's body inline. */
  function editSection(sectionId: string, body: string) {
    const seg = draftSections.value.get(sectionId);
    if (seg) {
      seg.body = body;
      seg.confirmed = true;
    }
  }

  /** User answered the pending ask — clear it. */
  function clearAsk() {
    pendingAsk.value = null;
  }

  // ─── One-shot interview (Sprint UX-2) ──────────────────────────────


  /** Streaming entry point for `sheet_open_interview`. Replaces any prior
   * in-flight interview and primes empty answers for every question. */
  function openInterview(payload: {
    sections: { id: string; title: string; draft_body: string }[];
    questions: {
      question_id: string;
      section_id: string;
      title: string;
      subtitle?: string;
      options: string[];
      multi?: boolean;
      has_other?: boolean;
    }[];
    notes_prompt: string;
  }) {
    const sections: InterviewSection[] = payload.sections.map((s) => ({
      id: s.id as InterviewSection['id'],
      title: s.title,
      draftBody: s.draft_body,
    }));
    const questions: InterviewQuestion[] = payload.questions.map((q) => ({
      questionId: q.question_id,
      sectionId: q.section_id as InterviewQuestion['sectionId'],
      title: q.title,
      subtitle: q.subtitle ?? null,
      options: q.options,
      multi: q.multi ?? false,
      hasOther: q.has_other ?? true,
    }));
    const answers = new Map<string, InterviewAnswer>();
    for (const q of questions) {
      answers.set(q.questionId, { selected: [], otherText: '' });
    }
    activeInterview.value = {
      sections,
      questions,
      notesPrompt: payload.notes_prompt,
      answers,
      sectionEdits: new Map(),
      notes: '',
      submitted: false,
    };
    if (phase.value === 'none' || phase.value === 'invite') {
      phase.value = 'interview';
    }
  }

  function setAnswer(questionId: string, selected: number[], otherText: string) {
    const iv = activeInterview.value;
    if (!iv) return;
    iv.answers.set(questionId, { selected: [...selected], otherText });
  }

  function setSectionEdit(sectionId: string, body: string) {
    const iv = activeInterview.value;
    if (!iv) return;
    iv.sectionEdits.set(sectionId, body);
  }

  function setNotes(text: string) {
    const iv = activeInterview.value;
    if (!iv) return;
    iv.notes = text;
  }

  /** True when every question has at least one selected option OR a
   * non-empty Other text. Drives the Submit button's disabled state. */
  const interviewReady = computed<boolean>(() => {
    const iv = activeInterview.value;
    if (!iv) return false;
    for (const q of iv.questions) {
      const a = iv.answers.get(q.questionId);
      if (!a) return false;
      const hasSel = a.selected.length > 0;
      const hasOther = a.otherText.trim().length > 0;
      if (!hasSel && !hasOther) return false;
    }
    return true;
  });

  /** Build the structured user message the agent parses on the next turn.
   * Format is documented in `prompts/references/32_build_sheets.md`. */
  function submitInterview(): string {
    const iv = activeInterview.value;
    if (!iv) {
      throw new Error('submitInterview called with no active interview');
    }
    iv.submitted = true;

    const orderedSections: InterviewSection[] = [...iv.sections].sort(
      (a, b) =>
        INTERVIEW_SECTION_ORDER.indexOf(a.id) -
        INTERVIEW_SECTION_ORDER.indexOf(b.id),
    );

    const lines: string[] = [];
    lines.push(
      '[INTERVIEW SUBMISSION — finalize the sheet now and then answer my original question]',
    );
    lines.push('');
    lines.push('## Sections');
    for (const s of orderedSections) {
      const body = iv.sectionEdits.get(s.id) ?? s.draftBody;
      lines.push(`- **${s.id}** (${s.title}): ${body}`);
    }
    lines.push('');
    lines.push('## Question answers');
    if (iv.questions.length === 0) {
      lines.push('- (no leverage questions on this turn)');
    } else {
      for (const q of iv.questions) {
        const ans = iv.answers.get(q.questionId);
        const selectedLabels = ans
          ? ans.selected.map((i) => q.options[i]).filter((x): x is string => Boolean(x))
          : [];
        const other = ans?.otherText.trim() ?? '';
        const parts: string[] = [];
        if (selectedLabels.length > 0) parts.push(selectedLabels.join(' + '));
        if (other) parts.push(`Other: ${other}`);
        const value = parts.length > 0 ? parts.join(' | ') : '(no answer)';
        lines.push(
          `- **${q.questionId}** [${q.sectionId}] (${q.title}): ${value}`,
        );
      }
    }
    lines.push('');
    lines.push('## Notes');
    lines.push(iv.notes.trim() || '(none)');
    lines.push('');
    lines.push(
      'Now call `sheet_finalize_request` with the merged payload (use these section bodies, derive defining_items / intent / known_gaps from them and from your earlier analysis), then answer my original question citing the persisted sheet.',
    );

    return lines.join('\n');
  }

  /** Drop the in-flight interview without submitting — bound to the
   * panel's Cancel button. The agent will re-emit on the next ask. */
  function cancelInterview() {
    activeInterview.value = null;
  }

  /** Rebuild the in-flight interview from a chat history snapshot.
   *  Mirrors `openInterview` for the agent payload, then layers any
   *  user edits saved on the segment back into the live state. Called
   *  by `chat.loadFromSaved` so closing the app mid-interview no longer
   *  loses the user's chip selections / Other-text / notes. */
  function rehydrateInterview(
    payload: {
      sections: { id: string; title: string; draft_body: string }[];
      questions: {
        question_id: string;
        section_id: string;
        title: string;
        subtitle?: string;
        options: string[];
        multi?: boolean;
        has_other?: boolean;
      }[];
      notes_prompt: string;
    },
    serializedState: {
      answers: Record<string, { selected: number[]; otherText: string }>;
      sectionEdits: Record<string, string>;
      notes: string;
      submitted: boolean;
    } | null,
  ) {
    openInterview(payload);
    const iv = activeInterview.value;
    if (!iv || !serializedState) return;
    iv.notes = serializedState.notes;
    iv.submitted = serializedState.submitted;
    for (const [sectionId, body] of Object.entries(serializedState.sectionEdits)) {
      iv.sectionEdits.set(sectionId, body);
    }
    for (const [questionId, ans] of Object.entries(serializedState.answers)) {
      iv.answers.set(questionId, {
        selected: [...ans.selected],
        otherText: ans.otherText,
      });
    }
  }

  /** Fired after `sheet_finalize_request` completes (or on next-turn
   * `get_active_build_sheet` returning a valid sheet). */
  function loadActiveSheet(sheet: ActiveSheet) {
    activeSheet.value = sheet;
    phase.value = 'finalized';
    draftSections.value.clear();
    pendingAsk.value = null;
    activeInterview.value = null;
  }

  /** Mark the active sheet as stale (fingerprint match, hash drift). */
  function markStale() {
    if (activeSheet.value !== null) phase.value = 'stale';
  }

  /** Drop the active sheet without resetting interview / draft state.
   *  Called by the BuildPicker after a successful delete so the sidebar
   *  card disappears immediately when the user removes the currently
   *  linked sheet. The store transitions back to `phase = 'none'`; the
   *  invite card will re-surface as long as a build is loaded. */
  function clearActiveSheet() {
    activeSheet.value = null;
    if (phase.value === 'finalized' || phase.value === 'stale') {
      phase.value = activeInterview.value !== null ? 'interview' : 'none';
    }
  }

  /** Reset on chat-switch / new chat. */
  function reset() {
    phase.value = 'none';
    draftSections.value.clear();
    pendingAsk.value = null;
    activeSheet.value = null;
    activeInterview.value = null;
  }

  return {
    phase,
    draftSections,
    pendingAsk,
    activeSheet,
    activeInterview,
    isInterviewing,
    sectionsList,
    interviewReady,
    applyDraftUpdate,
    applyAskUser,
    confirmSection,
    editSection,
    clearAsk,
    openInterview,
    setAnswer,
    setSectionEdit,
    setNotes,
    submitInterview,
    cancelInterview,
    rehydrateInterview,
    loadActiveSheet,
    markStale,
    clearActiveSheet,
    reset,
  };
});

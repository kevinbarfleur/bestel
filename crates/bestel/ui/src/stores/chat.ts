import { defineStore } from 'pinia';
import { computed, nextTick, ref, watch } from 'vue';

import { chatCancel, chatReset, chatStart } from '../api/tauri';
import { extractPanelSidecar, tryExtractPanelSidecarPartial } from '../api/markdown';
import type { AttachmentDto, UsageStats } from '../api/types';
import { useChatHistoryStore, type SavedChat } from './chatHistory';
import { useBuildStore } from './build';
import { useSheetStore } from './sheet';
import { useUiStore, type PanelArtifact } from './ui';

export type ChatRole = 'user' | 'assistant';
export type MessageStatus = 'streaming' | 'complete' | 'error' | 'cancelled';
export type ToolSegmentStatus = 'running' | 'done' | 'failed';

export interface TextSegment {
  kind: 'text';
  id: string;
  text: string;
  /**
   * Parsed side-panel sidecar payloads, indexed by marker name. Populated
   * once at message finalization (or on chat reload) by walking the text
   * for ⟦panel-data⟧{json}⟦/panel-data⟧, stripping it, and validating the
   * entries. Click handlers in ChatMessage.vue resolve panel buttons here.
   */
  panelMap?: Record<string, PanelArtifact>;
}

export interface ReasoningSegment {
  kind: 'reasoning';
  id: string;
  text: string;
  open: boolean;
}

export interface ToolSegment {
  kind: 'tool';
  id: string;
  name: string;
  detail: string | null;
  /**
   * Late-arriving formatted view of the tool's input arguments. Anthropic
   * streams tool input across multiple chunks and emits this once the
   * input is fully assembled (per `LlmDelta::ToolDetailUpdate`); Ollama
   * emits it alongside `tool_begin` for shape parity. The badge prefers
   * `summaryInput` over `detail` when both are present.
   */
  summaryInput?: string;
  status: ToolSegmentStatus;
  summary: string | null;
  output: string;
}

/** Build-Sheet drafted-section segment — emitted by the agent calling
 *  `sheet_propose_section`. Rendered inline as a `BSDraftedCard` inside an
 *  interview-styled agent bubble (see `32_build_sheets.md` for the flow). */
export interface SheetDraftSegment {
  kind: 'sheet_draft';
  id: string;
  sectionId: string;
  title: string;
  body: string;
  confirmed: boolean;
}

/** Build-Sheet ask-user segment — emitted by the agent calling `sheet_ask`.
 *  Rendered inline as a `BSAskCard` inside an interview-styled bubble. */
export interface SheetAskSegment {
  kind: 'sheet_ask';
  id: string;
  questionId: string;
  title: string;
  subtitle: string | null;
  options: string[];
  multi: boolean;
  hasOther: boolean;
}

/** Build-Sheet one-shot interview anchor (Sprint UX-2).
 *
 *  The segment carries BOTH the original agent payload (sections,
 *  questions, notes_prompt — used to rebuild the panel on reload) AND a
 *  serialized snapshot of the user's in-progress edits (answers,
 *  sectionEdits, notes, submitted). The sheet store remains the runtime
 *  source of truth during a session, but its state is mirrored here on
 *  every mutation so closing the app mid-interview doesn't lose the
 *  user's work — `loadFromSaved` rehydrates from this snapshot. Both
 *  fields are optional so older saved chats restore as empty
 *  read-only segments instead of crashing. */
export interface SerializedInterviewState {
  answers: Record<string, { selected: number[]; otherText: string }>;
  sectionEdits: Record<string, string>;
  notes: string;
  submitted: boolean;
}

export interface SheetInterviewSegment {
  kind: 'sheet_interview';
  id: string;
  payload?: {
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
  };
  state?: SerializedInterviewState;
}

/** Sprint UX-2.7 — small persistent confirmation segment emitted after
 *  the agent's `sheet_finalize_request` succeeds. Sits in the timeline as
 *  a centered amber-good banner; reloads with the rest of the
 *  conversation, just like text/tool/interview segments. */
export interface SheetFinalizedSegment {
  kind: 'sheet_finalized';
  id: string;
  sheetId: string;
  name: string;
}

/** CoVe verifier surface — slim tool card emitted after the post-draft
 * verification pipeline runs. Anchored at the start of the assistant's
 * turn so the user sees the audit BEFORE reading the (possibly revised)
 * answer. Empty `claims` means heuristic-skip or toggle-off — caller
 * should NOT render anything in that case. */
export interface VerifyClaimsSegment {
  kind: 'verify_claims';
  id: string;
  /** `pass` | `revise` | `fail`. Drives the card's accent colour. */
  status: 'pass' | 'revise' | 'fail' | string;
  claims: import('../api/types').VerifiedClaimDto[];
  correctionsCount: number;
  /** Joined finding summaries (mainly for dev panel parity). */
  findingsSummary: string;
}

export type Segment =
  | TextSegment
  | ReasoningSegment
  | ToolSegment
  | SheetDraftSegment
  | SheetAskSegment
  | SheetInterviewSegment
  | SheetFinalizedSegment
  | VerifyClaimsSegment;

export interface ChatMessageVm {
  id: string;
  role: ChatRole;
  segments: Segment[];
  attachments: AttachmentDto[];
  status: MessageStatus;
  sessionId: number | null;
  errorMessage: string | null;
  /** Token / cost telemetry for this assistant turn. Populated when the
   * provider emits an `LlmDelta::Usage` (Anthropic + DeepSeek today). */
  usage?: UsageStats;
  /** Wall-clock timestamp (Date.now()) of the most recent streaming
   * event for this message. Used by ChatMessage.vue to render an
   * inter-segment thinking indicator when the gap exceeds
   * THINKING_GAP_MS while status === 'streaming'. */
  lastDeltaAt?: number;
}

const segId = () => `s_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
const msgId = () => `m_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;

/**
 * Detects the primary panel marker (⟦panel*:type:name⟧) so the auto-open
 * trigger can fire as soon as it arrives in drained text — well before
 * end-of-message. Captures the marker NAME for sidecar lookup.
 */
const PRIMARY_MARKER_RE = /⟦panel\*:[a-z-]+:([^⟧]+?)⟧/;

/**
 * Adaptive drain factor for the streaming text buffer. Each RAF tick
 * pulls `ceil(buffer.length * DRAIN_FACTOR)` characters (min 1) from the
 * queue into the live text. Higher = faster catch-up after a network
 * burst, lower = smoother typewriter feel. 0.06 yields a ~0.3s settle
 * time after a dump regardless of burst size (exponential decay).
 */
const DRAIN_FACTOR = 0.06;

export const useChatStore = defineStore('chat', () => {
  const messages = ref<ChatMessageVm[]>([]);
  const activeSessionId = ref<number | null>(null);

  // ─── Streaming buffer (RAF drain) ──────────────────────────────────
  // Raw token deltas from the LLM accumulate here, then a
  // requestAnimationFrame loop drains a fraction per frame into the
  // live text segment. Smooths bursty network cadence into a steady
  // ~60fps render rhythm and decouples the markdown re-parse from
  // network jitter. Module-local state — never reactive.
  let streamBuffer = '';
  let rafHandle: number | null = null;
  /** True while `loadFromSaved` is repopulating in-memory messages from a
   *  persisted chat. Suppresses the autosave watch so the snapshot taken
   *  mid-restore (when the build store still holds the *default* build)
   *  does not overwrite the chat's real `attached_build_path`. ChatView /
   *  ChatPicker re-align the build via `setActive(...)` after restore;
   *  the next user message persists the correct path. */
  let restoreInProgress = false;

  // Per-turn flag: ensures the auto-open of the primary panel fires at
  // most once per assistant message, even though the drain loop scans
  // text on every tick. Reset in `startAssistant`.
  let alreadyAutoPromoted = false;

  const isStreaming = computed(() => activeSessionId.value !== null);

  const currentAssistant = computed<ChatMessageVm | null>(() => {
    if (activeSessionId.value === null) return null;
    for (let i = messages.value.length - 1; i >= 0; i--) {
      const m = messages.value[i];
      if (m.role === 'assistant' && m.sessionId === activeSessionId.value) return m;
    }
    return null;
  });

  function pushUser(text: string, attachments: AttachmentDto[]) {
    messages.value.push({
      id: msgId(),
      role: 'user',
      segments: [{ kind: 'text', id: segId(), text }],
      attachments,
      status: 'complete',
      sessionId: null,
      errorMessage: null,
    });
  }

  function startAssistant(sessionId: number) {
    // Defensive: any leftover buffer from a prior turn is discarded.
    // setCompleted/setCancelled/setError already flushed it; this guards
    // against unexpected paths (provider crash mid-stream, hot reload).
    streamBuffer = '';
    if (rafHandle !== null) {
      cancelAnimationFrame(rafHandle);
      rafHandle = null;
    }
    alreadyAutoPromoted = false;
    activeSessionId.value = sessionId;
    messages.value.push({
      id: msgId(),
      role: 'assistant',
      segments: [],
      attachments: [],
      status: 'streaming',
      sessionId,
      errorMessage: null,
      lastDeltaAt: Date.now(),
    });
  }

  function lastAssistantSegment(): Segment | null {
    const a = currentAssistant.value;
    if (!a || a.segments.length === 0) return null;
    return a.segments[a.segments.length - 1];
  }

  /**
   * Append a slice to the assistant's last text segment, creating one
   * if none exists or the last segment isn't text. Used both by the RAF
   * drain (regular path) and `flushStreamingText` (cancellation path).
   */
  function commitTextSlice(slice: string) {
    const a = currentAssistant.value;
    if (!a) return;
    const last = lastAssistantSegment();
    if (last && last.kind === 'text') {
      last.text += slice;
    } else {
      a.segments.push({ kind: 'text', id: segId(), text: slice });
    }
  }

  /**
   * Scan the assistant's text segments for the side-panel sidecar. As
   * soon as both ⟦panel-data⟧ tags are present in any segment, parse
   * the JSON, strip the block from that segment's text, and assign its
   * `panelMap`. Then scan ALL text segments for a primary marker and
   * resolve it against ANY segment's panelMap — sidecar and marker may
   * live in different segments (e.g. start-of-message sidecar followed
   * by reasoning, then prose with the marker). Idempotent: segments
   * with `panelMap` already set are skipped. Auto-open is gated by
   * `alreadyAutoPromoted` so it fires at most once per turn, and by
   * `panelStack.length === 0` so manual opens win.
   */
  function tryIncrementalPanelExtraction() {
    const a = currentAssistant.value;
    if (!a) return;

    for (const seg of a.segments) {
      if (seg.kind !== 'text') continue;
      if (seg.panelMap) continue;
      const partial = tryExtractPanelSidecarPartial(seg.text);
      if (partial) {
        seg.text = partial.stripped;
        seg.panelMap = partial.map;
      }
    }

    if (alreadyAutoPromoted) return;
    const ui = useUiStore();
    if (ui.panelStack.length > 0) return;

    let primaryKey: string | null = null;
    for (const seg of a.segments) {
      if (seg.kind !== 'text') continue;
      const m = PRIMARY_MARKER_RE.exec(seg.text);
      if (m) {
        primaryKey = m[1];
        break;
      }
    }
    if (!primaryKey) return;

    for (const seg of a.segments) {
      if (seg.kind !== 'text') continue;
      const artifact = seg.panelMap?.[primaryKey];
      if (artifact) {
        ui.openPanel({ ...artifact, source: 'agent' });
        alreadyAutoPromoted = true;
        return;
      }
    }
  }

  function drainTick() {
    rafHandle = null;
    if (streamBuffer.length === 0) return;
    const n = Math.min(
      streamBuffer.length,
      Math.max(1, Math.ceil(streamBuffer.length * DRAIN_FACTOR)),
    );
    const slice = streamBuffer.slice(0, n);
    streamBuffer = streamBuffer.slice(n);
    commitTextSlice(slice);
    tryIncrementalPanelExtraction();
    if (streamBuffer.length > 0) {
      rafHandle = requestAnimationFrame(drainTick);
    }
  }

  /**
   * Public entry point for streamed text deltas. Tokens are queued into
   * `streamBuffer` and an RAF drain is scheduled if not already pending.
   * Keeps the same public name (`appendText`) so the streaming
   * composable doesn't need to change.
   */
  function appendText(text: string) {
    const a = currentAssistant.value;
    if (!a) return;
    a.lastDeltaAt = Date.now();
    streamBuffer += text;
    if (rafHandle === null) {
      rafHandle = requestAnimationFrame(drainTick);
    }
  }

  /**
   * Empty the buffer immediately (no smoothing) and stop the RAF loop.
   * Called when the stream terminates so `setCompleted` finalizers see
   * the full text. Also runs `tryIncrementalPanelExtraction` so the
   * sidecar is parsed regardless of placement.
   */
  function flushStreamingText() {
    if (rafHandle !== null) {
      cancelAnimationFrame(rafHandle);
      rafHandle = null;
    }
    if (streamBuffer.length > 0) {
      const slice = streamBuffer;
      streamBuffer = '';
      commitTextSlice(slice);
    }
    tryIncrementalPanelExtraction();
  }

  function reasoningBegin() {
    const a = currentAssistant.value;
    if (!a) return;
    a.lastDeltaAt = Date.now();
    a.segments.push({ kind: 'reasoning', id: segId(), text: '', open: true });
  }

  function reasoningDelta(text: string) {
    const a = currentAssistant.value;
    if (!a) return;
    a.lastDeltaAt = Date.now();
    for (let i = a.segments.length - 1; i >= 0; i--) {
      const s = a.segments[i];
      if (s.kind === 'reasoning' && s.open) {
        s.text += text;
        return;
      }
    }
    a.segments.push({ kind: 'reasoning', id: segId(), text, open: true });
  }

  function reasoningEnd() {
    const a = currentAssistant.value;
    if (!a) return;
    for (let i = a.segments.length - 1; i >= 0; i--) {
      const s = a.segments[i];
      if (s.kind === 'reasoning' && s.open) {
        s.open = false;
        return;
      }
    }
  }

  function toolBegin(id: string, name: string, detail: string | null) {
    const a = currentAssistant.value;
    if (!a) return;
    a.lastDeltaAt = Date.now();
    a.segments.push({
      kind: 'tool',
      id,
      name,
      detail,
      status: 'running',
      summary: null,
      output: '',
    });
  }

  function toolDetailUpdate(id: string, summaryInput: string) {
    const a = currentAssistant.value;
    if (!a) return;
    a.lastDeltaAt = Date.now();
    const seg = a.segments.find((s): s is ToolSegment => s.kind === 'tool' && s.id === id);
    if (seg) seg.summaryInput = summaryInput;
  }

  function toolOutput(id: string, chunk: string) {
    const a = currentAssistant.value;
    if (!a) return;
    a.lastDeltaAt = Date.now();
    const seg = a.segments.find((s): s is ToolSegment => s.kind === 'tool' && s.id === id);
    if (seg) seg.output += chunk;
  }

  function toolEnd(id: string, status: ToolSegmentStatus, summary: string | null) {
    const a = currentAssistant.value;
    if (!a) return;
    a.lastDeltaAt = Date.now();
    const seg = a.segments.find((s): s is ToolSegment => s.kind === 'tool' && s.id === id);
    if (seg) {
      seg.status = status;
      seg.summary = summary;
    }
  }

  /** Append (or update in place) a Build-Sheet drafted section. The agent
   *  may re-draft an existing section after the user corrects it; we
   *  overwrite the matching segment instead of pushing a duplicate so the
   *  card position stays put in the chat scroll. */
  function sheetDraftUpdate(payload: {
    sectionId: string;
    title: string;
    body: string;
    confirmed: boolean;
  }) {
    const a = currentAssistant.value;
    if (!a) return;
    a.lastDeltaAt = Date.now();
    const existing = a.segments.find(
      (s): s is SheetDraftSegment => s.kind === 'sheet_draft' && s.sectionId === payload.sectionId,
    );
    if (existing) {
      existing.title = payload.title;
      existing.body = payload.body;
      existing.confirmed = payload.confirmed;
      return;
    }
    a.segments.push({
      kind: 'sheet_draft',
      id: segId(),
      sectionId: payload.sectionId,
      title: payload.title,
      body: payload.body,
      confirmed: payload.confirmed,
    });
  }

  /** Append a Build-Sheet ask-user picker. Multiple asks can land in a
   *  single turn in theory; we don't dedupe — each one stays as its own
   *  segment in the conversation timeline. */
  function sheetAskUser(payload: {
    questionId: string;
    title: string;
    subtitle: string | null;
    options: string[];
    multi: boolean;
    hasOther: boolean;
  }) {
    const a = currentAssistant.value;
    if (!a) return;
    a.lastDeltaAt = Date.now();
    a.segments.push({
      kind: 'sheet_ask',
      id: segId(),
      questionId: payload.questionId,
      title: payload.title,
      subtitle: payload.subtitle,
      options: payload.options,
      multi: payload.multi,
      hasOther: payload.hasOther,
    });
  }

  /** Anchor a one-shot Build Sheet interview (Sprint UX-2) in the current
   *  assistant message and stash the original agent payload on the
   *  segment so a reload can rebuild the panel. Idempotent within a turn:
   *  if the model emits two sheet_open_interview deltas in a single
   *  assistant message (shouldn't happen, but safe), the second
   *  overwrites the first's payload — we always want the latest one. */
  function sheetInterviewOpen(payload: SheetInterviewSegment['payload']) {
    const a = currentAssistant.value;
    if (!a) return;
    a.lastDeltaAt = Date.now();
    const existing = a.segments.find(
      (s): s is SheetInterviewSegment => s.kind === 'sheet_interview',
    );
    if (existing) {
      existing.payload = payload;
      existing.state = {
        answers: {},
        sectionEdits: {},
        notes: '',
        submitted: false,
      };
      return;
    }
    a.segments.push({
      kind: 'sheet_interview',
      id: segId(),
      payload,
      state: {
        answers: {},
        sectionEdits: {},
        notes: '',
        submitted: false,
      },
    });
  }

  /** Mirror the sheet store's in-progress interview state onto the most
   *  recent `sheet_interview` segment. Called by BSInterviewPanel on
   *  every mutation (chip toggle, Other-text input, body edit, notes
   *  edit). The deep-watch chat-history snapshot picks the patched
   *  segment up automatically and persists it; on reload, loadFromSaved
   *  rehydrates the sheet store from this state. */
  function updateSheetInterviewState(state: SerializedInterviewState) {
    for (let i = messages.value.length - 1; i >= 0; i--) {
      const m = messages.value[i];
      if (m.role !== 'assistant') continue;
      for (let j = m.segments.length - 1; j >= 0; j--) {
        const s = m.segments[j];
        if (s.kind === 'sheet_interview') {
          s.state = state;
          return;
        }
      }
    }
  }

  /** Append a verifier audit segment to the current assistant message.
   * Called once per turn (the verifier runs after the draft is fully
   * streamed). Idempotent — replaces an existing verify segment so a
   * retry doesn't stack two cards. Caller should pre-filter empty
   * results (heuristic skip / toggle off) and skip this entirely. */
  function verifierResult(payload: {
    status: string;
    claims: import('../api/types').VerifiedClaimDto[];
    correctionsCount: number;
    findingsSummary: string;
  }) {
    const a = currentAssistant.value;
    if (!a) return;
    a.lastDeltaAt = Date.now();
    const existing = a.segments.find(
      (s): s is VerifyClaimsSegment => s.kind === 'verify_claims',
    );
    if (existing) {
      existing.status = payload.status;
      existing.claims = payload.claims;
      existing.correctionsCount = payload.correctionsCount;
      existing.findingsSummary = payload.findingsSummary;
      return;
    }
    a.segments.push({
      kind: 'verify_claims',
      id: segId(),
      status: payload.status,
      claims: payload.claims,
      correctionsCount: payload.correctionsCount,
      findingsSummary: payload.findingsSummary,
    });
  }

  /** Anchor the `✓ Build Sheet saved` banner on the current assistant
   *  message after `sheet_finalize_request` reports success. Idempotent —
   *  if the agent somehow emits two finalize events in one turn, the
   *  later one updates the existing segment's name in place rather than
   *  duplicating the banner. */
  function sheetFinalized(sheetId: string, name: string) {
    const a = currentAssistant.value;
    if (!a) return;
    a.lastDeltaAt = Date.now();
    const existing = a.segments.find(
      (s): s is SheetFinalizedSegment => s.kind === 'sheet_finalized',
    );
    if (existing) {
      existing.sheetId = sheetId;
      existing.name = name;
      return;
    }
    a.segments.push({
      kind: 'sheet_finalized',
      id: segId(),
      sheetId,
      name,
    });
  }

  /** Find the latest sheet_interview segment that has a payload.
   *  Used by the chat view + the sheet store's rehydration path. */
  function findLatestSheetInterviewSegment(): SheetInterviewSegment | null {
    for (let i = messages.value.length - 1; i >= 0; i--) {
      const m = messages.value[i];
      if (m.role !== 'assistant') continue;
      for (let j = m.segments.length - 1; j >= 0; j--) {
        const s = m.segments[j];
        if (s.kind === 'sheet_interview' && s.payload) {
          return s;
        }
      }
    }
    return null;
  }

  /** Lookup helper used by useStreaming to read a completed tool's output
   *  (e.g. to promote `show_in_panel` results to the right adaptive panel). */
  function findToolSegment(id: string): ToolSegment | null {
    const a = currentAssistant.value;
    if (!a) return null;
    return (
      a.segments.find((s): s is ToolSegment => s.kind === 'tool' && s.id === id) ??
      null
    );
  }

  function setUsage(stats: UsageStats) {
    const a = currentAssistant.value;
    if (a) a.usage = stats;
  }

  /**
   * Strip the ⟦panel-data⟧ sidecar from each text segment of an assistant
   * message and populate `seg.panelMap`. Idempotent — segments that the
   * RAF drain already populated keep their existing map (running the
   * extractor on already-stripped text would return an empty map and
   * clobber the prior result).
   */
  function finalizeAssistantMessage(msg: ChatMessageVm | null) {
    if (!msg) return;
    for (const seg of msg.segments) {
      if (seg.kind !== 'text') continue;
      if (seg.panelMap !== undefined) continue;
      const { stripped, map } = extractPanelSidecar(seg.text);
      seg.text = stripped;
      seg.panelMap = map;
    }
  }

  function setError(message: string) {
    flushStreamingText();
    const a = currentAssistant.value;
    if (a) {
      a.status = 'error';
      a.errorMessage = message;
    }
    finalizeAssistantMessage(currentAssistant.value);
    activeSessionId.value = null;
    useChatHistoryStore().flushToDisk();
  }

  function setCompleted() {
    flushStreamingText();
    const a = currentAssistant.value;
    if (a) a.status = 'complete';
    finalizeAssistantMessage(currentAssistant.value);
    activeSessionId.value = null;
    useChatHistoryStore().flushToDisk();
  }

  function setCancelled() {
    flushStreamingText();
    const a = currentAssistant.value;
    if (a) a.status = 'cancelled';
    finalizeAssistantMessage(currentAssistant.value);
    activeSessionId.value = null;
    useChatHistoryStore().flushToDisk();
  }

  async function send(prompt: string, attachments: AttachmentDto[] = []) {
    const trimmed = prompt.trim();
    if (isStreaming.value) return;
    if (!trimmed && attachments.length === 0) return;
    pushUser(trimmed, attachments);
    try {
      const sid = await chatStart(trimmed, attachments);
      startAssistant(sid);
    } catch (e) {
      messages.value.push({
        id: msgId(),
        role: 'assistant',
        segments: [],
        attachments: [],
        status: 'error',
        sessionId: null,
        errorMessage: String(e),
      });
    }
  }

  async function cancel() {
    if (!isStreaming.value) return;
    try {
      await chatCancel();
    } catch {
      // swallow — cancellation is best-effort
    }
  }

  async function reset() {
    try {
      await chatReset();
    } catch {
      // ignore
    }
    messages.value = [];
    activeSessionId.value = null;
    // Detail panel from a previous chat must not bleed into the new
    // conversation; resetting the chat resets the right-rail too.
    useUiStore().closePanel();
    // Same for the sheet store — interview drafts from a previous chat
    // would otherwise stay visible in the sidebar / Build Sheets view.
    useSheetStore().reset();
    // Mark history "new" so the next message starts a fresh saved chat.
    useChatHistoryStore().startNew();
  }

  /** Load a saved chat into the in-memory state (replace current). */
  async function loadFromSaved(saved: SavedChat) {
    if (isStreaming.value) {
      try {
        await chatCancel();
      } catch {
        /* ignore */
      }
    }
    try {
      await chatReset();
    } catch {
      /* ignore */
    }
    restoreInProgress = true;
    try {
      messages.value = saved.messages.map((m) => ({
        ...m,
        // mark restored assistant messages as complete (no live streaming)
        status: m.status === 'streaming' ? 'complete' : m.status,
        segments: m.segments.map((s) => ({ ...s })),
      }));
      // Re-extract sidecars on every restored assistant text segment. Saved
      // chats predate panelMap so the field is missing; running the extractor
      // is cheap and the result is identical for already-stripped text.
      for (const m of messages.value) {
        if (m.role === 'assistant') finalizeAssistantMessage(m);
      }
      activeSessionId.value = null;
      // Detail panel from the previous chat would otherwise stay pinned
      // to the right rail showing a stale artifact.
      useUiStore().closePanel();
      // Sheet store — same logic as in `reset()` above.
      const sheetStore = useSheetStore();
      sheetStore.reset();
      // Sprint UX-2 — rehydrate any in-progress one-shot interview the
      // user left mid-edit. The latest sheet_interview segment that has
      // a payload AND has not been submitted is the live one; the
      // serialized state (answers / sectionEdits / notes / submitted)
      // re-populates the sheet store so BSInterviewPanel renders with
      // the user's prior selections intact.
      const liveInterview = findLatestSheetInterviewSegment();
      if (liveInterview && liveInterview.payload && !(liveInterview.state?.submitted)) {
        sheetStore.rehydrateInterview(liveInterview.payload, liveInterview.state ?? null);
      }
    } finally {
      // Release the autosave guard on the next tick so the deep watch's
      // post-flush observation also sees the lock in place.
      await nextTick();
      restoreInProgress = false;
    }
  }

  // ─── Autosave ──────────────────────────────────────────────────
  // Snapshot the current chat into history whenever the messages
  // settle (deep watch, debounced via the run loop).
  //
  // The `restoreInProgress` guard prevents a destructive race during
  // app boot: ChatView mounts → refreshActive() loads the *default* build
  // → loadFromSaved() populates messages → the watch below would fire
  // and snapshot with `attached_build_path = <default build>`, overwriting
  // the chat's actual associated build. Then `setActive(saved.attached_build_path)`
  // runs *after* and fixes the in-memory state, but the corrupted snapshot
  // is already in localStorage. Next app launch reads the corrupted path
  // and the chat opens with the wrong (default) build forever after.
  //
  // Fix: skip snapshots while a chat is being restored — the caller
  // (ChatView / ChatPicker) is responsible for re-aligning the build via
  // setActive(), and once that completes the next message OR a manual
  // re-snapshot will persist the correct path.
  watch(
    messages,
    () => {
      if (restoreInProgress) return;
      if (messages.value.length === 0) return;
      const buildPath = useBuildStore().current?.source_file ?? null;
      useChatHistoryStore().snapshot(messages.value, buildPath);
    },
    { deep: true, flush: 'post' },
  );

  return {
    messages,
    activeSessionId,
    isStreaming,
    currentAssistant,
    appendText,
    reasoningBegin,
    reasoningDelta,
    reasoningEnd,
    toolBegin,
    toolDetailUpdate,
    toolOutput,
    toolEnd,
    sheetDraftUpdate,
    sheetAskUser,
    sheetInterviewOpen,
    sheetFinalized,
    verifierResult,
    updateSheetInterviewState,
    findLatestSheetInterviewSegment,
    findToolSegment,
    setUsage,
    setError,
    setCompleted,
    setCancelled,
    send,
    cancel,
    reset,
    loadFromSaved,
  };
});

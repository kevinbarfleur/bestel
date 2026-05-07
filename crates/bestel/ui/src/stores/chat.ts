import { defineStore } from 'pinia';
import { computed, ref, watch } from 'vue';

import { chatCancel, chatReset, chatStart } from '../api/tauri';
import { extractPanelSidecar, tryExtractPanelSidecarPartial } from '../api/markdown';
import type { AttachmentDto, UsageStats } from '../api/types';
import { useChatHistoryStore, type SavedChat } from './chatHistory';
import { useBuildStore } from './build';
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
  status: ToolSegmentStatus;
  summary: string | null;
  output: string;
}

export type Segment = TextSegment | ReasoningSegment | ToolSegment;

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
    if (!currentAssistant.value) return;
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
    a.segments.push({ kind: 'reasoning', id: segId(), text: '', open: true });
  }

  function reasoningDelta(text: string) {
    const a = currentAssistant.value;
    if (!a) return;
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

  function toolOutput(id: string, chunk: string) {
    const a = currentAssistant.value;
    if (!a) return;
    const seg = a.segments.find((s): s is ToolSegment => s.kind === 'tool' && s.id === id);
    if (seg) seg.output += chunk;
  }

  function toolEnd(id: string, status: ToolSegmentStatus, summary: string | null) {
    const a = currentAssistant.value;
    if (!a) return;
    const seg = a.segments.find((s): s is ToolSegment => s.kind === 'tool' && s.id === id);
    if (seg) {
      seg.status = status;
      seg.summary = summary;
    }
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
  }

  function setCompleted() {
    flushStreamingText();
    const a = currentAssistant.value;
    if (a) a.status = 'complete';
    finalizeAssistantMessage(currentAssistant.value);
    activeSessionId.value = null;
  }

  function setCancelled() {
    flushStreamingText();
    const a = currentAssistant.value;
    if (a) a.status = 'cancelled';
    finalizeAssistantMessage(currentAssistant.value);
    activeSessionId.value = null;
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
  }

  // ─── Autosave ──────────────────────────────────────────────────
  // Snapshot the current chat into history whenever the messages
  // settle (deep watch, debounced via the run loop).
  watch(
    messages,
    () => {
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
    toolOutput,
    toolEnd,
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

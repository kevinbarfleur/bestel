import { defineStore } from 'pinia';
import { computed, ref, watch } from 'vue';

import { chatCancel, chatReset, chatStart } from '../api/tauri';
import { extractPanelSidecar } from '../api/markdown';
import type { AttachmentDto, UsageStats } from '../api/types';
import { useChatHistoryStore, type SavedChat } from './chatHistory';
import { useBuildStore } from './build';
import type { PanelArtifact } from './ui';

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

export const useChatStore = defineStore('chat', () => {
  const messages = ref<ChatMessageVm[]>([]);
  const activeSessionId = ref<number | null>(null);

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

  function appendText(text: string) {
    const a = currentAssistant.value;
    if (!a) return;
    const last = lastAssistantSegment();
    if (last && last.kind === 'text') {
      last.text += text;
    } else {
      a.segments.push({ kind: 'text', id: segId(), text });
    }
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
   * message and populate `seg.panelMap`. Idempotent — running on text that
   * no longer contains a sidecar leaves it unchanged.
   */
  function finalizeAssistantMessage(msg: ChatMessageVm | null) {
    if (!msg) return;
    for (const seg of msg.segments) {
      if (seg.kind !== 'text') continue;
      const { stripped, map } = extractPanelSidecar(seg.text);
      seg.text = stripped;
      seg.panelMap = map;
    }
  }

  function setError(message: string) {
    const a = currentAssistant.value;
    if (a) {
      a.status = 'error';
      a.errorMessage = message;
    }
    finalizeAssistantMessage(currentAssistant.value);
    activeSessionId.value = null;
  }

  function setCompleted() {
    const a = currentAssistant.value;
    if (a) a.status = 'complete';
    finalizeAssistantMessage(currentAssistant.value);
    activeSessionId.value = null;
  }

  function setCancelled() {
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

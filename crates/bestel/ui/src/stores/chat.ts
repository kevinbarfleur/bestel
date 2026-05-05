import { defineStore } from 'pinia';
import { computed, ref, watch } from 'vue';

import { chatCancel, chatReset, chatStart } from '../api/tauri';
import type { AttachmentDto } from '../api/types';
import { useChatHistoryStore, type SavedChat } from './chatHistory';
import { useBuildStore } from './build';

export type ChatRole = 'user' | 'assistant';
export type MessageStatus = 'streaming' | 'complete' | 'error' | 'cancelled';
export type ToolSegmentStatus = 'running' | 'done' | 'failed';

export interface TextSegment {
  kind: 'text';
  id: string;
  text: string;
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

  function setError(message: string) {
    const a = currentAssistant.value;
    if (a) {
      a.status = 'error';
      a.errorMessage = message;
    }
    activeSessionId.value = null;
  }

  function setCompleted() {
    const a = currentAssistant.value;
    if (a) a.status = 'complete';
    activeSessionId.value = null;
  }

  function setCancelled() {
    const a = currentAssistant.value;
    if (a) a.status = 'cancelled';
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
    setError,
    setCompleted,
    setCancelled,
    send,
    cancel,
    reset,
    loadFromSaved,
  };
});

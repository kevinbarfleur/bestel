import { onBeforeUnmount, onMounted } from 'vue';
import type { UnlistenFn } from '@tauri-apps/api/event';

import { onLlmDelta } from '../api/tauri';
import { useChatStore, type TextSegment } from '../stores/chat';
import { useToastsStore } from '../stores/toasts';
import { useUiStore } from '../stores/ui';

/**
 * Regex used to detect a starred ⟦panel*:type:name⟧ marker — the agent's
 * way of marking ONE primary artifact per message that should auto-open
 * after streaming completes.
 */
const PRIMARY_MARKER_RE = /⟦panel\*:[a-z-]+:([^⟧]+?)⟧/;

export function useStreaming() {
  const chat = useChatStore();
  const toasts = useToastsStore();
  const ui = useUiStore();
  let unlisten: UnlistenFn | null = null;

  /**
   * After an assistant message finalizes, scan it for a primary panel
   * marker (⟦panel*:type:name⟧) and auto-open the matching artifact.
   * Skips silently if no primary marker exists, the sidecar didn't carry
   * the entry, or the user already has a panel open (don't stomp focus).
   *
   * Called from both `message_end` (soft-close on some providers) and
   * `completed`. The `panelStack.length > 0` guard naturally dedups: the
   * second call lands with the panel already open and no-ops.
   */
  function maybeAutoPromotePrimary() {
    if (ui.panelStack.length > 0) return;
    const all = chat.messages;
    for (let i = all.length - 1; i >= 0; i--) {
      const m = all[i];
      if (m.role !== 'assistant') continue;
      for (const seg of m.segments) {
        if (seg.kind !== 'text') continue;
        const t = seg as TextSegment;
        const match = PRIMARY_MARKER_RE.exec(t.text);
        if (!match) continue;
        const key = match[1];
        const artifact = t.panelMap?.[key];
        if (artifact) {
          ui.openPanel({ ...artifact, source: 'agent' });
          return;
        }
      }
      break;
    }
  }

  onMounted(async () => {
    unlisten = await onLlmDelta((ev) => {
      switch (ev.kind) {
        case 'text':
          chat.appendText(ev.text);
          break;
        case 'reasoning_begin':
          chat.reasoningBegin();
          break;
        case 'reasoning_delta':
          chat.reasoningDelta(ev.text);
          break;
        case 'reasoning_end':
          chat.reasoningEnd();
          break;
        case 'tool_begin':
          chat.toolBegin(ev.id, ev.name, ev.detail);
          break;
        case 'tool_output':
          chat.toolOutput(ev.id, ev.chunk);
          break;
        case 'tool_end':
          chat.toolEnd(ev.id, ev.status, ev.summary);
          break;
        case 'usage':
          chat.setUsage({
            input_tokens: ev.input_tokens,
            cached_input_tokens: ev.cached_input_tokens,
            cache_creation_tokens: ev.cache_creation_tokens,
            output_tokens: ev.output_tokens,
            cost_usd: ev.cost_usd,
          });
          break;
        case 'message_end':
          // Soft-close marker; some providers fire this without a following
          // 'completed' event. Trigger auto-promote here as a backup —
          // panelStack.length > 0 dedupes if 'completed' also fires.
          maybeAutoPromotePrimary();
          break;
        case 'completed':
          chat.setCompleted();
          maybeAutoPromotePrimary();
          break;
        case 'cancelled':
          chat.setCancelled();
          toasts.push({ variant: 'info', title: 'Conversation cancelled.' });
          break;
        case 'error':
          chat.setError(ev.message);
          toasts.push({ variant: 'error', title: 'Provider error', body: ev.message });
          break;
      }
    });
  });

  onBeforeUnmount(() => {
    if (unlisten) unlisten();
    unlisten = null;
  });
}

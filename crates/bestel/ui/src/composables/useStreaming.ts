import { onBeforeUnmount, onMounted } from 'vue';
import type { UnlistenFn } from '@tauri-apps/api/event';

import { onLlmDelta } from '../api/tauri';
import { useChatStore } from '../stores/chat';
import { useToastsStore } from '../stores/toasts';

export function useStreaming() {
  const chat = useChatStore();
  const toasts = useToastsStore();
  let unlisten: UnlistenFn | null = null;

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
          // Soft-close marker; some providers fire this without a
          // following 'completed' event. The chat store's RAF drain
          // handles auto-promote incrementally, so nothing to do here.
          break;
        case 'completed':
          chat.setCompleted();
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

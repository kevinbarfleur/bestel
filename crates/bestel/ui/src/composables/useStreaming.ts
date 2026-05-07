import { onBeforeUnmount, onMounted } from 'vue';
import type { UnlistenFn } from '@tauri-apps/api/event';

import { onLlmDelta } from '../api/tauri';
import { useChatStore } from '../stores/chat';
import { useToastsStore } from '../stores/toasts';
import {
  useUiStore,
  type PanelArtifact,
  type PanelArtifactType,
} from '../stores/ui';

const PANEL_TYPES: ReadonlySet<PanelArtifactType> = new Set([
  'item-card',
  'gem-detail',
  'mechanic',
  'markdown',
]);

function tryParsePanelPayload(raw: string): PanelArtifact | null {
  try {
    const obj = JSON.parse(raw);
    if (
      obj &&
      typeof obj === 'object' &&
      typeof obj.type === 'string' &&
      typeof obj.title === 'string' &&
      'payload' in obj &&
      PANEL_TYPES.has(obj.type as PanelArtifactType)
    ) {
      return {
        id: '', // filled by caller (uses tool segment id)
        type: obj.type as PanelArtifactType,
        title: obj.title,
        payload: obj.payload,
        source: 'agent',
      };
    }
  } catch {
    /* malformed — fall through */
  }
  return null;
}

export function useStreaming() {
  const chat = useChatStore();
  const toasts = useToastsStore();
  const ui = useUiStore();
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
        case 'tool_end': {
          chat.toolEnd(ev.id, ev.status, ev.summary);
          // show_in_panel completion → promote payload to the right
          // adaptive panel. Uses the tool segment id so repeated calls with
          // the same id update in place (no back-stack growth on re-emit).
          if (ev.status === 'done') {
            const seg = chat.findToolSegment(ev.id);
            if (seg && seg.name === 'show_in_panel' && seg.output) {
              const parsed = tryParsePanelPayload(seg.output);
              if (parsed) {
                ui.openPanel({ ...parsed, id: ev.id });
              }
            }
          }
          break;
        }
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
          // soft-close marker; treat as completion if no Completed follows
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

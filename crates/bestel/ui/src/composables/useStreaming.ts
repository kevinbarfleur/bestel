import { onBeforeUnmount, onMounted } from 'vue';
import type { UnlistenFn } from '@tauri-apps/api/event';

import { onLlmDelta } from '../api/tauri';
import { useChatStore } from '../stores/chat';
import { useSheetStore } from '../stores/sheet';
import { useToastsStore } from '../stores/toasts';

export function useStreaming() {
  const chat = useChatStore();
  const sheet = useSheetStore();
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
        case 'tool_detail_update':
          chat.toolDetailUpdate(ev.id, ev.summary_input);
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
        case 'sheet_draft_update': {
          const payload = {
            sectionId: ev.section_id,
            title: ev.title,
            body: ev.body,
            confirmed: ev.confirmed,
          };
          chat.sheetDraftUpdate(payload);
          sheet.applyDraftUpdate(payload);
          break;
        }
        case 'sheet_ask_user': {
          const payload = {
            questionId: ev.question_id,
            title: ev.title,
            subtitle: ev.subtitle,
            options: ev.options,
            multi: ev.multi,
            hasOther: ev.has_other,
          };
          chat.sheetAskUser(payload);
          sheet.applyAskUser(payload);
          break;
        }
        case 'sheet_interview_open': {
          // Single-panel one-shot interview (Sprint UX-2). The agent has
          // done its deep PoB analysis and now ships every section + every
          // leverage question + a notes prompt in one delta. Stash the
          // payload on the chat segment for restoration after reload, and
          // open the live interview in the sheet store.
          chat.sheetInterviewOpen(ev.payload);
          sheet.openInterview(ev.payload);
          break;
        }
        case 'sheet_finalized': {
          // The agent's `sheet_finalize_request` succeeded — the sheet is
          // persisted in `build_sheets`. Anchor a permanent "✓ Build Sheet
          // saved" banner segment in the chat timeline. The companion
          // `sheet_loaded` event populates the sidebar card.
          chat.sheetFinalized(ev.sheet_id, ev.name);
          break;
        }
        case 'sheet_loaded': {
          // A validated sheet became active for this chat. Fired by both
          // finalize (right after persist) and `get_active_build_sheet`
          // (lookup by fingerprint), so the sidebar `BSLinkedSheetCard`
          // populates without waiting for the next user turn.
          sheet.loadActiveSheet({
            sheetId: ev.sheet_id,
            fingerprint: ev.fingerprint,
            name: ev.name,
            pobHash: ev.pob_hash,
            authoredAt: ev.authored_at,
            updatedAt: ev.updated_at,
            schemaVersion: ev.schema_version,
            payload: ev.payload,
          });
          if (ev.stale) sheet.markStale();
          break;
        }
        case 'verifier': {
          // Empty `claims_checked` means the heuristic short-circuited
          // (cheap draft) or the toggle is off — render nothing in
          // those cases so the chat stays uncluttered. Anything beyond
          // that surfaces as a slim tool card on the assistant turn.
          if (ev.claims_checked.length === 0) break;
          chat.verifierResult({
            status: ev.status,
            claims: ev.claims_checked,
            correctionsCount: ev.corrections_count,
            findingsSummary: ev.findings_summary,
          });
          break;
        }
        case 'mode_assigned': {
          // Sprint v3 — deterministic turn classifier output. The default
          // mode is suppressed server-side, so anything reaching here is
          // a user-visible chip. Pin the value on the current assistant
          // message so a reload reproduces the chip from chat history.
          chat.setAssistantMode(ev.mode);
          break;
        }
      }
    });
  });

  onBeforeUnmount(() => {
    if (unlisten) unlisten();
    unlisten = null;
  });
}

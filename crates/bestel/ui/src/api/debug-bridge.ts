// Debug bridge — exposes a stable `window.__bestel` API for the
// `bestel-driver` harness to drive the frontend over CDP. The bridge is
// installed ONLY when the backend reports `debug_is_enabled = true`,
// which itself is gated by the `BESTEL_DEBUG_PORT` env var at launch
// (see `crates/bestel/src/main.rs::maybe_enable_cdp_debug`).
//
// Without that env var, no bridge is attached — production users have
// no exposed surface, even in DevTools. The bridge wraps existing
// Pinia store actions; it does NOT add new logic that could diverge
// from the user-facing flow.

import { invoke } from '@tauri-apps/api/core';

import { useBuildStore } from '../stores/build';
import { useChatStore } from '../stores/chat';
import { useSettingsStore } from '../stores/settings';
import { useSheetStore } from '../stores/sheet';

interface MemoryStats {
  /** JS heap currently used (bytes). Edge/Chromium-only — null on others. */
  heap: number | null;
  /** Heap soft limit reported by V8. */
  heapLimit: number | null;
  /** Total DOM node count under document. */
  domNodes: number;
  /** Approximate localStorage payload size in bytes (UTF-16 = 2× chars). */
  localStorageBytes: number;
}

interface BestelDebugBridge {
  /** Resets the chat store and optionally attaches a PoB build by absolute path. */
  newChat: (buildPath?: string) => Promise<void>;
  /** Enqueues a user message; the existing chat.send pipeline takes over. */
  sendMessage: (text: string) => Promise<void>;
  /** Snapshot of the messages array as plain JSON (Vue proxies stripped). */
  getChatState: () => unknown;
  /** True while the assistant is mid-stream. */
  isStreaming: () => boolean;
  /** Heap / DOM / localStorage probe — sampled at call time. */
  getMemoryStats: () => MemoryStats;
  /** Aborts the in-flight stream if any. */
  cancelStreaming: () => Promise<void>;
  /** Currently-attached build (or null). */
  getActiveBuild: () => unknown;
  /** Currently-active model profile (DTO). */
  getActiveModel: () => unknown;
  /** Switch the active model profile by id. */
  setActiveModel: (id: string) => Promise<unknown>;
  /** Generic Tauri command bridge — useful for any backend probe. */
  invoke: (cmd: string, args?: unknown) => Promise<unknown>;
  /** Sheet store probe — returns the active interview payload (sections,
   * questions, notes prompt) as plain JSON, or null when no interview is
   * in flight. Driver scenarios use this to detect when the agent has
   * opened a one-shot interview panel after `Build sheet: absent`. */
  getActiveInterview: () => unknown;
  /** Auto-fill the active interview with sensible defaults (first option
   * for every leverage question, empty notes, no section edits) — or with
   * user-provided overrides — then synthesize the structured
   * `[INTERVIEW SUBMISSION ...]` user message and SEND it via the chat
   * pipeline. The agent's next turn finalizes the sheet and answers the
   * original question. Returns the submission text that was sent. Throws
   * if no interview is in flight. */
  autoSubmitInterview: (opts?: {
    /** Map of question_id → selected option indices. Missing questions
     * default to [0]. */
    answers?: Record<string, number[]>;
    /** Map of section_id → custom body (overrides the agent's draft). */
    sections?: Record<string, string>;
    /** Optional notes prompt response. Empty by default. */
    notes?: string;
  }) => Promise<string>;
  /** Schema version — bumped on breaking bridge changes. */
  readonly version: 3;
}

declare global {
  interface Window {
    __bestel?: BestelDebugBridge;
  }
}

function getMemoryStats(): MemoryStats {
  const perf = performance as unknown as {
    memory?: { usedJSHeapSize: number; jsHeapSizeLimit: number };
  };
  let lsBytes = 0;
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    if (key === null) continue;
    const value = localStorage.getItem(key) ?? '';
    // UTF-16: 2 bytes per char.
    lsBytes += (key.length + value.length) * 2;
  }
  return {
    heap: perf.memory?.usedJSHeapSize ?? null,
    heapLimit: perf.memory?.jsHeapSizeLimit ?? null,
    domNodes: document.querySelectorAll('*').length,
    localStorageBytes: lsBytes,
  };
}

/** Installs `window.__bestel` if the backend reports debug mode. Idempotent —
 * subsequent calls are no-ops. Errors are swallowed so a misconfigured
 * backend never breaks the regular UI boot. */
export async function installDebugBridge(): Promise<void> {
  let enabled = false;
  try {
    enabled = await invoke<boolean>('debug_is_enabled');
  } catch {
    enabled = false;
  }
  if (!enabled) return;
  if (window.__bestel) return;

  const chat = useChatStore();
  const build = useBuildStore();
  const settings = useSettingsStore();
  const sheet = useSheetStore();

  window.__bestel = {
    version: 3,
    async newChat(buildPath?: string) {
      chat.reset();
      if (buildPath) {
        try {
          await build.setActive(buildPath);
        } catch (e) {
          console.error('[bestel debug] newChat: setActive failed', e);
        }
      }
    },
    async sendMessage(text: string) {
      await chat.send(text);
    },
    getChatState() {
      // Strip Vue reactivity proxies via JSON round-trip.
      return JSON.parse(JSON.stringify(chat.messages));
    },
    isStreaming() {
      return chat.isStreaming;
    },
    getMemoryStats,
    async cancelStreaming() {
      await chat.cancel();
    },
    getActiveBuild() {
      return build.current ? JSON.parse(JSON.stringify(build.current)) : null;
    },
    getActiveModel() {
      return settings.activeModel
        ? JSON.parse(JSON.stringify(settings.activeModel))
        : null;
    },
    async setActiveModel(id: string) {
      const dto = await settings.setActive(id);
      return dto ? JSON.parse(JSON.stringify(dto)) : null;
    },
    async invoke(cmd: string, args?: unknown) {
      return await invoke(cmd, args as Record<string, unknown> | undefined);
    },
    getActiveInterview() {
      const iv = sheet.activeInterview;
      if (!iv) return null;
      // Serialize the Map fields explicitly — JSON.stringify drops Map entries.
      return {
        sections: iv.sections.map((s) => ({
          id: s.id,
          title: s.title,
          draftBody: s.draftBody,
        })),
        questions: iv.questions.map((q) => ({
          questionId: q.questionId,
          sectionId: q.sectionId,
          title: q.title,
          subtitle: q.subtitle,
          options: q.options,
          multi: q.multi,
          hasOther: q.hasOther,
        })),
        notesPrompt: iv.notesPrompt,
        answers: Object.fromEntries(
          Array.from(iv.answers.entries()).map(([k, v]) => [
            k,
            { selected: [...v.selected], otherText: v.otherText },
          ]),
        ),
        sectionEdits: Object.fromEntries(iv.sectionEdits.entries()),
        notes: iv.notes,
        submitted: iv.submitted,
      };
    },
    async autoSubmitInterview(opts) {
      const iv = sheet.activeInterview;
      if (!iv) throw new Error('autoSubmitInterview: no active interview');
      const answersOverride = opts?.answers ?? {};
      const sectionsOverride = opts?.sections ?? {};
      const notes = opts?.notes ?? '';
      for (const q of iv.questions) {
        const userAns = answersOverride[q.questionId];
        const selected =
          Array.isArray(userAns) && userAns.length > 0 ? userAns : [0];
        sheet.setAnswer(q.questionId, selected, '');
      }
      for (const [sectionId, body] of Object.entries(sectionsOverride)) {
        sheet.setSectionEdit(sectionId, body);
      }
      sheet.setNotes(notes);
      const message = sheet.submitInterview();
      await chat.send(message);
      return message;
    },
  };
  console.log('[bestel.debug] window.__bestel installed', { version: 3 });
}

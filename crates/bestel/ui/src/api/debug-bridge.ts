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
  /** Schema version — bumped on breaking bridge changes. */
  readonly version: 2;
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

  window.__bestel = {
    version: 2,
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
  };
  // Loud log so the driver's `logs --filter bestel.debug` immediately picks
  // up that the bridge is live and version-stamped.
  console.log('[bestel.debug] window.__bestel installed', { version: 2 });
}

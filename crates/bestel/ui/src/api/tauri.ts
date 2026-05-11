import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

import type {
  AttachmentDto,
  BuildEvent,
  BuildSheetDetailDto,
  BuildSheetSummaryDto,
  DebugRunDto,
  DetectionDto,
  KeyStatusDto,
  LintReport,
  LlmDeltaEvent,
  ModelProfileDto,
  PobBuildDto,
  PobBuildSummaryDto,
  ProviderStatusEvent,
  SettingsDto,
} from './types';

export const ping = (name: string): Promise<string> => invoke('ping', { name });

export const windowMinimize = (): Promise<void> => invoke('window_minimize');
export const windowToggleMaximize = (): Promise<void> => invoke('window_toggle_maximize');
export const windowClose = (): Promise<void> => invoke('window_close');
export const isMaximized = async (): Promise<boolean> => getCurrentWindow().isMaximized();

export const listModels = (): Promise<ModelProfileDto[]> => invoke('list_models');
export const getActiveModel = (): Promise<ModelProfileDto> => invoke('get_active_model');
export const setActiveModel = (id: string): Promise<ModelProfileDto> =>
  invoke('set_active_model', { id });

export const detectProviders = (): Promise<DetectionDto> => invoke('detect_providers');

export const listBuilds = (): Promise<PobBuildSummaryDto[]> => invoke('list_builds');
export const getActiveBuild = (): Promise<PobBuildDto | null> => invoke('get_active_build');
export const setActiveBuild = (path: string): Promise<PobBuildDto> =>
  invoke('set_active_build', { path });
export const clearActiveBuild = (): Promise<void> => invoke('clear_active_build');
export const previewBuild = (path: string): Promise<PobBuildDto> =>
  invoke('preview_build', { path });

export const listBuildSheets = (): Promise<BuildSheetSummaryDto[]> =>
  invoke('list_build_sheets');
export const getBuildSheet = (id: string): Promise<BuildSheetDetailDto | null> =>
  invoke('get_build_sheet', { id });
export const deleteBuildSheet = (id: string): Promise<boolean> =>
  invoke('delete_build_sheet', { id });

/** DTO returned by `get_active_build_sheet_for_ui` — the BuildSheetDetailDto
 * shape plus `pob_hash_match` so the caller can decide fresh-vs-stale
 * presentation. */
export interface ActiveBuildSheetDto extends BuildSheetDetailDto {
  pob_hash_match: boolean;
}

/** Frontend-side lookup of the persisted Build Sheet (if any) for the
 * currently active build. Resolves to null when no build is attached OR
 * when no sheet exists for the build's fingerprint. Triggered by the
 * build store on app boot, build attach, and chat switch — the sheet
 * sidebar card no longer needs the agent to volunteer
 * `get_active_build_sheet` to discover an existing sheet. */
export const getActiveBuildSheetForUi = (): Promise<ActiveBuildSheetDto | null> =>
  invoke('get_active_build_sheet_for_ui');

export const chatStart = (
  prompt: string,
  attachments?: AttachmentDto[],
): Promise<number> =>
  invoke('chat_start', {
    prompt,
    attachments: attachments && attachments.length > 0 ? attachments : null,
  });
export const chatCancel = (): Promise<void> => invoke('chat_cancel');
export const chatReset = (): Promise<void> => invoke('chat_reset');

/** Write the current chat's full state to `~/.bestel/runtime/conversation-logs/{chatId}.json`.
 *  Returns the absolute path on success. Used by the autosave watch in
 *  the chat-history store so a coding agent can pick up the latest log
 *  for debugging without the user having to copy/paste the timeline. */
export const chatLogPersist = (chatId: string, json: string): Promise<string> =>
  invoke('chat_log_persist', { chatId, json });

/** Resolve the absolute path of the conversation-logs directory. Used by
 *  the debug button to surface the directory in the OS file manager. */
export const chatLogDir = (): Promise<string> => invoke('chat_log_dir');

export const openExternal = (url: string): Promise<void> => invoke('open_external', { url });

/** Mount or refresh the in-app link viewer sub-webview at the given logical bounds. */
export const openLinkModal = (
  url: string,
  x: number,
  y: number,
  w: number,
  h: number,
): Promise<void> => invoke('open_link_modal', { url, x, y, w, h });

/** Re-position / resize the in-app link viewer (e.g. on window resize). */
export const updateLinkModalBounds = (
  x: number,
  y: number,
  w: number,
  h: number,
): Promise<void> => invoke('update_link_modal_bounds', { x, y, w, h });

/** Tear down the in-app link viewer. */
export const closeLinkModal = (): Promise<void> => invoke('close_link_modal');

/**
 * Single entry-point for clicking a link in the chat / pickers.
 * Consults `useSettingsStore.linkBehavior` and either opens the URL in the
 * in-app overlay (default) or hands it to the OS's default browser.
 *
 * Lazy-imports the store so this module remains usable from anywhere in the
 * app without circular deps.
 */
export async function openLink(url: string): Promise<void> {
  if (!url) return;
  // Lazy require to avoid circular imports between api/tauri and stores/.
  const { useSettingsStore } = await import('../stores/settings');
  const { useUiStore } = await import('../stores/ui');
  const settings = useSettingsStore();
  if (settings.linkBehavior === 'browser') {
    await openExternal(url);
    return;
  }
  // 'modal' — let the LinkViewerModal mount, it forwards bounds to backend.
  useUiStore().openLinkViewer(url);
}

/**
 * Click handler for any `v-html` container that may render markdown
 * `<a>` links. Intercepts the click, prevents the default webview
 * navigation (which would otherwise replace the whole app view with the
 * external page — the bug pattern fixed in the link-bug sprint), and
 * routes the URL through {@link openLink} which respects the user's
 * `linkBehavior` setting (in-app overlay vs OS browser).
 *
 * Does NOT handle panel-button clicks — those still live in
 * `ChatMessage.handleClick` because they need access to the message's
 * `panelMap` for artifact resolution.
 *
 * Idempotent: it's safe to register this on a container that already
 * has another `@click` listener — the inner handler's `e.preventDefault()`
 * only fires when an anchor is the actual target.
 */
export function interceptAnchorClick(e: MouseEvent): void {
  const target = e.target as HTMLElement | null;
  if (!target) return;
  const anchor = target.closest('a') as HTMLAnchorElement | null;
  if (!anchor) return;
  const href = anchor.dataset.wikiUrl ?? anchor.getAttribute('href');
  if (!href) return;
  // Allow in-document anchors and mailto/javascript scheme to fall
  // through. Only swallow http(s) and file: URLs that would otherwise
  // hijack the main webview.
  if (
    href.startsWith('#') ||
    href.startsWith('mailto:') ||
    href.startsWith('javascript:')
  ) {
    return;
  }
  // Same-origin tauri.localhost navigation (in-app routes) is also
  // safe — vue-router or other internal mechanisms own those.
  try {
    const u = new URL(href, window.location.href);
    if (u.origin === window.location.origin) {
      return;
    }
  } catch {
    // malformed URL — fall through and treat as external for safety.
  }
  e.preventDefault();
  void openLink(href);
}

/**
 * Document-level defense in depth. Attaches a single capturing-phase
 * `click` listener that catches ANY anchor click anywhere in the page —
 * including v-html containers that forgot to wire {@link interceptAnchorClick}
 * directly. Without this guard, an unhandled `<a href="https://...">` click
 * navigates the entire WebView to the external URL, replacing the app
 * (and stranding the user, since the title bar's window controls go with
 * the navigation).
 *
 * Idempotent: re-calling does NOT register multiple listeners (uses a
 * Symbol marker on `document`).
 */
const GUARD_INSTALLED = Symbol.for('bestel.linkGuard');

export function installGlobalAnchorGuard(): void {
  // The Symbol-keyed sentinel is sticky across hot-reloads, so we don't
  // accumulate listeners in dev mode either.
  const doc = document as Document & { [k: symbol]: boolean };
  if (doc[GUARD_INSTALLED]) return;
  doc[GUARD_INSTALLED] = true;
  // Capture phase so we beat any inline `<a>` default before bubbling
  // hits the WebView's navigation logic.
  document.addEventListener('click', interceptAnchorClick, { capture: true });
}

export const listDebugRuns = (): Promise<DebugRunDto[]> => invoke('list_debug_runs');
export const getDebugRun = (id: string): Promise<DebugRunDto | null> =>
  invoke('get_debug_run', { id });
export const deleteDebugRun = (id: string): Promise<void> =>
  invoke('delete_debug_run', { id });
export const deleteAllDebugRuns = (): Promise<number> => invoke('delete_all_debug_runs');
export const lintDebugRun = (id: string): Promise<LintReport | null> =>
  invoke('lint_debug_run', { id });

export const listApiKeys = (): Promise<KeyStatusDto[]> => invoke('list_api_keys');
export const setApiKey = (envName: string, value: string): Promise<void> =>
  invoke('set_api_key', { envName, value });
export const deleteApiKey = (envName: string): Promise<void> =>
  invoke('delete_api_key', { envName });

export const settingsGet = (): Promise<SettingsDto> => invoke('settings_get');
export const settingsSetVerifyEnabled = (enabled: boolean): Promise<void> =>
  invoke('settings_set_verify_enabled', { enabled });

export const onLlmDelta = (cb: (e: LlmDeltaEvent) => void): Promise<UnlistenFn> =>
  listen<LlmDeltaEvent>('llm:delta', (ev) => cb(ev.payload));

export const onPobBuild = (cb: (e: BuildEvent) => void): Promise<UnlistenFn> =>
  listen<BuildEvent>('pob:build', (ev) => cb(ev.payload));

export const onPobCleared = (cb: () => void): Promise<UnlistenFn> =>
  listen<unknown>('pob:cleared', () => cb());

export const onProviderStatus = (cb: (e: ProviderStatusEvent) => void): Promise<UnlistenFn> =>
  listen<ProviderStatusEvent>('provider:status', (ev) => cb(ev.payload));

export interface PromptFileMeta {
  rel_path: string;
  kind: 'shipped' | 'custom';
  modified_vs_bundled: boolean;
  byte_size: number;
  line_count: number;
  description: string;
}

export interface PromptGroup {
  label: string;
  description: string;
  items: PromptFileMeta[];
}

export interface PromptTree {
  groups: PromptGroup[];
}

export interface PromptFile {
  rel_path: string;
  content: string;
  kind: 'shipped' | 'custom';
  bundled: string | null;
}

export interface PromptsChangedEvent {
  rel_path: string;
  reason: string;
}

export const promptsList = (): Promise<PromptTree> => invoke('prompts_list');
export const promptsRead = (path: string): Promise<PromptFile> =>
  invoke('prompts_read', { path });
export const promptsWrite = (path: string, content: string): Promise<void> =>
  invoke('prompts_write', { path, content });
export const promptsReset = (path: string): Promise<void> =>
  invoke('prompts_reset', { path });
export const promptsResetAll = (): Promise<void> => invoke('prompts_reset_all');
export const promptsOpenEditor = (): Promise<void> => invoke('prompts_open_editor');
export const promptsRevealInFinder = (path: string): Promise<void> =>
  invoke('prompts_reveal_in_finder', { path });

export const onPromptsChanged = (
  cb: (e: PromptsChangedEvent) => void,
): Promise<UnlistenFn> =>
  listen<PromptsChangedEvent>('prompts:changed', (ev) => cb(ev.payload));

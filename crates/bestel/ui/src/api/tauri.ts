import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

import type {
  AttachmentDto,
  BuildEvent,
  DebugRunDto,
  DetectionDto,
  KeyStatusDto,
  LlmDeltaEvent,
  ModelProfileDto,
  PobBuildDto,
  PobBuildSummaryDto,
  ProviderStatusEvent,
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

export const listDebugRuns = (): Promise<DebugRunDto[]> => invoke('list_debug_runs');
export const getDebugRun = (id: string): Promise<DebugRunDto | null> =>
  invoke('get_debug_run', { id });
export const deleteDebugRun = (id: string): Promise<void> =>
  invoke('delete_debug_run', { id });
export const deleteAllDebugRuns = (): Promise<number> => invoke('delete_all_debug_runs');

export const listApiKeys = (): Promise<KeyStatusDto[]> => invoke('list_api_keys');
export const setApiKey = (envName: string, value: string): Promise<void> =>
  invoke('set_api_key', { envName, value });
export const deleteApiKey = (envName: string): Promise<void> =>
  invoke('delete_api_key', { envName });

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
}

export interface PromptGroup {
  label: string;
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

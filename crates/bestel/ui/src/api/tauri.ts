import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

import type {
  AttachmentDto,
  BuildEvent,
  DebugRunDto,
  DetectionDto,
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

export const listDebugRuns = (): Promise<DebugRunDto[]> => invoke('list_debug_runs');
export const getDebugRun = (id: string): Promise<DebugRunDto | null> =>
  invoke('get_debug_run', { id });
export const deleteDebugRun = (id: string): Promise<void> =>
  invoke('delete_debug_run', { id });
export const deleteAllDebugRuns = (): Promise<number> => invoke('delete_all_debug_runs');

export const onLlmDelta = (cb: (e: LlmDeltaEvent) => void): Promise<UnlistenFn> =>
  listen<LlmDeltaEvent>('llm:delta', (ev) => cb(ev.payload));

export const onPobBuild = (cb: (e: BuildEvent) => void): Promise<UnlistenFn> =>
  listen<BuildEvent>('pob:build', (ev) => cb(ev.payload));

export const onPobCleared = (cb: () => void): Promise<UnlistenFn> =>
  listen<unknown>('pob:cleared', () => cb());

export const onProviderStatus = (cb: (e: ProviderStatusEvent) => void): Promise<UnlistenFn> =>
  listen<ProviderStatusEvent>('provider:status', (ev) => cb(ev.payload));

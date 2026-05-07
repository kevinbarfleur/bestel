import { defineStore } from 'pinia';
import { computed, ref, watch } from 'vue';
import { emitTo } from '@tauri-apps/api/event';

import {
  deleteApiKey as deleteApiKeyIpc,
  detectProviders,
  getActiveModel,
  listApiKeys,
  listModels,
  setActiveModel as setActiveModelIpc,
  setApiKey as setApiKeyIpc,
} from '../api/tauri';
import type {
  DetectionDto,
  KeyStatusDto,
  ModelProfileDto,
  ProviderKindLabel,
} from '../api/types';

export type Theme = 'light' | 'dark';
export type LinkBehavior = 'modal' | 'browser';
export type Density = 'compact' | 'standard' | 'airy';

const THEME_KEY = 'bestel.theme';
const LINK_BEHAVIOR_KEY = 'bestel.linkBehavior';
const DENSITY_KEY = 'bestel.density';
const DEFAULT_MODEL_KEY = 'bestel.defaultModelId';
const WATCHER_FOLDERS_KEY = 'bestel.watcherFolders';

function readStoredTheme(): Theme {
  try {
    const raw = localStorage.getItem(THEME_KEY);
    if (raw === 'dark' || raw === 'light') return raw;
  } catch {
    /* localStorage unavailable */
  }
  return 'light';
}

function readStoredLinkBehavior(): LinkBehavior {
  try {
    const raw = localStorage.getItem(LINK_BEHAVIOR_KEY);
    if (raw === 'modal' || raw === 'browser') return raw;
  } catch {
    /* localStorage unavailable */
  }
  return 'modal';
}

function readStoredDensity(): Density {
  try {
    const raw = localStorage.getItem(DENSITY_KEY);
    if (raw === 'compact' || raw === 'standard' || raw === 'airy') return raw;
  } catch {
    /* localStorage unavailable */
  }
  return 'standard';
}

function applyDensityToDom(d: Density) {
  if (typeof document === 'undefined') return;
  document.documentElement.setAttribute('data-density', d);
}

function readStoredString(key: string): string | null {
  try {
    const raw = localStorage.getItem(key);
    return raw && raw.trim() ? raw : null;
  } catch {
    return null;
  }
}

function readStoredFolders(): string[] {
  try {
    const raw = localStorage.getItem(WATCHER_FOLDERS_KEY);
    if (!raw) return [];
    const arr = JSON.parse(raw);
    return Array.isArray(arr) ? arr.filter((s) => typeof s === 'string') : [];
  } catch {
    return [];
  }
}

function applyThemeToDom(theme: Theme) {
  if (typeof document === 'undefined') return;
  document.documentElement.classList.toggle('theme-dark', theme === 'dark');
}

export const useSettingsStore = defineStore('settings', () => {
  const models = ref<ModelProfileDto[]>([]);
  const activeModel = ref<ModelProfileDto | null>(null);
  const detection = ref<DetectionDto | null>(null);
  const apiKeys = ref<KeyStatusDto[]>([]);
  const theme = ref<Theme>(readStoredTheme());
  const linkBehavior = ref<LinkBehavior>(readStoredLinkBehavior());
  const density = ref<Density>(readStoredDensity());
  const defaultModelId = ref<string | null>(readStoredString(DEFAULT_MODEL_KEY));
  const watcherFolders = ref<string[]>(readStoredFolders());

  applyThemeToDom(theme.value);
  applyDensityToDom(density.value);

  watch(theme, (next) => {
    applyThemeToDom(next);
    try {
      localStorage.setItem(THEME_KEY, next);
    } catch {
      /* ignore */
    }
    // Sync the theme to the prompt-editor window when it's open. emitTo
    // delivers to whatever windows match the label and silently noops
    // otherwise — safe to fire-and-forget.
    emitTo('prompt-editor', 'theme:changed', { theme: next }).catch(() => {});
  });

  watch(linkBehavior, (next) => {
    try {
      localStorage.setItem(LINK_BEHAVIOR_KEY, next);
    } catch {
      /* ignore */
    }
  });

  watch(density, (next) => {
    applyDensityToDom(next);
    try {
      localStorage.setItem(DENSITY_KEY, next);
    } catch {
      /* ignore */
    }
  });

  watch(defaultModelId, (next) => {
    try {
      if (next) localStorage.setItem(DEFAULT_MODEL_KEY, next);
      else localStorage.removeItem(DEFAULT_MODEL_KEY);
    } catch {
      /* ignore */
    }
  });

  watch(
    watcherFolders,
    (next) => {
      try {
        localStorage.setItem(WATCHER_FOLDERS_KEY, JSON.stringify(next));
      } catch {
        /* ignore */
      }
    },
    { deep: true },
  );

  function setTheme(next: Theme) {
    theme.value = next;
  }

  function toggleTheme() {
    theme.value = theme.value === 'dark' ? 'light' : 'dark';
  }

  function setLinkBehavior(next: LinkBehavior) {
    linkBehavior.value = next;
  }

  function setDensity(next: Density) {
    density.value = next;
  }

  function setDefaultModelId(id: string | null) {
    defaultModelId.value = id;
  }

  function setWatcherFolders(folders: string[]) {
    watcherFolders.value = [...folders];
  }

  const providerAvailability = computed<Record<ProviderKindLabel, boolean>>(() => {
    const map: Record<ProviderKindLabel, boolean> = {
      anthropic: false,
      ollama: false,
    };
    if (!detection.value) return map;
    for (const probe of detection.value.probes) {
      const name = probe.name.toLowerCase();
      if (name.includes('anthropic')) map.anthropic = probe.installed;
      else if (name.includes('ollama')) map.ollama = probe.installed;
    }
    return map;
  });

  async function refreshModels() {
    try {
      models.value = await listModels();
    } catch {
      models.value = [];
    }
    try {
      activeModel.value = await getActiveModel();
    } catch {
      activeModel.value = null;
    }
  }

  async function refreshDetection() {
    try {
      detection.value = await detectProviders();
    } catch {
      detection.value = null;
    }
  }

  async function setActive(id: string): Promise<ModelProfileDto | null> {
    try {
      const dto = await setActiveModelIpc(id);
      activeModel.value = dto;
      return dto;
    } catch {
      return null;
    }
  }

  function applyDetection(d: DetectionDto) {
    detection.value = d;
  }

  async function refreshKeys() {
    try {
      apiKeys.value = await listApiKeys();
    } catch {
      apiKeys.value = [];
    }
  }

  /** Returns null on success, or a human-readable error message on failure. */
  async function saveApiKey(envName: string, value: string): Promise<string | null> {
    try {
      await setApiKeyIpc(envName, value);
      await Promise.all([refreshKeys(), refreshDetection(), refreshModels()]);
      return null;
    } catch (e: unknown) {
      return e instanceof Error ? e.message : String(e);
    }
  }

  async function deleteApiKey(envName: string): Promise<string | null> {
    try {
      await deleteApiKeyIpc(envName);
      await Promise.all([refreshKeys(), refreshDetection(), refreshModels()]);
      return null;
    } catch (e: unknown) {
      return e instanceof Error ? e.message : String(e);
    }
  }

  function keyStatus(envName: string): KeyStatusDto | null {
    return apiKeys.value.find((k) => k.env_name === envName) ?? null;
  }

  return {
    models,
    activeModel,
    detection,
    apiKeys,
    theme,
    linkBehavior,
    density,
    defaultModelId,
    watcherFolders,
    providerAvailability,
    setTheme,
    toggleTheme,
    setLinkBehavior,
    setDensity,
    setDefaultModelId,
    setWatcherFolders,
    refreshModels,
    refreshDetection,
    refreshKeys,
    saveApiKey,
    deleteApiKey,
    keyStatus,
    setActive,
    applyDetection,
  };
});

import { defineStore } from 'pinia';
import { computed, ref, watch } from 'vue';

import {
  detectProviders,
  getActiveModel,
  listModels,
  setActiveModel as setActiveModelIpc,
} from '../api/tauri';
import type { DetectionDto, ModelProfileDto, ProviderKindLabel } from '../api/types';

export type Theme = 'light' | 'dark';
const THEME_KEY = 'bestel.theme';

function readStoredTheme(): Theme {
  try {
    const raw = localStorage.getItem(THEME_KEY);
    if (raw === 'dark' || raw === 'light') return raw;
  } catch {
    /* localStorage unavailable */
  }
  return 'light';
}

function applyThemeToDom(theme: Theme) {
  if (typeof document === 'undefined') return;
  document.documentElement.classList.toggle('theme-dark', theme === 'dark');
}

export const useSettingsStore = defineStore('settings', () => {
  const models = ref<ModelProfileDto[]>([]);
  const activeModel = ref<ModelProfileDto | null>(null);
  const detection = ref<DetectionDto | null>(null);
  const theme = ref<Theme>(readStoredTheme());

  applyThemeToDom(theme.value);

  watch(theme, (next) => {
    applyThemeToDom(next);
    try {
      localStorage.setItem(THEME_KEY, next);
    } catch {
      /* ignore */
    }
  });

  function setTheme(next: Theme) {
    theme.value = next;
  }

  function toggleTheme() {
    theme.value = theme.value === 'dark' ? 'light' : 'dark';
  }

  const providerAvailability = computed<Record<ProviderKindLabel, boolean>>(() => {
    const map: Record<ProviderKindLabel, boolean> = {
      anthropic: false,
      codex_cli: false,
      claude_cli: false,
    };
    if (!detection.value) return map;
    for (const probe of detection.value.probes) {
      const name = probe.name.toLowerCase();
      if (name.includes('anthropic')) map.anthropic = probe.installed;
      else if (name.includes('codex')) map.codex_cli = probe.installed;
      else if (name.includes('claude')) map.claude_cli = probe.installed;
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

  return {
    models,
    activeModel,
    detection,
    theme,
    providerAvailability,
    setTheme,
    toggleTheme,
    refreshModels,
    refreshDetection,
    setActive,
    applyDetection,
  };
});

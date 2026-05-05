import { defineStore } from 'pinia';
import { ref } from 'vue';

import {
  clearActiveBuild as clearActiveBuildIpc,
  getActiveBuild,
  listBuilds,
  setActiveBuild as setActiveBuildIpc,
} from '../api/tauri';
import type { PobBuildDto, PobBuildSummaryDto } from '../api/types';

export const useBuildStore = defineStore('build', () => {
  const current = ref<PobBuildDto | null>(null);
  const list = ref<PobBuildSummaryDto[]>([]);
  const loadingList = ref(false);

  async function refreshActive() {
    try {
      current.value = await getActiveBuild();
    } catch {
      current.value = null;
    }
  }

  async function refreshList() {
    loadingList.value = true;
    try {
      list.value = await listBuilds();
    } catch {
      list.value = [];
    } finally {
      loadingList.value = false;
    }
  }

  async function setActive(path: string): Promise<PobBuildDto | null> {
    try {
      const dto = await setActiveBuildIpc(path);
      current.value = dto;
      return dto;
    } catch {
      return null;
    }
  }

  async function clearActive(): Promise<boolean> {
    try {
      await clearActiveBuildIpc();
      current.value = null;
      return true;
    } catch {
      return false;
    }
  }

  function applyBuildEvent(build: PobBuildDto) {
    current.value = build;
  }

  function applyClearedEvent() {
    current.value = null;
  }

  return {
    current,
    list,
    loadingList,
    refreshActive,
    refreshList,
    setActive,
    clearActive,
    applyBuildEvent,
    applyClearedEvent,
  };
});

import { defineStore } from 'pinia';
import { ref } from 'vue';

import {
  clearActiveBuild as clearActiveBuildIpc,
  getActiveBuild,
  getActiveBuildSheetForUi,
  listBuilds,
  setActiveBuild as setActiveBuildIpc,
} from '../api/tauri';
import type { PobBuildDto, PobBuildSummaryDto } from '../api/types';
import { useSheetStore } from './sheet';

export const useBuildStore = defineStore('build', () => {
  const current = ref<PobBuildDto | null>(null);
  const list = ref<PobBuildSummaryDto[]>([]);
  const loadingList = ref(false);

  /** Look up the persisted Build Sheet for the currently active build's
   * fingerprint and load it into the sheet store. Triggered after any
   * change to `current` (boot, attach, switch chat) so the sidebar
   * sheet card rehydrates without waiting for the agent to volunteer
   * `get_active_build_sheet`. Null `current` → sheet store cleared. */
  async function rehydrateSheet() {
    const sheet = useSheetStore();
    if (current.value === null) {
      sheet.clearActiveSheet();
      return;
    }
    try {
      const dto = await getActiveBuildSheetForUi();
      if (!dto) {
        sheet.clearActiveSheet();
        return;
      }
      sheet.loadActiveSheet({
        sheetId: dto.id,
        fingerprint: dto.fingerprint,
        name: dto.name,
        pobHash: dto.pob_hash,
        authoredAt: dto.authored_at,
        updatedAt: dto.updated_at,
        schemaVersion: dto.schema_version,
        payload: dto.payload,
        authoredSignatures: dto.authored_signatures,
        currentSignatures: dto.current_signatures,
      });
      if (!dto.pob_hash_match) {
        sheet.markStale();
      }
    } catch {
      // Best-effort; absence of a sheet is the common case and is fine.
    }
  }

  async function refreshActive() {
    try {
      current.value = await getActiveBuild();
    } catch {
      current.value = null;
    }
    await rehydrateSheet();
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
      await rehydrateSheet();
      return dto;
    } catch {
      return null;
    }
  }

  async function clearActive(): Promise<boolean> {
    try {
      await clearActiveBuildIpc();
      current.value = null;
      useSheetStore().clearActiveSheet();
      return true;
    } catch {
      return false;
    }
  }

  function applyBuildEvent(build: PobBuildDto) {
    current.value = build;
    // Fire-and-forget; the watcher event arrives off-main-thread.
    void rehydrateSheet();
  }

  function applyClearedEvent() {
    current.value = null;
    useSheetStore().clearActiveSheet();
  }

  return {
    current,
    list,
    loadingList,
    refreshActive,
    refreshList,
    rehydrateSheet,
    setActive,
    clearActive,
    applyBuildEvent,
    applyClearedEvent,
  };
});

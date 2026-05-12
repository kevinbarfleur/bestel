import { defineStore } from 'pinia';
import { ref } from 'vue';

import { getActiveBuildSheetForUi, listBuilds } from '../api/tauri';
import type { PobBuildDto, PobBuildSummaryDto } from '../api/types';
import { useSheetStore } from './sheet';

/**
 * Build store — narrowed in Sprint v3.1.
 *
 * Used to also hold the global "currently attached build" singleton. That
 * responsibility moved to the chat store (`chat.activeBuild`) because the
 * active build is conceptually a property of one chat, not of the whole
 * app. This store now only exposes:
 *
 *   - `list`: PoB files the watcher discovered on disk (across all
 *     watcher folders), regardless of which one — if any — the current
 *     chat is using.
 *   - `rehydrateSheet(build)`: helper that fetches the linked Build Sheet
 *     from the backend whenever the chat's active build changes, so the
 *     sidebar sheet card lights up immediately on attach without waiting
 *     for the agent to volunteer `get_active_build_sheet`.
 *
 * Mutation of "which build is in use" is owned by the chat store.
 */
export const useBuildStore = defineStore('build', () => {
  const list = ref<PobBuildSummaryDto[]>([]);
  const loadingList = ref(false);

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

  /** Load (or clear) the Build Sheet linked to a given build. Called by
   *  the chat store on attach / detach / chat-restore. Null `build` →
   *  sheet store cleared. Errors are swallowed: most builds don't have
   *  a sheet yet, which is the normal case.
   *
   *  The IPC reads the sheet for whatever BuildContext currently holds
   *  on the backend, so callers must call `set_active_build` first. */
  async function rehydrateSheetForActive(build: PobBuildDto | null) {
    const sheet = useSheetStore();
    if (build === null) {
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
      // Best-effort; absence of a sheet is the common case.
    }
  }

  return {
    list,
    loadingList,
    refreshList,
    rehydrateSheetForActive,
  };
});

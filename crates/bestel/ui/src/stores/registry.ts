import { defineStore } from 'pinia';
import { ref } from 'vue';
import {
  onRegistryChanged,
  registryAdd,
  registryCheckPaths,
  registryGetPobPath,
  registryList,
  registryRefresh,
  registryRemove,
  registryTouch,
} from '../api/tauri';
import type { RegistryEntryDto } from '../api/types';

/**
 * Pinia store wrapping the Build Registry IPC commands. Loads lazily and
 * re-fetches whenever the `registry:changed` Tauri event fires (every
 * window listens, so an add in Settings reflects immediately in the
 * chat header picker).
 */
export const useRegistryStore = defineStore('registry', () => {
  const entries = ref<RegistryEntryDto[]>([]);
  /** IDs of entries whose `pob_path` does not currently resolve on disk.
   *  Populated alongside `entries` by `load()`; used by the modal to badge
   *  rows red and disable their primary action. */
  const missingIds = ref<Set<number>>(new Set());
  const loading = ref(false);
  const loaded = ref(false);
  let unlisten: (() => void) | null = null;

  async function load(force = false) {
    if (loaded.value && !force) return;
    loading.value = true;
    try {
      const [list, exists] = await Promise.all([
        registryList(),
        registryCheckPaths().catch(() => [] as Array<[number, boolean]>),
      ]);
      entries.value = list;
      missingIds.value = new Set(
        exists.filter(([, ok]) => !ok).map(([id]) => id),
      );
      loaded.value = true;
    } finally {
      loading.value = false;
    }
  }

  async function add(pobPath: string): Promise<RegistryEntryDto> {
    const entry = await registryAdd(pobPath);
    // The `registry:changed` event will trigger a re-load below; do not
    // patch the local entries array here to keep the source of truth in
    // SQLite.
    return entry;
  }

  async function remove(id: number): Promise<boolean> {
    const removed = await registryRemove(id);
    return removed;
  }

  async function refresh(id: number): Promise<RegistryEntryDto> {
    return await registryRefresh(id);
  }

  async function touch(id: number): Promise<void> {
    await registryTouch(id);
  }

  async function pobPathFor(id: number): Promise<string | null> {
    return await registryGetPobPath(id);
  }

  async function start() {
    await load();
    if (unlisten) return;
    unlisten = await onRegistryChanged(() => {
      void load(true);
    });
  }

  return {
    entries,
    missingIds,
    loading,
    loaded,
    load,
    add,
    remove,
    refresh,
    touch,
    pobPathFor,
    start,
  };
});

import { defineStore } from 'pinia';
import { ref } from 'vue';
import {
  onRegistryChanged,
  registryAdd,
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
  const loading = ref(false);
  const loaded = ref(false);
  let unlisten: (() => void) | null = null;

  async function load(force = false) {
    if (loaded.value && !force) return;
    loading.value = true;
    try {
      entries.value = await registryList();
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

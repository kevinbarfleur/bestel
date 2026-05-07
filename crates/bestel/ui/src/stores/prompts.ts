import { defineStore } from 'pinia';
import { computed, ref } from 'vue';

import {
  promptsList,
  promptsRead,
  promptsWrite,
  promptsReset,
  promptsResetAll,
  promptsRevealInFinder,
  onPromptsChanged,
  type PromptFile,
  type PromptTree,
} from '../api/tauri';

const SAVE_DEBOUNCE_MS = 500;

export const usePromptsStore = defineStore('prompts', () => {
  const tree = ref<PromptTree>({ groups: [] });
  const activeFile = ref<string | null>(null);
  const openTabs = ref<string[]>([]);
  const contents = ref<Record<string, string>>({});
  const bundled = ref<Record<string, string | null>>({});
  const kinds = ref<Record<string, 'shipped' | 'custom'>>({});
  const dirty = ref<Set<string>>(new Set());
  const lastSavedAt = ref<Record<string, number>>({});
  const treeLoaded = ref(false);
  const watcherUnlisten = ref<(() => void) | null>(null);
  const saveTimers = new Map<string, ReturnType<typeof setTimeout>>();

  const activeContent = computed<string>(() => {
    const path = activeFile.value;
    return path != null ? contents.value[path] ?? '' : '';
  });

  const activeMeta = computed(() => {
    const path = activeFile.value;
    if (path == null) return null;
    for (const group of tree.value.groups) {
      const found = group.items.find((it) => it.rel_path === path);
      if (found) return found;
    }
    return null;
  });

  const activeIsDirty = computed(() => {
    const path = activeFile.value;
    return path != null && dirty.value.has(path);
  });

  function applyFile(file: PromptFile) {
    contents.value[file.rel_path] = file.content;
    bundled.value[file.rel_path] = file.bundled;
    kinds.value[file.rel_path] = file.kind;
  }

  async function loadTree() {
    try {
      tree.value = await promptsList();
      treeLoaded.value = true;
    } catch (e) {
      console.error('promptsList failed', e);
      tree.value = { groups: [] };
      treeLoaded.value = true;
    }
  }

  async function openFile(path: string): Promise<void> {
    if (!openTabs.value.includes(path)) {
      openTabs.value.push(path);
    }
    activeFile.value = path;
    if (!(path in contents.value)) {
      try {
        const file = await promptsRead(path);
        applyFile(file);
      } catch (e) {
        console.error('promptsRead failed', path, e);
        contents.value[path] = '';
      }
    }
  }

  function closeTab(path: string) {
    const idx = openTabs.value.indexOf(path);
    if (idx === -1) return;
    if (dirty.value.has(path)) {
      const ok = confirm(`${path} has unsaved changes. Close anyway?`);
      if (!ok) return;
    }
    openTabs.value.splice(idx, 1);
    delete contents.value[path];
    delete bundled.value[path];
    delete kinds.value[path];
    dirty.value.delete(path);
    delete lastSavedAt.value[path];
    if (activeFile.value === path) {
      activeFile.value = openTabs.value[openTabs.value.length - 1] ?? null;
    }
  }

  function setContent(path: string, value: string) {
    if (contents.value[path] === value) return;
    contents.value[path] = value;
    dirty.value.add(path);
    scheduleSave(path);
  }

  function scheduleSave(path: string) {
    const existing = saveTimers.get(path);
    if (existing) clearTimeout(existing);
    const handle = setTimeout(() => {
      void save(path);
    }, SAVE_DEBOUNCE_MS);
    saveTimers.set(path, handle);
  }

  async function save(path: string): Promise<void> {
    const content = contents.value[path];
    if (content == null) return;
    try {
      await promptsWrite(path, content);
      dirty.value.delete(path);
      lastSavedAt.value[path] = Date.now();
      // Refresh tree after a save so 'modified_vs_bundled' chips update.
      void loadTree();
    } catch (e) {
      console.error('promptsWrite failed', path, e);
    }
  }

  async function saveActive(): Promise<void> {
    if (activeFile.value != null) {
      const t = saveTimers.get(activeFile.value);
      if (t) {
        clearTimeout(t);
        saveTimers.delete(activeFile.value);
      }
      await save(activeFile.value);
    }
  }

  async function resetCurrent(): Promise<void> {
    const path = activeFile.value;
    if (path == null) return;
    if (kinds.value[path] !== 'shipped') return;
    try {
      await promptsReset(path);
      const file = await promptsRead(path);
      applyFile(file);
      dirty.value.delete(path);
      lastSavedAt.value[path] = Date.now();
      void loadTree();
    } catch (e) {
      console.error('promptsReset failed', path, e);
    }
  }

  async function resetAll(): Promise<void> {
    try {
      await promptsResetAll();
      for (const path of openTabs.value) {
        try {
          const file = await promptsRead(path);
          applyFile(file);
          dirty.value.delete(path);
          lastSavedAt.value[path] = Date.now();
        } catch {
          /* file may have been removed; skip */
        }
      }
      void loadTree();
    } catch (e) {
      console.error('promptsResetAll failed', e);
    }
  }

  async function reveal(path: string | null): Promise<void> {
    try {
      await promptsRevealInFinder(path ?? '');
    } catch (e) {
      console.error('promptsRevealInFinder failed', e);
    }
  }

  async function watchExternalChanges(): Promise<void> {
    if (watcherUnlisten.value != null) return;
    try {
      const un = await onPromptsChanged(async (ev) => {
        const path = ev.rel_path.replace(/\\/g, '/');
        if (!openTabs.value.includes(path)) {
          await loadTree();
          return;
        }
        if (dirty.value.has(path)) return;
        try {
          const file = await promptsRead(path);
          applyFile(file);
          lastSavedAt.value[path] = Date.now();
        } catch (e) {
          console.error('reload after watcher event failed', path, e);
        }
      });
      watcherUnlisten.value = un;
    } catch (e) {
      console.warn('prompts watcher subscribe failed', e);
    }
  }

  function stopWatching() {
    if (watcherUnlisten.value != null) {
      watcherUnlisten.value();
      watcherUnlisten.value = null;
    }
  }

  return {
    tree,
    activeFile,
    openTabs,
    contents,
    kinds,
    dirty,
    lastSavedAt,
    treeLoaded,
    activeContent,
    activeMeta,
    activeIsDirty,
    loadTree,
    openFile,
    closeTab,
    setContent,
    saveActive,
    save,
    resetCurrent,
    resetAll,
    reveal,
    watchExternalChanges,
    stopWatching,
  };
});

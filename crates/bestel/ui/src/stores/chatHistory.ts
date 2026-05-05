import { defineStore } from 'pinia';
import { computed, ref } from 'vue';

import type { ChatMessageVm } from './chat';

export interface SavedChat {
  id: string;
  title: string;
  messages: ChatMessageVm[];
  attached_build_path: string | null;
  created_at: number;
  updated_at: number;
}

const STORAGE_KEY = 'bestel.chats.v1';
const ACTIVE_KEY = 'bestel.chats.active';
const MAX_CHATS = 50;
const TITLE_MAX = 64;

const newId = (): string => `c_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 8)}`;

function safeRead<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return fallback;
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

function safeWrite(key: string, value: unknown): void {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    /* quota / disabled */
  }
}

function deriveTitle(messages: ChatMessageVm[]): string {
  for (const m of messages) {
    if (m.role !== 'user') continue;
    for (const seg of m.segments) {
      if (seg.kind === 'text') {
        const t = seg.text.trim().replace(/\s+/g, ' ');
        if (t) return t.length > TITLE_MAX ? `${t.slice(0, TITLE_MAX - 1)}…` : t;
      }
    }
  }
  return 'New chat';
}

export const useChatHistoryStore = defineStore('chatHistory', () => {
  const chats = ref<SavedChat[]>(safeRead<SavedChat[]>(STORAGE_KEY, []));
  const activeId = ref<string | null>(safeRead<string | null>(ACTIVE_KEY, null));

  const sortedChats = computed(() =>
    [...chats.value].sort((a, b) => b.updated_at - a.updated_at),
  );

  function persist(): void {
    safeWrite(STORAGE_KEY, chats.value);
    safeWrite(ACTIVE_KEY, activeId.value);
  }

  function findActive(): SavedChat | null {
    if (!activeId.value) return null;
    return chats.value.find((c) => c.id === activeId.value) ?? null;
  }

  /** Snapshot the current in-memory chat into the saved list. */
  function snapshot(messages: ChatMessageVm[], buildPath: string | null): void {
    if (messages.length === 0) return;
    const now = Date.now();
    let active = findActive();
    if (!active) {
      active = {
        id: newId(),
        title: deriveTitle(messages),
        messages: [],
        attached_build_path: buildPath,
        created_at: now,
        updated_at: now,
      };
      chats.value.push(active);
      activeId.value = active.id;
    }
    active.messages = messages.map((m) => ({ ...m, segments: [...m.segments] }));
    active.attached_build_path = buildPath;
    active.title = deriveTitle(active.messages);
    active.updated_at = now;
    // Trim to MAX
    if (chats.value.length > MAX_CHATS) {
      const sorted = [...chats.value].sort((a, b) => b.updated_at - a.updated_at);
      chats.value = sorted.slice(0, MAX_CHATS);
    }
    persist();
  }

  function startNew(): void {
    activeId.value = null;
    persist();
  }

  function select(id: string): SavedChat | null {
    const chat = chats.value.find((c) => c.id === id);
    if (!chat) return null;
    activeId.value = chat.id;
    persist();
    return chat;
  }

  function remove(id: string): void {
    chats.value = chats.value.filter((c) => c.id !== id);
    if (activeId.value === id) activeId.value = null;
    persist();
  }

  function clear(): void {
    chats.value = [];
    activeId.value = null;
    persist();
  }

  return {
    chats,
    activeId,
    sortedChats,
    snapshot,
    startNew,
    select,
    remove,
    clear,
    findActive,
  };
});

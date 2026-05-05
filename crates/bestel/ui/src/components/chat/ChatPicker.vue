<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import { useChatHistoryStore } from '../../stores/chatHistory';
import { useChatStore } from '../../stores/chat';
import { useBuildStore } from '../../stores/build';
import { useToastsStore } from '../../stores/toasts';
import RunicInput from '../runic/RunicInput.vue';
import RunicModal from '../runic/RunicModal.vue';

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>();

const history = useChatHistoryStore();
const chat = useChatStore();
const buildStore = useBuildStore();
const toasts = useToastsStore();

const { sortedChats } = storeToRefs(history);

const search = ref('');
const inputRef = ref<InstanceType<typeof RunicInput> | null>(null);

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return sortedChats.value;
  return sortedChats.value.filter((c) => c.title.toLowerCase().includes(q));
});

const close = () => emit('update:modelValue', false);

const fmtRelative = (ms: number): string => {
  const diff = Date.now() - ms;
  const m = Math.floor(diff / 60_000);
  if (m < 1) return 'just now';
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  const d = Math.floor(h / 24);
  if (d < 7) return `${d}d ago`;
  return new Date(ms).toLocaleDateString();
};

const choose = async (id: string) => {
  const saved = history.select(id);
  if (!saved) {
    toasts.push({ variant: 'error', title: 'Chat not found', body: 'It may have been deleted.' });
    close();
    return;
  }
  await chat.loadFromSaved(saved);
  if (saved.attached_build_path && buildStore.current?.source_file !== saved.attached_build_path) {
    const ok = await buildStore.setActive(saved.attached_build_path);
    if (!ok) {
      toasts.push({
        variant: 'warning',
        title: 'Build no longer available',
        body: 'Open the build picker to load a fresh one.',
      });
    }
  } else if (!saved.attached_build_path && buildStore.current) {
    await buildStore.clearActive();
  }
  toasts.push({ variant: 'info', title: 'Chat restored', body: saved.title });
  close();
};

const newChat = async () => {
  await chat.reset();
  toasts.push({ variant: 'info', title: 'New chat', body: 'Starting a fresh conversation.' });
  close();
};

const remove = (id: string, event: MouseEvent) => {
  event.stopPropagation();
  history.remove(id);
};

watch(
  () => props.modelValue,
  async (open) => {
    if (!open) return;
    search.value = '';
    await nextTick();
    inputRef.value?.focus();
  },
);
</script>

<template>
  <RunicModal
    :model-value="modelValue"
    title="Chats"
    icon="◆"
    max-width="lg"
    @update:model-value="(v) => emit('update:modelValue', v)"
  >
    <div class="chat-picker">
      <div class="chat-picker__bar">
        <RunicInput
          ref="inputRef"
          v-model="search"
          icon="search"
          placeholder="Filter chats…"
        />
        <button type="button" class="chat-picker__new" @click="newChat">+ new chat</button>
      </div>

      <p v-if="!filtered.length && !sortedChats.length" class="chat-picker__hint">
        No saved chats yet — your conversations are auto-saved as you talk.
      </p>
      <p v-else-if="!filtered.length" class="chat-picker__hint">
        No chat matches "{{ search }}".
      </p>

      <ul class="chat-picker__list runic-scrollbar" role="listbox">
        <li
          v-for="c in filtered"
          :key="c.id"
          class="chat-picker__row"
          role="option"
          @click="choose(c.id)"
        >
          <span class="chat-picker__title">{{ c.title }}</span>
          <span v-if="c.attached_build_path" class="chat-picker__build">
            ◆ {{ c.attached_build_path.split(/[/\\]/).pop() }}
          </span>
          <span class="chat-picker__time">{{ fmtRelative(c.updated_at) }}</span>
          <button
            type="button"
            class="chat-picker__remove"
            aria-label="Delete chat"
            @click="(e) => remove(c.id, e)"
          >×</button>
        </li>
      </ul>
    </div>
  </RunicModal>
</template>

<style scoped>
.chat-picker {
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
  min-height: 320px;
}

.chat-picker__bar {
  display: flex;
  gap: 8px;
  align-items: center;
}
.chat-picker__new {
  flex: none;
  padding: 6px 12px;
  background: var(--ink);
  color: var(--paper);
  border: 0;
  border-radius: 6px;
  cursor: pointer;
  font-family: var(--hand);
  font-size: 12px;
  font-weight: 500;
}
.chat-picker__new:hover { background: #1a1a18; }

.chat-picker__hint {
  margin: 0;
  text-align: center;
  font-family: var(--hand);
  color: var(--ink-faint);
}

.chat-picker__list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  max-height: 50vh;
  overflow-y: auto;
}

.chat-picker__row {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.5rem 0.4rem;
  border-bottom: 1px solid var(--paper-line);
  cursor: pointer;
  font-family: var(--hand);
  font-size: 13px;
  color: var(--ink);
  transition: background 0.15s ease;
}
.chat-picker__row:hover {
  background: var(--paper-shade);
}
.chat-picker__row:hover .chat-picker__title { color: var(--amber); }

.chat-picker__title {
  flex: 1;
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.chat-picker__build {
  font-family: var(--hand);
  font-size: 10.5px;
  color: var(--amber);
  white-space: nowrap;
  flex-shrink: 0;
}
.chat-picker__time {
  font-family: var(--hand);
  font-size: 10.5px;
  color: var(--ink-faint);
  white-space: nowrap;
  flex-shrink: 0;
}
.chat-picker__remove {
  width: 18px;
  height: 18px;
  border: 0;
  background: transparent;
  color: var(--ink-faint);
  font-size: 16px;
  line-height: 1;
  cursor: pointer;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  flex-shrink: 0;
}
.chat-picker__remove:hover {
  color: var(--bad);
}
</style>

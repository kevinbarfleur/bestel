<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import { useBuildStore } from '../../stores/build';
import { useChatHistoryStore, type SavedChat } from '../../stores/chatHistory';
import { useChatStore } from '../../stores/chat';
import { useToastsStore } from '../../stores/toasts';
import RunicModal from '../runic/RunicModal.vue';
import RunicButton from '../runic/RunicButton.vue';
import {
  PickerLayout,
  PickerListItem,
  PickerSearchInput,
  PickerSectionHead,
} from '../pickers';
import { usePickerNav } from '../../composables/usePickerNav';

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>();

const history = useChatHistoryStore();
const chat = useChatStore();
const buildStore = useBuildStore();
const toasts = useToastsStore();

const { sortedChats, activeId } = storeToRefs(history);

const search = ref('');
const confirmingDelete = ref<string | null>(null);

const filtered = computed<SavedChat[]>(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return sortedChats.value;
  return sortedChats.value.filter((c) => c.title.toLowerCase().includes(q));
});

const close = () => emit('update:modelValue', false);

const fmtRelative = (ms: number): string => {
  const diff = Date.now() - ms;
  const m = Math.floor(diff / 60_000);
  if (m < 1) return 'just now';
  if (m < 60) return `${m} min ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h} h ago`;
  const d = Math.floor(h / 24);
  if (d < 7) return `${d} d ago`;
  return new Date(ms).toLocaleDateString();
};

const fmtAbsolute = (ms: number): string => new Date(ms).toLocaleString();

const fileFromPath = (p: string | null | undefined): string => {
  if (!p) return 'none';
  return p.split(/[/\\]/).pop() ?? p;
};

const messageCount = (c: SavedChat): number => c.messages.length;

const lastMessageText = (c: SavedChat): string | null => {
  for (let i = c.messages.length - 1; i >= 0; i -= 1) {
    const m = c.messages[i];
    for (const seg of m.segments) {
      if (seg.kind === 'text' && typeof seg.text === 'string' && seg.text.trim()) {
        const t = seg.text.trim().replace(/\s+/g, ' ');
        return t.length > 320 ? `${t.slice(0, 320)}…` : t;
      }
    }
  }
  return null;
};

const lastMessageRole = (c: SavedChat): 'user' | 'assistant' | null => {
  for (let i = c.messages.length - 1; i >= 0; i -= 1) {
    const m = c.messages[i];
    for (const seg of m.segments) {
      if (seg.kind === 'text' && typeof seg.text === 'string' && seg.text.trim()) {
        return m.role === 'user' ? 'user' : 'assistant';
      }
    }
  }
  return null;
};

const isCurrent = (c: SavedChat) => activeId.value === c.id;

const choose = async (c: SavedChat) => {
  const saved = history.select(c.id);
  if (!saved) {
    toasts.push({
      variant: 'error',
      title: 'Chat not found',
      body: 'It may have been deleted.',
    });
    close();
    return;
  }
  await chat.loadFromSaved(saved);
  if (
    saved.attached_build_path &&
    buildStore.current?.source_file !== saved.attached_build_path
  ) {
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
  toasts.push({
    variant: 'info',
    title: 'New chat',
    body: 'Starting a fresh conversation.',
  });
  close();
};

const requestDelete = (id: string) => {
  confirmingDelete.value = id;
};
const cancelDelete = () => {
  confirmingDelete.value = null;
};
const confirmDelete = (id: string) => {
  history.remove(id);
  confirmingDelete.value = null;
  toasts.push({ variant: 'info', title: 'Chat deleted' });
};

const { highlighted, selected, onKeydown } = usePickerNav<SavedChat>(
  filtered,
  choose,
);

const detail = computed(() => selected.value);

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    search.value = '';
    confirmingDelete.value = null;
    highlighted.value = 0;
  },
);
</script>

<template>
  <RunicModal
    :model-value="modelValue"
    title="Select a chat"
    subtitle="Resume a previous conversation, or start a new one. Each chat keeps its own model and build context."
    kbd="Ctrl+H"
    max-width="panes"
    @update:model-value="(v) => emit('update:modelValue', v)"
  >
    <PickerLayout>
      <template #sidebar>
        <div class="chat-picker__new-row">
          <RunicButton variant="secondary" no-runes icon="plus" kbd="⌘N" @click="newChat">
            Start a new chat
          </RunicButton>
        </div>

        <PickerSearchInput
          v-model="search"
          placeholder="Search chats by title…"
          :count="filtered.length"
          @keydown="onKeydown"
        />

        <p
          v-if="!filtered.length && !sortedChats.length"
          class="chat-picker__hint"
        >
          No saved chats yet — your conversations are auto-saved as you talk.
        </p>
        <p v-else-if="!filtered.length" class="chat-picker__hint">
          No chat matches "{{ search }}".
        </p>

        <PickerListItem
          v-for="(c, idx) in filtered"
          :key="c.id"
          :active="isCurrent(c)"
          :highlighted="idx === highlighted"
          active-label="OPEN"
          @click="choose(c)"
          @mouseenter="highlighted = idx"
        >
          <template #name>{{ c.title }}</template>
          <template #meta>
            <span>{{ fmtRelative(c.updated_at) }}</span>
            <span class="chat-picker__sep">·</span>
            <span>{{ messageCount(c) }} message{{ messageCount(c) === 1 ? '' : 's' }}</span>
            <template v-if="c.attached_build_path">
              <span class="chat-picker__sep">·</span>
              <span class="chat-picker__build">{{ fileFromPath(c.attached_build_path) }}</span>
            </template>
          </template>
        </PickerListItem>
      </template>

      <template #main>
        <div v-if="detail" class="chat-picker__detail">
          <header class="chat-picker__title-row">
            <div class="chat-picker__title-block">
              <h1 class="chat-picker__title">{{ detail.title }}</h1>
              <p class="chat-picker__subtitle">
                {{ fmtRelative(detail.updated_at) }} · {{ messageCount(detail) }} message{{ messageCount(detail) === 1 ? '' : 's' }}
                <template v-if="detail.attached_build_path">
                  · build <strong>{{ fileFromPath(detail.attached_build_path) }}</strong>
                </template>
              </p>
            </div>
            <span v-if="isCurrent(detail)" class="chat-picker__active-chip">Currently open</span>
          </header>

          <!-- LAST MESSAGE -->
          <section class="chat-picker__section">
            <PickerSectionHead>
              Last message
              <template v-if="lastMessageRole(detail)" #right>
                from {{ lastMessageRole(detail) === 'user' ? 'you' : 'Bestel' }}
              </template>
            </PickerSectionHead>
            <div v-if="lastMessageText(detail)" class="chat-picker__preview">
              {{ lastMessageText(detail) }}
            </div>
            <p v-else class="chat-picker__preview chat-picker__preview--empty">
              No message recorded yet.
            </p>
          </section>

          <!-- META GRID -->
          <section class="chat-picker__meta-grid">
            <div class="chat-picker__meta-col">
              <PickerSectionHead>Conversation</PickerSectionHead>
              <div class="leader-row">
                <span class="leader-row__k">started</span>
                <span class="leader-row__dots" />
                <span class="leader-row__v">{{ fmtAbsolute(detail.created_at) }}</span>
              </div>
              <div class="leader-row">
                <span class="leader-row__k">last activity</span>
                <span class="leader-row__dots" />
                <span class="leader-row__v">{{ fmtRelative(detail.updated_at) }}</span>
              </div>
              <div class="leader-row">
                <span class="leader-row__k">messages</span>
                <span class="leader-row__dots" />
                <span class="leader-row__v">{{ messageCount(detail) }} turn{{ messageCount(detail) === 1 ? '' : 's' }}</span>
              </div>
            </div>
            <div class="chat-picker__meta-col">
              <PickerSectionHead>Context</PickerSectionHead>
              <div class="leader-row">
                <span class="leader-row__k">build attached</span>
                <span class="leader-row__dots" />
                <span
                  class="leader-row__v"
                  :style="{ color: detail.attached_build_path ? 'var(--ink)' : 'var(--ink-faint)' }"
                >
                  {{ fileFromPath(detail.attached_build_path) }}
                </span>
              </div>
              <div class="leader-row">
                <span class="leader-row__k">created</span>
                <span class="leader-row__dots" />
                <span class="leader-row__v">{{ new Date(detail.created_at).toLocaleDateString() }}</span>
              </div>
            </div>
          </section>

          <!-- DELETE row, low emphasis -->
          <div class="chat-picker__delete-row">
            <template v-if="confirmingDelete === detail.id">
              <span class="chat-picker__confirm">
                Delete this chat permanently? Its messages can't be recovered.
              </span>
              <span style="flex: 1" />
              <button type="button" class="link link--soft" @click="cancelDelete">Cancel</button>
              <RunicButton
                variant="primary"
                no-runes
                size="sm"
                icon="trash"
                danger
                @click="confirmDelete(detail.id)"
              >
                Yes, delete
              </RunicButton>
            </template>
            <template v-else>
              <span class="chat-picker__delete-prompt">Done with this conversation?</span>
              <button
                type="button"
                class="link link--danger"
                @click="requestDelete(detail.id)"
              >
                Delete chat…
              </button>
            </template>
          </div>
        </div>
        <p v-else class="chat-picker__empty-detail">
          Pick a conversation on the left to see its details — or start a new one.
        </p>
      </template>

      <template v-if="detail" #actionBar>
        <span class="chat-picker__hint">
          <template v-if="isCurrent(detail)">
            This conversation is already open.
          </template>
          <template v-else-if="detail.attached_build_path">
            Opening will load <strong>{{ fileFromPath(detail.attached_build_path) }}</strong>.
          </template>
          <template v-else>
            Opening will detach the current build.
          </template>
        </span>
        <RunicButton variant="secondary" no-runes @click="close">Cancel</RunicButton>
        <RunicButton
          variant="primary"
          no-runes
          icon="open"
          kbd="⏎"
          :disabled="isCurrent(detail)"
          :disabled-reason="isCurrent(detail) ? 'Already open' : undefined"
          @click="choose(detail)"
        >
          Open chat
        </RunicButton>
      </template>

      <template #footer>
        <span class="chat-picker__hint-row"><span class="chat-picker__kbd">↑↓</span><span>navigate</span></span>
        <span class="chat-picker__hint-row"><span class="chat-picker__kbd">⏎</span><span>open</span></span>
        <span class="chat-picker__hint-row"><span class="chat-picker__kbd">esc</span><span>close</span></span>
        <span style="flex: 1" />
        <span>{{ sortedChats.length }} saved</span>
      </template>
    </PickerLayout>
  </RunicModal>
</template>

<style scoped>
.chat-picker__hint,
.chat-picker__empty-detail {
  margin: 1rem;
  text-align: center;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-faint);
}

.chat-picker__new-row {
  padding: 12px 14px;
  border-bottom: 1px solid var(--paper-line);
  display: flex;
}
.chat-picker__new-row :deep(.runic-button) {
  width: 100%;
  justify-content: center;
}

.chat-picker__sep { color: var(--ink-faint); margin: 0 4px; }
.chat-picker__build { color: var(--amber); }

.chat-picker__detail {
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.chat-picker__title-row {
  display: flex;
  align-items: flex-start;
  gap: 16px;
}
.chat-picker__title-block {
  flex: 1;
  min-width: 0;
}
.chat-picker__title {
  margin: 0;
  font-family: var(--hand);
  font-size: var(--fs-h1);
  font-weight: var(--fw-bold);
  line-height: 1.2;
  color: var(--ink);
}
.chat-picker__subtitle {
  margin: 6px 0 0;
  font-family: var(--hand);
  font-size: var(--fs-body);
  color: var(--ink-soft);
  font-weight: var(--fw-regular);
}
.chat-picker__active-chip {
  font-family: var(--label);
  font-size: var(--fs-micro);
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--amber);
  font-weight: var(--fw-bold);
  padding: 4px 10px;
  border: 1.4px solid var(--amber);
  border-radius: 3px;
  background: var(--amber-glow);
  flex: none;
  white-space: nowrap;
}

.chat-picker__section {
  display: flex;
  flex-direction: column;
}

.chat-picker__preview {
  padding: 14px 18px;
  background: var(--paper-shade);
  border-left: 3px solid var(--paper-line);
  border-radius: 0 4px 4px 0;
  font-family: var(--hand);
  font-size: var(--fs-body);
  color: var(--ink);
  line-height: 1.6;
  max-width: 720px;
}
.chat-picker__preview--empty {
  color: var(--ink-faint);
}

.chat-picker__meta-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 4px 32px;
  max-width: 720px;
}
.chat-picker__meta-col {
  display: flex;
  flex-direction: column;
}

.chat-picker__delete-row {
  margin-top: 6px;
  padding-top: 16px;
  border-top: 1px dotted var(--paper-line);
  display: flex;
  align-items: center;
  gap: 12px;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
  max-width: 720px;
}
.chat-picker__delete-prompt {
  flex: 1;
}
.chat-picker__confirm {
  color: var(--ink);
  font-size: var(--fs-meta);
}
.chat-picker__danger-btn {
  padding: 8px 14px;
  background: var(--bad);
  color: var(--paper);
  border: 1.5px solid var(--bad);
  border-radius: 4px 6px 5px 7px / 6px 5px 7px 4px;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  font-weight: var(--fw-semibold);
  cursor: pointer;
}
.chat-picker__danger-btn:hover {
  filter: brightness(1.05);
}

.chat-picker__hint {
  flex: 1;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}
.chat-picker__hint-row {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.chat-picker__kbd {
  font-family: var(--label);
  font-size: var(--fs-micro);
  padding: 1px 5px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  color: var(--ink-soft);
}
</style>

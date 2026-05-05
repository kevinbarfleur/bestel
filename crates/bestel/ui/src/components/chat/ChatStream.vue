<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import { useChatStore } from '../../stores/chat';
import ChatMessage from './ChatMessage.vue';

const chat = useChatStore();
const { messages } = storeToRefs(chat);

const scrollHost = ref<HTMLElement | null>(null);
const stickToBottom = ref(true);

const isAtBottom = (): boolean => {
  const el = scrollHost.value;
  if (!el) return true;
  return el.scrollHeight - el.scrollTop - el.clientHeight < 48;
};

const scrollToBottom = (smooth = false) => {
  const el = scrollHost.value;
  if (!el) return;
  el.scrollTo({ top: el.scrollHeight, behavior: smooth ? 'smooth' : 'auto' });
};

const onScroll = () => {
  stickToBottom.value = isAtBottom();
};

watch(
  messages,
  async () => {
    if (!stickToBottom.value) return;
    await nextTick();
    scrollToBottom();
  },
  { deep: true },
);

onMounted(() => scrollToBottom());

const turnCount = computed(() => messages.value.length);
</script>

<template>
  <div class="chat-wrap">
    <!-- Thread header — almanach grammar -->
    <div class="chat-thread">
      <span class="chat-thread__label">thread</span>
      <span class="chat-thread__title">conversation</span>
      <span v-if="turnCount" class="chat-thread__meta">· {{ turnCount }} turns</span>
      <span class="chat-thread__grow" />
      <button type="button" class="chat-thread__action">share</button>
      <button type="button" class="chat-thread__action">archive</button>
    </div>

    <div ref="scrollHost" class="chat-stream runic-scrollbar" @scroll.passive="onScroll">
      <div v-if="messages.length === 0" class="chat-empty">
        <span class="chat-empty__rune">◆</span>
        <p class="chat-empty__title">Bestel awaits your question.</p>
        <p class="chat-empty__hint">
          Ask him to review your build, suggest a synergy, or explain a mechanic.
        </p>
        <div class="chat-empty__suggestions">
          <span class="chip">analyze my defense</span>
          <span class="chip">how do I raise my ehp?</span>
          <span class="chip">which uniques could help?</span>
        </div>
      </div>

      <ChatMessage v-for="m in messages" :key="m.id" :message="m" />
    </div>
  </div>
</template>

<style scoped>
.chat-wrap {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--paper);
}

/* Thread header — small caps + Garamond title + script italic meta */
.chat-thread {
  height: 36px;
  padding: 0 24px;
  border-bottom: 1px solid var(--paper-line);
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: nowrap;
  white-space: nowrap;
  flex: none;
}
.chat-thread__label {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 500;
}
.chat-thread__title {
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 500;
  color: var(--ink);
}
.chat-thread__meta {
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink-faint);
}
.chat-thread__grow { flex: 1; }
.chat-thread__action {
  font-family: var(--hand);
  font-size: 11px;
  color: var(--ink-faint);
  cursor: pointer;
  background: none;
  border: 0;
  padding: 0;
  transition: color 0.15s ease;
}
.chat-thread__action:hover { color: var(--amber); }

.chat-stream {
  flex: 1;
  overflow-y: auto;
  padding: 24px 28px;
  display: flex;
  flex-direction: column;
  gap: 18px;
  min-height: 0;
  width: 100%;
}

.chat-empty {
  margin: auto;
  text-align: center;
  padding: 2.5rem 1.5rem;
  max-width: 30rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}
.chat-empty__rune {
  font-size: 1.8rem;
  color: var(--amber);
  opacity: 0.65;
}
.chat-empty__title {
  margin: 0;
  font-family: var(--hand-display);
  font-size: 22px;
  font-weight: 700;
  color: var(--ink);
  letter-spacing: 0.02em;
}
.chat-empty__hint {
  margin: 0;
  max-width: 22rem;
  text-align: center;
  font-family: var(--hand);
  font-size: 13px;
  color: var(--ink-faint);
}
.chat-empty__suggestions {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 6px;
  margin-top: 8px;
}
</style>

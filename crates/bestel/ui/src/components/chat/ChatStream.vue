<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import { useChatStore } from '../../stores/chat';
import { useChatHistoryStore } from '../../stores/chatHistory';
import { useToastsStore } from '../../stores/toasts';
import { chatLogDir } from '../../api/tauri';
import ChatMessage from './ChatMessage.vue';

const chat = useChatStore();
const chatHistory = useChatHistoryStore();
const toasts = useToastsStore();
const { messages } = storeToRefs(chat);

/** Copy the entire current conversation as JSON to the clipboard. Mirrors
 *  what the autosave persists to `~/.bestel/runtime/conversation-logs/{id}.json`
 *  so the user can paste it directly to a coding agent for debugging. */
async function copyDebugSnapshot() {
  const active = chatHistory.findActive();
  const payload = active ?? {
    id: 'unsaved',
    title: 'Unsaved conversation',
    messages: messages.value,
    attached_build_path: null,
    created_at: Date.now(),
    updated_at: Date.now(),
  };
  const json = JSON.stringify(payload, null, 2);
  let logDir: string | null = null;
  try {
    logDir = await chatLogDir();
  } catch {
    /* ignore */
  }
  try {
    await navigator.clipboard.writeText(json);
    toasts.push({
      variant: 'info',
      title: 'Conversation copied',
      body: logDir
        ? `Latest log file is at ${logDir}\\${payload.id}.json`
        : 'Paste it into your coding agent for debugging.',
    });
  } catch (e) {
    toasts.push({
      variant: 'error',
      title: 'Copy failed',
      body: String(e),
    });
  }
}

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
      <span class="chat-thread__build-tag" title="Build marker — confirms the running binary contains the latest layout fixes">UX-2.6</span>
      <button type="button" class="chat-thread__action">share</button>
      <button type="button" class="chat-thread__action">archive</button>
      <button
        type="button"
        class="chat-thread__action chat-thread__action--debug"
        title="Copy the full conversation as JSON for debugging"
        @click="copyDebugSnapshot"
      >
        <span aria-hidden="true">⟐</span> debug
      </button>
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
  min-width: 0;
  /* CSS containment: isolate inner layout from outer width context. Same
   * defense-in-depth pattern as `.chat-view__main` — a nested element
   * that requests an intrinsic min-content larger than the chat column
   * cannot push this box wider than its parent's flex slot. */
  contain: inline-size;
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
  font-size: var(--fs-caps);
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: var(--fw-semibold);
}
.chat-thread__title {
  font-family: var(--hand);
  font-size: var(--fs-body);
  font-weight: var(--fw-medium);
  color: var(--ink);
}
.chat-thread__meta {
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}
.chat-thread__grow { flex: 1; }
.chat-thread__action {
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
  cursor: pointer;
  background: none;
  border: 0;
  padding: 0;
  transition: color 0.15s ease;
}
.chat-thread__action:hover { color: var(--amber); }

/* Build marker — small amber tag in the header so the dev can verify
 * at a glance which build the running binary corresponds to. Tagged as
 * `UX-2.6` for the layout-containment pass; bump the literal in the
 * template each time we ship a CSS-only chat-layout patch so a visual
 * comparison against an old screenshot is trivial. */
.chat-thread__build-tag {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--paper);
  background: var(--amber);
  padding: 2px 6px;
  border-radius: 3px;
  font-weight: 700;
}

/* Layout chain for chat content (window resize behavior).
 *
 *   .app-shell (100vw)
 *     └── .app-body (flex)
 *         └── .chat-view (flex: 1)
 *             ├── .chat-view__sidebar    (fixed 360px — intentional)
 *             ├── .chat-view__main       (flex: 1, min-width: 0, overflow: hidden)
 *             │   └── .chat-wrap         (flex column)
 *             │       └── .chat-stream   (THIS — flex: 1, overflow-x: hidden)
 *             │           └── article.turn (flex row, min-width: 0, max-width: 100%)
 *             │               ├── .turn__gutter (fixed 72px — labels)
 *             │               └── .turn__body (flex: 1, min-width: 0)
 *             │                   └── any artifact / ToolCallBadge / BSInterviewPanel
 *             └── .chat-view__panel      (fixed 380px when open — intentional)
 *
 * The 360 + 380 widths are the only HARD sizes. Everything else is
 * `flex: 1` + `min-width: 0` so the chat content shrinks and grows
 * with the window. Children that risk content overflow (long tool
 * detail strings, long chip labels) are clipped via `overflow: hidden`
 * at the chat-stream level + ellipsis at the leaf level.
 *
 * The `.chat-stream > *` rule below is the final safety net: any
 * direct descendant that forgets `min-width: 0` is force-constrained
 * to the chat column. This means we never have to chase per-component
 * width regressions when adding a new artifact / panel kind. */
.chat-stream {
  flex: 1;
  overflow-x: hidden;
  overflow-y: auto;
  padding: 24px 32px 20px;
  display: flex;
  flex-direction: column;
  gap: 22px;
  min-height: 0;
  min-width: 0;
  width: 100%;
  box-sizing: border-box;
}
.chat-stream > * {
  /* Force every direct child to respect the chat column width. Combined
   * with `overflow-x: hidden` on `.chat-stream` itself, this means a
   * child that fails to define its own `min-width: 0` (or contains
   * non-shrinkable inline content) gets clipped instead of breaking the
   * right rail. The `max-width` is the visible-width clamp; the
   * `min-width: 0` is what actually allows flex shrinking inside. */
  min-width: 0;
  max-width: 100%;
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

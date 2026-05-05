<script setup lang="ts">
import { computed } from 'vue';

import { openExternal } from '../../api/tauri';
import { renderMarkdown } from '../../api/markdown';
import { useBuildStore } from '../../stores/build';
import type { ChatMessageVm, ReasoningSegment, TextSegment, ToolSegment } from '../../stores/chat';
import ToolCallBadge from './ToolCallBadge.vue';
import ArtThinking from './artifacts/ArtThinking.vue';
import ArtPoBImport from './artifacts/ArtPoBImport.vue';
import ArtWikiPage from './artifacts/ArtWikiPage.vue';
import AttachmentChip from './artifacts/AttachmentChip.vue';

const props = defineProps<{ message: ChatMessageVm }>();

const buildStore = useBuildStore();
const game = computed(() => buildStore.current?.game ?? 'poe1');

const renderText = (seg: TextSegment) => renderMarkdown(seg.text, game.value);

const handleClick = (e: MouseEvent) => {
  const target = e.target as HTMLElement | null;
  if (!target) return;
  const anchor = target.closest('a') as HTMLAnchorElement | null;
  if (!anchor) return;
  const href = anchor.dataset.wikiUrl ?? anchor.getAttribute('href');
  if (!href) return;
  e.preventDefault();
  void openExternal(href);
};

const isAssistant = computed(() => props.message.role === 'assistant');
const isStreaming = computed(() => props.message.status === 'streaming');

const userText = computed(() => {
  if (isAssistant.value) return '';
  const seg = props.message.segments.find((s): s is TextSegment => s.kind === 'text');
  return seg?.text ?? '';
});

const isThinkingOnly = computed(
  () => isAssistant.value && isStreaming.value && props.message.segments.length === 0,
);

interface Turn {
  key: string;
  label: string;
  color: string;
  kind: 'text' | 'reasoning' | 'tool-pob' | 'tool-wiki-page' | 'tool-generic' | 'placeholder';
  segment: TextSegment | ReasoningSegment | ToolSegment | null;
  isLast: boolean;
}

const TOOL_KIND_LABEL: Record<string, { label: string; color: string }> = {
  // Build context
  get_active_build: { label: 'import', color: 'var(--el-lit)' },
  // Wiki tools
  wiki_search: { label: 'search', color: 'var(--ink-soft)' },
  wiki_parse: { label: 'wiki', color: 'var(--el-cold)' },
  wiki_synergies: { label: 'sweep', color: 'var(--good)' },
  wiki_cargo: { label: 'cargo', color: 'var(--ink-soft)' },
  // Legacy alias for the renamed wiki_synergies tool
  find_synergies: { label: 'sweep', color: 'var(--good)' },
  // Trade tools
  trade_resolve_stats: { label: 'trade', color: 'var(--ink-soft)' },
  trade_search_url: { label: 'trades', color: 'var(--ink-soft)' },
  trade_search: { label: 'trades', color: 'var(--ink-soft)' },
  // Generic web fetch (allowlisted)
  web_fetch: { label: 'fetch', color: 'var(--ink-soft)' },
  web_search: { label: 'search', color: 'var(--ink-soft)' },
};

function toolLabel(name: string): { label: string; color: string } {
  return TOOL_KIND_LABEL[name] ?? { label: 'tool', color: 'var(--ink-soft)' };
}

function toolKind(name: string): 'tool-pob' | 'tool-wiki-page' | 'tool-generic' {
  if (name === 'get_active_build') return 'tool-pob';
  if (name === 'wiki_parse') return 'tool-wiki-page';
  return 'tool-generic';
}

const turns = computed<Turn[]>(() => {
  if (!isAssistant.value) return [];
  const segs = props.message.segments;
  const out: Turn[] = [];
  segs.forEach((seg, idx) => {
    const isLast = idx === segs.length - 1;
    if (seg.kind === 'text') {
      out.push({
        key: seg.id,
        label: 'bestel',
        color: 'var(--ink)',
        kind: 'text',
        segment: seg,
        isLast,
      });
    } else if (seg.kind === 'reasoning') {
      out.push({
        key: seg.id,
        label: 'thinking',
        color: 'var(--el-chaos)',
        kind: 'reasoning',
        segment: seg,
        isLast,
      });
    } else {
      const meta = toolLabel(seg.name);
      out.push({
        key: seg.id,
        label: meta.label,
        color: meta.color,
        kind: toolKind(seg.name),
        segment: seg,
        isLast,
      });
    }
  });
  if (isThinkingOnly.value) {
    out.push({
      key: 'placeholder',
      label: 'bestel',
      color: 'var(--ink)',
      kind: 'placeholder',
      segment: null,
      isLast: true,
    });
  }
  return out;
});
</script>

<template>
  <!-- USER turn — small caps "you" + Kalam italic body -->
  <article v-if="!isAssistant" class="turn">
    <div class="turn__gutter">
      <span class="turn__label" style="color: var(--amber)">you</span>
    </div>
    <div class="turn__body">
      <div v-if="message.attachments.length" class="turn__attachments">
        <AttachmentChip
          v-for="(att, idx) in message.attachments"
          :key="idx"
          :attachment="att"
          :removable="false"
        />
      </div>
      <p v-if="userText" class="turn__user-text">{{ userText }}</p>
    </div>
  </article>

  <!-- ASSISTANT turns — one turn per segment, hairline divider before -->
  <template v-else>
    <hr class="sk-hr turn-divider" />
    <article
      v-for="(t, i) in turns"
      :key="t.key"
      class="turn"
      :class="[`turn--${t.kind}`, { 'turn--first-asst': i === 0 }]"
    >
      <div class="turn__gutter">
        <span class="turn__label" :style="{ color: t.color }">{{ t.label }}</span>
      </div>
      <div class="turn__body">
        <!-- TEXT segment → markdown body in Garamond -->
        <template v-if="t.kind === 'text' && t.segment">
          <div
            class="markdown-body"
            v-html="renderText(t.segment as TextSegment)"
            @click="handleClick"
          />
          <span
            v-if="isStreaming && t.isLast"
            class="streaming-cursor"
            aria-hidden="true"
          />
        </template>

        <!-- REASONING segment → ArtThinking marginalia -->
        <ArtThinking
          v-else-if="t.kind === 'reasoning' && t.segment"
          :segment="t.segment as ReasoningSegment"
        />

        <!-- TOOL get_active_build → ArtPoBImport -->
        <ArtPoBImport
          v-else-if="t.kind === 'tool-pob' && t.segment"
          :segment="t.segment as ToolSegment"
        />

        <!-- TOOL wiki_parse → ArtWikiPage (title, sections, excerpt) -->
        <ArtWikiPage
          v-else-if="t.kind === 'tool-wiki-page' && t.segment"
          :segment="t.segment as ToolSegment"
        />

        <!-- Other tools → slim ToolCallBadge -->
        <ToolCallBadge
          v-else-if="t.kind === 'tool-generic' && t.segment"
          :segment="t.segment as ToolSegment"
        />

        <!-- Placeholder while waiting for first segment -->
        <span v-else-if="t.kind === 'placeholder'" class="streaming-cursor" aria-hidden="true" />
      </div>
    </article>

    <p v-if="message.status === 'error' && message.errorMessage" class="turn-error">
      {{ message.errorMessage }}
    </p>
  </template>
</template>

<style scoped>
/* Turn — gutter (72px label) + 14px gap + body flex 1.
 * Baseline alignment so the gutter label sits on the same baseline as the
 * first body line (avoids the TOOL/web misalignment seen in screenshots). */
.turn {
  display: flex;
  gap: 14px;
  align-items: baseline;
}
.turn__gutter {
  width: 72px;
  flex: none;
  text-align: right;
}
.turn__label {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  font-weight: 600;
  white-space: nowrap;
}
.turn__body {
  flex: 1;
  min-width: 0;
}

/* User text — Kalam italic 16/1.5 */
.turn__user-text {
  margin: 0;
  font-family: var(--script);
  font-size: 16px;
  line-height: 1.5;
  color: var(--ink);
  font-style: italic;
}
.turn__attachments {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 8px;
}

/* Hairline between user and assistant */
.turn-divider {
  margin: 4px 0;
  border-top: 1px solid var(--paper-line);
}

/* Error message */
.turn-error {
  margin: 0 0 0 86px;
  padding: 0.5rem 0.7rem;
  border-left: 2px solid var(--bad);
  font-family: var(--hand);
  font-size: 13px;
  color: var(--bad);
  background: transparent;
}
</style>

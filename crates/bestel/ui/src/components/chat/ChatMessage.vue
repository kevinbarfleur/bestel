<script setup lang="ts">
import { computed } from 'vue';

import { openLink } from '../../api/tauri';
import { renderMarkdown } from '../../api/markdown';
import { useBuildStore } from '../../stores/build';
import type { ChatMessageVm, ReasoningSegment, TextSegment, ToolSegment } from '../../stores/chat';
import ToolCallBadge from './ToolCallBadge.vue';
import ArtThinking from './artifacts/ArtThinking.vue';
import ArtPoBImport from './artifacts/ArtPoBImport.vue';
import ArtWikiPage from './artifacts/ArtWikiPage.vue';
import AttachmentChip from './artifacts/AttachmentChip.vue';
import { useUiStore, type PanelArtifactType } from '../../stores/ui';

const props = defineProps<{ message: ChatMessageVm }>();

const buildStore = useBuildStore();
const ui = useUiStore();
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
  void openLink(href);
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
  kind:
    | 'text'
    | 'reasoning'
    | 'tool-pob'
    | 'tool-wiki-page'
    | 'tool-panel'
    | 'tool-generic'
    | 'placeholder';
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
  // Right adaptive panel promotion
  show_in_panel: { label: 'highlight', color: 'var(--ink)' },
};

function toolLabel(name: string): { label: string; color: string } {
  return TOOL_KIND_LABEL[name] ?? { label: 'tool', color: 'var(--ink-soft)' };
}

function toolKind(
  name: string,
): 'tool-pob' | 'tool-wiki-page' | 'tool-panel' | 'tool-generic' {
  if (name === 'get_active_build') return 'tool-pob';
  if (name === 'wiki_parse') return 'tool-wiki-page';
  if (name === 'show_in_panel') return 'tool-panel';
  return 'tool-generic';
}

interface PanelHint {
  type: PanelArtifactType;
  title: string;
  payload: unknown;
}

/** Parse a show_in_panel tool segment's output JSON. Tolerant — returns
 *  null if the output is incomplete (still streaming) or malformed. */
function parsePanelHint(seg: ToolSegment): PanelHint | null {
  if (!seg.output) return null;
  try {
    const obj = JSON.parse(seg.output);
    if (
      obj &&
      typeof obj === 'object' &&
      typeof obj.type === 'string' &&
      typeof obj.title === 'string' &&
      'payload' in obj
    ) {
      return {
        type: obj.type as PanelArtifactType,
        title: obj.title,
        payload: obj.payload,
      };
    }
  } catch {
    /* incomplete during stream */
  }
  return null;
}

function reopenInPanel(seg: ToolSegment) {
  const hint = parsePanelHint(seg);
  if (!hint) return;
  ui.openPanel({
    id: seg.id,
    type: hint.type,
    title: hint.title,
    payload: hint.payload,
    source: 'click',
  });
}

const fmtTokens = (n: number): string => {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
  return `${n}`;
};

const fmtCost = (n: number): string => {
  if (n < 0.001) return `<$0.001`;
  if (n < 0.01) return `$${n.toFixed(4)}`;
  if (n < 1) return `$${n.toFixed(3)}`;
  return `$${n.toFixed(2)}`;
};

const usageLine = computed(() => {
  const u = props.message.usage;
  if (!u) return null;
  const totalIn = u.input_tokens + u.cached_input_tokens;
  if (totalIn === 0 && u.output_tokens === 0) return null;
  const parts: string[] = [];
  if (u.cost_usd !== null && u.cost_usd !== undefined) parts.push(`~${fmtCost(u.cost_usd)}`);
  if (u.cached_input_tokens > 0 && totalIn > 0) {
    const pct = Math.round((u.cached_input_tokens / totalIn) * 100);
    parts.push(`${pct}% cached`);
  }
  parts.push(`${fmtTokens(totalIn)} in / ${fmtTokens(u.output_tokens)} out`);
  return parts.join(' · ');
});

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

        <!-- TOOL show_in_panel → pure pill, no icon. The visual difference
             with web-link pills (which carry an external-link icon) is the
             whole UX language: pill + icon = web, pill alone = side panel. -->
        <button
          v-else-if="t.kind === 'tool-panel' && t.segment"
          type="button"
          class="turn__panel-hint"
          :title="`Open ${parsePanelHint(t.segment as ToolSegment)?.title ?? 'in side panel'}`"
          :disabled="!parsePanelHint(t.segment as ToolSegment)"
          @click="reopenInPanel(t.segment as ToolSegment)"
        >
          {{ parsePanelHint(t.segment as ToolSegment)?.title ?? 'highlighted' }}
        </button>

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

    <p v-if="usageLine" class="turn-usage">
      {{ usageLine }}
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
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  font-weight: var(--fw-semibold);
  white-space: nowrap;
}
.turn__body {
  flex: 1;
  min-width: 0;
}

/* User text — Kalam italic 17/1.5 */
.turn__user-text {
  margin: 0;
  font-family: var(--script);
  font-size: 17px;
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

/* show_in_panel inline hint — *exactly* the markdown pill style minus the
 * external-link icon. Pill alone = "this opens the right side panel".
 * Hover bumps fill, never shifts layout. */
.turn__panel-hint {
  display: inline-flex;
  align-items: baseline;
  padding: 0px 7px;
  border-radius: 3px;
  background: rgba(0, 0, 0, 0.04);
  border: 1px solid var(--ink-ghost);
  color: var(--ink);
  font-family: var(--hand);
  font-size: 0.95em;
  font-weight: 500;
  line-height: inherit;
  white-space: nowrap;
  vertical-align: baseline;
  cursor: pointer;
  transition: background 0.15s ease, border-color 0.15s ease;
}
.theme-dark .turn__panel-hint {
  background: rgba(255, 255, 255, 0.04);
}
.turn__panel-hint:hover {
  background: rgba(0, 0, 0, 0.08);
  border-color: var(--ink-soft);
}
.theme-dark .turn__panel-hint:hover {
  background: rgba(255, 255, 255, 0.08);
}
.turn__panel-hint:disabled {
  cursor: default;
  opacity: 0.55;
}

/* Error message */
.turn-error {
  margin: 0 0 0 86px;
  padding: 0.5rem 0.7rem;
  border-left: 2px solid var(--bad);
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--bad);
  background: transparent;
}

/* Usage telemetry footer — small caps caption below the assistant turn */
.turn-usage {
  margin: 8px 0 0 86px;
  font-family: var(--label);
  font-size: var(--fs-caps);
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: var(--fw-medium);
}
</style>

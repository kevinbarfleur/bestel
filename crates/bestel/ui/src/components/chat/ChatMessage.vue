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
import { useUiStore } from '../../stores/ui';

const props = defineProps<{ message: ChatMessageVm }>();

const buildStore = useBuildStore();
const ui = useUiStore();
const game = computed(() => buildStore.current?.game ?? 'poe1');

const renderText = (seg: TextSegment) => renderMarkdown(seg.text, game.value);

const handleClick = (e: MouseEvent) => {
  const target = e.target as HTMLElement | null;
  if (!target) return;
  // Side-panel button (loupe icon) — opens the matching artifact from the
  // segment's panelMap. Resolved by walking the message's text segments
  // until one carries the data-panel-key.
  const panelBtn = target.closest('button.panel-btn') as HTMLButtonElement | null;
  if (panelBtn) {
    const key = panelBtn.dataset.panelKey;
    if (!key) return;
    e.preventDefault();
    for (const seg of props.message.segments) {
      if (seg.kind !== 'text') continue;
      const map = (seg as TextSegment).panelMap;
      const artifact = map?.[key];
      if (artifact) {
        ui.openPanel({ ...artifact, source: 'click' });
        return;
      }
    }
    return;
  }
  // External / wiki link.
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
  // Legacy show_in_panel calls survive in old saved chats — render as a
  // faded ToolCallBadge so the timeline doesn't break. New panel buttons
  // live in assistant text via ⟦panel:…⟧ markers (see api/markdown.ts).
  show_in_panel: { label: 'highlight (legacy)', color: 'var(--ink-faint)' },
};

function toolLabel(name: string): { label: string; color: string } {
  return TOOL_KIND_LABEL[name] ?? { label: 'tool', color: 'var(--ink-soft)' };
}

function toolKind(
  name: string,
): 'tool-pob' | 'tool-wiki-page' | 'tool-generic' {
  if (name === 'get_active_build') return 'tool-pob';
  if (name === 'wiki_parse') return 'tool-wiki-page';
  return 'tool-generic';
}

/** True for any turn that should sit inside the indented artifact stack
 *  (vertical line on the left). Excludes plain text turns from Bestel
 *  and the streaming-cursor placeholder. */
function isArtifactKind(k: Turn['kind']): boolean {
  return (
    k === 'reasoning' ||
    k === 'tool-pob' ||
    k === 'tool-wiki-page' ||
    k === 'tool-generic'
  );
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
      :class="[
        `turn--${t.kind}`,
        {
          'turn--first-asst': i === 0,
          'turn--artifact': isArtifactKind(t.kind),
        },
      ]"
    >
      <div class="turn__gutter">
        <span class="turn__label" :style="{ color: t.color }">{{ t.label }}</span>
      </div>
      <div class="turn__body">
        <!-- TEXT segment → markdown body in Garamond -->
        <template v-if="t.kind === 'text' && t.segment">
          <div
            class="markdown-body"
            :class="{ 'markdown-body--streaming': isStreaming && t.isLast }"
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

/* Artifact stack — reasoning + tool turns sit inside an indented
 * sub-block grouped by a single thin vertical line on the left. Same
 * affordance as Slack thread replies or GitHub conversation nesting:
 * the line says "these belong together as the work that produced the
 * answer below". The bestel text turns above and below are NOT
 * indented; the line begins at the first artifact's top and ends at
 * the last one's bottom.
 *
 * The line is drawn by an absolutely-positioned `::before` on the
 * article itself, anchored at `left: 86px` — the column where the
 * body sits in the article's inner flex layout (72px gutter +
 * 14px `.turn` gap). Going through ::before instead of `border-left`
 * has two benefits:
 *   - Labels (`thinking`, `wiki`, `search`) keep their original
 *     gutter position aligned with `bestel` above and below — the
 *     article's box is unchanged.
 *   - The line spans `top: 0` to `height: 100%` of the article's
 *     padding box, so it covers the full article height regardless
 *     of where the inner flex baseline-alignment places the body.
 *
 * Between two adjacent artifact turns, the parent flex `gap: 22px` is
 * canceled with `margin-top: -22px` and replaced by `padding-top: 22px`
 * on the article. The ::before line covers the padding region too,
 * so consecutive articles sit flush and the line is uninterrupted
 * across the visible 22px spacing. Keep the 22px value in sync with
 * `.chat-stream`'s gap (ChatStream.vue) and the 86px in sync with
 * the gutter width (72px) + .turn gap (14px). */
.turn--artifact {
  position: relative;
}
.turn--artifact::before {
  content: '';
  position: absolute;
  left: 86px;
  top: 0;
  height: 100%;
  width: 1px;
  background: var(--paper-line);
}
.turn--artifact .turn__body {
  padding-left: 14px;
}
.turn--artifact + .turn--artifact {
  margin-top: -22px;
  padding-top: 22px;
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

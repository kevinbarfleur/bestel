<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';

import { openLink } from '../../api/tauri';
import { renderMarkdown } from '../../api/markdown';
import { useBuildStore } from '../../stores/build';
import { useChatStore } from '../../stores/chat';
import type {
  ChatMessageVm,
  ReasoningSegment,
  SheetAskSegment,
  SheetDraftSegment,
  SheetFinalizedSegment,
  SheetInterviewSegment,
  TextSegment,
  ToolSegment,
  VerifyClaimsSegment,
} from '../../stores/chat';
import { useSheetStore } from '../../stores/sheet';
import ToolCallBadge from './ToolCallBadge.vue';
import VerifyClaimsCard from './VerifyClaimsCard.vue';
import ArtThinking from './artifacts/ArtThinking.vue';
import ArtPoBImport from './artifacts/ArtPoBImport.vue';
import ArtWikiPage from './artifacts/ArtWikiPage.vue';
import ArtInterviewSubmission from './artifacts/ArtInterviewSubmission.vue';
import AttachmentChip from './artifacts/AttachmentChip.vue';
import BSDraftedCard from '../build-sheet/BSDraftedCard.vue';
import BSAskCard from '../build-sheet/BSAskCard.vue';
import BSInterviewPanel from '../build-sheet/BSInterviewPanel.vue';
import BSSheetSavedBanner from '../build-sheet/BSSheetSavedBanner.vue';
import { useUiStore } from '../../stores/ui';

/**
 * Threshold for the inter-segment thinking indicator. When the assistant
 * is streaming and the gap since the last delta exceeds this, the turn
 * list grows a placeholder turn so the user sees Bestel is still working
 * (between the last `tool_end` and the next `tool_begin` or final text,
 * the wire is silent — without this the UI looks frozen). Tune higher to
 * surface the spinner only during real stalls.
 */
const THINKING_GAP_MS = 800;
const THINKING_TICK_MS = 250;

const props = defineProps<{ message: ChatMessageVm }>();

const buildStore = useBuildStore();
const ui = useUiStore();
const chat = useChatStore();
const sheet = useSheetStore();
const game = computed(() => buildStore.current?.game ?? 'poe1');

/** Confirm a drafted section. Updates the local sheet store optimistically
 *  so the card switches to its `confirmed` style immediately, then sends a
 *  short user message back to the agent so the next turn knows the user
 *  approved the draft and the interview can advance to the next section
 *  (or to `sheet_finalize_request` if this was the last one). */
function onConfirmDraft(seg: SheetDraftSegment) {
  if (seg.confirmed) return;
  sheet.confirmSection(seg.sectionId);
  void chat.send(`Confirmed: ${seg.title} section is correct as drafted.`);
}

/** Edit a drafted section. Opens a native prompt with the current body so
 *  the user can revise it inline; the new text is sent back to the agent
 *  as a corrected user message. The native prompt is intentionally minimal
 *  — a richer inline editor can land later, but the click handlers are
 *  what currently block the interview from advancing. */
function onEditDraft(seg: SheetDraftSegment) {
  if (seg.confirmed) return;
  const next = window.prompt(
    `Edit the ${seg.title} section. Press OK to send the corrected text to Bestel.`,
    seg.body,
  );
  if (next === null) return;
  const trimmed = next.trim();
  if (!trimmed) return;
  sheet.editSection(seg.sectionId, trimmed);
  void chat.send(`Edited: ${seg.title} section should read instead — ${trimmed}`);
}

/** Set of entity names that have a side-panel artifact in this message,
 *  collected across ALL text segments. The markdown renderer uses this
 *  to suppress the wiki backtick pill for those entities — the panel
 *  button supersedes it (its chrome carries the wiki link). */
const panelKeys = computed<Set<string>>(() => {
  const set = new Set<string>();
  for (const seg of props.message.segments) {
    if (seg.kind !== 'text') continue;
    const map = (seg as TextSegment).panelMap;
    if (!map) continue;
    for (const k of Object.keys(map)) set.add(k);
  }
  return set;
});

const renderText = (seg: TextSegment) =>
  renderMarkdown(seg.text, game.value, panelKeys.value);

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

/** Detect the structured `[INTERVIEW SUBMISSION ...]` user message
 * emitted by `useSheetStore().submitInterview()` so we can replace the
 * raw markdown body with a compact, persistent artifact card. The
 * tag is fixed and documented in `prompts/references/32_build_sheets.md`
 * § Phase 3 — any drift here is a bug to fix at the source. */
const isInterviewSubmission = computed(() =>
  !isAssistant.value && userText.value.startsWith('[INTERVIEW SUBMISSION'),
);

const isThinkingOnly = computed(
  () => isAssistant.value && isStreaming.value && props.message.segments.length === 0,
);

// Heartbeat used by `isThinkingGap` to re-evaluate the gap-since-last-
// delta computation. Only ticks while the message is actively streaming;
// stopped on completion / cancellation / error so an idle viewport
// doesn't pay the cost.
const nowTick = ref(Date.now());
let tickHandle: ReturnType<typeof setInterval> | null = null;

const startTick = () => {
  if (tickHandle !== null) return;
  tickHandle = setInterval(() => {
    nowTick.value = Date.now();
  }, THINKING_TICK_MS);
};
const stopTick = () => {
  if (tickHandle === null) return;
  clearInterval(tickHandle);
  tickHandle = null;
};

watch(
  isStreaming,
  (streaming) => {
    if (streaming) {
      nowTick.value = Date.now();
      startTick();
    } else {
      stopTick();
    }
  },
  { immediate: true },
);
onBeforeUnmount(stopTick);

/**
 * True when the assistant has streamed at least one segment and the gap
 * since the last delta exceeds `THINKING_GAP_MS`. Drives the inter-
 * segment placeholder turn so the spinner is visible during dead air
 * between tool calls or before the final text starts streaming.
 */
const isThinkingGap = computed(() => {
  if (!isAssistant.value || !isStreaming.value) return false;
  if (props.message.segments.length === 0) return false;
  const last = props.message.lastDeltaAt;
  if (typeof last !== 'number') return false;
  return nowTick.value - last > THINKING_GAP_MS;
});

interface Turn {
  key: string;
  label: string;
  color: string;
  kind:
    | 'text'
    | 'narration'
    | 'reasoning'
    | 'tool-pob'
    | 'tool-wiki-page'
    | 'tool-generic'
    | 'sheet-draft'
    | 'sheet-ask'
    | 'sheet-interview'
    | 'sheet-finalized'
    | 'verify-claims'
    | 'placeholder';
  segment:
    | TextSegment
    | ReasoningSegment
    | ToolSegment
    | SheetDraftSegment
    | SheetAskSegment
    | SheetInterviewSegment
    | SheetFinalizedSegment
    | VerifyClaimsSegment
    | null;
  isLast: boolean;
}

/** Strip side-panel marker syntax from a narration text segment so the
 *  raw `⟦panel*:item-card:Mageblood⟧` literal doesn't leak into the
 *  rendered prose. Markers are reserved for the final answer; if the
 *  model emits them mid-narration anyway, they're hidden from the
 *  narration block (the panelMap on the segment is preserved either
 *  way, and the marker fires once it shows up in the actual answer
 *  text). */
const PANEL_MARKER_LITERAL_RE = /⟦panel\*?:[a-z-]+:[^⟧]+⟧/g;
function cleanNarration(text: string): string {
  return text.replace(PANEL_MARKER_LITERAL_RE, '').trim();
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
 *  (vertical line on the left). Includes narration — text fragments the
 *  model emitted between tool calls, demoted from the bestel prose
 *  level. Excludes the final answer text and the streaming-cursor
 *  placeholder. */
function isArtifactKind(k: Turn['kind']): boolean {
  return (
    k === 'narration' ||
    k === 'reasoning' ||
    k === 'tool-pob' ||
    k === 'tool-wiki-page' ||
    k === 'tool-generic' ||
    k === 'sheet-draft' ||
    k === 'sheet-ask' ||
    k === 'verify-claims'
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

  // Find the LAST text segment — only that one gets the bestel-prose
  // treatment. Anthropic streams text deltas chronologically with
  // tool_use blocks, so when the model narrates between tool calls,
  // each fragment becomes its own text segment ("I'll analyze your
  // current ch" → tool → "est piece and recommend an upgrade…"). All
  // text segments BEFORE the last one are status updates, not the
  // answer; demote them to narration so the final prose isn't
  // diluted by sentence stumps. Evaluated dynamically — during
  // streaming, the currently-growing text is "answer" until a new
  // text segment appears after a tool, at which point it becomes
  // narration retroactively.
  let lastTextIdx = -1;
  for (let i = segs.length - 1; i >= 0; i--) {
    if (segs[i].kind === 'text') {
      lastTextIdx = i;
      break;
    }
  }

  const out: Turn[] = [];
  segs.forEach((seg, idx) => {
    const isLast = idx === segs.length - 1;
    if (seg.kind === 'text') {
      const isAnswer = idx === lastTextIdx;
      // Sprint UX-2 — suppress pre-final text segments entirely. The
      // model interleaves text with tool_use blocks; the SSE stream cuts
      // those text blocks at byte boundaries (mid-word). Showing them as
      // faded narration left long stutter trails in the timeline. Only
      // render the LAST text segment as the answer; the dead air between
      // tool calls is covered by the existing thinking-gap placeholder.
      if (!isAnswer) return;
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
    } else if (seg.kind === 'sheet_draft') {
      out.push({
        key: seg.id,
        label: 'interview',
        color: 'var(--amber)',
        kind: 'sheet-draft',
        segment: seg,
        isLast,
      });
    } else if (seg.kind === 'sheet_ask') {
      out.push({
        key: seg.id,
        label: 'interview',
        color: 'var(--amber)',
        kind: 'sheet-ask',
        segment: seg,
        isLast,
      });
    } else if (seg.kind === 'sheet_interview') {
      out.push({
        key: seg.id,
        label: 'interview',
        color: 'var(--amber)',
        kind: 'sheet-interview',
        segment: seg,
        isLast,
      });
    } else if (seg.kind === 'sheet_finalized') {
      out.push({
        key: seg.id,
        label: 'sheet',
        color: 'var(--good)',
        kind: 'sheet-finalized',
        segment: seg,
        isLast,
      });
    } else if (seg.kind === 'verify_claims') {
      out.push({
        key: seg.id,
        label: 'verify',
        color: seg.correctionsCount > 0 ? 'var(--amber)' : 'var(--good)',
        kind: 'verify-claims',
        segment: seg,
        isLast,
      });
    } else if (seg.kind === 'tool') {
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
  if (isThinkingOnly.value || isThinkingGap.value) {
    if (out.length > 0) {
      out[out.length - 1].isLast = false;
    }
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
      <ArtInterviewSubmission
        v-if="isInterviewSubmission"
        :text="userText"
      />
      <p v-else-if="userText" class="turn__user-text">{{ userText }}</p>
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

        <!-- NARRATION segment — assistant text emitted between tool
             calls. Rendered as faded italic prose so it reads as a
             status update, not part of the final answer. Panel
             markers are stripped (the panelMap on the segment is
             still used for click resolution, but the literal marker
             syntax doesn't leak into the narration view). -->
        <p
          v-else-if="t.kind === 'narration' && t.segment"
          class="turn__narration"
        >{{ cleanNarration((t.segment as TextSegment).text) }}</p>

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

        <!-- Build-Sheet drafted-section card. Rendered inline as the
             agent surfaces each section for confirm/edit. -->
        <BSDraftedCard
          v-else-if="t.kind === 'sheet-draft' && t.segment"
          :title="(t.segment as SheetDraftSegment).title"
          :status="(t.segment as SheetDraftSegment).confirmed ? 'confirmed' : 'pending'"
          @confirm="onConfirmDraft(t.segment as SheetDraftSegment)"
          @edit="onEditDraft(t.segment as SheetDraftSegment)"
        >
          <p class="bs-msg__body">{{ (t.segment as SheetDraftSegment).body }}</p>
        </BSDraftedCard>

        <!-- Build-Sheet questions_v2-style picker. -->
        <BSAskCard
          v-else-if="t.kind === 'sheet-ask' && t.segment"
          :title="(t.segment as SheetAskSegment).title"
          :subtitle="(t.segment as SheetAskSegment).subtitle ?? undefined"
          :options="(t.segment as SheetAskSegment).options"
          :multi="(t.segment as SheetAskSegment).multi"
          :has-other="(t.segment as SheetAskSegment).hasOther"
        />

        <!-- Build-Sheet one-shot interview panel (Sprint UX-2). The panel
             reads its data from the sheet store; the segment is just an
             anchor so the panel renders inline at the right place in the
             assistant bubble. -->
        <BSInterviewPanel
          v-else-if="t.kind === 'sheet-interview'"
        />

        <!-- Persistent confirmation banner once `sheet_finalize_request`
             succeeds. Pure visual marker — the source of truth lives in
             the SQLite `build_sheets` row. -->
        <BSSheetSavedBanner
          v-else-if="t.kind === 'sheet-finalized' && t.segment"
          :name="(t.segment as SheetFinalizedSegment).name"
        />

        <!-- Slim CoVe verifier audit card. Anchored before the final text
             of the turn so the user sees the verification status before
             reading the (possibly revised) reply. -->
        <VerifyClaimsCard
          v-else-if="t.kind === 'verify-claims' && t.segment"
          :segment="(t.segment as VerifyClaimsSegment)"
        />

        <!-- Placeholder while waiting for first segment, OR while
             Bestel sits silent between tool calls / before final text. -->
        <span v-else-if="t.kind === 'placeholder'" class="thinking-dots" aria-label="Bestel is thinking…">
          <span class="thinking-dots__dot" />
          <span class="thinking-dots__dot" />
          <span class="thinking-dots__dot" />
        </span>
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
  /* Constrain to parent (chat-stream column flex). Without `min-width: 0`
   * a child with intrinsic min-content larger than the chat width (e.g.
   * the BSInterviewPanel's chip rows) pushes the article past the right
   * rail and breaks the layout. The `max-width: 100%` is belt-and-braces;
   * with `min-width: 0` alone the article still expands beyond the chat
   * column on Windows + Chrome's flex sizing on certain widths. */
  min-width: 0;
  max-width: 100%;
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

/* Narration — assistant text fragments emitted between tool calls.
 * Faded italic to read as a status update, not the answer. Sits
 * inside the artifact stack via .turn--artifact (above) so the
 * vertical guide line groups it with its surrounding tool calls. */
.turn__narration {
  margin: 0;
  font-family: var(--hand);
  font-size: 13px;
  font-style: italic;
  line-height: 1.55;
  color: var(--ink-faint);
  white-space: pre-wrap;
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

/* Build-Sheet drafted-section body slotted into BSDraftedCard. */
.bs-msg__body {
  margin: 0;
  font-family: var(--hand);
  font-size: 15px;
  color: var(--ink);
  line-height: 1.6;
}
</style>

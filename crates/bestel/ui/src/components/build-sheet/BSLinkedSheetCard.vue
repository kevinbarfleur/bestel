<script setup lang="ts">
import { computed } from 'vue';
import { storeToRefs } from 'pinia';

import { useChatStore } from '../../stores/chat';
import { useSheetStore } from '../../stores/sheet';
import { INTERVIEW_SECTION_ORDER } from '../../stores/sheet';

/**
 * Sidebar card surfaced once a Build Sheet is validated for the active
 * PoB. Sprint UX-2.8 enriches the previous minimal version with the
 * design's full layout (`build-sheet.jsx::SheetPanel finalized`,
 * lines 256-322): header badge + relative time, archetype subtitle,
 * defining-item rows, 6-section dots, and a footer with `View full`
 * (opens a modal showing the entire sheet) and `↻ Refresh` (sends a
 * predefined message in the current chat to relaunch the interview).
 *
 * Hides when `sheet.activeSheet` is null. When the agent finalizes a
 * fresh sheet via `sheet_finalize_request`, or when `get_active_build_sheet`
 * loads an existing sheet by fingerprint, the streaming layer emits
 * `SheetLoaded` and the store flips `activeSheet` non-null.
 */

const sheet = useSheetStore();
const chat = useChatStore();
const { activeSheet, phase } = storeToRefs(sheet);

const emit = defineEmits<{
  (e: 'view-full', sheetId: string): void;
}>();

interface BuildSheetPayload {
  sections?: Array<{ id: string; label?: string; body?: string; confirmed?: boolean }>;
  defining_items?: Array<{ name: string; role: string; purpose?: string | null }>;
}

const payload = computed<BuildSheetPayload | null>(
  () => (activeSheet.value?.payload as BuildSheetPayload | undefined) ?? null,
);

/** First sentence / paragraph of the archetype section, truncated. The
 *  agent authors the body during `sheet_finalize_request` — a single
 *  short paragraph like "Inquisitor · Penance Brand of Dissipation ·
 *  crit-lightning self-cast · …". We grab the leading chunk so the card
 *  surfaces the build's signature without sprawling. */
const archetypeText = computed<string | null>(() => {
  const section = payload.value?.sections?.find((s) => s.id === 'archetype');
  const body = (section?.body ?? '').trim();
  if (!body) return null;
  // Split on first paragraph break or sentence end. Keep within ~120 chars.
  const firstParagraph = body.split(/\n\n+/)[0] ?? body;
  const trimmed = firstParagraph.length > 120
    ? `${firstParagraph.slice(0, 117).trimEnd()}…`
    : firstParagraph;
  return trimmed;
});

/** Order roles by load-bearing priority so the rendered list always
 *  shows engine + defining items first, even when the agent emits the
 *  array in arbitrary order. Anything beyond the top 3 falls behind a
 *  `+N more` summary line. */
const ROLE_PRIORITY: Record<string, number> = {
  engine: 0,
  defining: 1,
  amplifier: 2,
  enabler: 3,
  filler: 4,
};

const sortedItems = computed(() => {
  const items = payload.value?.defining_items ?? [];
  return [...items].sort(
    (a, b) =>
      (ROLE_PRIORITY[a.role] ?? 99) - (ROLE_PRIORITY[b.role] ?? 99) ||
      a.name.localeCompare(b.name),
  );
});

const topItems = computed(() => sortedItems.value.slice(0, 3));
const moreItems = computed(() => Math.max(0, sortedItems.value.length - topItems.value.length));

/** Compute a status dot for each canonical section. Validated sheets
 *  always have non-empty section bodies, so by default everything is
 *  done. When the sheet is `stale`, we still show all dots as done —
 *  the staleness is surfaced in the badge above. */
type SectionId = (typeof INTERVIEW_SECTION_ORDER)[number];
const SECTION_LABELS: Record<SectionId, string> = {
  identity: 'Identity',
  archetype: 'Archetype',
  damage: 'Damage',
  defense: 'Defense',
  items: 'Items',
  intent: 'Intent',
};

const sectionStatus = computed<Record<SectionId, 'done' | 'queued'>>(() => {
  const out: Record<SectionId, 'done' | 'queued'> = {
    identity: 'queued',
    archetype: 'queued',
    damage: 'queued',
    defense: 'queued',
    items: 'queued',
    intent: 'queued',
  };
  const sections = payload.value?.sections ?? [];
  for (const s of sections) {
    if ((INTERVIEW_SECTION_ORDER as readonly string[]).includes(s.id)) {
      out[s.id as SectionId] = (s.body ?? '').trim().length > 0 ? 'done' : 'queued';
    }
  }
  return out;
});

const versionLabel = computed(() => {
  const v = activeSheet.value?.schemaVersion;
  return v ? `Build Sheet · v${v}` : 'Build Sheet';
});

const isStale = computed(() => phase.value === 'stale');

const relativeTime = computed<string | null>(() => {
  const ts = activeSheet.value?.updatedAt;
  if (!ts) return null;
  const ms = new Date(ts).getTime();
  if (!Number.isFinite(ms)) return null;
  const delta = Math.max(0, Date.now() - ms);
  if (delta < 60_000) return 'just now';
  if (delta < 60 * 60_000) return `${Math.floor(delta / 60_000)} min ago`;
  if (delta < 24 * 60 * 60_000) return `${Math.floor(delta / (60 * 60_000))} h ago`;
  const days = Math.floor(delta / (24 * 60 * 60_000));
  return days <= 1 ? 'yesterday' : `${days} d ago`;
});

const REFRESH_MESSAGE =
  'I would like to refresh my Build Sheet — please re-run the interview from scratch so I can update my answers and capture any changes since the last finalize.';

function onViewFull() {
  if (activeSheet.value) emit('view-full', activeSheet.value.sheetId);
}

async function onRefresh() {
  await chat.send(REFRESH_MESSAGE);
}
</script>

<template>
  <div v-if="activeSheet" class="bs-link" :class="{ 'bs-link--stale': isStale }">
    <!-- Header — badge + grow + relative time -->
    <div class="bs-link__head">
      <span class="bs-link__badge" :class="{ 'bs-link__badge--stale': isStale }">
        <span class="bs-link__badge-glyph" aria-hidden="true">{{ isStale ? '!' : '✓' }}</span>
        {{ isStale ? `${versionLabel} · stale` : versionLabel }}
      </span>
      <span class="bs-link__grow" />
      <span v-if="relativeTime" class="bs-link__time">{{ relativeTime }}</span>
    </div>

    <!-- Archetype subtitle, when authored -->
    <div v-if="archetypeText" class="bs-link__sub">
      <div class="bs-link__sub-label">archetype</div>
      <div class="bs-link__sub-text">{{ archetypeText }}</div>
    </div>

    <!-- Defining items rows (top-priority, +N more) -->
    <div v-if="topItems.length" class="bs-link__sub">
      <div class="bs-link__sub-label">defining items</div>
      <div
        v-for="it in topItems"
        :key="it.name"
        class="bs-link__item"
      >
        <span class="bs-link__role" :class="`bs-link__role--${it.role}`">{{ it.role }}</span>
        <span class="bs-link__name">{{ it.name }}</span>
      </div>
      <div v-if="moreItems > 0" class="bs-link__more">+{{ moreItems }} more</div>
    </div>

    <!-- Section status dots — small inline row -->
    <div class="bs-link__dots">
      <span
        v-for="id in INTERVIEW_SECTION_ORDER"
        :key="id"
        class="bs-link__dot"
        :class="`bs-link__dot--${sectionStatus[id as SectionId]}`"
        :title="SECTION_LABELS[id as SectionId]"
      />
    </div>

    <!-- Footer — View full + Refresh -->
    <div class="bs-link__foot">
      <button
        type="button"
        class="bs-link__btn bs-link__btn--outline"
        @click="onViewFull"
      >
        View full
      </button>
      <span class="bs-link__grow" />
      <button
        type="button"
        class="bs-link__btn bs-link__btn--link"
        @click="onRefresh"
      >
        ↻ Refresh
      </button>
    </div>
  </div>
</template>

<style scoped>
.bs-link {
  flex: 0 0 auto;
  width: 100%;
  padding: 14px 16px;
  border: 1px solid var(--paper-line);
  border-radius: 5px;
  background: var(--paper);
  display: flex;
  flex-direction: column;
  gap: 12px;
  box-sizing: border-box;
}
.bs-link--stale {
  border-color: var(--amber);
  background: rgba(175, 96, 37, 0.06);
}

/* ─── Header ──────────────────────────────────────────────────────────── */
.bs-link__head {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.bs-link__badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 2px 8px;
  border: 1px solid var(--good);
  border-radius: 3px;
  background: rgba(84, 124, 74, 0.10);
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--good);
  font-weight: 700;
  white-space: nowrap;
}
.bs-link__badge--stale {
  border-color: var(--amber);
  background: rgba(175, 96, 37, 0.12);
  color: var(--amber);
}
.bs-link__badge-glyph {
  font-family: var(--hand);
  font-size: 12px;
  line-height: 1;
  letter-spacing: 0;
}
.bs-link__grow { flex: 1; }
.bs-link__time {
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink-faint);
  white-space: nowrap;
}

/* ─── Sub blocks (archetype, defining items) ─────────────────────────── */
.bs-link__sub {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-top: 10px;
  border-top: 1px dotted var(--paper-line);
}
.bs-link__sub-label {
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 700;
  margin-bottom: 2px;
}
.bs-link__sub-text {
  font-family: var(--hand);
  font-size: 13px;
  color: var(--ink);
  line-height: 1.5;
  font-weight: 500;
}

.bs-link__item {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-size: 13px;
  color: var(--ink);
  min-width: 0;
}
.bs-link__role {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 700;
  width: 64px;
  flex: none;
}
.bs-link__role--engine { color: var(--amber); }
.bs-link__role--defining { color: var(--ink); }
.bs-link__role--amplifier { color: var(--ink-soft); }
.bs-link__role--enabler { color: var(--ink-soft); }
.bs-link__role--filler { color: var(--ink-faint); }
.bs-link__name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--ink);
  font-weight: 500;
}
.bs-link__more {
  margin-top: 2px;
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.10em;
  color: var(--ink-faint);
  text-transform: uppercase;
}

/* ─── Section dots row ───────────────────────────────────────────────── */
.bs-link__dots {
  display: flex;
  gap: 6px;
  padding-top: 10px;
  border-top: 1px dotted var(--paper-line);
}
.bs-link__dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  flex: none;
}
.bs-link__dot--done {
  background: var(--good);
}
.bs-link__dot--queued {
  border: 1.2px dashed var(--ink-faint);
  background: transparent;
}

/* ─── Footer ──────────────────────────────────────────────────────────── */
.bs-link__foot {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-top: 10px;
  border-top: 1px dotted var(--paper-line);
}
.bs-link__btn {
  font-family: var(--hand);
  cursor: pointer;
  transition: border-color 0.15s ease, background 0.15s ease, color 0.15s ease;
}
.bs-link__btn--outline {
  display: inline-flex;
  align-items: center;
  padding: 5px 11px;
  border: 1px solid var(--ink-soft);
  border-radius: 4px;
  background: var(--paper);
  font-size: 12px;
  font-weight: 500;
  color: var(--ink);
}
.bs-link__btn--outline:hover {
  border-color: var(--amber);
  background: var(--amber-glow);
}
.bs-link__btn--link {
  background: transparent;
  border: 0;
  padding: 0;
  font-size: 13px;
  color: var(--ink-soft);
  text-decoration: underline;
  text-underline-offset: 3px;
}
.bs-link__btn--link:hover {
  color: var(--amber);
}
</style>

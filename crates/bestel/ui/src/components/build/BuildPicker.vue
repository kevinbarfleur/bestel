<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import {
  deleteBuildSheet,
  getBuildSheet,
  listBuildSheets,
  previewBuild,
} from '../../api/tauri';
import { useBuildStore } from '../../stores/build';
import { useSheetStore } from '../../stores/sheet';
import { useToastsStore } from '../../stores/toasts';
import RunicModal from '../runic/RunicModal.vue';
import RunicButton from '../runic/RunicButton.vue';
import BSSheetFullView from '../build-sheet/BSSheetFullView.vue';
import {
  PickerLayout,
  PickerListItem,
  PickerSearchInput,
  PickerSectionHead,
  PickerStatusDot,
} from '../pickers';
import { usePickerNav } from '../../composables/usePickerNav';
import type {
  BuildSheetDetailDto,
  BuildSheetSummaryDto,
  PobBuildDto,
  PobBuildSummaryDto,
} from '../../api/types';

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>();

const buildStore = useBuildStore();
const sheetStore = useSheetStore();
const { list, loadingList, current } = storeToRefs(buildStore);
const { activeSheet } = storeToRefs(sheetStore);
const toasts = useToastsStore();

/** Top-of-modal toggle: which catalogue we're browsing. The toggle sits
 *  above the search input so the user picks a mode first, then filters. */
const mode = ref<'builds' | 'sheets'>('builds');

const search = ref('');
const previewCache = ref(new Map<string, PobBuildDto>());
const previewLoading = ref(false);

// ─── Builds — filtered by search (existing) ─────────────────────────
const filtered = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return list.value;
  return list.value.filter((b) => {
    const haystack = [
      b.header,
      b.file_name,
      b.class,
      b.ascendancy ?? '',
      b.main_skill_hint ?? '',
      b.game,
    ]
      .join(' ')
      .toLowerCase();
    return haystack.includes(q);
  });
});

// ─── Build Sheets — listed via Tauri command, filtered by same search ─
const sheets = ref<BuildSheetSummaryDto[]>([]);
const sheetsLoading = ref(false);
const sheetDetailCache = ref(new Map<string, BuildSheetDetailDto>());
const sheetDetailLoading = ref(false);
/** Two-stage delete: first click sets pending; second click commits. */
const pendingDeleteId = ref<string | null>(null);

const filteredSheets = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return sheets.value;
  return sheets.value.filter((s) => {
    const haystack = [s.name, s.fingerprint, s.id].join(' ').toLowerCase();
    return haystack.includes(q);
  });
});

async function refreshSheets() {
  sheetsLoading.value = true;
  try {
    sheets.value = await listBuildSheets();
  } catch (e) {
    toasts.push({ variant: 'error', title: 'Failed to load sheets', body: String(e) });
  } finally {
    sheetsLoading.value = false;
  }
}

const close = () => emit('update:modelValue', false);

const choose = async (b: PobBuildSummaryDto) => {
  const dto = await buildStore.setActive(b.path);
  if (dto) {
    toasts.push({ variant: 'success', title: 'Build loaded', body: b.header });
    close();
  } else {
    toasts.push({
      variant: 'error',
      title: 'Failed to load build',
      body: b.header,
    });
  }
};

const { highlighted, selected, onKeydown } = usePickerNav<PobBuildSummaryDto>(
  filtered,
  choose,
);

const sheetNoop = () => {
  /* sheets pane has no Enter-to-load action — Delete is in the action bar. */
};
const {
  highlighted: sheetHighlighted,
  selected: selectedSheet,
  onKeydown: onSheetKeydown,
} = usePickerNav<BuildSheetSummaryDto>(filteredSheets, sheetNoop);

function onSearchKeydown(e: KeyboardEvent) {
  return mode.value === 'builds' ? onKeydown(e) : onSheetKeydown(e);
}

const detail = computed<PobBuildDto | null>(() => {
  const summary = selected.value;
  if (!summary) return null;
  // If the summary points at the currently-loaded build, prefer the live DTO
  // (avoids re-parsing and stays in sync with watcher updates).
  if (current.value && current.value.source_file === summary.path) {
    return current.value;
  }
  return previewCache.value.get(summary.path) ?? null;
});

const isActive = (b: PobBuildSummaryDto) =>
  current.value?.source_file === b.path;

watch(
  () => props.modelValue,
  async (open) => {
    if (!open) return;
    search.value = '';
    highlighted.value = 0;
    sheetHighlighted.value = 0;
    pendingDeleteId.value = null;
    previewCache.value = new Map();
    sheetDetailCache.value = new Map();
    await buildStore.refreshList();
    if (mode.value === 'sheets') await refreshSheets();
  },
);

watch(mode, async (m) => {
  // Reset shared state on mode swap so the user lands in a clean view.
  search.value = '';
  pendingDeleteId.value = null;
  highlighted.value = 0;
  sheetHighlighted.value = 0;
  if (m === 'sheets' && sheets.value.length === 0) {
    await refreshSheets();
  }
});

// Lazy-load full sheet payload on selection so the right pane has data.
let sheetDetailTimer: number | null = null;
watch(selectedSheet, (s) => {
  if (!s) return;
  if (sheetDetailCache.value.has(s.id)) return;
  if (sheetDetailTimer != null) window.clearTimeout(sheetDetailTimer);
  sheetDetailTimer = window.setTimeout(async () => {
    sheetDetailLoading.value = true;
    try {
      const dto = await getBuildSheet(s.id);
      if (dto) sheetDetailCache.value.set(s.id, dto);
    } catch (e) {
      toasts.push({ variant: 'warning', title: 'Sheet preview failed', body: s.name });
    } finally {
      sheetDetailLoading.value = false;
    }
  }, 80);
});

const selectedSheetDetail = computed<BuildSheetDetailDto | null>(() => {
  const s = selectedSheet.value;
  if (!s) return null;
  return sheetDetailCache.value.get(s.id) ?? null;
});

const selectedSheetGame = computed<'poe1' | 'poe2'>(() => {
  // Fingerprint format: "<asc_lower>:<skill_lower>:<sorted_uniques>". The
  // game hint isn't in there. Fall back to the active build's game when
  // the selected sheet matches our fingerprint, otherwise default poe1.
  return current.value?.game ?? 'poe1';
});

async function onDeleteClick() {
  const s = selectedSheet.value;
  if (!s) return;
  if (pendingDeleteId.value !== s.id) {
    pendingDeleteId.value = s.id;
    return;
  }
  try {
    const ok = await deleteBuildSheet(s.id);
    if (ok) {
      sheets.value = sheets.value.filter((x) => x.id !== s.id);
      sheetDetailCache.value.delete(s.id);
      if (activeSheet.value?.sheetId === s.id) {
        sheetStore.clearActiveSheet();
      }
      pendingDeleteId.value = null;
      // Re-anchor highlight at the position now occupied (or last row).
      sheetHighlighted.value = Math.min(
        sheetHighlighted.value,
        Math.max(0, filteredSheets.value.length - 1),
      );
      toasts.push({
        variant: 'success',
        title: 'Build Sheet deleted',
        body: s.name,
      });
    } else {
      toasts.push({
        variant: 'warning',
        title: 'Sheet not found',
        body: 'It may have been deleted in another window.',
      });
      sheets.value = sheets.value.filter((x) => x.id !== s.id);
      pendingDeleteId.value = null;
    }
  } catch (e) {
    toasts.push({ variant: 'error', title: 'Delete failed', body: String(e) });
    pendingDeleteId.value = null;
  }
}

function relativeSheetTime(iso: string | null | undefined): string {
  if (!iso) return 'unknown';
  const ms = new Date(iso).getTime();
  if (!Number.isFinite(ms)) return 'unknown';
  const delta = Math.max(0, Date.now() - ms);
  if (delta < 60_000) return 'just now';
  if (delta < 60 * 60_000) return `${Math.floor(delta / 60_000)} min ago`;
  if (delta < 24 * 60 * 60_000) return `${Math.floor(delta / (60 * 60_000))} h ago`;
  const days = Math.floor(delta / (24 * 60 * 60_000));
  return days <= 1 ? 'yesterday' : `${days} d ago`;
}

function fingerprintSummary(fp: string): string {
  // Fingerprint shape: "<asc_lower>:<skill_lower>:<unique1+unique2+…>"
  const parts = fp.split(':');
  if (parts.length < 2) return fp;
  const asc = parts[0]?.replace(/\b\w/g, (c) => c.toUpperCase()) ?? '';
  const skill = parts[1]?.replace(/\b\w/g, (c) => c.toUpperCase()) ?? '';
  return [asc, skill].filter(Boolean).join(' · ');
}

// Lazy-load full DTO for the highlighted item, with a tiny debounce so cycling
// fast through the list doesn't fire dozens of parses.
let previewTimer: number | null = null;
watch(selected, (summary) => {
  if (!summary) return;
  if (current.value?.source_file === summary.path) return;
  if (previewCache.value.has(summary.path)) return;
  if (previewTimer != null) window.clearTimeout(previewTimer);
  previewTimer = window.setTimeout(async () => {
    previewLoading.value = true;
    try {
      const dto = await previewBuild(summary.path);
      previewCache.value.set(summary.path, dto);
    } catch (e) {
      toasts.push({
        variant: 'warning',
        title: 'Preview failed',
        body: summary.header,
      });
    } finally {
      previewLoading.value = false;
    }
  }, 120);
});

function relativeTime(mtime: number | null): string {
  if (!mtime) return 'unknown';
  const diff = Date.now() - mtime;
  const m = Math.floor(diff / 60000);
  if (m < 1) return 'just now';
  if (m < 60) return `${m} min ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h} h ago`;
  const d = Math.floor(h / 24);
  if (d < 7) return `${d} d ago`;
  const w = Math.floor(d / 7);
  return `${w} wk ago`;
}

function gameTag(game: string): string {
  return game === 'poe2' ? 'PoE2' : 'PoE1';
}

function fmt(n: number | null | undefined): string {
  if (n == null || !Number.isFinite(n)) return '—';
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
  if (n >= 1000) return n.toLocaleString('en-US', { maximumFractionDigits: 0 });
  return Math.round(n).toString();
}

interface ResistView {
  el: 'fire' | 'cold' | 'lightning' | 'chaos';
  label: string;
  value: number | null;
  cap: number;
  status: 'ok' | 'under' | 'neg' | 'missing';
  color: string;
  fillPct: number; // 0..1, capped at 1
  capMarkerPct: number; // position of cap line on the bar (0..1 normalized over 100)
}

function resistView(d: PobBuildDto): ResistView[] {
  const seq: Array<['fire' | 'cold' | 'lightning' | 'chaos', string]> = [
    ['fire', 'fire'],
    ['cold', 'cold'],
    ['lightning', 'lit'],
    ['chaos', 'chaos'],
  ];
  return seq.map(([el, label]) => {
    const r = d.resistances.find((x) => x.name === el);
    const value = r?.value ?? null;
    const cap = r?.cap ?? 75;
    let status: ResistView['status'];
    let color: string;
    if (value == null) {
      status = 'missing';
      color = 'var(--ink-faint)';
    } else if (value < 0) {
      status = 'neg';
      color = 'var(--bad)';
    } else if (value < cap) {
      status = 'under';
      color = 'var(--note)';
    } else {
      status = 'ok';
      color = `var(--el-${label}-deep)`;
    }
    // Bar normalized so 100 = full track. Negatives and over-caps clamp.
    const fillPct = value == null ? 0 : Math.max(0, Math.min(1, value / 100));
    const capMarkerPct = Math.max(0, Math.min(1, cap / 100));
    return { el, label, value, cap, status, color, fillPct, capMarkerPct };
  });
}

interface VitalTile {
  k: string;
  v: string;
  muted: boolean;
}

function vitals(d: PobBuildDto): VitalTile[] {
  const isPoe2 = d.game === 'poe2';
  const fourth: VitalTile = isPoe2
    ? { k: 'spirit', v: fmt(d.spirit), muted: d.spirit == null }
    : { k: 'evasion', v: fmt(d.evasion), muted: d.evasion == null };
  return [
    { k: 'life', v: fmt(d.life), muted: d.life == null },
    {
      k: 'energy shield',
      v: d.energy_shield && d.energy_shield > 0 ? fmt(d.energy_shield) : '—',
      muted: !d.energy_shield,
    },
    { k: 'mana', v: fmt(d.mana), muted: d.mana == null },
    fourth,
  ];
}

function mainSkillRow(d: PobBuildDto) {
  if (!d.main_skill) return null;
  return d.main_skill;
}

function dpsLabel(d: PobBuildDto): string {
  if (d.dps == null) return '—';
  return `${fmt(d.dps)} dps`;
}

function ehpLabel(d: PobBuildDto): string {
  if (d.ehp == null) return '';
  return `EHP ${fmt(d.ehp)}`;
}

function actionLabel(b: PobBuildSummaryDto): string {
  return isActive(b) ? 'Already loaded' : 'Load this build';
}
</script>

<template>
  <RunicModal
    :model-value="modelValue"
    :title="mode === 'builds' ? 'Select a build' : 'Build Sheets'"
    :subtitle="mode === 'builds'
      ? 'Path of Building XML files detected in your watched folders. Pick one to load it as Bestel\'s working build.'
      : 'Saved Build Sheets — pick one to view its full content, or delete to start fresh next time.'"
    kbd="Ctrl+B"
    max-width="panes"
    @update:model-value="(v) => emit('update:modelValue', v)"
  >
    <PickerLayout>
      <template #sidebar>
        <!-- Mode toggle — sits above the search input (Sprint UX-2.9). -->
        <div class="picker-mode" role="tablist" aria-label="Builds or Build Sheets">
          <button
            type="button"
            role="tab"
            class="picker-mode__btn"
            :class="{ 'picker-mode__btn--active': mode === 'builds' }"
            :aria-selected="mode === 'builds'"
            @click="mode = 'builds'"
          >
            <span class="picker-mode__label">Builds</span>
            <span class="picker-mode__count">{{ list.length }}</span>
          </button>
          <button
            type="button"
            role="tab"
            class="picker-mode__btn"
            :class="{ 'picker-mode__btn--active': mode === 'sheets' }"
            :aria-selected="mode === 'sheets'"
            @click="mode = 'sheets'"
          >
            <span class="picker-mode__label">Build Sheets</span>
            <span v-if="sheets.length" class="picker-mode__count">{{ sheets.length }}</span>
          </button>
        </div>

        <PickerSearchInput
          v-model="search"
          :placeholder="mode === 'builds' ? 'Search by file, class, skill…' : 'Search by sheet name, fingerprint…'"
          :count="mode === 'builds' ? filtered.length : filteredSheets.length"
          @keydown="onSearchKeydown"
        />

        <!-- BUILDS list -->
        <template v-if="mode === 'builds'">
          <p v-if="loadingList" class="build-picker__hint">Loading…</p>
          <p v-else-if="!filtered.length" class="build-picker__hint">No builds found.</p>

          <PickerListItem
            v-for="(b, idx) in filtered"
            :key="b.path"
            :active="isActive(b)"
            :highlighted="idx === highlighted"
            active-label="LOADED"
            @click="choose(b)"
            @mouseenter="highlighted = idx"
          >
            <template #name>
              <span class="build-picker__row-name">
                <span class="build-picker__game-chip">{{ gameTag(b.game) }}</span>
                <span class="build-picker__file-name">{{ b.file_name }}</span>
              </span>
            </template>
            <template #meta>
              <span class="build-picker__row-meta">
                {{ b.class }}<template v-if="b.ascendancy"> / {{ b.ascendancy }}</template>
                <template v-if="b.level"> · lvl {{ b.level }}</template>
                <template v-if="b.main_skill_hint"> · {{ b.main_skill_hint }}</template>
              </span>
            </template>
            <template #right>
              <span class="build-picker__time">{{ relativeTime(b.mtime_ms) }}</span>
            </template>
          </PickerListItem>

          <div class="build-picker__watcher">
            <PickerStatusDot kind="on" label="Watcher live · auto-import on save" />
          </div>
        </template>

        <!-- SHEETS list -->
        <template v-else>
          <p v-if="sheetsLoading" class="build-picker__hint">Loading…</p>
          <p v-else-if="!filteredSheets.length && search" class="build-picker__hint">
            No sheets match this search.
          </p>
          <p v-else-if="!filteredSheets.length" class="build-picker__hint">
            No Build Sheets stored yet. Once Bestel finalizes one, it'll show up here.
          </p>

          <PickerListItem
            v-for="(s, idx) in filteredSheets"
            :key="s.id"
            :active="activeSheet?.sheetId === s.id"
            :highlighted="idx === sheetHighlighted"
            active-label="LINKED"
            @click="sheetHighlighted = idx"
            @mouseenter="sheetHighlighted = idx"
          >
            <template #name>
              <span class="build-picker__row-name">
                <span class="build-picker__sheet-chip">v{{ s.schema_version }}</span>
                <span class="build-picker__file-name build-picker__file-name--sheet">{{ s.name }}</span>
              </span>
            </template>
            <template #meta>
              <span class="build-picker__row-meta">{{ fingerprintSummary(s.fingerprint) }}</span>
            </template>
            <template #right>
              <span class="build-picker__time">{{ relativeSheetTime(s.updated_at) }}</span>
            </template>
          </PickerListItem>
        </template>
      </template>

      <template #main>
        <!-- BUILDS detail (existing) -->
        <div v-if="mode === 'builds' && detail" class="build-picker__detail">
          <header class="build-picker__title-row">
            <div class="build-picker__title-block">
              <h1 class="build-picker__title">
                {{ detail.class }}<span v-if="detail.ascendancy"> / {{ detail.ascendancy }}</span>
                <span class="build-picker__title-game">{{ gameTag(detail.game) }}</span>
              </h1>
              <p class="build-picker__subtitle">
                lvl {{ detail.level ?? '—' }}
                <template v-if="detail.main_skill">
                  · main skill <strong>{{ detail.main_skill }}</strong>
                </template>
                · <code class="build-picker__file-mono">{{ detail.file_name }}</code>
              </p>
            </div>
            <span v-if="current?.source_file === detail.source_file" class="build-picker__active-chip">
              Currently loaded
            </span>
          </header>

          <!-- VITALS -->
          <section class="build-picker__section">
            <PickerSectionHead>
              Vitals
              <template v-if="detail.ehp" #right>{{ ehpLabel(detail) }}</template>
            </PickerSectionHead>
            <div class="build-picker__big-grid">
              <div v-for="t in vitals(detail)" :key="t.k" class="build-picker__big-tile">
                <div class="build-picker__big-label">{{ t.k }}</div>
                <div
                  class="build-picker__big-value"
                  :class="{ 'build-picker__big-value--muted': t.muted }"
                >
                  {{ t.v }}
                </div>
              </div>
            </div>
          </section>

          <!-- RESISTANCES -->
          <section class="build-picker__section">
            <PickerSectionHead>Resistances</PickerSectionHead>
            <div class="build-picker__big-grid">
              <div v-for="r in resistView(detail)" :key="r.el" class="build-picker__resist-tile">
                <div
                  class="build-picker__big-label"
                  :style="{ color: `var(--el-${r.label})` }"
                >
                  {{ r.label }}
                </div>
                <div class="build-picker__resist-value-row">
                  <span class="build-picker__big-value" :style="{ color: r.color }">
                    {{ r.value == null ? '—' : `${Math.round(r.value)}%` }}
                  </span>
                  <span class="build-picker__resist-cap">/ {{ Math.round(r.cap) }}</span>
                </div>
                <div class="build-picker__bar">
                  <div
                    class="build-picker__bar-fill"
                    :style="{
                      width: `${r.fillPct * 100}%`,
                      background: r.status === 'neg' ? 'var(--bad)' : r.status === 'under' ? 'var(--note)' : `var(--el-${r.label})`,
                    }"
                  />
                  <div
                    class="build-picker__bar-cap"
                    :style="{ left: `${r.capMarkerPct * 100}%` }"
                  />
                </div>
              </div>
            </div>
          </section>

          <!-- SKILLS + DPS -->
          <section class="build-picker__skills-row">
            <div class="build-picker__skills">
              <PickerSectionHead>
                Skills
                <template v-if="detail.skill_groups?.length" #right>
                  {{ detail.skill_groups.length }} group{{ detail.skill_groups.length === 1 ? '' : 's' }}
                </template>
              </PickerSectionHead>
              <div v-if="mainSkillRow(detail)" class="leader-row">
                <span class="leader-row__k" style="color: var(--el-lit-deep);">
                  <span style="color: var(--el-lit); margin-right: 4px;">◆</span>{{ mainSkillRow(detail) }}
                </span>
                <span class="leader-row__dots" />
                <span class="leader-row__v" style="color: var(--el-lit-deep);">{{ dpsLabel(detail) }}</span>
              </div>
              <div
                v-for="g in detail.skill_groups.filter((g) => !g.is_main).slice(0, 4)"
                :key="g.label"
                class="leader-row"
              >
                <span class="leader-row__k">{{ g.label }}</span>
                <span class="leader-row__dots" />
                <span class="leader-row__v">{{ g.gems.length }} gem{{ g.gems.length === 1 ? '' : 's' }}</span>
              </div>
              <p v-if="!detail.skill_groups?.length" class="build-picker__missing">
                No skill groups parsed.
              </p>
            </div>
            <div class="build-picker__dps">
              <PickerSectionHead>Estimated DPS</PickerSectionHead>
              <div class="build-picker__dps-value">{{ fmt(detail.dps) }}</div>
              <div class="build-picker__dps-meta">
                {{ detail.dps == null ? 'no DPS computed in PoB' : 'shaper config · top main skill' }}
              </div>
            </div>
          </section>
        </div>
        <p
          v-else-if="mode === 'builds' && previewLoading"
          class="build-picker__empty-detail"
        >
          Parsing build…
        </p>
        <p v-else-if="mode === 'builds'" class="build-picker__empty-detail">
          Pick a build on the left to see its details.
        </p>

        <!-- SHEETS detail (new) -->
        <div
          v-else-if="mode === 'sheets' && selectedSheetDetail"
          class="build-picker__detail build-picker__sheet-detail"
        >
          <header class="build-picker__title-row">
            <div class="build-picker__title-block">
              <h1 class="build-picker__title">
                {{ selectedSheetDetail.name }}
                <span class="build-picker__title-game">v{{ selectedSheetDetail.schema_version }}</span>
              </h1>
              <p class="build-picker__subtitle">
                <span>{{ relativeSheetTime(selectedSheetDetail.updated_at) }}</span>
                <span v-if="selectedSheetDetail.fingerprint">
                  · <code class="build-picker__file-mono">{{ selectedSheetDetail.fingerprint }}</code>
                </span>
              </p>
            </div>
            <span
              v-if="activeSheet?.sheetId === selectedSheetDetail.id"
              class="build-picker__active-chip"
            >
              Currently linked
            </span>
          </header>
          <BSSheetFullView
            :payload="selectedSheetDetail.payload"
            :game="selectedSheetGame"
          />
        </div>
        <p
          v-else-if="mode === 'sheets' && sheetDetailLoading"
          class="build-picker__empty-detail"
        >
          Loading sheet…
        </p>
        <p v-else-if="mode === 'sheets'" class="build-picker__empty-detail">
          Pick a Build Sheet on the left to read its full content.
        </p>
      </template>

      <!-- BUILDS action bar -->
      <template v-if="mode === 'builds' && selected" #actionBar>
        <span class="build-picker__hint">
          Loading <strong>{{ selected.file_name }}</strong> will replace the build attached to this chat.
        </span>
        <RunicButton variant="secondary" no-runes @click="close">Cancel</RunicButton>
        <RunicButton
          variant="primary"
          no-runes
          icon="check"
          kbd="⏎"
          :disabled="isActive(selected)"
          :disabled-reason="isActive(selected) ? 'Already loaded' : undefined"
          @click="choose(selected)"
        >
          {{ actionLabel(selected) }}
        </RunicButton>
      </template>

      <!-- SHEETS action bar — two-stage delete -->
      <template v-else-if="mode === 'sheets' && selectedSheet" #actionBar>
        <span
          class="build-picker__hint"
          :class="{ 'build-picker__hint--danger': pendingDeleteId === selectedSheet.id }"
        >
          <template v-if="pendingDeleteId === selectedSheet.id">
            <strong>Click again to permanently delete</strong> "{{ selectedSheet.name }}".
            This cannot be undone.
          </template>
          <template v-else>
            Deleting <strong>{{ selectedSheet.name }}</strong> removes its row from the
            local database. Bestel will re-author from scratch the next time the build is loaded.
          </template>
        </span>
        <RunicButton
          variant="secondary"
          no-runes
          @click="pendingDeleteId === selectedSheet.id ? (pendingDeleteId = null) : close()"
        >
          Cancel
        </RunicButton>
        <RunicButton
          variant="primary"
          no-runes
          icon="trash"
          :danger="true"
          @click="onDeleteClick"
        >
          {{ pendingDeleteId === selectedSheet.id ? 'Confirm delete' : 'Delete sheet' }}
        </RunicButton>
      </template>

      <template #footer>
        <span class="build-picker__hint-row"><span class="build-picker__kbd">↑↓</span><span>navigate</span></span>
        <span v-if="mode === 'builds'" class="build-picker__hint-row">
          <span class="build-picker__kbd">⏎</span><span>load</span>
        </span>
        <span class="build-picker__hint-row"><span class="build-picker__kbd">esc</span><span>close</span></span>
        <span style="flex: 1" />
        <span v-if="mode === 'builds'">
          {{ filtered.length }} build{{ filtered.length === 1 ? '' : 's' }}
        </span>
        <span v-else>
          {{ filteredSheets.length }} sheet{{ filteredSheets.length === 1 ? '' : 's' }}
        </span>
      </template>
    </PickerLayout>
  </RunicModal>
</template>

<style scoped>
.build-picker__hint,
.build-picker__empty-detail {
  margin: 1rem;
  text-align: center;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-faint);
}
.build-picker__hint--danger {
  color: var(--bad);
}

/* ─── Mode toggle (Builds / Build Sheets) — sits above search ─────── */
.picker-mode {
  display: flex;
  gap: 6px;
  padding: 12px 16px 4px;
}
.picker-mode__btn {
  flex: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 7px 14px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 4px;
  font-family: var(--hand);
  font-size: 13px;
  font-weight: 500;
  color: var(--ink-soft);
  cursor: pointer;
  transition: border-color 0.15s ease, background 0.15s ease, color 0.15s ease;
}
.picker-mode__btn:hover {
  border-color: var(--ink-soft);
  color: var(--ink);
  background: var(--paper-shade);
}
.picker-mode__btn--active {
  border-color: var(--amber);
  background: var(--amber-glow);
  color: var(--ink);
  font-weight: 600;
}
.picker-mode__btn--active:hover {
  background: var(--amber-glow);
}
.picker-mode__label {
  letter-spacing: 0.02em;
}
.picker-mode__count {
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.10em;
  color: var(--ink-faint);
  background: var(--paper-shade);
  padding: 1px 6px;
  border-radius: 8px;
  font-weight: 600;
}
.picker-mode__btn--active .picker-mode__count {
  color: var(--amber);
  background: var(--paper);
}

.build-picker__row-name {
  display: inline-flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}
.build-picker__game-chip {
  font-family: var(--mono);
  font-size: var(--fs-micro);
  color: var(--ink-faint);
  padding: 1px 5px;
  border: 1px solid var(--paper-line);
  border-radius: 3px;
  letter-spacing: 0.04em;
  flex: none;
}
.build-picker__sheet-chip {
  font-family: var(--label);
  font-size: var(--fs-micro);
  color: var(--good);
  padding: 1px 6px;
  border: 1px solid var(--good);
  border-radius: 3px;
  letter-spacing: 0.10em;
  font-weight: 700;
  flex: none;
}
.build-picker__file-name--sheet {
  font-family: var(--hand);
  font-weight: 600;
  color: var(--ink);
}
.build-picker__sheet-detail {
  /* Slightly narrower gap — the sheet view ships its own internal
   * spacing so we don't double the gap. */
  gap: 18px;
}
.build-picker__file-name {
  font-family: var(--mono);
  font-size: var(--fs-meta);
  color: var(--ink);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.build-picker__row-meta {
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}
.build-picker__time {
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}
.build-picker__watcher {
  margin-top: auto;
  padding: 12px 16px;
  border-top: 1px solid var(--paper-line);
}

.build-picker__detail {
  display: flex;
  flex-direction: column;
  gap: 26px;
}

.build-picker__title-row {
  display: flex;
  align-items: flex-start;
  gap: 16px;
}
.build-picker__title-block {
  flex: 1;
  min-width: 0;
}
.build-picker__title {
  display: flex;
  align-items: baseline;
  gap: 12px;
  margin: 0;
  font-family: var(--hand);
  font-size: var(--fs-display);
  font-weight: var(--fw-bold);
  line-height: 1.05;
  color: var(--ink);
}
.build-picker__title-game {
  font-family: var(--mono);
  font-size: var(--fs-caps);
  color: var(--ink-soft);
  padding: 3px 8px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  font-weight: var(--fw-regular);
  letter-spacing: 0.04em;
}
.build-picker__subtitle {
  margin: 6px 0 0;
  font-family: var(--hand);
  font-size: var(--fs-body);
  color: var(--ink-soft);
}
.build-picker__file-mono {
  font-family: var(--mono);
  font-size: var(--fs-meta);
  color: var(--ink);
}
.build-picker__active-chip {
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

.build-picker__big-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 18px;
}
.build-picker__big-tile,
.build-picker__resist-tile {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.build-picker__big-label {
  font-family: var(--label);
  font-size: var(--fs-micro);
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: var(--fw-semibold);
}
.build-picker__big-value {
  font-family: var(--hand);
  font-size: 28px;
  font-weight: var(--fw-bold);
  color: var(--ink);
  line-height: 1;
}
.build-picker__big-value--muted {
  color: var(--ink-faint);
}
.build-picker__resist-value-row {
  display: flex;
  align-items: baseline;
  gap: 6px;
}
.build-picker__resist-cap {
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}
.build-picker__bar {
  margin-top: 6px;
  position: relative;
  height: 4px;
  background: var(--paper-shade);
  border: 1px solid var(--paper-line);
  border-radius: 2px;
  overflow: hidden;
}
.build-picker__bar-fill {
  height: 100%;
}
.build-picker__bar-cap {
  position: absolute;
  top: -2px;
  bottom: -2px;
  width: 1px;
  background: var(--ink-soft);
  opacity: 0.5;
}

.build-picker__skills-row {
  display: grid;
  grid-template-columns: 1.5fr 1fr;
  gap: 32px;
}
.build-picker__skills .leader-row {
  padding: 4px 0;
}
.build-picker__missing {
  margin: 8px 0 0;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-faint);
}
.build-picker__dps-value {
  font-family: var(--hand);
  font-size: 28px;
  font-weight: var(--fw-bold);
  color: var(--el-lit-deep);
  line-height: 1;
}
.build-picker__dps-meta {
  margin-top: 4px;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}

.build-picker__hint {
  flex: 1;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}
.build-picker__hint-row {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.build-picker__kbd {
  font-family: var(--label);
  font-size: var(--fs-micro);
  padding: 1px 5px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  color: var(--ink-soft);
}
</style>

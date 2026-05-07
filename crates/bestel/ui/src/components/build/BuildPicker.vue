<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import { previewBuild } from '../../api/tauri';
import { useBuildStore } from '../../stores/build';
import { useToastsStore } from '../../stores/toasts';
import RunicModal from '../runic/RunicModal.vue';
import RunicButton from '../runic/RunicButton.vue';
import {
  PickerLayout,
  PickerListItem,
  PickerSearchInput,
  PickerSectionHead,
  PickerStatusDot,
} from '../pickers';
import { usePickerNav } from '../../composables/usePickerNav';
import type { PobBuildDto, PobBuildSummaryDto } from '../../api/types';

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>();

const buildStore = useBuildStore();
const { list, loadingList, current } = storeToRefs(buildStore);
const toasts = useToastsStore();

const search = ref('');
const previewCache = ref(new Map<string, PobBuildDto>());
const previewLoading = ref(false);

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
    previewCache.value = new Map();
    await buildStore.refreshList();
  },
);

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
    title="Select a build"
    subtitle="Path of Building XML files detected in your watched folders. Pick one to load it as Bestel's working build."
    kbd="Ctrl+B"
    max-width="panes"
    @update:model-value="(v) => emit('update:modelValue', v)"
  >
    <PickerLayout>
      <template #sidebar>
        <PickerSearchInput
          v-model="search"
          placeholder="Search by file, class, skill…"
          :count="filtered.length"
          @keydown="onKeydown"
        />

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

      <template #main>
        <div v-if="detail" class="build-picker__detail">
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
        <p v-else-if="previewLoading" class="build-picker__empty-detail">Parsing build…</p>
        <p v-else class="build-picker__empty-detail">
          Pick a build on the left to see its details.
        </p>
      </template>

      <template v-if="selected" #actionBar>
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

      <template #footer>
        <span class="build-picker__hint-row"><span class="build-picker__kbd">↑↓</span><span>navigate</span></span>
        <span class="build-picker__hint-row"><span class="build-picker__kbd">⏎</span><span>load</span></span>
        <span class="build-picker__hint-row"><span class="build-picker__kbd">esc</span><span>close</span></span>
        <span style="flex: 1" />
        <span>{{ filtered.length }} build{{ filtered.length === 1 ? '' : 's' }}</span>
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

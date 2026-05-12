<script setup lang="ts">
import { computed } from 'vue';
import RunicButton from '../runic/RunicButton.vue';
import RunicIcon from '../runic/RunicIcon.vue';
import type { DriftAxis } from './DriftChip.vue';

interface DiffSection {
  label: string;
  count: number;
  items: string[];
  removed?: boolean;
  added?: boolean;
  changed?: boolean;
}
interface DriftDrawerContents {
  title: string;
  authored: string;
  sections: DiffSection[];
}

interface Props {
  kind: DriftAxis;
  authoredAt: string;
  contents?: DriftDrawerContents | null;
  /** When true, the drawer renders a placeholder layout while diff data
   *  loads from the backend. */
  loading?: boolean;
}
const props = withDefaults(defineProps<Props>(), { contents: null, loading: false });
const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'reauthor'): void;
  (e: 'keep'): void;
}>();

const titleByKind: Record<DriftAxis, string> = {
  identity: 'Identity signature drift',
  tree: 'Tree signature drift',
  gear: 'Gear signature drift',
  skill: 'Skill signature drift',
  config: 'Config signature drift',
};

const heading = computed(() => props.contents?.title ?? titleByKind[props.kind]);
</script>

<template>
  <aside class="drift-drawer" role="dialog" aria-label="Drifted signature">
    <header class="drift-drawer__header">
      <div class="drift-drawer__heading">
        <span class="drift-drawer__eyebrow">drifted signature</span>
        <h2 class="drift-drawer__title">{{ heading }}</h2>
        <div class="drift-drawer__meta">authored {{ authoredAt }} · compared against live PoB</div>
      </div>
      <button class="drift-drawer__close" type="button" aria-label="Close" @click="emit('close')">
        <RunicIcon name="close" :size="14" />
      </button>
    </header>

    <div class="drift-drawer__body">
      <div v-if="loading" class="drift-drawer__loading">Computing diff…</div>
      <template v-else-if="contents">
        <section v-for="(s, i) in contents.sections" :key="i" class="drift-drawer__section">
          <div class="drift-drawer__section-head">
            <span class="drift-drawer__section-label">{{ s.label }}</span>
            <span class="drift-drawer__section-rule" />
            <span class="drift-drawer__section-count">{{ s.count }}</span>
          </div>
          <ul class="drift-drawer__items">
            <li
              v-for="(it, j) in s.items"
              :key="j"
              class="drift-drawer__item"
              :class="{
                'drift-drawer__item--removed': s.removed,
                'drift-drawer__item--added': s.added,
                'drift-drawer__item--changed': s.changed,
              }"
            >
              <span class="drift-drawer__sigil">{{ s.removed ? '−' : s.added ? '+' : '~' }}</span>
              <span>{{ it }}</span>
            </li>
          </ul>
        </section>
      </template>
      <p v-else class="drift-drawer__empty">No structured diff available yet for this axis.</p>
    </div>

    <footer class="drift-drawer__footer">
      <RunicButton variant="primary" icon="check" @click="emit('reauthor')">
        Re-author sheet from current PoB
      </RunicButton>
      <RunicButton variant="secondary" @click="emit('keep')">Keep current sheet</RunicButton>
    </footer>
  </aside>
</template>

<style scoped>
.drift-drawer {
  width: 520px;
  max-width: 100%;
  height: 100%;
  background: var(--paper);
  border-left: 1px solid var(--paper-line);
  box-shadow: -14px 0 30px rgba(60, 40, 20, 0.10);
  display: flex;
  flex-direction: column;
  font-family: var(--hand);
  color: var(--ink);
}
.drift-drawer__header {
  padding: 18px 22px 14px;
  border-bottom: 1px solid var(--paper-line);
  display: flex;
  align-items: flex-start;
  gap: 12px;
}
.drift-drawer__heading {
  flex: 1;
}
.drift-drawer__eyebrow {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--note);
  font-weight: 700;
  display: inline-block;
  margin-bottom: 4px;
}
.drift-drawer__title {
  font-size: 22px;
  font-weight: 700;
  line-height: 1.15;
  margin: 0;
}
.drift-drawer__meta {
  font-size: 13px;
  color: var(--ink-soft);
  margin-top: 4px;
}
.drift-drawer__close {
  width: 28px;
  height: 28px;
  padding: 0;
  background: transparent;
  color: var(--ink-soft);
  border: 1px solid transparent;
  border-radius: 4px;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.drift-drawer__close:hover {
  background: var(--paper-shade);
}
.drift-drawer__body {
  flex: 1;
  padding: 18px 22px;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.drift-drawer__loading,
.drift-drawer__empty {
  font-size: 14px;
  color: var(--ink-faint);
  font-style: italic;
}
.drift-drawer__section-head {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 8px;
}
.drift-drawer__section-label {
  font-family: var(--label);
  font-size: 12px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 600;
}
.drift-drawer__section-rule {
  flex: 1;
  height: 1px;
  background: var(--paper-line);
}
.drift-drawer__section-count {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  color: var(--ink-faint);
}
.drift-drawer__items {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.drift-drawer__item {
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding: 6px 10px;
  background: var(--paper-shade);
  border-left: 2px solid var(--note);
  border-radius: 0 3px 3px 0;
  font-size: 14px;
  color: var(--ink);
}
.drift-drawer__item--removed {
  background: rgba(164, 72, 72, 0.06);
  border-left-color: var(--bad);
}
.drift-drawer__item--added {
  background: rgba(84, 124, 74, 0.06);
  border-left-color: var(--good);
}
.drift-drawer__sigil {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  font-weight: 700;
  width: 12px;
  text-align: center;
  flex: none;
  color: var(--note);
}
.drift-drawer__item--removed .drift-drawer__sigil { color: var(--bad); }
.drift-drawer__item--added .drift-drawer__sigil { color: var(--good); }
.drift-drawer__footer {
  padding: 16px 22px;
  border-top: 1px solid var(--paper-line);
  background: var(--paper-shade);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
</style>

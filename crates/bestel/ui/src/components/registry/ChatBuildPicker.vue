<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount } from 'vue';
import { useRegistryStore } from '../../stores/registry';
import FreshPill from '../sheet/FreshPill.vue';
import DriftChipStrip, { type DriftSignatures } from '../sheet/DriftChipStrip.vue';
import RunicIcon from '../runic/RunicIcon.vue';
import type { RegistryEntryDto } from '../../api/types';

const props = defineProps<{
  /** Identifier of the currently-selected registry entry for this chat,
   *  or null when the active build is ad-hoc (or absent). */
  activeRegistryId: number | null;
  /** Displayed name of the live attached build, even when ad-hoc. */
  activeBuildName: string | null;
}>();
const emit = defineEmits<{
  (e: 'pick', id: number): void;
  (e: 'add-adhoc'): void;
  (e: 'manage'): void;
}>();

const registry = useRegistryStore();
const open = ref(false);
const trigger = ref<HTMLElement | null>(null);

onMounted(() => {
  registry.load();
  document.addEventListener('mousedown', handleOutsideClick);
});
onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleOutsideClick);
});

function handleOutsideClick(ev: MouseEvent) {
  if (!open.value) return;
  const target = ev.target as Node | null;
  if (target && trigger.value && !trigger.value.contains(target)) {
    open.value = false;
  }
}

function pick(id: number) {
  emit('pick', id);
  open.value = false;
}

function adhoc() {
  emit('add-adhoc');
  open.value = false;
}

function manage() {
  emit('manage');
  open.value = false;
}

const visibleEntries = computed<RegistryEntryDto[]>(() => registry.entries.slice(0, 8));

function entryStatus(e: RegistryEntryDto) {
  if (!e.linked_sheet_id) return 'no-sheet';
  return 'fresh';
}

const dummySigs: DriftSignatures = {
  identity: 'match',
  tree: 'match',
  gear: 'match',
  skill: 'match',
  config: 'match',
};
function sigsFor(_e: RegistryEntryDto): DriftSignatures {
  // Without a live comparison we can't compute real drift here — the
  // dropdown displays the entry's last-known shape. The chat-level sheet
  // sidebar carries the authoritative drift indicator.
  return dummySigs;
}
</script>

<template>
  <div ref="trigger" class="chat-build-picker">
    <button
      type="button"
      class="chat-build-picker__trigger"
      :class="{ 'chat-build-picker__trigger--empty': !activeBuildName }"
      @click="open = !open"
    >
      <span class="chat-build-picker__label">build</span>
      <span v-if="activeBuildName" class="chat-build-picker__name">{{ activeBuildName }}</span>
      <span v-else class="chat-build-picker__placeholder">attach a PoB…</span>
      <span class="chat-build-picker__caret">▾</span>
    </button>

    <div v-if="open" class="chat-build-picker__dropdown" role="listbox">
      <div class="chat-build-picker__heading">
        <span>From registry</span>
        <span class="chat-build-picker__rule" />
        <span class="chat-build-picker__count">{{ registry.entries.length }} builds</span>
      </div>
      <div v-if="registry.entries.length === 0" class="chat-build-picker__empty">
        No builds yet. Use “Add ad-hoc PoB…” below or open Settings → Active Builds to register one.
      </div>
      <button
        v-for="e in visibleEntries"
        :key="e.id"
        type="button"
        class="chat-build-picker__item"
        :class="{ 'chat-build-picker__item--active': e.id === activeRegistryId }"
        @click="pick(e.id)"
      >
        <span class="chat-build-picker__game">{{ e.game }}</span>
        <span class="chat-build-picker__item-meta">
          <span class="chat-build-picker__item-name">{{ e.display_name }}</span>
          <span class="chat-build-picker__item-summary">
            {{ e.summary.class }} · {{ e.summary.ascendancy ?? '—' }}
            <template v-if="e.summary.level"> · lvl {{ e.summary.level }}</template>
          </span>
        </span>
        <span class="chat-build-picker__item-status">
          <FreshPill v-if="entryStatus(e) === 'fresh'" dense />
          <span v-else class="chat-build-picker__no-sheet">no sheet</span>
        </span>
      </button>

      <div class="chat-build-picker__footer">
        <button type="button" class="chat-build-picker__footer-row" @click="adhoc">
          <RunicIcon name="plus" :size="14" />
          <span class="chat-build-picker__footer-label">Add ad-hoc PoB…</span>
          <span class="chat-build-picker__tag">session-only</span>
        </button>
        <button type="button" class="chat-build-picker__footer-row" @click="manage">
          <RunicIcon name="gear" :size="13" />
          <span class="chat-build-picker__footer-label">Manage registry…</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.chat-build-picker {
  position: relative;
  display: inline-flex;
}
.chat-build-picker__trigger {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 6px 12px;
  border: 1.4px solid var(--ink-soft);
  border-radius: 16px;
  background: var(--paper-shade);
  font-family: var(--hand);
  color: var(--ink);
  cursor: pointer;
}
.chat-build-picker__trigger--empty {
  border-color: var(--ink-faint);
  border-style: dashed;
  background: transparent;
  color: var(--ink-soft);
}
.chat-build-picker__label {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 600;
}
.chat-build-picker__name {
  font-size: 14px;
  font-weight: 600;
}
.chat-build-picker__placeholder {
  font-size: 13px;
  font-style: italic;
}
.chat-build-picker__caret {
  font-size: 13px;
  color: var(--ink-soft);
}
.chat-build-picker__dropdown {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  min-width: 480px;
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 6px;
  box-shadow: 0 14px 30px rgba(60, 40, 20, 0.14);
  overflow: hidden;
  z-index: 50;
}
.theme-dark .chat-build-picker__dropdown {
  box-shadow: 0 14px 30px rgba(0, 0, 0, 0.4);
}
.chat-build-picker__heading {
  padding: 10px 16px 6px;
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}
.chat-build-picker__rule {
  flex: 1;
  height: 1px;
  background: var(--paper-line);
}
.chat-build-picker__count {
  font-size: 12px;
  color: var(--ink-faint);
}
.chat-build-picker__empty {
  padding: 10px 16px;
  font-size: 13px;
  color: var(--ink-faint);
  font-style: italic;
}
.chat-build-picker__item {
  width: 100%;
  text-align: left;
  background: transparent;
  border: none;
  border-left: 3px solid transparent;
  padding: 10px 16px;
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
}
.chat-build-picker__item:hover {
  background: var(--paper-shade);
}
.chat-build-picker__item--active {
  border-left-color: var(--ink);
  background: var(--paper-shade);
}
.chat-build-picker__game {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: var(--ink-faint);
  padding: 1px 5px;
  border: 1px solid var(--paper-line);
  border-radius: 3px;
  flex: none;
}
.chat-build-picker__item-meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.chat-build-picker__item-name {
  font-size: 14px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.chat-build-picker__item--active .chat-build-picker__item-name {
  font-weight: 600;
}
.chat-build-picker__item-summary {
  font-size: 12px;
  color: var(--ink-soft);
}
.chat-build-picker__no-sheet {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink-faint);
}
.chat-build-picker__footer {
  border-top: 1px solid var(--paper-line);
  padding: 8px 0;
}
.chat-build-picker__footer-row {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 16px;
  background: transparent;
  border: none;
  cursor: pointer;
  text-align: left;
  color: var(--ink);
  font-family: var(--hand);
  font-size: 14px;
}
.chat-build-picker__footer-row:hover {
  background: var(--paper-shade);
}
.chat-build-picker__footer-label {
  flex: 1;
}
.chat-build-picker__tag {
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  padding: 1px 6px;
  border: 1px dashed var(--ink-faint);
  border-radius: 3px;
}
</style>

<script setup lang="ts">
import { computed } from 'vue';
import { storeToRefs } from 'pinia';

import { useChatStore } from '../../stores/chat';
import { useSheetStore } from '../../stores/sheet';
import { useUiStore } from '../../stores/ui';
import RunicModal from '../runic/RunicModal.vue';
import BSSheetFullView from './BSSheetFullView.vue';

/**
 * Modal wrapper that surfaces the active Build Sheet in a centered
 * RunicModal. Triggered from `BSLinkedSheetCard`'s `View full` button
 * via `useUiStore().openSheetModal()`. Body rendering is delegated to
 * `BSSheetFullView` so the same component is reused inside the
 * BuildPicker's sheets pane.
 */

const ui = useUiStore();
const sheet = useSheetStore();
const chat = useChatStore();

const { sheetModalOpen } = storeToRefs(ui);
const { activeSheet } = storeToRefs(sheet);

const game = computed(() => chat.activeBuild?.game ?? 'poe1');

const versionLabel = computed(() => {
  const v = activeSheet.value?.schemaVersion;
  return v ? `Build Sheet · v${v}` : 'Build Sheet';
});

const subtitleText = computed<string | null>(() => {
  const a = activeSheet.value;
  if (!a) return null;
  const parts: string[] = [];
  if (a.name) parts.push(a.name);
  if (a.updatedAt) {
    const ms = new Date(a.updatedAt).getTime();
    if (Number.isFinite(ms)) {
      const delta = Math.max(0, Date.now() - ms);
      if (delta < 60_000) parts.push('updated just now');
      else if (delta < 60 * 60_000) parts.push(`updated ${Math.floor(delta / 60_000)} min ago`);
      else if (delta < 24 * 60 * 60_000) parts.push(`updated ${Math.floor(delta / (60 * 60_000))} h ago`);
      else parts.push(`updated ${Math.floor(delta / (24 * 60 * 60_000))} d ago`);
    }
  }
  return parts.join(' · ') || null;
});

function onUpdate(value: boolean) {
  if (value) ui.openSheetModal();
  else ui.closeSheetModal();
}

function onClose() {
  ui.closeSheetModal();
}
</script>

<template>
  <RunicModal
    :model-value="sheetModalOpen"
    :title="versionLabel"
    :subtitle="subtitleText ?? undefined"
    max-width="lg"
    @update:model-value="onUpdate"
    @close="onClose"
  >
    <BSSheetFullView
      v-if="activeSheet"
      :payload="activeSheet.payload"
      :game="game"
    />
    <div v-else class="bs-modal__empty">
      No active Build Sheet to display.
    </div>

    <template #footer>
      <button type="button" class="bs-modal__close-btn" @click="onClose">Close</button>
    </template>
  </RunicModal>
</template>

<style scoped>
.bs-modal__empty {
  padding: 32px 8px;
  text-align: center;
  color: var(--ink-faint);
  font-family: var(--hand);
  font-style: italic;
}
.bs-modal__close-btn {
  padding: 6px 16px;
  border: 1px solid var(--ink-soft);
  border-radius: 4px;
  background: var(--paper);
  font-family: var(--hand);
  font-size: 13px;
  font-weight: 500;
  color: var(--ink);
  cursor: pointer;
  transition: border-color 0.15s ease, background 0.15s ease;
}
.bs-modal__close-btn:hover {
  border-color: var(--amber);
  background: var(--amber-glow);
}
</style>

<script setup lang="ts">
import { computed } from 'vue';
import { storeToRefs } from 'pinia';

import { useUiStore } from '../../stores/ui';
import RunicIcon from '../runic/RunicIcon.vue';
import PanelItemCard from './PanelItemCard.vue';
import PanelGemDetail from './PanelGemDetail.vue';
import PanelMechanic from './PanelMechanic.vue';
import PanelMarkdown from './PanelMarkdown.vue';

const ui = useUiStore();
const { panelArtifact, panelHasHistory } = storeToRefs(ui);

const kindLabel = computed(() => {
  switch (panelArtifact.value?.type) {
    case 'item-card': return 'Item';
    case 'gem-detail': return 'Gem';
    case 'mechanic': return 'Mechanic';
    case 'markdown': return 'Note';
    default: return '';
  }
});
</script>

<template>
  <aside v-if="panelArtifact" class="panel-view runic-scrollbar">
    <header class="panel-view__head">
      <div class="panel-view__head-left">
        <button
          v-if="panelHasHistory"
          type="button"
          class="panel-view__icon-btn"
          title="Back"
          aria-label="Back"
          @click="ui.goBackPanel()"
        >
          <RunicIcon name="back" :size="16" />
        </button>
        <div class="panel-view__title-block">
          <span v-if="kindLabel" class="panel-view__kind">{{ kindLabel }}</span>
          <h2 class="panel-view__title">{{ panelArtifact.title }}</h2>
        </div>
      </div>
      <button
        type="button"
        class="panel-view__icon-btn"
        title="Close"
        aria-label="Close panel"
        @click="ui.closePanel()"
      >
        <RunicIcon name="close" :size="16" />
      </button>
    </header>

    <div class="panel-view__body">
      <PanelItemCard
        v-if="panelArtifact.type === 'item-card'"
        :payload="panelArtifact.payload"
      />
      <PanelGemDetail
        v-else-if="panelArtifact.type === 'gem-detail'"
        :payload="panelArtifact.payload"
      />
      <PanelMechanic
        v-else-if="panelArtifact.type === 'mechanic'"
        :payload="panelArtifact.payload"
      />
      <PanelMarkdown
        v-else-if="panelArtifact.type === 'markdown'"
        :payload="panelArtifact.payload"
      />
    </div>
  </aside>
</template>

<style scoped>
.panel-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  width: 100%;
  overflow-y: auto;
  background: var(--paper-shade);
  font-family: var(--hand);
  color: var(--ink);
}

.panel-view__head {
  flex: none;
  position: sticky;
  top: 0;
  z-index: 1;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 18px 12px;
  background: var(--paper-shade);
  border-bottom: 1px solid var(--paper-line);
}

.panel-view__head-left {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 10px;
}

.panel-view__title-block {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.panel-view__kind {
  font-family: var(--label);
  font-size: var(--fs-caps);
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: var(--fw-semibold);
}

.panel-view__title {
  margin: 0;
  font-family: var(--hand);
  font-size: var(--fs-h1);
  font-weight: var(--fw-bold);
  line-height: 1.15;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.panel-view__icon-btn {
  flex: none;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  color: var(--ink-soft);
  cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease;
}
.panel-view__icon-btn:hover {
  background: var(--paper-line);
  color: var(--ink);
}

.panel-view__body {
  flex: 1;
  min-height: 0;
  padding: 18px 22px 24px;
}
</style>

<script setup lang="ts">
import { computed } from 'vue';
import { storeToRefs } from 'pinia';

import { useUiStore } from '../../stores/ui';
import { useBuildStore } from '../../stores/build';
import { openLink } from '../../api/tauri';
import { makeWikiUrl } from '../../api/markdown';
import RunicIcon from '../runic/RunicIcon.vue';
import PanelItemCard from './PanelItemCard.vue';
import PanelGemDetail from './PanelGemDetail.vue';
import PanelMechanic from './PanelMechanic.vue';
import PanelMarkdown from './PanelMarkdown.vue';

const ui = useUiStore();
const buildStore = useBuildStore();
const { panelArtifact, panelStack, panelHasHistory } = storeToRefs(ui);
const game = computed(() => buildStore.current?.game ?? 'poe1');

const kindLabel = computed(() => {
  switch (panelArtifact.value?.type) {
    case 'item-card': return 'Item card';
    case 'gem-detail': return 'Gem detail';
    case 'mechanic': return 'Mechanic';
    case 'markdown': return 'Note';
    default: return '';
  }
});

const prevTitle = computed(() => {
  const stack = panelStack.value;
  return stack.length > 1 ? stack[stack.length - 2].title : null;
});

/** Open the wiki page for the current artifact in the in-app webview.
 *  This is the affordance that replaces the per-entity backtick pill in
 *  prose: when an entity has a panel, its panel chrome carries the
 *  external link instead of cluttering the chat with a duplicate pill. */
function onExternal() {
  const artifact = panelArtifact.value;
  if (!artifact) return;
  const url = makeWikiUrl(artifact.title, game.value);
  void openLink(url);
}
</script>

<template>
  <aside v-if="panelArtifact" class="panel-view runic-scrollbar">
    <header class="panel-view__head">
      <div class="panel-view__head-row">
        <span v-if="kindLabel" class="panel-view__kind">{{ kindLabel }}</span>
        <span class="panel-view__head-spacer" />
        <button
          type="button"
          class="panel-view__icon-btn"
          title="Open external"
          aria-label="Open external"
          @click="onExternal"
        >
          <RunicIcon name="open" :size="14" />
        </button>
        <button
          type="button"
          class="panel-view__icon-btn"
          title="Close panel"
          aria-label="Close panel"
          @click="ui.closePanel()"
        >
          <RunicIcon name="close" :size="14" />
        </button>
      </div>
      <button
        v-if="panelHasHistory && prevTitle"
        type="button"
        class="panel-view__back"
        :title="`Back to ${prevTitle}`"
        @click="ui.goBackPanel()"
      >
        <span aria-hidden="true">←</span>
        <span class="panel-view__back-label">{{ prevTitle }}</span>
      </button>
      <h2 class="panel-view__title">{{ panelArtifact.title }}</h2>
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
/* v9 chrome — asymmetric border-radius keeps the hand-drawn feel; right
 * border is heavier (1.5px ink-soft) for a subtle depth cue, the other
 * three sides hairline (1px paper-line). Body bg is paper-shade so the
 * panel reads as a recessed surface compared to the chat. */
.panel-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  width: 100%;
  overflow: hidden;
  background: var(--paper-shade);
  border-top: 1px solid var(--paper-line);
  border-bottom: 1px solid var(--paper-line);
  border-left: 1px solid var(--paper-line);
  border-right: 1.5px solid var(--ink-soft);
  border-radius: 4px 6px 5px 7px / 6px 5px 7px 4px;
  font-family: var(--hand);
  color: var(--ink);
}

.panel-view__head {
  flex: none;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px 18px 12px;
  background: var(--paper);
  border-bottom: 1px solid var(--paper-line);
}

.panel-view__head-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.panel-view__head-spacer {
  flex: 1;
}

.panel-view__kind {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--amber);
  font-weight: 700;
}

.panel-view__back {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0;
  background: transparent;
  border: 0;
  cursor: pointer;
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink-faint);
  text-align: left;
  transition: color 0.15s ease;
}
.panel-view__back:hover {
  color: var(--ink);
}
.panel-view__back-label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.panel-view__title {
  margin: 0;
  font-family: var(--hand);
  font-size: 22px;
  font-weight: 700;
  line-height: 1.15;
  color: var(--ink);
}

.panel-view__icon-btn {
  flex: none;
  width: 26px;
  height: 26px;
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
  background: var(--paper-shade);
  color: var(--ink);
}

.panel-view__body {
  flex: 1;
  min-height: 0;
  padding: 16px 18px 20px;
  overflow-y: auto;
}
</style>

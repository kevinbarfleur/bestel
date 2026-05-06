<script setup lang="ts">
import { computed } from 'vue';
import { RouterView, useRoute } from 'vue-router';
import { storeToRefs } from 'pinia';

import TopBar from './components/topbar/TopBar.vue';
import RunicToast from './components/runic/RunicToast.vue';
import RunicTooltip from './components/runic/RunicTooltip.vue';
import ItemTooltip from './components/runic/ItemTooltip.vue';
import NodeTooltip from './components/runic/NodeTooltip.vue';
import GemTooltip from './components/runic/GemTooltip.vue';
import BuildPicker from './components/build/BuildPicker.vue';
import ModelPicker from './components/build/ModelPicker.vue';
import ChatPicker from './components/chat/ChatPicker.vue';
import { useTooltip } from './composables/useTooltip';
import { useToastsStore } from './stores/toasts';
import { useUiStore } from './stores/ui';

const route = useRoute();
const isComponentsRoute = computed(() => route.path === '/components');
const isDebugRoute = computed(() => route.path === '/debug');
const hideTopBar = computed(() => isComponentsRoute.value || isDebugRoute.value);

const { state: tooltipState } = useTooltip();
const toastsStore = useToastsStore();
const { toasts } = storeToRefs(toastsStore);

const ui = useUiStore();
const { picker } = storeToRefs(ui);

const buildOpen = computed({
  get: () => picker.value === 'build',
  set: (v: boolean) => {
    if (!v) ui.close();
    else ui.openBuild();
  },
});

const modelOpen = computed({
  get: () => picker.value === 'model',
  set: (v: boolean) => {
    if (!v) ui.close();
    else ui.openModel();
  },
});

const chatOpen = computed({
  get: () => picker.value === 'chat',
  set: (v: boolean) => {
    if (!v) ui.close();
    else ui.openChat();
  },
});
</script>

<template>
  <div class="app-shell">
    <TopBar v-if="!hideTopBar" />
    <main class="app-body">
      <RouterView />
    </main>

    <BuildPicker v-model="buildOpen" />
    <ModelPicker v-model="modelOpen" />
    <ChatPicker v-model="chatOpen" />

    <RunicTooltip
      :visible="tooltipState.visible"
      :target="tooltipState.target"
      :title="tooltipState.kind === 'text' ? (tooltipState.title ?? undefined) : undefined"
      :preferred-side="tooltipState.side"
      :card="tooltipState.kind !== 'text'"
    >
      <ItemTooltip
        v-if="tooltipState.kind === 'item'"
        :raw="tooltipState.content"
        :slot-name="tooltipState.title"
      />
      <NodeTooltip
        v-else-if="tooltipState.kind === 'node'"
        :name="tooltipState.title ?? '?'"
        :payload="tooltipState.content"
      />
      <GemTooltip
        v-else-if="tooltipState.kind === 'gem'"
        :payload="tooltipState.content"
      />
      <template v-else>{{ tooltipState.content }}</template>
    </RunicTooltip>

    <Teleport to="body">
      <div class="toast-stack">
        <Transition-group name="toast">
          <RunicToast
            v-for="t in toasts"
            :key="t.id"
            :variant="t.variant"
            :title="t.title"
            @dismiss="toastsStore.dismiss(t.id)"
          >
            <template v-if="t.body">{{ t.body }}</template>
          </RunicToast>
        </Transition-group>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  width: 100vw;
  height: 100vh;
  background: var(--paper-shade);
}

.app-body {
  flex: 1;
  display: flex;
  min-height: 0;
}
</style>

<style>
.toast-stack {
  position: fixed;
  bottom: 1.5rem;
  right: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
  z-index: 11000;
  pointer-events: none;
}
.toast-stack > * { pointer-events: auto; }
.toast-enter-from,
.toast-leave-to {
  transform: translateX(120%);
  opacity: 0;
}
.toast-enter-active,
.toast-leave-active {
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.25s ease;
}
</style>

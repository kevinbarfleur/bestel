<script setup lang="ts">
import { computed } from 'vue';

import { usePromptsStore } from '../../stores/prompts';

const props = defineProps<{ line: number; col: number }>();

const store = usePromptsStore();

const wordCount = computed(() => {
  const text = store.activeContent;
  if (!text) return 0;
  return (text.match(/\S+/g) ?? []).length;
});

const charCount = computed(() => store.activeContent.length);
</script>

<template>
  <div class="pe-status">
    <span class="pe-status__cell pe-status__cell--mono">Ln {{ props.line }}, Col {{ props.col }}</span>
    <span class="pe-status__div" />
    <span class="pe-status__cell">UTF-8</span>
    <span class="pe-status__div" />
    <span class="pe-status__cell">Markdown</span>
    <span class="pe-status__div" />
    <span class="pe-status__cell">{{ wordCount.toLocaleString() }} words · {{ charCount.toLocaleString() }} chars</span>
    <span class="pe-status__spacer" />
    <span class="pe-status__kbd-pair"><span class="pe-status__kbd">⌃S</span><span>save</span></span>
    <span class="pe-status__kbd-pair"><span class="pe-status__kbd">⌃P</span><span>filter</span></span>
    <span class="pe-status__kbd-pair"><span class="pe-status__kbd">⌃W</span><span>close</span></span>
  </div>
</template>

<style scoped>
.pe-status {
  padding: 8px 18px;
  border-top: 1px solid var(--paper-line);
  background: var(--paper-shade);
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 13px;
  color: var(--ink-soft);
}

.pe-status__cell {
  flex: none;
}

.pe-status__cell--mono {
  font-family: var(--mono);
  font-size: 12px;
}

.pe-status__div {
  width: 1px;
  height: 14px;
  background: var(--paper-line);
}

.pe-status__spacer {
  flex: 1;
}

.pe-status__kbd-pair {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.pe-status__kbd {
  font-family: var(--mono);
  font-size: 11px;
  padding: 1px 5px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  color: var(--ink-faint);
}
</style>

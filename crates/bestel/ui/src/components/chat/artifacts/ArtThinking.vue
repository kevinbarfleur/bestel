<script setup lang="ts">
import { computed, ref } from 'vue';

import type { ReasoningSegment } from '../../../stores/chat';

const props = defineProps<{ segment: ReasoningSegment }>();

const open = ref(false);

const teaser = computed(() => {
  const t = props.segment.text.trim().replace(/\s+/g, ' ');
  if (t.length <= 180) return t;
  return t.slice(0, 180).trimEnd() + '…';
});

const stepCount = computed(() => {
  const paras = props.segment.text.split(/\n\s*\n/).filter((p) => p.trim().length).length;
  return Math.max(1, paras);
});

const toggle = () => {
  open.value = !open.value;
};
</script>

<template>
  <div class="art-thinking" @click="toggle">
    <div class="art-thinking__text">
      {{ open ? segment.text : teaser }}
    </div>
    <div class="art-thinking__foot">
      <span>thought for {{ stepCount }}s</span>
      <span class="art-thinking__sep">·</span>
      <span class="art-thinking__toggle">{{ open ? 'collapse' : 'read full thinking' }}</span>
    </div>
  </div>
</template>

<style scoped>
.art-thinking {
  font-family: var(--script);
  font-size: 13px;
  line-height: 1.6;
  color: var(--ink-soft);
  font-style: italic;
  padding-left: 14px;
  border-left: 1px dashed var(--paper-line);
  cursor: pointer;
}
.art-thinking__text {
  white-space: pre-wrap;
}
.art-thinking__foot {
  margin-top: 6px;
  font-family: var(--hand);
  font-size: 11px;
  color: var(--ink-faint);
  display: flex;
  gap: 8px;
}
.art-thinking__sep { opacity: 0.5; }
.art-thinking__toggle {
  text-decoration: underline dotted;
  text-decoration-color: var(--ink-faint);
  text-underline-offset: 2px;
}
</style>

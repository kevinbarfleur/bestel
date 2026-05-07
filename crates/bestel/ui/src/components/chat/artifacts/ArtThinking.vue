<script setup lang="ts">
import { computed, ref } from 'vue';

import type { ReasoningSegment } from '../../../stores/chat';

const props = defineProps<{ segment: ReasoningSegment }>();

const open = ref(false);

const teaser = computed(() => {
  const t = props.segment.text.trim().replace(/\s+/g, ' ');
  if (t.length <= 200) return t;
  return t.slice(0, 200).trimEnd() + '…';
});

const stepCount = computed(() => {
  const paras = props.segment.text
    .split(/\n\s*\n/)
    .filter((p) => p.trim().length).length;
  return Math.max(1, paras);
});

const toggle = () => {
  open.value = !open.value;
};
</script>

<template>
  <div class="art-thinking">
    <span class="art-thinking__tag">thinking</span>
    <div class="art-thinking__body" @click="toggle">
      <div class="art-thinking__text">
        {{ open ? segment.text : teaser }}
      </div>
      <div class="art-thinking__foot">
        <span>thought for {{ stepCount }}s</span>
        <span class="art-thinking__sep">·</span>
        <span class="art-thinking__toggle">{{ open ? 'collapse' : 'read full thinking' }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Marginalia thinking — Kalam italic body, vertical "THINKING" label,
 * 2px solid paper-line border-left. Only italic exception in the UI. */
.art-thinking {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  padding: 4px 0;
}

.art-thinking__tag {
  flex: none;
  margin-top: 4px;
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: var(--fw-semibold);
  font-style: normal;
  /* Vertical orientation — reads top-to-bottom along the thinking column. */
  writing-mode: vertical-rl;
  transform: rotate(180deg);
  white-space: nowrap;
}

.art-thinking__body {
  flex: 1;
  min-width: 0;
  padding-left: 14px;
  border-left: 2px solid var(--paper-line);
  cursor: pointer;
}

.art-thinking__text {
  font-family: var(--script);
  font-size: 14px;
  line-height: 1.6;
  color: var(--ink-soft);
  font-style: italic;
  white-space: pre-wrap;
}

.art-thinking__foot {
  margin-top: 8px;
  font-family: var(--hand);
  font-size: var(--fs-caps);
  color: var(--ink-faint);
  font-style: normal;
  font-weight: var(--fw-regular);
  display: flex;
  gap: 8px;
}
.art-thinking__sep { opacity: 0.5; }
.art-thinking__toggle {
  text-decoration: underline dotted;
  text-decoration-color: var(--ink-faint);
  text-underline-offset: 2px;
}
.art-thinking__toggle:hover {
  color: var(--ink-soft);
  text-decoration-color: var(--amber);
}
</style>

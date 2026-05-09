<script setup lang="ts">
import { computed, ref } from 'vue';

import type { ToolSegment } from '../../stores/chat';

const props = defineProps<{ segment: ToolSegment }>();

const expanded = ref(false);

const statusLabel = computed(() => {
  switch (props.segment.status) {
    case 'running':
      return '· running';
    case 'done':
      return '✓ done';
    case 'failed':
      return '✗ failed';
  }
  return '';
});

const statusTone = computed(() => {
  switch (props.segment.status) {
    case 'done': return 'var(--good)';
    case 'failed': return 'var(--bad)';
    default: return 'var(--ink-faint)';
  }
});

const inlineDetail = computed(() => props.segment.summaryInput ?? props.segment.detail ?? null);

const hasDetails = computed(
  () => Boolean(inlineDetail.value) || Boolean(props.segment.summary) || Boolean(props.segment.output),
);
</script>

<template>
  <div class="tool" :class="{ 'tool--running': segment.status === 'running' }">
    <div class="tool__head">
      <span class="tool__name">{{ segment.name }}</span>
      <span v-if="inlineDetail" class="aside tool__detail">· {{ inlineDetail }}</span>
      <span class="tool__grow" />
      <span v-if="segment.status === 'running'" class="tool__spinner" aria-hidden="true">
        <span class="tool__spinner-dot" />
        <span class="tool__spinner-dot" />
        <span class="tool__spinner-dot" />
      </span>
      <span class="tool__status" :style="{ color: statusTone }">{{ statusLabel }}</span>
      <button
        v-if="hasDetails"
        type="button"
        class="tool__chev"
        @click="expanded = !expanded"
        :aria-expanded="expanded"
      >{{ expanded ? '▾' : '▸' }}</button>
    </div>
    <div v-if="expanded && hasDetails" class="tool__body">
      <p v-if="segment.summary" class="tool__summary">{{ segment.summary }}</p>
      <pre v-if="segment.output" class="tool__output">{{ segment.output }}</pre>
    </div>
  </div>
</template>

<style scoped>
.tool {
  display: flex;
  flex-direction: column;
  gap: 6px;
  /* Hard-bound the tool row to its parent flex item. Without these,
   * a tool with a multi-kilobyte `summary_input` (e.g. the full JSON
   * blob of `sheet_open_interview` inputs) pushed the row past the
   * chat column's right edge — `min-width: 0` lets the row shrink to
   * its container, `max-width: 100%` is a belt-and-braces guard, and
   * the inner ellipsis on `.tool__detail` truncates the visible
   * detail string. The full string is still available behind the
   * expand chevron. */
  min-width: 0;
  max-width: 100%;
}
.tool__head {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex-wrap: nowrap;
  white-space: nowrap;
  /* Same containment as `.tool` — without this, Chrome's flex
   * algorithm declines to shrink `.tool__detail` to zero even when
   * `min-width: 0` is set, because the head's own min-content
   * overflows the parent. The combination forces the ellipsis. */
  min-width: 0;
  max-width: 100%;
  overflow: hidden;
}
.tool__name {
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
  flex: 0 0 auto;
}
.tool__detail {
  font-style: italic;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  /* `flex: 1 1 0` lets the detail fill the remaining horizontal space
   * AND shrink to zero (with `min-width: 0`). This is what makes the
   * ellipsis actually trigger when the input string is huge. */
  flex: 1 1 0;
  min-width: 0;
}
/* `.tool__grow` is now redundant — the detail spans the full mid-row
 * and pushes status/chev to the right via flex distribution. Kept as a
 * hidden no-op to avoid breaking older saved-chat replays that may
 * still reference the class. */
.tool__grow { display: none; }
.tool__status {
  font-family: var(--hand);
  font-size: 11px;
  white-space: nowrap;
}
.tool__chev {
  border: 0;
  background: transparent;
  cursor: pointer;
  font-size: 11px;
  color: var(--ink-faint);
  padding: 0 4px;
}

/* Running indicator — three pulsing amber dots */
.tool__spinner {
  display: inline-flex;
  gap: 3px;
  align-items: center;
  margin-right: 4px;
}
.tool__spinner-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--amber);
  opacity: 0.35;
  animation: tool-pulse 1.1s ease-in-out infinite;
}
.tool__spinner-dot:nth-child(2) { animation-delay: 0.18s; }
.tool__spinner-dot:nth-child(3) { animation-delay: 0.36s; }
@keyframes tool-pulse {
  0%, 80%, 100% { opacity: 0.25; transform: scale(0.85); }
  40%           { opacity: 1; transform: scale(1.1); }
}

.tool--running .tool__name {
  color: var(--amber);
}
.tool__body {
  padding-left: 14px;
  border-left: 1px dotted var(--paper-line);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.tool__summary {
  margin: 0;
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink-soft);
  line-height: 1.5;
}
.tool__output {
  margin: 0;
  padding: 6px 8px;
  background: var(--paper-shade);
  border: 1px solid var(--paper-line);
  border-radius: 3px;
  font-family: 'Consolas', 'Menlo', monospace;
  font-size: 11px;
  color: var(--ink);
  white-space: pre-wrap;
  max-height: 240px;
  overflow: auto;
}
</style>

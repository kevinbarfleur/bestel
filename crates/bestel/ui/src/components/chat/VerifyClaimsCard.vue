<script setup lang="ts">
import { computed, ref } from 'vue';

import type { VerifyClaimsSegment } from '../../stores/chat';

const props = defineProps<{ segment: VerifyClaimsSegment }>();

const expanded = ref(false);

const total = computed(() => props.segment.claims.length);

const correctionsTone = computed(() => {
  if (props.segment.correctionsCount > 0) return 'var(--amber)';
  return 'var(--good)';
});

const summaryLine = computed(() => {
  const total_ = total.value;
  const corr = props.segment.correctionsCount;
  if (corr > 0) {
    return `· ${total_} checked · ${corr} corrected`;
  }
  return `· ${total_} checked`;
});

function statusGlyph(status: string): string {
  switch (status) {
    case 'ok':
      return '✓';
    case 'wrong':
      return '✗';
    case 'unverified':
      return '?';
    default:
      return '·';
  }
}

function statusTone(status: string): string {
  switch (status) {
    case 'ok':
      return 'var(--good)';
    case 'wrong':
      return 'var(--amber)';
    case 'unverified':
      return 'var(--ink-faint)';
    default:
      return 'var(--ink-faint)';
  }
}
</script>

<template>
  <div class="verify" :class="{ 'verify--has-corrections': segment.correctionsCount > 0 }">
    <div class="verify__head">
      <span class="verify__name">verify_claims</span>
      <span class="verify__summary" :style="{ color: correctionsTone }">{{ summaryLine }}</span>
      <span class="verify__grow" />
      <button
        type="button"
        class="verify__chev"
        @click="expanded = !expanded"
        :aria-expanded="expanded"
      >{{ expanded ? '▾' : '▸' }}</button>
    </div>
    <div v-if="expanded" class="verify__body">
      <div
        v-for="(c, i) in segment.claims"
        :key="`${segment.id}-${i}`"
        class="verify__row"
      >
        <span class="verify__glyph" :style="{ color: statusTone(c.status) }">{{ statusGlyph(c.status) }}</span>
        <div class="verify__col">
          <div class="verify__statement">{{ c.statement }}</div>
          <div v-if="c.status === 'wrong' && c.correction" class="verify__fix">
            <span class="verify__lbl">fix:</span> {{ c.correction }}
          </div>
          <div v-else-if="c.evidence_excerpt" class="verify__evidence">
            <span class="verify__lbl">evidence:</span> {{ c.evidence_excerpt }}
          </div>
          <div v-else class="verify__evidence verify__evidence--empty">
            <span class="verify__lbl">evidence:</span> not found in KB
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.verify {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
  max-width: 100%;
}
.verify__head {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex-wrap: nowrap;
  white-space: nowrap;
  min-width: 0;
  max-width: 100%;
  overflow: hidden;
}
.verify__name {
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
  flex: 0 0 auto;
}
.verify__summary {
  font-family: var(--hand);
  font-size: 12px;
  font-style: italic;
  flex: 0 0 auto;
}
.verify__grow {
  flex: 1 1 0;
  min-width: 0;
}
.verify__chev {
  border: 0;
  background: transparent;
  cursor: pointer;
  font-size: 11px;
  color: var(--ink-faint);
  padding: 0 4px;
  flex: 0 0 auto;
}
.verify--has-corrections .verify__name {
  color: var(--amber);
}
.verify__body {
  padding-left: 14px;
  border-left: 1px dotted var(--paper-line);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.verify__row {
  display: flex;
  gap: 8px;
  align-items: flex-start;
}
.verify__glyph {
  flex: 0 0 14px;
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 700;
  line-height: 1.4;
}
.verify__col {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1 1 auto;
}
.verify__statement {
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink);
  line-height: 1.4;
}
.verify__fix {
  font-family: var(--hand);
  font-size: 11.5px;
  color: var(--ink-soft);
  line-height: 1.4;
}
.verify__evidence {
  font-family: var(--hand);
  font-size: 11.5px;
  color: var(--ink-soft);
  line-height: 1.4;
  font-style: italic;
}
.verify__evidence--empty {
  color: var(--ink-faint);
}
.verify__lbl {
  color: var(--ink-faint);
  font-weight: 600;
  font-style: normal;
  margin-right: 3px;
}
</style>

<script setup lang="ts">
import { computed, ref } from 'vue';

import type { ToolSegment } from '../../../stores/chat';
import { openLink } from '../../../api/tauri';

const props = defineProps<{ segment: ToolSegment }>();

interface WikiPagePayload {
  title?: string;
  url?: string;
  sections?: { level?: number; heading?: string; anchor?: string }[];
  plain_text?: string;
}

const expanded = ref(false);

const parsed = computed<WikiPagePayload | null>(() => {
  const out = props.segment.output;
  if (!out) return null;
  try {
    return JSON.parse(out) as WikiPagePayload;
  } catch {
    return null;
  }
});

const headingsPreview = computed(() => {
  const list = parsed.value?.sections ?? [];
  return list
    .filter((s) => (s.level ?? 1) <= 2 && s.heading)
    .slice(0, 6)
    .map((s) => s.heading as string);
});

const excerpt = computed(() => {
  const text = parsed.value?.plain_text ?? '';
  if (!text) return '';
  return text.length > 280 ? `${text.slice(0, 280)}…` : text;
});

const statusLabel = computed(() => {
  switch (props.segment.status) {
    case 'running':
      return '· fetching';
    case 'done':
      return '✓ parsed';
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

const open = () => {
  if (parsed.value?.url) void openLink(parsed.value.url);
};
</script>

<template>
  <div class="art-wiki" :class="{ 'art-wiki--running': segment.status === 'running' }">
    <div class="art-wiki__head">
      <span class="art-wiki__rune">◆</span>
      <span class="art-wiki__kind">wiki</span>
      <a
        v-if="parsed?.url"
        class="link art-wiki__title"
        @click.prevent="open"
      >{{ parsed?.title ?? segment.detail ?? segment.name }}</a>
      <span v-else class="art-wiki__title">{{ parsed?.title ?? segment.detail ?? segment.name }}</span>
      <span class="art-wiki__grow" />
      <span v-if="segment.status === 'running'" class="art-wiki__spinner" aria-hidden="true">
        <span class="art-wiki__spinner-dot" />
        <span class="art-wiki__spinner-dot" />
        <span class="art-wiki__spinner-dot" />
      </span>
      <span class="art-wiki__status" :style="{ color: statusTone }">{{ statusLabel }}</span>
      <button
        v-if="parsed"
        type="button"
        class="art-wiki__chev"
        @click="expanded = !expanded"
        :aria-expanded="expanded"
      >{{ expanded ? '▾' : '▸' }}</button>
    </div>

    <div v-if="parsed && headingsPreview.length" class="art-wiki__sections">
      <span
        v-for="(h, i) in headingsPreview"
        :key="i"
        class="art-wiki__section"
      >{{ h }}</span>
    </div>

    <div v-if="expanded && parsed" class="art-wiki__body">
      <p v-if="excerpt" class="art-wiki__excerpt">{{ excerpt }}</p>
      <p v-if="parsed.url" class="art-wiki__url">
        <a class="link" @click.prevent="open">{{ parsed.url }}</a>
      </p>
    </div>
  </div>
</template>

<style scoped>
.art-wiki {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.art-wiki__head {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex-wrap: nowrap;
}
.art-wiki__rune {
  color: var(--amber);
  font-size: 12px;
}
.art-wiki__kind {
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 600;
}
.art-wiki__title {
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.art-wiki__grow { flex: 1; }
.art-wiki__status {
  font-family: var(--hand);
  font-size: 11px;
  white-space: nowrap;
}
.art-wiki__chev {
  border: 0;
  background: transparent;
  cursor: pointer;
  font-size: 11px;
  color: var(--ink-faint);
  padding: 0 4px;
}

.art-wiki__spinner {
  display: inline-flex;
  gap: 3px;
  align-items: center;
}
.art-wiki__spinner-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--amber);
  opacity: 0.35;
  animation: wiki-pulse 1.1s ease-in-out infinite;
}
.art-wiki__spinner-dot:nth-child(2) { animation-delay: 0.18s; }
.art-wiki__spinner-dot:nth-child(3) { animation-delay: 0.36s; }
@keyframes wiki-pulse {
  0%, 80%, 100% { opacity: 0.25; transform: scale(0.85); }
  40%           { opacity: 1; transform: scale(1.1); }
}
.art-wiki--running .art-wiki__title {
  color: var(--amber);
}

.art-wiki__sections {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding-left: 14px;
}
.art-wiki__section {
  font-family: var(--hand);
  font-size: 11px;
  color: var(--ink-soft);
  border-bottom: 1px dotted var(--paper-line);
}

.art-wiki__body {
  padding-left: 14px;
  border-left: 1px dotted var(--paper-line);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.art-wiki__excerpt {
  margin: 0;
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink-soft);
  line-height: 1.5;
}
.art-wiki__url {
  margin: 0;
  font-size: 11px;
}
</style>

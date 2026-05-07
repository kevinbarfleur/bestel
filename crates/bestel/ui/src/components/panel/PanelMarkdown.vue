<script setup lang="ts">
import { computed } from 'vue';
import { useBuildStore } from '../../stores/build';
import { renderMarkdown } from '../../api/markdown';

export interface MarkdownPayload {
  body_md?: string;
}

const props = defineProps<{ payload: unknown }>();

const buildStore = useBuildStore();

const body = computed(() => {
  const p = (props.payload ?? {}) as Partial<MarkdownPayload>;
  return typeof p.body_md === 'string' ? p.body_md : '';
});

const game = computed(() => buildStore.current?.game ?? 'poe1');

const html = computed(() => renderMarkdown(body.value, game.value));
</script>

<template>
  <div class="panel-md">
    <div v-if="body" class="markdown-body panel-md__body" v-html="html" />
    <p v-else class="panel-md__empty">No content.</p>
  </div>
</template>

<style scoped>
.panel-md {
  display: flex;
  flex-direction: column;
}

/* v9 markdown panel — slightly larger body and looser line-height than the
 * chat surface, since the panel has the reader's full attention and
 * readability beats density. Inline code uses JetBrains Mono per spec. */
.panel-md__body {
  font-size: 15px;
  line-height: 1.65;
}
.panel-md__body :deep(p),
.panel-md__body :deep(ol),
.panel-md__body :deep(ul) {
  margin: 0 0 12px;
}
.panel-md__body :deep(p:last-child),
.panel-md__body :deep(ol:last-child),
.panel-md__body :deep(ul:last-child) {
  margin-bottom: 0;
}
.panel-md__body :deep(code) {
  font-family: 'JetBrains Mono', 'Consolas', 'Menlo', monospace;
  font-size: 13px;
}

.panel-md__empty {
  margin: 0;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink-faint);
}
</style>

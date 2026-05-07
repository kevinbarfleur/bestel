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
    <div v-if="body" class="markdown-body" v-html="html" />
    <p v-else class="panel-md__empty">No content.</p>
  </div>
</template>

<style scoped>
.panel-md {
  display: flex;
  flex-direction: column;
}

.panel-md__empty {
  margin: 0;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-faint);
}
</style>

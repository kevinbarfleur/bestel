<script setup lang="ts">
import { computed } from 'vue';

interface StatusBadge {
  label: string;
  tone?: 'ink' | 'good' | 'bad';
}

const props = withDefaults(
  defineProps<{
    rune?: string;
    kind?: string;
    title?: string;
    meta?: string;
    status?: StatusBadge | null;
    open?: boolean | null;
  }>(),
  {
    rune: '',
    kind: '',
    title: '',
    meta: '',
    status: null,
    open: null,
  },
);

const emit = defineEmits<{ toggle: [] }>();

const statusColor = computed(() => {
  if (!props.status) return 'var(--ink-faint)';
  switch (props.status.tone) {
    case 'good': return 'var(--good)';
    case 'bad':  return 'var(--bad)';
    default:     return 'var(--ink-faint)';
  }
});
</script>

<template>
  <div class="art-head">
    <a v-if="title && kind" class="link art-head__title">{{ title }}</a>
    <span v-else-if="title" class="art-head__title-plain">{{ title }}</span>
    <span v-if="meta" class="aside art-head__meta">{{ meta }}</span>
    <span class="art-head__grow" />
    <span v-if="status" class="art-head__status" :style="{ color: statusColor }">
      {{ status.label }}
    </span>
    <span
      v-if="open !== null"
      class="art-head__chev"
      role="button"
      @click.stop="emit('toggle')"
    >{{ open ? '▾' : '▸' }}</span>
  </div>
</template>

<style scoped>
.art-head {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex-wrap: nowrap;
  white-space: nowrap;
}
.art-head__title {
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
}
.art-head__title-plain {
  font-family: var(--hand-display);
  font-size: 13px;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
}
.art-head__meta {
  white-space: nowrap;
}
.art-head__grow {
  flex: 1;
}
.art-head__status {
  font-family: var(--hand);
  font-size: 11px;
  white-space: nowrap;
}
.art-head__chev {
  cursor: pointer;
  font-size: 11px;
  color: var(--ink-faint);
  padding: 0 4px;
}
</style>

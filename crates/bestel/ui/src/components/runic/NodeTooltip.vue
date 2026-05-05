<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  name: string;
  payload: string;
}>();

const data = computed(() => {
  try {
    const parsed = JSON.parse(props.payload || '{}');
    return {
      kind: typeof parsed.kind === 'string' ? parsed.kind : 'Passive',
      description: typeof parsed.description === 'string' ? parsed.description : '',
      ascendancy: typeof parsed.ascendancy === 'string' ? parsed.ascendancy : null,
    };
  } catch {
    return { kind: 'Passive', description: '', ascendancy: null };
  }
});

interface KindMeta {
  color: string;
  bg: string;
  label: string;
}

const KIND_META: Record<string, KindMeta> = {
  Keystone: { color: 'var(--amber)', bg: 'rgba(175, 96, 37, 0.06)', label: 'keystone' },
  Notable: { color: '#c9a227', bg: 'rgba(201, 162, 39, 0.06)', label: 'notable' },
  Passive: { color: 'var(--ink-soft)', bg: 'transparent', label: 'passive' },
  Mastery: { color: 'var(--el-cold)', bg: 'rgba(58, 110, 138, 0.06)', label: 'mastery' },
  'Ascendancy keystone': { color: 'var(--el-chaos)', bg: 'rgba(110, 61, 128, 0.06)', label: 'ascendancy ◆' },
  'Ascendancy notable': { color: 'var(--el-chaos)', bg: 'rgba(110, 61, 128, 0.04)', label: 'ascendancy' },
  Ascendancy: { color: 'var(--el-chaos)', bg: 'transparent', label: 'ascendancy' },
};

const meta = computed<KindMeta>(
  () => KIND_META[data.value.kind] ?? { color: 'var(--ink-soft)', bg: 'transparent', label: data.value.kind.toLowerCase() },
);

const description = computed(() => data.value.description?.trim() || 'No description in dictionary.');
</script>

<template>
  <div class="node-tt" :style="{ background: meta.bg }">
    <div class="node-tt__band" :style="{ background: meta.color }">
      <span>{{ meta.label }}</span>
      <span v-if="data.ascendancy" class="node-tt__band-ascend">· {{ data.ascendancy }}</span>
    </div>
    <div class="node-tt__body">
      <div class="node-tt__name" :style="{ color: meta.color }">{{ name }}</div>
      <div class="node-tt__sep" />
      <div class="node-tt__desc">{{ description }}</div>
    </div>
  </div>
</template>

<style scoped>
.node-tt {
  font-family: var(--hand);
  margin: -8px -12px;
  padding: 0 0 10px;
  min-width: 220px;
  max-width: 320px;
}
.node-tt__band {
  padding: 4px 12px;
  font-family: var(--label);
  font-size: 9px;
  letter-spacing: 0.20em;
  text-transform: uppercase;
  font-weight: 600;
  color: var(--paper);
  display: flex;
  align-items: center;
  gap: 6px;
}
.node-tt__band-ascend {
  font-family: var(--hand);
  letter-spacing: 0.05em;
  text-transform: none;
  font-size: 11px;
  opacity: 0.95;
}
.node-tt__body {
  padding: 8px 12px 0;
  display: flex;
  flex-direction: column;
}
.node-tt__name {
  font-family: var(--hand-display);
  font-size: 15px;
  font-weight: 700;
  line-height: 1.2;
}
.node-tt__sep {
  height: 1px;
  background: var(--paper-line);
  margin: 6px 0;
}
.node-tt__desc {
  font-family: var(--hand);
  font-size: 13px;
  line-height: 1.5;
  color: var(--ink);
  white-space: pre-wrap;
}
</style>

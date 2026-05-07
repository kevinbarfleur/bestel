<script setup lang="ts">
import { computed } from 'vue';

import { usePromptsStore } from '../../stores/prompts';

interface Heading {
  level: number;
  text: string;
  line: number;
}

const props = defineProps<{ activeLine: number }>();
const emit = defineEmits<{ jump: [line: number] }>();

const store = usePromptsStore();

const headings = computed<Heading[]>(() => {
  const out: Heading[] = [];
  const lines = store.activeContent.split('\n');
  let inFence = false;
  for (let i = 0; i < lines.length; i++) {
    const ln = lines[i];
    if (/^```/.test(ln)) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;
    const m = /^(#{1,3})\s+(.+?)\s*$/.exec(ln);
    if (m) {
      out.push({ level: m[1].length, text: m[2], line: i + 1 });
    }
  }
  return out;
});

const activeHeadingIndex = computed(() => {
  let active = -1;
  for (let i = 0; i < headings.value.length; i++) {
    if (headings.value[i].line <= props.activeLine) {
      active = i;
    } else {
      break;
    }
  }
  return active;
});

function metaPath() {
  return store.activeFile ? `prompts/${store.activeFile}` : '—';
}

function metaLines() {
  if (!store.activeContent) return '0';
  return String(store.activeContent.split('\n').length);
}

function metaKind() {
  const path = store.activeFile;
  if (path == null) return '—';
  return store.kinds[path] === 'shipped' ? 'shipped' : 'custom';
}
</script>

<template>
  <div class="pe-outline">
    <div class="pe-outline__head">
      <span class="pe-outline__rune">◆</span>
      <span>Outline</span>
      <span class="pe-outline__rule" />
    </div>
    <div class="pe-outline__body">
      <div
        v-for="(h, i) in headings"
        :key="i"
        class="pe-outline__row"
        :class="{
          'pe-outline__row--here': i === activeHeadingIndex,
          'pe-outline__row--l1': h.level === 1,
          'pe-outline__row--l2': h.level === 2,
          'pe-outline__row--l3': h.level === 3,
        }"
        @click="emit('jump', h.line)"
      >
        {{ h.text }}
      </div>
      <div v-if="headings.length === 0" class="pe-outline__empty">No headings</div>
    </div>

    <div class="pe-outline__about">
      <div class="pe-outline__about-head">
        <span class="pe-outline__rune">◆</span>
        <span>About this file</span>
        <span class="pe-outline__rule" />
      </div>
      <div class="pe-meta">
        <span class="pe-meta__k">Format</span>
        <span class="pe-meta__rule" />
        <span class="pe-meta__v">markdown</span>
      </div>
      <div class="pe-meta">
        <span class="pe-meta__k">Kind</span>
        <span class="pe-meta__rule" />
        <span class="pe-meta__v">{{ metaKind() }}</span>
      </div>
      <div class="pe-meta">
        <span class="pe-meta__k">Path</span>
        <span class="pe-meta__rule" />
        <span class="pe-meta__v pe-meta__v--mono">{{ metaPath() }}</span>
      </div>
      <div class="pe-meta">
        <span class="pe-meta__k">Lines</span>
        <span class="pe-meta__rule" />
        <span class="pe-meta__v pe-meta__v--mono">{{ metaLines() }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pe-outline {
  border-left: 1px solid var(--paper-line);
  background: var(--paper-shade);
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.pe-outline__head,
.pe-outline__about-head {
  padding: 12px 18px 8px;
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.pe-outline__rune {
  color: var(--amber);
  opacity: 0.6;
}

.pe-outline__rule {
  flex: 1;
  height: 1px;
  background: var(--paper-line);
}

.pe-outline__body {
  flex: 1;
  overflow: auto;
  padding: 0 0 10px;
}

.pe-outline__row {
  padding: 4px 18px;
  font-family: var(--hand);
  font-size: 13px;
  color: var(--ink-soft);
  cursor: pointer;
  border-left: 2px solid transparent;
  background: transparent;
}

.pe-outline__row--l1 {
  font-size: 14px;
  font-weight: 600;
}

.pe-outline__row--l2 {
  padding-left: 30px;
}

.pe-outline__row--l3 {
  padding-left: 42px;
  font-size: 12.5px;
}

.pe-outline__row:hover {
  color: var(--ink);
}

.pe-outline__row--here {
  color: var(--amber);
  border-left: 2px solid var(--amber);
  background: var(--amber-glow);
}

.pe-outline__empty {
  padding: 18px;
  font-size: 13px;
  font-style: italic;
  color: var(--ink-faint);
  text-align: center;
}

.pe-outline__about {
  border-top: 1px solid var(--paper-line);
  padding: 0 18px 14px;
  background: var(--paper);
}

.pe-meta {
  display: flex;
  align-items: baseline;
  gap: 6px;
  margin-top: 3px;
}

.pe-meta__k {
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 500;
  flex: none;
}

.pe-meta__rule {
  flex: 1;
  border-bottom: 1px dotted var(--paper-line);
  transform: translateY(-3px);
}

.pe-meta__v {
  font-family: var(--hand);
  font-size: 13px;
  color: var(--ink);
  flex: 0 0 auto;
  white-space: nowrap;
}

.pe-meta__v--mono {
  font-family: var(--mono);
  font-size: 12px;
}
</style>

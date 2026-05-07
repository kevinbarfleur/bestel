<script setup lang="ts">
import { computed, ref } from 'vue';

import { usePromptsStore } from '../../stores/prompts';
import type { PromptFileMeta } from '../../api/tauri';

const store = usePromptsStore();
const filterInput = ref<HTMLInputElement | null>(null);
const filter = ref('');

const filteredGroups = computed(() => {
  const q = filter.value.trim().toLowerCase();
  if (!q) return store.tree.groups;
  return store.tree.groups
    .map((g) => ({
      ...g,
      items: g.items.filter((it) => it.rel_path.toLowerCase().includes(q)),
    }))
    .filter((g) => g.items.length > 0);
});

defineExpose({
  focusFilter() {
    filterInput.value?.focus();
    filterInput.value?.select();
  },
});

function onClick(item: PromptFileMeta) {
  void store.openFile(item.rel_path);
}

function shortName(rel: string) {
  return rel.split('/').pop() ?? rel;
}

function metaFor(item: PromptFileMeta) {
  if (item.description && item.description.trim().length > 0) {
    return item.description;
  }
  return `${item.line_count} line${item.line_count === 1 ? '' : 's'}`;
}
</script>

<template>
  <div class="pe-tree">
    <div class="pe-tree__filter">
      <svg viewBox="0 0 14 14" width="14" height="14" aria-hidden="true">
        <circle cx="6" cy="6" r="4" stroke="currentColor" stroke-width="1.2" fill="none" />
        <path d="M9 9 L13 13" stroke="currentColor" stroke-width="1.4" />
      </svg>
      <input
        ref="filterInput"
        v-model="filter"
        type="text"
        placeholder="Filter files…"
        spellcheck="false"
      />
      <span class="pe-tree__kbd">⌃P</span>
    </div>

    <div class="pe-tree__body">
      <div v-for="group in filteredGroups" :key="group.label" class="pe-tree__group">
        <div class="pe-tree__group-head">
          <span class="pe-tree__rune">◆</span>
          <span>{{ group.label }}</span>
          <span class="pe-tree__group-rule" />
          <span class="pe-tree__group-count">{{ group.items.length }}</span>
        </div>
        <p v-if="group.description" class="pe-tree__group-desc">{{ group.description }}</p>
        <div
          v-for="item in group.items"
          :key="item.rel_path"
          class="pe-tree__row"
          :class="{ 'pe-tree__row--active': item.rel_path === store.activeFile }"
          @click="onClick(item)"
        >
          <div class="pe-tree__row-main">
            <div class="pe-tree__row-name">
              {{ shortName(item.rel_path) }}
              <span v-if="item.modified_vs_bundled" class="pe-tree__dot" title="Modified vs bundled">•</span>
            </div>
            <div class="pe-tree__row-meta">{{ metaFor(item) }}</div>
          </div>
          <span
            v-if="item.kind === 'shipped' && item.rel_path !== store.activeFile"
            class="pe-tree__badge"
          >shipped</span>
          <span
            v-else-if="item.kind === 'custom'"
            class="pe-tree__badge pe-tree__badge--custom"
          >custom</span>
        </div>
      </div>

      <div v-if="filteredGroups.length === 0" class="pe-tree__empty">
        No files match
      </div>
    </div>

    <div class="pe-tree__footer">
      Drop any <code>.md</code> file into <code>~/.bestel/prompts/</code> to add it.
    </div>
  </div>
</template>

<style scoped>
.pe-tree {
  border-right: 1px solid var(--paper-line);
  background: var(--paper);
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.pe-tree__filter {
  padding: 10px 14px;
  border-bottom: 1px solid var(--paper-line);
  display: flex;
  align-items: center;
  gap: 9px;
  color: var(--ink-faint);
}

.pe-tree__filter input {
  flex: 1;
  min-width: 0;
  background: transparent;
  border: none;
  outline: none;
  font-family: var(--hand);
  font-size: 14px;
  color: var(--ink);
}

.pe-tree__kbd {
  font-family: var(--mono);
  font-size: 11px;
  padding: 1px 5px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  color: var(--ink-faint);
  flex: none;
}

.pe-tree__body {
  flex: 1;
  overflow: auto;
  padding: 4px 0 14px;
}

.pe-tree__group-head {
  padding: 12px 18px 4px;
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

.pe-tree__rune {
  color: var(--amber);
  opacity: 0.6;
}

.pe-tree__group-rule {
  flex: 1;
  height: 1px;
  background: var(--paper-line);
}

.pe-tree__group-count {
  font-weight: 400;
  letter-spacing: 0.02em;
}

.pe-tree__group-desc {
  margin: 4px 18px 6px;
  font-size: 12px;
  font-style: italic;
  line-height: 1.4;
  color: var(--ink-faint);
}

.pe-tree__row {
  padding: 7px 18px;
  border-left: 3px solid transparent;
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 10px;
}

.pe-tree__row:hover {
  background: var(--paper-shade);
}

.pe-tree__row--active {
  border-left: 3px solid var(--ink);
  background: var(--paper-shade);
}

.pe-tree__row-main {
  flex: 1;
  min-width: 0;
}

.pe-tree__row-name {
  font-family: var(--mono);
  font-size: 13px;
  color: var(--ink-soft);
  font-weight: 400;
  display: flex;
  align-items: baseline;
  gap: 7px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pe-tree__row--active .pe-tree__row-name {
  color: var(--ink);
  font-weight: 600;
}

.pe-tree__dot {
  color: var(--amber);
  font-size: 13px;
  font-weight: 700;
  line-height: 1;
}

.pe-tree__row-meta {
  font-family: var(--hand);
  font-size: 12px;
  color: var(--ink-faint);
  margin-top: 1px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pe-tree__badge {
  font-family: var(--label);
  font-size: 9.5px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink-faint);
  font-weight: 600;
  flex: none;
}

.pe-tree__badge--custom {
  color: var(--amber);
  font-weight: 700;
}

.pe-tree__empty {
  padding: 18px;
  font-size: 13px;
  font-style: italic;
  color: var(--ink-faint);
  text-align: center;
}

.pe-tree__footer {
  border-top: 1px solid var(--paper-line);
  padding: 10px 16px;
  font-size: 12.5px;
  color: var(--ink-faint);
  font-style: italic;
  line-height: 1.4;
  background: var(--paper-shade);
}

.pe-tree__footer code {
  font-family: var(--mono);
  font-style: normal;
}
</style>

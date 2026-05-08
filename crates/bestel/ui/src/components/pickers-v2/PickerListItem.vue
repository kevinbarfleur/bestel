<!--
  Picker sidebar list row. Selected vs Active are visually distinct on
  purpose — a model the user TAPPED (selected) might not yet be the one
  POWERING the chat (active). Same for builds (selected vs loaded) and
  chats (selected vs current).

  - Selected: 3 px ink stroke on the left edge + paper-shade fill.
    Triggered on click.
  - Active: an amber `active` chip rendered next to the name.
    Driven by app state.
  - status="off": fades the row to ink-faint (no key, daemon down…).

  `right` slot lets callers drop a status dot, a count, etc.
-->
<script setup lang="ts">
import { computed } from 'vue';

type Status = 'on' | 'warn' | 'off';

const props = withDefaults(
  defineProps<{
    selected?: boolean;
    active?: boolean;
    name: string;
    meta?: string;
    status?: Status;
  }>(),
  { selected: false, active: false, status: 'on' },
);

defineEmits<{ click: [MouseEvent] }>();

const dimmed = computed(() => props.status === 'off');
</script>

<template>
  <div
    class="pli"
    :class="{ 'pli--selected': selected, 'pli--dimmed': dimmed }"
    @click="$emit('click', $event)"
  >
    <div class="pli__main">
      <div class="pli__row">
        <span class="pli__name" :class="{ 'pli__name--selected': selected }">
          {{ name }}
        </span>
        <span v-if="active" class="pli__active-chip">active</span>
      </div>
      <div v-if="meta" class="pli__meta">{{ meta }}</div>
    </div>
    <div v-if="$slots.right" class="pli__right">
      <slot name="right" />
    </div>
  </div>
</template>

<style scoped>
.pli {
  padding: 10px 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  border-left: 3px solid transparent;
  background: transparent;
  cursor: pointer;
  position: relative;
  transition: background 100ms ease, border-color 100ms ease;
}
.pli:hover {
  background: var(--paper-shade);
}
.pli--selected {
  border-left-color: var(--ink);
  background: var(--paper-shade);
}
.pli--dimmed .pli__name,
.pli--dimmed .pli__meta {
  color: var(--ink-faint);
}

.pli__main {
  flex: 1;
  min-width: 0;
}
.pli__row {
  display: flex;
  align-items: baseline;
  gap: 8px;
}
.pli__name {
  font-size: 14px;
  font-weight: 500;
  color: var(--ink);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.pli__name--selected {
  font-weight: 600;
}
.pli__active-chip {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--amber);
  font-weight: 700;
  padding: 1px 6px;
  border: 1px solid var(--amber);
  border-radius: 3px;
  background: var(--amber-glow);
  flex: none;
}
.pli__meta {
  font-size: 14px;
  color: var(--ink-soft);
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.pli__right {
  flex: none;
  text-align: right;
}
</style>

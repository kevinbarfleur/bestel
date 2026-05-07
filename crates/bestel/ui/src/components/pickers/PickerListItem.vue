<script setup lang="ts">
withDefaults(
  defineProps<{
    /** Active = currently the chosen item (e.g. loaded build). Renders an
     *  amber "ACTIVE" chip to the right of the name. */
    active?: boolean;
    /** Highlighted = currently focused via keyboard or hover. Renders a
     *  paper-shade background + 3px ink left bar. */
    highlighted?: boolean;
    /** Disabled = unavailable (e.g. provider key missing). Still focusable. */
    disabled?: boolean;
    /** Custom active-chip label, default "ACTIVE". */
    activeLabel?: string;
  }>(),
  { active: false, highlighted: false, disabled: false, activeLabel: 'ACTIVE' },
);
</script>

<template>
  <button
    type="button"
    class="picker-row"
    :class="{
      'picker-row--active': active,
      'picker-row--highlighted': highlighted,
      'picker-row--disabled': disabled,
    }"
    role="option"
    :aria-selected="highlighted"
    :aria-disabled="disabled"
  >
    <div class="picker-row__body">
      <div class="picker-row__title-line">
        <span class="picker-row__name">
          <slot name="name"><slot /></slot>
        </span>
        <span v-if="active" class="picker-row__active-chip">{{ activeLabel }}</span>
      </div>
      <div v-if="$slots.meta" class="picker-row__meta">
        <slot name="meta" />
      </div>
    </div>
    <div v-if="$slots.right" class="picker-row__right">
      <slot name="right" />
    </div>
  </button>
</template>

<style scoped>
.picker-row {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 10px 16px;
  border: 0;
  border-left: 3px solid transparent;
  background: transparent;
  text-align: left;
  cursor: pointer;
  font-family: var(--hand);
  color: var(--ink);
  transition: background 0.15s ease, border-color 0.15s ease;
}
.picker-row:hover { background: var(--paper-shade); }

.picker-row--highlighted {
  background: var(--paper-shade);
  border-left-color: var(--ink);
}

.picker-row--active {
  background: var(--paper-shade);
}
.picker-row--active .picker-row__name {
  font-weight: var(--fw-semibold);
}

.picker-row--disabled {
  cursor: not-allowed;
  opacity: 0.6;
}
.picker-row--disabled .picker-row__name {
  color: var(--ink-faint);
}

.picker-row__body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.picker-row__title-line {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}

.picker-row__name {
  font-size: var(--fs-body);
  color: var(--ink);
  font-weight: var(--fw-medium);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.picker-row__active-chip {
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--amber);
  font-weight: var(--fw-bold);
  padding: 1px 6px;
  border: 1px solid var(--amber);
  border-radius: 3px;
  background: var(--amber-glow);
  flex: none;
  line-height: 1.4;
}

.picker-row__meta {
  font-size: var(--fs-meta);
  color: var(--ink-soft);
  font-weight: var(--fw-regular);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.picker-row__right {
  flex: none;
  text-align: right;
}
</style>

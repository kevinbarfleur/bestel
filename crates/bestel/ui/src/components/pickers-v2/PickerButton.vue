<!--
  Single button component covering the v2 family A trio:
  - variant="primary" — solid ink fill, paper text, the "Use this model" CTA
  - variant="outline" — paper bg, ink border, secondary action like Cancel
  - variant="link"    — no border, accent text with dotted underline
                       (replace key, remove, etc.)
  - variant="icon"    — square ghost icon button (close, theme toggle)

  Variants converge in a single component because the bulk of the spec
  (radius 4 px, 0.01 em letter-spacing, 1.2 line-height, EB Garamond at
  14-15 px) is shared. The `danger` modifier flips primary fill to red
  and tints link underlines.
-->
<script setup lang="ts">
import { computed } from 'vue';
import PickerIcon from './PickerIcon.vue';
import PickerKbd from './PickerKbd.vue';

type Variant = 'primary' | 'outline' | 'link' | 'icon';

const props = withDefaults(
  defineProps<{
    variant?: Variant;
    icon?: string;
    iconRight?: string;
    kbd?: string;
    danger?: boolean;
    fullWidth?: boolean;
    disabled?: boolean;
    label?: string;
    iconSize?: number;
  }>(),
  { variant: 'primary', iconSize: 16 },
);

defineEmits<{ click: [MouseEvent] }>();

const cls = computed(() => ({
  pbtn: true,
  [`pbtn--${props.variant}`]: true,
  'pbtn--danger': !!props.danger,
  'pbtn--full': !!props.fullWidth,
}));
</script>

<template>
  <button
    type="button"
    :class="cls"
    :disabled="disabled"
    :title="label"
    :aria-label="label"
    @click="$emit('click', $event)"
  >
    <PickerIcon v-if="icon" :name="(icon as any)" :size="iconSize" />
    <span v-if="$slots.default || (variant !== 'icon')" class="pbtn__label">
      <slot />
    </span>
    <PickerIcon v-if="iconRight" :name="(iconRight as any)" :size="iconSize" />
    <PickerKbd v-if="kbd" :on-dark="variant === 'primary' && !disabled">{{ kbd }}</PickerKbd>
  </button>
</template>

<style scoped>
.pbtn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 9px;
  border-radius: 4px;
  font-family: var(--hand);
  font-weight: 600;
  letter-spacing: 0.01em;
  line-height: 1.2;
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
}
.pbtn:disabled {
  cursor: not-allowed;
}
.pbtn--full {
  width: 100%;
}

/* Primary — solid ink fill */
.pbtn--primary {
  padding: 11px 20px;
  background: var(--ink);
  color: var(--paper);
  border: 1px solid var(--ink);
  font-size: 15px;
}
.pbtn--primary:hover:not(:disabled) {
  background: var(--ink-soft);
  border-color: var(--ink-soft);
}
.pbtn--primary:disabled {
  background: var(--paper-shade);
  color: var(--ink-faint);
  border-color: var(--paper-line);
}
.pbtn--primary.pbtn--danger:not(:disabled) {
  background: var(--bad);
  border-color: var(--bad);
}
.pbtn--primary.pbtn--danger:hover:not(:disabled) {
  filter: brightness(1.06);
}

/* Outline — paper bg, ink border */
.pbtn--outline {
  padding: 10px 19px;
  background: var(--paper);
  color: var(--ink);
  border: 1px solid var(--ink-soft);
  font-size: 14.5px;
  font-weight: 500;
}
.pbtn--outline:hover:not(:disabled) {
  background: var(--paper-shade);
  border-color: var(--ink);
}
.pbtn--outline:disabled {
  color: var(--ink-faint);
  border-color: var(--paper-line);
}

/* Link — secondary text action with dotted underline */
.pbtn--link {
  padding: 4px 0;
  background: transparent;
  border: none;
  font-size: 14px;
  font-weight: 500;
  color: var(--amber);
  text-decoration: underline;
  text-decoration-style: dotted;
  text-decoration-color: var(--amber-soft);
  text-underline-offset: 3px;
  letter-spacing: 0;
  gap: 5px;
}
.pbtn--link:hover:not(:disabled) {
  color: var(--amber-soft);
  text-decoration-style: solid;
}
.pbtn--link.pbtn--danger {
  color: var(--bad);
  text-decoration-color: rgba(164, 72, 72, 0.5);
}

/* Icon — square ghost button */
.pbtn--icon {
  width: 32px;
  height: 32px;
  padding: 0;
  background: transparent;
  color: var(--ink-soft);
  border: 1px solid transparent;
  gap: 0;
}
.pbtn--icon:hover:not(:disabled) {
  background: var(--paper-shade);
  color: var(--ink);
}
.pbtn--icon .pbtn__label {
  display: none;
}
</style>

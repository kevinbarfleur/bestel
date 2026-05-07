<script setup lang="ts">
import { computed } from 'vue';
import { RouterLink } from 'vue-router';
import RunicIcon from './RunicIcon.vue';

type RunicIconName =
  | 'plus' | 'check' | 'arrow' | 'open' | 'trash' | 'search'
  | 'close' | 'eye' | 'save' | 'sun' | 'moon'
  | 'gear' | 'reload' | 'back' | 'forward';

interface Props {
  to?: string;
  href?: string;
  external?: boolean;
  variant?: 'primary' | 'secondary' | 'ghost' | 'twitch' | 'youtube' | 'danger';
  size?: 'xs' | 'sm' | 'md' | 'lg';
  /** Legacy SVG icon set (twitch/youtube/collection/etc.) — rendered inline. */
  icon?:
    | RunicIconName
    | 'twitch'
    | 'youtube'
    | 'collection'
    | 'catalogue'
    | 'arrow-right'
    | 'external'
    | 'logout'
    | 'document'
    | 'settings'
    | 'record'
    | 'play'
    | 'edit';
  /** Optional second icon on the right side of the label (v5 set only). */
  iconRight?: RunicIconName;
  iconOnly?: boolean;
  runeLeft?: string;
  runeRight?: string;
  noRunes?: boolean;
  disabled?: boolean;
  /** Mark the primary as destructive (red background instead of ink). */
  danger?: boolean;
  /** Optional keyboard shortcut chip rendered at the right edge of the button
   *  (e.g. "⏎", "⌘N"). When set, the right rune is suppressed automatically. */
  kbd?: string;
  /** When the button is disabled, show this label instead of the slot content
   *  (e.g. "API key required", "Already active"). Lets us reuse a single
   *  primary CTA component for stateful actions. */
  disabledReason?: string;
}

const RUNIC_V5_ICONS: ReadonlySet<string> = new Set([
  'plus', 'check', 'arrow', 'open', 'trash', 'search',
  'close', 'eye', 'save', 'sun', 'moon',
  'gear', 'reload', 'back', 'forward',
]);

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  external: true,
  iconOnly: false,
  runeLeft: '◆',
  runeRight: '◆',
  noRunes: false,
  disabled: false,
  danger: false,
});

const showLeftRune = computed(
  () => !!props.runeLeft && !props.icon && !props.noRunes && !props.kbd,
);
const showRightRune = computed(
  () => !!props.runeRight && !props.icon && !props.noRunes && !props.kbd && !props.iconRight,
);

const isV5Icon = computed(() => !!props.icon && RUNIC_V5_ICONS.has(props.icon));
const isLegacyIcon = computed(() => !!props.icon && !RUNIC_V5_ICONS.has(props.icon));

const emit = defineEmits<{
  click: [event: MouseEvent];
}>();

const componentTag = computed(() => {
  if (props.to) return RouterLink;
  if (props.href) return 'a';
  return 'button';
});

const componentProps = computed(() => {
  if (props.to) return { to: props.to };
  if (props.href) {
    if (props.external) {
      return { href: props.href, target: '_blank', rel: 'noopener noreferrer' };
    }
    return { href: props.href };
  }
  return { type: 'button', disabled: props.disabled };
});

const handleClick = (event: MouseEvent) => {
  if (!props.disabled) emit('click', event);
};
</script>

<template>
  <component
    :is="componentTag"
    v-bind="componentProps"
    class="runic-button"
    :class="[
      `runic-button--${variant}`,
      `runic-button--${size}`,
      { 'runic-button--disabled': disabled },
      { 'runic-button--danger': danger },
      { 'runic-button--has-icon': icon },
      { 'runic-button--icon-only': iconOnly },
    ]"
    @click="handleClick"
  >
    <RunicIcon v-if="isV5Icon" :name="(icon as RunicIconName)" :size="size === 'sm' || size === 'xs' ? 14 : 16" class="runic-button__icon" />
    <svg v-else-if="icon === 'twitch'" class="runic-button__icon" viewBox="0 0 24 24" fill="currentColor">
      <path d="M11.571 4.714h1.715v5.143H11.57zm4.715 0H18v5.143h-1.714zM6 0L1.714 4.286v15.428h5.143V24l4.286-4.286h3.428L22.286 12V0zm14.571 11.143l-3.428 3.428h-3.429l-3 3v-3H6.857V1.714h13.714z" />
    </svg>
    <svg v-else-if="icon === 'youtube'" class="runic-button__icon" viewBox="0 0 24 24" fill="currentColor">
      <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z" />
    </svg>
    <svg v-else-if="icon === 'collection'" class="runic-button__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path stroke-linecap="round" stroke-linejoin="round" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
    </svg>
    <svg v-else-if="icon === 'catalogue'" class="runic-button__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path stroke-linecap="round" stroke-linejoin="round" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
    </svg>
    <svg v-else-if="icon === 'arrow-right'" class="runic-button__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path stroke-linecap="round" stroke-linejoin="round" d="M14 5l7 7m0 0l-7 7m7-7H3" />
    </svg>
    <svg v-else-if="icon === 'external'" class="runic-button__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path stroke-linecap="round" stroke-linejoin="round" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
    </svg>
    <svg v-else-if="icon === 'logout'" class="runic-button__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path stroke-linecap="round" stroke-linejoin="round" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
    </svg>
    <svg v-else-if="icon === 'document'" class="runic-button__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
    </svg>
    <svg v-else-if="icon === 'settings'" class="runic-button__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
      <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
    </svg>
    <svg v-else-if="icon === 'close'" class="runic-button__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
    </svg>
    <svg v-else-if="icon === 'record'" class="runic-button__icon runic-button__icon--record" viewBox="0 0 24 24" fill="currentColor">
      <circle cx="12" cy="12" r="8" />
    </svg>
    <svg v-else-if="icon === 'play'" class="runic-button__icon" viewBox="0 0 24 24" fill="currentColor">
      <polygon points="6 4 20 12 6 20 6 4" />
    </svg>
    <svg v-else-if="icon === 'edit'" class="runic-button__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path stroke-linecap="round" stroke-linejoin="round" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
    </svg>

    <span v-if="showLeftRune" class="runic-button__rune runic-button__rune--left">{{ runeLeft }}</span>
    <span v-if="!iconOnly" class="runic-button__text">
      <template v-if="disabled && disabledReason">{{ disabledReason }}</template>
      <slot v-else />
    </span>
    <RunicIcon v-if="iconRight" :name="iconRight" :size="size === 'sm' || size === 'xs' ? 14 : 16" class="runic-button__icon runic-button__icon--right" />
    <span v-if="kbd" class="runic-button__kbd">{{ kbd }}</span>
    <span v-if="showRightRune" class="runic-button__rune runic-button__rune--right">{{ runeRight }}</span>
  </component>
</template>

<style scoped>
.runic-button {
  position: relative;
  display: inline-flex;
  flex-direction: row;
  flex-wrap: nowrap;
  align-items: center;
  justify-content: center;
  gap: 9px;
  font-family: var(--hand);
  font-weight: var(--fw-semibold);
  letter-spacing: 0.01em;
  text-decoration: none;
  white-space: nowrap;
  cursor: pointer;
  border-radius: 4px;
  line-height: 1.2;
  transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  background: var(--paper);
  color: var(--ink);
  border: 1px solid var(--ink-soft);
}

.runic-button::before { content: none; }

.runic-button--xs { padding: 6px 12px; font-size: 12.5px; gap: 6px; }
.runic-button--xs .runic-button__rune { display: none; }

.runic-button--sm { padding: 8px 16px; font-size: 13.5px; gap: 7px; }
.runic-button--md { padding: 11px 20px; font-size: 15px; gap: 9px; }
.runic-button--lg { padding: 13px 24px; font-size: 16px; gap: 10px; }

.runic-button--primary {
  color: var(--paper);
  background: var(--ink);
  border-color: var(--ink);
}
.runic-button--primary:hover {
  background: var(--amber);
  border-color: var(--amber);
}
.runic-button--primary.runic-button--danger {
  background: var(--bad);
  border-color: var(--bad);
}
.runic-button--primary.runic-button--danger:hover {
  filter: brightness(1.08);
}

.runic-button--secondary {
  color: var(--ink);
  background: var(--paper);
  border-color: var(--ink-soft);
}
.runic-button--secondary--md,
.runic-button--secondary {
  font-size: 14.5px;
  padding: 10px 19px;
}
.runic-button--secondary.runic-button--sm {
  font-size: 13.5px;
  padding: 7px 15px;
}
.runic-button--secondary:hover {
  border-color: var(--amber);
  background: var(--paper-shade);
}

.runic-button--ghost {
  color: var(--ink-soft);
  background: transparent;
  border: 1.2px dashed var(--ink-faint);
}
.runic-button--ghost:hover {
  color: var(--ink);
  border-color: var(--amber);
  background: var(--amber-bg);
}

.runic-button--twitch {
  color: var(--paper);
  background: #6c3fbf;
  border-color: #6c3fbf;
}
.runic-button--twitch:hover { background: #4f2c8a; border-color: #4f2c8a; }

.runic-button--youtube {
  color: var(--paper);
  background: var(--bad);
  border-color: var(--bad);
}
.runic-button--youtube:hover { background: #8a3434; border-color: #8a3434; }

.runic-button--danger {
  color: var(--paper);
  background: var(--bad);
  border-color: var(--bad);
}
.runic-button--danger:hover { background: #8a3434; border-color: #8a3434; }

.runic-button__rune {
  font-size: 0.7em;
  opacity: 0.55;
  color: var(--amber);
  transition: opacity 0.18s ease;
}
.runic-button:hover .runic-button__rune { opacity: 1; }
.runic-button--primary .runic-button__rune,
.runic-button--danger .runic-button__rune,
.runic-button--youtube .runic-button__rune,
.runic-button--twitch .runic-button__rune { color: var(--paper); opacity: 0.7; }

.runic-button__icon {
  display: inline-block;
  flex-shrink: 0;
  width: 16px;
  height: 16px;
  vertical-align: middle;
  transition: transform 0.3s ease;
}
.runic-button--xs .runic-button__icon { width: 12px; height: 12px; }
.runic-button--sm .runic-button__icon { width: 14px; height: 14px; }
.runic-button--lg .runic-button__icon { width: 18px; height: 18px; }

.runic-button--has-icon { gap: 0.5rem; }
.runic-button--has-icon.runic-button--xs { gap: 0.25rem; }
.runic-button--has-icon.runic-button--sm { gap: 0.375rem; }
.runic-button--has-icon.runic-button--lg { gap: 0.75rem; }

.runic-button--icon-only { gap: 0; aspect-ratio: 1; }
.runic-button--icon-only.runic-button--xs { padding: 0.375rem; }
.runic-button--icon-only.runic-button--sm { padding: 0.5rem; }
.runic-button--icon-only.runic-button--md { padding: 0.625rem; }
.runic-button--icon-only.runic-button--lg { padding: 0.75rem; }

.runic-button:hover .runic-button__icon { transform: scale(1.1); }
.runic-button__icon--record { color: #e53935; }
.runic-button--recording .runic-button__icon--record { animation: pulse-record 1s ease-in-out infinite; }

@keyframes pulse-record {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.6; transform: scale(0.9); }
}

.runic-button__text {
  position: relative;
  display: inline-block;
  vertical-align: middle;
}

.runic-button--disabled {
  cursor: not-allowed;
  pointer-events: none;
}
.runic-button--primary.runic-button--disabled {
  background: var(--paper-shade);
  color: var(--ink-faint);
  border-color: var(--paper-line);
}
.runic-button--secondary.runic-button--disabled {
  background: var(--paper);
  color: var(--ink-faint);
  border-color: var(--paper-line);
}

/* Keyboard shortcut chip — rendered to the right of the label.
 * Picks up paper-tinted background on solid-ink buttons, ink-soft frame
 * on outlined buttons. */
.runic-button__kbd {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.02em;
  font-weight: var(--fw-medium);
  padding: 1px 5px;
  border-radius: 3px;
  border: 1px solid var(--ink-faint);
  color: var(--ink-soft);
  background: transparent;
  line-height: 1.3;
  flex: none;
}
.runic-button--primary .runic-button__kbd {
  border-color: rgba(244, 241, 234, 0.45);
  color: rgba(244, 241, 234, 0.85);
  background: transparent;
}
.runic-button--disabled .runic-button__kbd {
  opacity: 0.55;
  border-color: var(--paper-line);
  color: var(--ink-faint);
  background: transparent;
}

@media (max-width: 640px) {
  .runic-button--xs { padding: 0.3125rem 0.625rem; font-size: 0.6875rem; gap: 0.25rem; }
  .runic-button--sm { padding: 0.5rem 0.875rem; font-size: 0.8125rem; gap: 0.5rem; }
  .runic-button--md { padding: 0.625rem 1.25rem; font-size: 0.875rem; gap: 0.75rem; }
  .runic-button--lg { padding: 0.75rem 1.5rem; font-size: 0.9375rem; gap: 0.875rem; }
  .runic-button--xs .runic-button__icon { width: 10px; height: 10px; }
  .runic-button--sm .runic-button__icon { width: 12px; height: 12px; }
  .runic-button--md .runic-button__icon, .runic-button__icon { width: 14px; height: 14px; }
  .runic-button--lg .runic-button__icon { width: 16px; height: 16px; }
  .runic-button__rune { display: none; }
}
</style>

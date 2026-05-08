<!--
  Picker modal chrome — header / two-column body / footer. The body is
  exposed as a single default slot using `display: grid` with the
  callers responsible for filling the two columns. This keeps the
  modal completely generic: ModelPicker, BuildPicker, ChatPicker all
  drop into the same shell.

  Why a single slot vs `<template #sidebar>` and `<template #detail>`:
  some pickers want a 1-col layout (ChatPicker empty state) and others
  want unequal splits (Settings 240/1fr). Letting the caller own the
  grid is the more flexible primitive.
-->
<script setup lang="ts">
import PickerButton from './PickerButton.vue';
import PickerIcon from './PickerIcon.vue';

withDefaults(
  defineProps<{
    title: string;
    subtitle?: string;
    kbd?: string;
    width?: number | string;
    height?: number | string;
    /** Body grid template — defaults to two-column (340/1fr) */
    grid?: string;
    /** When true, hides the close button (used when caller provides ESC handling) */
    noClose?: boolean;
    /** Hide the bottom hints row entirely */
    noFooter?: boolean;
  }>(),
  {
    width: 1280,
    height: 820,
    grid: '340px 1fr',
    noClose: false,
    noFooter: false,
  },
);

defineEmits<{ close: [] }>();

defineSlots<{
  default: () => any;
  footer: () => any;
}>();
</script>

<template>
  <div class="pmodal">
    <!-- Backdrop. Click eats through to the parent close handler. -->
    <div class="pmodal__backdrop" @click="$emit('close')" />

    <!-- Paper card. Inset so the modal feels like an artefact on the desk. -->
    <div class="pmodal__card" :style="{ width: typeof width === 'number' ? `${width}px` : width, height: typeof height === 'number' ? `${height}px` : height }">
      <header class="pmodal__head">
        <div class="pmodal__head-text">
          <div class="pmodal__title">{{ title }}</div>
          <div v-if="subtitle" class="pmodal__sub">{{ subtitle }}</div>
        </div>
        <span v-if="kbd" class="pmodal__kbd">{{ kbd }}</span>
        <PickerButton
          v-if="!noClose"
          variant="icon"
          icon="close"
          label="Close"
          @click="$emit('close')"
        />
      </header>

      <div class="pmodal__body" :style="{ gridTemplateColumns: grid }">
        <slot />
      </div>

      <footer v-if="!noFooter" class="pmodal__foot">
        <span class="pmodal__hint">
          <span class="pmodal__hint-key">↑↓</span>
          <span>navigate</span>
        </span>
        <span class="pmodal__hint">
          <span class="pmodal__hint-key">⏎</span>
          <span>select</span>
        </span>
        <span class="pmodal__hint">
          <span class="pmodal__hint-key">esc</span>
          <span>close</span>
        </span>
        <span class="pmodal__spacer" />
        <slot name="footer" />
      </footer>
    </div>
  </div>
</template>

<style scoped>
.pmodal {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal, 10000);
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--hand);
  color: var(--ink);
}
.pmodal__backdrop {
  position: absolute;
  inset: 0;
  background: rgba(60, 55, 50, 0.35);
}
:global(.theme-dark) .pmodal__backdrop {
  background: rgba(8, 6, 4, 0.55);
}

.pmodal__card {
  position: relative;
  background: var(--paper);
  border-radius: 6px;
  box-shadow: 0 14px 30px rgba(60, 40, 20, 0.18);
  display: grid;
  grid-template-rows: auto 1fr auto;
  overflow: hidden;
  max-width: calc(100vw - 64px);
  max-height: calc(100vh - 48px);
}
:global(.theme-dark) .pmodal__card {
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.5);
}

.pmodal__head {
  padding: 16px 24px 14px;
  border-bottom: 1px solid var(--paper-line);
  display: flex;
  align-items: center;
  gap: 16px;
}
.pmodal__head-text {
  flex: 1;
  min-width: 0;
}
.pmodal__title {
  font-size: 19px;
  font-weight: 600;
  color: var(--ink);
}
.pmodal__sub {
  font-size: 14px;
  color: var(--ink-soft);
  margin-top: 2px;
}
.pmodal__kbd {
  font-family: var(--label);
  font-size: 13px;
  padding: 3px 8px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  color: var(--ink-soft);
}

.pmodal__body {
  display: grid;
  min-height: 0;
  overflow: hidden;
}

.pmodal__foot {
  padding: 10px 24px;
  border-top: 1px solid var(--paper-line);
  background: var(--paper-shade);
  display: flex;
  align-items: center;
  gap: 18px;
}
.pmodal__hint {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  color: var(--ink-soft);
}
.pmodal__hint-key {
  font-family: var(--label);
  font-size: 12px;
  padding: 2px 6px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  color: var(--ink-soft);
  line-height: 1.2;
}
.pmodal__spacer {
  flex: 1;
}
</style>

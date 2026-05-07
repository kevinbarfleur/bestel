<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue';

import RunicIcon from './RunicIcon.vue';

const props = withDefaults(
  defineProps<{
    modelValue: boolean;
    title?: string;
    /** Optional 13px ink-soft sub-line under the title. */
    subtitle?: string;
    /** Optional keyboard-shortcut chip rendered top-right (e.g. "Ctrl+P"). */
    kbd?: string;
    /** Legacy ornament — flanking ◆ around the title. Not rendered in v2
     *  pane modals (use subtitle + kbd instead). */
    icon?: string;
    maxWidth?: 'sm' | 'md' | 'lg' | 'xl' | 'panes';
    showCloseButton?: boolean;
    closeOnOverlay?: boolean;
    closeOnEscape?: boolean;
  }>(),
  {
    maxWidth: 'md',
    showCloseButton: true,
    closeOnOverlay: true,
    closeOnEscape: true,
  },
);

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  close: [];
}>();

const closeModal = () => {
  emit('update:modelValue', false);
  emit('close');
};

const handleOverlayClick = () => {
  if (props.closeOnOverlay) closeModal();
};

const handleKeydown = (e: KeyboardEvent) => {
  if (props.closeOnEscape && e.key === 'Escape') closeModal();
};

onMounted(() => {
  if (props.closeOnEscape) document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown);
});

watch(
  () => props.closeOnEscape,
  (enabled) => {
    if (enabled) document.addEventListener('keydown', handleKeydown);
    else document.removeEventListener('keydown', handleKeydown);
  },
);
</script>

<template>
  <Teleport to="body">
    <Transition name="runic-modal">
      <div v-if="modelValue" class="runic-modal">
        <div class="runic-modal__overlay" @click="handleOverlayClick" />
        <div
          class="runic-modal__content runic-scrollbar"
          :class="`runic-modal__content--${maxWidth}`"
        >
          <div v-if="title || $slots.header" class="runic-modal__header">
            <slot name="header">
              <div class="runic-modal__title-block">
                <h2 class="runic-modal__title">
                  <span v-if="icon" class="runic-modal__icon">{{ icon }}</span>
                  {{ title }}
                </h2>
                <p v-if="subtitle" class="runic-modal__subtitle">{{ subtitle }}</p>
              </div>
            </slot>
            <span v-if="kbd" class="runic-modal__kbd">{{ kbd }}</span>
            <button
              v-if="showCloseButton"
              type="button"
              class="runic-modal__close"
              aria-label="Close"
              @click="closeModal"
            >
              <RunicIcon name="close" :size="16" />
            </button>
          </div>

          <div class="runic-modal__body">
            <slot />
          </div>

          <div v-if="$slots.footer" class="runic-modal__footer">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.runic-modal {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}

.runic-modal__overlay {
  position: absolute;
  inset: 0;
  background: rgba(40, 35, 30, 0.45);
  backdrop-filter: blur(2px);
}

.runic-modal__content {
  position: relative;
  width: 100%;
  min-width: 320px;
  max-height: 90vh;
  overflow-y: auto;
  background: var(--paper);
  border-radius: 6px;
  border: 1px solid var(--paper-line);
  box-shadow: 0 12px 32px rgba(40, 35, 30, 0.18);
  font-family: var(--hand);
  color: var(--ink);
}

.runic-modal__content--sm { max-width: 24rem; }
.runic-modal__content--md { max-width: 32rem; }
.runic-modal__content--lg { max-width: 42rem; }
.runic-modal__content--xl { max-width: 56rem; }
.runic-modal__content--panes {
  width: 92vw;
  max-width: 1280px;
  min-width: 720px;
  height: 88vh;
  max-height: 820px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  border-radius: 6px;
  border: none;
  box-shadow: 0 18px 40px rgba(60, 40, 20, 0.22);
}
.theme-dark .runic-modal__content--panes {
  box-shadow: 0 22px 50px rgba(0, 0, 0, 0.55);
}
.runic-modal__content--panes .runic-modal__body {
  flex: 1;
  min-height: 0;
  padding: 0;
  overflow: hidden;
}

.runic-modal__header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 24px 14px;
  border-bottom: 1px solid var(--paper-line);
  background: var(--paper);
}

.runic-modal__title-block {
  flex: 1;
  min-width: 0;
}

.runic-modal__title {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin: 0;
  font-family: var(--hand);
  font-size: var(--fs-h2);
  font-weight: var(--fw-semibold);
  color: var(--ink);
  letter-spacing: 0;
  text-transform: none;
}

.runic-modal__subtitle {
  margin: 2px 0 0;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  font-weight: var(--fw-regular);
  color: var(--ink-soft);
  line-height: 1.45;
}

.runic-modal__icon {
  font-size: var(--fs-meta);
  color: var(--amber);
  opacity: 0.65;
}

.runic-modal__kbd {
  flex: none;
  font-family: var(--label);
  font-size: var(--fs-caps);
  padding: 3px 8px;
  border: 1px solid var(--ink-faint);
  border-radius: 3px;
  color: var(--ink-soft);
  background: transparent;
  letter-spacing: 0.06em;
}

.runic-modal__close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  color: var(--ink-soft);
  cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  flex: none;
}
.runic-modal__close:hover {
  color: var(--ink);
  background: var(--paper-shade);
}

.runic-modal__body { padding: 1.1rem 1.25rem; }

.runic-modal__footer {
  display: flex;
  justify-content: flex-end;
  padding: 0.75rem 1.25rem;
  border-top: 1px solid var(--paper-line);
  background: var(--paper);
}

.runic-modal-enter-active,
.runic-modal-leave-active { transition: all 0.3s ease; }
.runic-modal-enter-active .runic-modal__content,
.runic-modal-leave-active .runic-modal__content { transition: all 0.3s ease; }
.runic-modal-enter-from,
.runic-modal-leave-to { opacity: 0; }
.runic-modal-enter-from .runic-modal__content,
.runic-modal-leave-to .runic-modal__content {
  opacity: 0;
  transform: scale(0.95) translateY(-10px);
}

@media (max-width: 640px) {
  .runic-modal { padding: 0.5rem; align-items: flex-end; }
  .runic-modal__content { max-height: 85vh; border-radius: 12px 12px 0 0; }
  .runic-modal__header { padding: 1rem; }
  .runic-modal__title { font-size: 1rem; }
  .runic-modal__body { padding: 1rem; }
  .runic-modal__footer { padding: 1rem; }
}
</style>

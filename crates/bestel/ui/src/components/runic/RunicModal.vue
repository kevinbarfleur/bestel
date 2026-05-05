<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue';

const props = withDefaults(
  defineProps<{
    modelValue: boolean;
    title?: string;
    icon?: string;
    maxWidth?: 'sm' | 'md' | 'lg' | 'xl';
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
              <h2 class="runic-modal__title">
                <span v-if="icon" class="runic-modal__icon">{{ icon }}</span>
                {{ title }}
                <span v-if="icon" class="runic-modal__icon">{{ icon }}</span>
              </h2>
            </slot>
            <button
              v-if="showCloseButton"
              type="button"
              class="runic-modal__close"
              aria-label="Close"
              @click="closeModal"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
              </svg>
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

.runic-modal__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.85rem 1.25rem;
  border-bottom: 1px solid var(--paper-line);
  background: var(--paper);
}

.runic-modal__title {
  display: flex;
  align-items: baseline;
  gap: 0.6rem;
  margin: 0;
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  font-weight: 600;
  color: var(--ink-soft);
}

.runic-modal__icon {
  font-size: 0.8rem;
  color: var(--amber);
  opacity: 0.55;
}

.runic-modal__close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  padding: 0;
  background: transparent;
  border: 0;
  color: var(--ink-faint);
  cursor: pointer;
  transition: color 0.15s ease;
}
.runic-modal__close:hover {
  color: var(--bad);
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

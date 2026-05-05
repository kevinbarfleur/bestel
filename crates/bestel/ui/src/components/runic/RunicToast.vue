<script setup lang="ts">
interface Props {
  variant?: 'info' | 'success' | 'error' | 'warning';
  title?: string;
}

withDefaults(defineProps<Props>(), {
  variant: 'info',
  title: undefined,
});

const emit = defineEmits<{ dismiss: [] }>();
</script>

<template>
  <div class="runic-toast" :class="`runic-toast--${variant}`" role="status">
    <div class="runic-toast__icon">
      <svg v-if="variant === 'info'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" />
        <path stroke-linecap="round" stroke-linejoin="round" d="M12 16v-4m0-4h.01" />
      </svg>
      <svg v-else-if="variant === 'success'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
      </svg>
      <svg v-else-if="variant === 'error'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" />
        <path stroke-linecap="round" stroke-linejoin="round" d="M15 9l-6 6m0-6l6 6" />
      </svg>
      <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01M5.07 19h13.86c1.54 0 2.5-1.67 1.73-3L13.73 4a2 2 0 00-3.46 0L3.34 16c-.77 1.33.19 3 1.73 3z" />
      </svg>
    </div>

    <div class="runic-toast__content">
      <h4 v-if="title" class="runic-toast__title">{{ title }}</h4>
      <div class="runic-toast__body"><slot /></div>
    </div>

    <button class="runic-toast__close" type="button" aria-label="Close" @click="emit('dismiss')">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.runic-toast {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 0.7rem;
  min-width: 280px;
  max-width: 400px;
  padding: 0.7rem 0.8rem;
  font-family: var(--hand);
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-left: 2px solid var(--ink-soft);
  border-radius: 4px;
  color: var(--ink);
  box-shadow: 0 6px 18px rgba(40, 35, 30, 0.15);
}

.runic-toast--info    { border-left-color: var(--el-cold); }
.runic-toast--info    .runic-toast__icon { color: var(--el-cold); }
.runic-toast--success { border-left-color: var(--good); }
.runic-toast--success .runic-toast__icon { color: var(--good); }
.runic-toast--error   { border-left-color: var(--bad); }
.runic-toast--error   .runic-toast__icon { color: var(--bad); }
.runic-toast--warning { border-left-color: var(--note); }
.runic-toast--warning .runic-toast__icon { color: var(--note); }

.runic-toast__icon {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  margin-top: 2px;
}
.runic-toast__icon svg {
  width: 100%;
  height: 100%;
}

.runic-toast__content {
  flex: 1;
  min-width: 0;
}

.runic-toast__title {
  margin: 0 0 0.15rem;
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  font-weight: 600;
  color: var(--ink-soft);
}

.runic-toast__body {
  font-family: var(--hand);
  font-size: 13px;
  line-height: 1.4;
  color: var(--ink-soft);
}

.runic-toast__close {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  margin-top: 1px;
  background: transparent;
  border: 0;
  color: var(--ink-faint);
  cursor: pointer;
  transition: color 0.15s ease;
}
.runic-toast__close:hover {
  color: var(--bad);
}
</style>

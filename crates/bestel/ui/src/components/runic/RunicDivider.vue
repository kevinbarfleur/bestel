<script setup lang="ts">
withDefaults(
  defineProps<{
    rune?: string;
    showRune?: boolean;
    variant?: 'default' | 'subtle' | 'accent';
  }>(),
  {
    rune: '◆',
    showRune: true,
    variant: 'default',
  },
);
</script>

<template>
  <div
    class="runic-divider"
    :class="`runic-divider--${variant}`"
    role="separator"
  >
    <span v-if="showRune" class="runic-divider__rune">{{ rune }}</span>
  </div>
</template>

<style scoped>
.runic-divider {
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 1.25rem 0;
  position: relative;
}

.runic-divider::before,
.runic-divider::after {
  content: "";
  flex: 1;
  height: 1px;
}

.runic-divider--default::before,
.runic-divider--default::after {
  background: var(--ink-faint);
  opacity: 0.5;
}

.runic-divider--default .runic-divider__rune {
  color: var(--ink-faint);
}

.runic-divider--subtle::before,
.runic-divider--subtle::after {
  background: var(--ink-ghost);
  opacity: 0.5;
}

.runic-divider--subtle .runic-divider__rune {
  color: var(--ink-ghost);
}

.runic-divider--accent::before,
.runic-divider--accent::after {
  background: var(--amber);
  opacity: 0.4;
}

.runic-divider--accent .runic-divider__rune {
  color: var(--amber);
}

.runic-divider__rune {
  padding: 0 1rem;
  font-size: 0.5rem;
  transition: color 0.2s ease;
}
</style>

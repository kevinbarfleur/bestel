<script setup lang="ts">
defineProps<{
  /** Section label rendered as small caps with a leading ◆ rune. */
  label: string;
  /** Optional descriptive sub-line under the label. */
  hint?: string;
}>();
</script>

<template>
  <div class="settings-field">
    <div class="settings-field__label">◆ {{ label }}</div>
    <p v-if="hint" class="settings-field__hint">{{ hint }}</p>
    <div class="settings-field__body">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.settings-field {
  display: flex;
  flex-direction: column;
}
.settings-field__label {
  font-family: var(--label);
  font-size: var(--fs-caps);
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: var(--fw-semibold);
  margin-bottom: 4px;
}
.settings-field__hint {
  margin: 0 0 12px;
  max-width: 620px;
  font-family: var(--hand);
  font-size: var(--fs-meta);
  color: var(--ink-soft);
  line-height: 1.5;
}
.settings-field:not(:has(.settings-field__hint)) .settings-field__label {
  margin-bottom: 12px;
}
.settings-field__body {
  display: flex;
  flex-direction: column;
}
</style>

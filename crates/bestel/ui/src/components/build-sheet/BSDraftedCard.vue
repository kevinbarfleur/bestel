<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  title: string;
  status: 'pending' | 'confirmed' | 'edited';
}>();

const emit = defineEmits<{
  (e: 'confirm'): void;
  (e: 'edit'): void;
}>();

const confirmed = computed(() => props.status === 'confirmed');
</script>

<template>
  <div class="bs-draft" :class="{ 'bs-draft--confirmed': confirmed }">
    <div class="bs-draft__head">
      <span class="bs-draft__tag">drafted section</span>
      <span class="bs-draft__rule" />
      <span class="bs-draft__title">{{ title }}</span>
      <span v-if="confirmed" class="bs-draft__check">✓ confirmed</span>
    </div>
    <div class="bs-draft__body">
      <slot />
      <div v-if="!confirmed" class="bs-draft__actions">
        <button
          type="button"
          class="bs-draft__btn bs-draft__btn--primary"
          @click="emit('confirm')"
        >
          ✓ Confirm
        </button>
        <button
          type="button"
          class="bs-draft__btn bs-draft__btn--outline"
          @click="emit('edit')"
        >
          + Edit
        </button>
        <span class="bs-draft__spacer" />
        <span class="bs-draft__hint">You can correct any section later.</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.bs-draft {
  margin-top: 10px;
  border: 1px solid var(--amber);
  border-radius: 5px;
  background: var(--paper);
  overflow: hidden;
}
.bs-draft--confirmed {
  border-color: var(--paper-line);
}
.bs-draft__head {
  padding: 8px 14px;
  border-bottom: 1px solid var(--paper-line);
  background: var(--amber-glow);
  display: flex;
  align-items: center;
  gap: 10px;
}
.bs-draft--confirmed .bs-draft__head {
  background: var(--paper-shade);
}
.bs-draft__tag {
  font-family: var(--label);
  font-size: 11px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--amber);
  font-weight: 700;
}
.bs-draft--confirmed .bs-draft__tag {
  color: var(--ink-soft);
}
.bs-draft__rule {
  flex: 1;
  height: 1px;
  background: var(--paper-line);
}
.bs-draft__title {
  font-size: 14px;
  color: var(--ink);
  font-weight: 600;
}
.bs-draft__check {
  font-family: var(--label);
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--good);
  font-weight: 700;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.bs-draft__body {
  padding: 12px 14px 14px;
}
.bs-draft__actions {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px dotted var(--paper-line);
}
.bs-draft__btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  font-family: var(--hand);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border-radius: 4px;
}
.bs-draft__btn--primary {
  background: var(--ink);
  color: var(--paper);
  border: 1px solid var(--ink);
}
.bs-draft__btn--primary:hover {
  background: var(--ink-soft);
}
.bs-draft__btn--outline {
  background: transparent;
  color: var(--ink);
  border: 1px solid var(--ink-soft);
}
.bs-draft__btn--outline:hover {
  border-color: var(--amber);
  background: var(--amber-glow);
}
.bs-draft__spacer {
  flex: 1;
}
.bs-draft__hint {
  font-size: 13px;
  color: var(--ink-faint);
}
</style>

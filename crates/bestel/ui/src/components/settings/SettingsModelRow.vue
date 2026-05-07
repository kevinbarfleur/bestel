<script setup lang="ts">
defineProps<{
  active: boolean;
  name: string;
  sub: string;
  disabled?: boolean;
}>();
defineEmits<{ pick: [] }>();
</script>

<template>
  <button
    type="button"
    class="model-row"
    :class="{ 'model-row--disabled': disabled }"
    :disabled="disabled"
    @click="!disabled && $emit('pick')"
  >
    <span
      class="model-row__bullet"
      :class="{ 'model-row__bullet--on': active }"
    >
      <span v-if="active" class="model-row__dot" />
    </span>
    <div class="model-row__body">
      <div class="model-row__name">{{ name }}</div>
      <div class="model-row__sub">{{ sub }}</div>
    </div>
    <span v-if="active" class="model-row__chip">boot</span>
  </button>
</template>

<style scoped>
.model-row {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 11px 14px;
  border: 0;
  border-top: 1px dotted var(--paper-line);
  background: transparent;
  cursor: pointer;
  text-align: left;
  font-family: var(--hand);
  color: var(--ink);
  transition: background 0.12s ease;
}
.model-row:first-child {
  border-top: 0;
}
.model-row:hover {
  background: var(--paper-shade);
}
.model-row--disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.model-row--disabled:hover {
  background: transparent;
}

.model-row__bullet {
  width: 14px;
  height: 14px;
  border: 1.5px solid var(--ink-faint);
  border-radius: 50%;
  position: relative;
  flex: none;
}
.model-row__bullet--on {
  border-color: var(--ink);
}
.model-row__dot {
  position: absolute;
  inset: 2.5px;
  background: var(--ink);
  border-radius: 50%;
}

.model-row__body {
  flex: 1;
  min-width: 0;
}
.model-row__name {
  font-size: var(--fs-body);
  font-weight: var(--fw-semibold);
  color: var(--ink);
}
.model-row__sub {
  margin-top: 1px;
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}

.model-row__chip {
  flex: none;
  font-family: var(--label);
  font-size: var(--fs-micro);
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink);
  font-weight: var(--fw-bold);
  padding: 2px 7px;
  border: 1px solid var(--ink);
  border-radius: 3px;
}
</style>

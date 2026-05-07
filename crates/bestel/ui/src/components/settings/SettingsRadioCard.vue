<script setup lang="ts">
import RunicIcon from '../runic/RunicIcon.vue';

type RunicIconName =
  | 'plus' | 'check' | 'arrow' | 'open' | 'trash' | 'search'
  | 'close' | 'eye' | 'save' | 'sun' | 'moon'
  | 'gear' | 'reload' | 'back' | 'forward';

defineProps<{
  selected: boolean;
  icon?: RunicIconName;
  title: string;
  body: string;
  badge?: string;
}>();
defineEmits<{ pick: [] }>();
</script>

<template>
  <button
    type="button"
    class="radio-card"
    :class="{ 'radio-card--selected': selected }"
    @click="$emit('pick')"
  >
    <span
      class="radio-card__bullet"
      :class="{ 'radio-card__bullet--on': selected }"
    >
      <span v-if="selected" class="radio-card__dot" />
    </span>
    <div class="radio-card__body">
      <div class="radio-card__title-row">
        <RunicIcon v-if="icon" :name="icon" :size="16" class="radio-card__icon" />
        <span class="radio-card__title">{{ title }}</span>
        <span v-if="badge" class="radio-card__badge">{{ badge }}</span>
      </div>
      <p class="radio-card__text">{{ body }}</p>
    </div>
  </button>
</template>

<style scoped>
.radio-card {
  display: flex;
  gap: 14px;
  padding: 14px 16px;
  border: 1.5px solid var(--paper-line);
  border-radius: 5px;
  background: var(--paper);
  cursor: pointer;
  text-align: left;
  font-family: var(--hand);
  color: var(--ink);
  transition: border-color 0.15s ease, background 0.15s ease;
}
.radio-card:hover {
  border-color: var(--ink-soft);
}
.radio-card--selected {
  border-color: var(--ink);
  background: var(--paper-shade);
}

.radio-card__bullet {
  width: 16px;
  height: 16px;
  border: 1.5px solid var(--ink-faint);
  border-radius: 50%;
  position: relative;
  flex: none;
  margin-top: 3px;
}
.radio-card__bullet--on {
  border-color: var(--ink);
}
.radio-card__dot {
  position: absolute;
  inset: 3px;
  background: var(--ink);
  border-radius: 50%;
}

.radio-card__body {
  flex: 1;
  min-width: 0;
}

.radio-card__title-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 4px;
}
.radio-card__icon {
  color: var(--ink-soft);
  flex: none;
}
.radio-card__title {
  font-size: var(--fs-h3);
  font-weight: var(--fw-semibold);
  color: var(--ink);
}
.radio-card__badge {
  font-family: var(--label);
  font-size: var(--fs-micro);
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--ink);
  font-weight: var(--fw-bold);
  padding: 1px 7px;
  border: 1px solid var(--ink);
  border-radius: 3px;
}

.radio-card__text {
  margin: 0;
  font-size: var(--fs-meta);
  color: var(--ink-soft);
  line-height: 1.5;
  max-width: 580px;
}
</style>

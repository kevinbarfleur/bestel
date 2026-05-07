<script setup lang="ts">
defineProps<{
  /** 'light' | 'dark' — drives the mini-preview color and the active state. */
  value: 'light' | 'dark';
  selected: boolean;
  label: string;
  sub: string;
}>();
defineEmits<{ pick: [] }>();
</script>

<template>
  <button
    type="button"
    class="theme-card"
    :class="{ 'theme-card--selected': selected, [`theme-card--${value}`]: true }"
    @click="$emit('pick')"
  >
    <span v-if="selected" class="theme-card__chip">selected</span>
    <div class="theme-card__preview" :class="`theme-card__preview--${value}`">
      <div class="theme-card__line theme-card__line--head" />
      <div class="theme-card__line theme-card__line--body theme-card__line--w70" />
      <div class="theme-card__line theme-card__line--body theme-card__line--w55" />
      <div class="theme-card__spacer" />
      <div class="theme-card__btn">BUTTON</div>
    </div>
    <div class="theme-card__caption">
      <div class="theme-card__title">{{ label }}</div>
      <div class="theme-card__sub">{{ sub }}</div>
    </div>
  </button>
</template>

<style scoped>
.theme-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 12px;
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
.theme-card:hover {
  border-color: var(--ink-soft);
}
.theme-card--selected {
  border-color: var(--ink);
  background: var(--paper-shade);
}

.theme-card__chip {
  position: absolute;
  top: 10px;
  right: 12px;
  font-family: var(--label);
  font-size: var(--fs-micro);
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--ink);
  font-weight: var(--fw-bold);
}

.theme-card__preview {
  height: 110px;
  border-radius: 4px;
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.theme-card__preview--light {
  background: #f3f3f1;
  border: 1px solid #d2d0cb;
}
.theme-card__preview--dark {
  background: #161617;
  border: 1px solid #2a2a2c;
}

.theme-card__line {
  border-radius: 2px;
}
.theme-card__line--head {
  height: 4px;
  width: 40%;
}
.theme-card__line--body {
  height: 3px;
}
.theme-card__line--w70 { width: 70%; }
.theme-card__line--w55 { width: 55%; }
.theme-card__preview--light .theme-card__line--head { background: #1d1d1b; }
.theme-card__preview--light .theme-card__line--body { background: rgba(29, 29, 27, 0.4); }
.theme-card__preview--dark .theme-card__line--head { background: #ececea; }
.theme-card__preview--dark .theme-card__line--body { background: rgba(236, 236, 234, 0.45); }

.theme-card__spacer { flex: 1; }

.theme-card__btn {
  align-self: flex-start;
  padding: 3px 8px;
  font-size: 9px;
  font-weight: var(--fw-bold);
  letter-spacing: 0.08em;
  border-radius: 2px;
}
.theme-card__preview--light .theme-card__btn { background: #1d1d1b; color: #f3f3f1; }
.theme-card__preview--dark .theme-card__btn { background: #ececea; color: #161617; }

.theme-card__caption {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.theme-card__title {
  font-size: var(--fs-h3);
  font-weight: var(--fw-semibold);
  color: var(--ink);
}
.theme-card__sub {
  font-size: var(--fs-meta);
  color: var(--ink-soft);
}
</style>

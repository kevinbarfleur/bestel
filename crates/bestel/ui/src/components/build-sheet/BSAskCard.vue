<script setup lang="ts">
withDefaults(
  defineProps<{
    title: string;
    subtitle?: string;
    options: string[];
    selected?: number[];
    multi?: boolean;
    hasOther?: boolean;
  }>(),
  { subtitle: undefined, selected: () => [], multi: false, hasOther: true },
);
</script>

<template>
  <div class="bs-ask">
    <div class="bs-ask__title">{{ title }}</div>
    <div v-if="subtitle" class="bs-ask__subtitle">{{ subtitle }}</div>
    <div class="bs-ask__chips">
      <span
        v-for="(opt, i) in options"
        :key="i"
        class="bs-ask__chip"
        :class="{ 'bs-ask__chip--sel': selected.includes(i) }"
      >
        <span
          v-if="multi"
          class="bs-ask__check"
          :class="{ 'bs-ask__check--sel': selected.includes(i) }"
        >{{ selected.includes(i) ? '✓' : '' }}</span>
        {{ opt }}
      </span>
      <span v-if="hasOther" class="bs-ask__other">Other…</span>
    </div>
    <div v-if="multi" class="bs-ask__hint">
      Pick one or more · <span class="bs-ask__hint-strong">{{ selected.length }} selected</span>
    </div>
  </div>
</template>

<style scoped>
.bs-ask {
  margin-top: 10px;
  border: 1px solid var(--paper-line);
  border-radius: 5px;
  background: var(--paper);
  padding: 14px 16px 16px;
}
.bs-ask__title {
  font-size: 16px;
  font-weight: 600;
  color: var(--ink);
  line-height: 1.3;
}
.bs-ask__subtitle {
  font-size: 14px;
  color: var(--ink-soft);
  line-height: 1.5;
  margin-top: 4px;
}
.bs-ask__chips {
  margin-top: 12px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.bs-ask__chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  border: 1px solid var(--ink-soft);
  border-radius: 18px;
  background: var(--paper);
  color: var(--ink);
  font-family: var(--hand);
  font-size: 14px;
  font-weight: 500;
  line-height: 1.2;
  cursor: pointer;
}
.bs-ask__chip--sel {
  background: var(--ink);
  color: var(--paper);
  border-color: var(--ink);
}
.bs-ask__check {
  width: 12px;
  height: 12px;
  border-radius: 2px;
  border: 1.4px solid var(--ink-soft);
  background: transparent;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--ink);
  font-size: 9px;
  font-weight: 800;
}
.bs-ask__check--sel {
  border-color: var(--paper);
  background: var(--paper);
}
.bs-ask__other {
  display: inline-flex;
  align-items: center;
  padding: 7px 14px;
  border: 1px dashed var(--ink-faint);
  border-radius: 18px;
  background: var(--paper-shade);
  color: var(--ink-faint);
  font-family: var(--hand);
  font-size: 14px;
  font-style: italic;
  line-height: 1.2;
  cursor: text;
}
.bs-ask__hint {
  margin-top: 12px;
  font-size: 13px;
  color: var(--ink-faint);
  font-family: var(--label);
  letter-spacing: 0.04em;
}
.bs-ask__hint-strong {
  color: var(--ink-soft);
}
</style>

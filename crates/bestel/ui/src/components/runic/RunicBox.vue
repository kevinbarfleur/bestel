<script setup lang="ts">
interface Props {
  padding?: 'none' | 'sm' | 'md' | 'lg';
  maxWidth?: string;
  centered?: boolean;
  attached?: boolean;
}

withDefaults(defineProps<Props>(), {
  padding: 'md',
  maxWidth: undefined,
  centered: false,
  attached: false,
});
</script>

<template>
  <div
    class="runic-box"
    :class="[
      `runic-box--padding-${padding}`,
      { 'runic-box--centered': centered },
      { 'runic-box--attached': attached },
    ]"
    :style="maxWidth ? { maxWidth } : {}"
  >
    <div class="runic-box__corner runic-box__corner--tl"></div>
    <div class="runic-box__corner runic-box__corner--tr"></div>
    <div class="runic-box__corner runic-box__corner--bl"></div>
    <div class="runic-box__corner runic-box__corner--br"></div>

    <div class="runic-box__content">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.runic-box {
  position: relative;
  width: 100%;
  background: var(--paper);
  border: 1.4px solid var(--ink-soft);
  border-radius: 4px 7px 5px 6px / 6px 4px 7px 5px;
  font-family: var(--hand);
  color: var(--ink);
}

.runic-box::before { content: none; }

.runic-box--padding-none { padding: 0; }
.runic-box--padding-sm { padding: 0.75rem 1rem; }
.runic-box--padding-md { padding: 1rem 1.25rem; }
.runic-box--padding-lg { padding: 1.5rem 1.75rem; }

@media (min-width: 640px) {
  .runic-box--padding-sm { padding: 1rem 1.5rem; }
  .runic-box--padding-md { padding: 1.5rem 2rem; }
  .runic-box--padding-lg { padding: 2.5rem 3rem; }
}

.runic-box--centered {
  text-align: center;
}

.runic-box--centered .runic-box__content {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.runic-box--attached {
  border-radius: 0 0 6px 6px;
  border-top: none;
}

.runic-box__content {
  position: relative;
  z-index: 1;
}

.runic-box__corner {
  position: absolute;
  width: 20px;
  height: 20px;
  pointer-events: none;
  z-index: 2;
}

.runic-box__corner::before,
.runic-box__corner::after {
  content: "";
  position: absolute;
  background: var(--amber);
  opacity: 0.4;
}

.runic-box__corner--tl { top: 8px; left: 8px; }
.runic-box__corner--tr { top: 8px; right: 8px; transform: rotate(90deg); }
.runic-box__corner--bl { bottom: 8px; left: 8px; transform: rotate(-90deg); }
.runic-box__corner--br { bottom: 8px; right: 8px; transform: rotate(180deg); }

.runic-box__corner::before { width: 20px; height: 1px; top: 0; left: 0; }
.runic-box__corner::after { width: 1px; height: 20px; top: 0; left: 0; }

@media (max-width: 640px) {
  .runic-box--padding-md { padding: 1.25rem 1.5rem; }
  .runic-box--padding-lg { padding: 2rem 2rem; }
}
</style>

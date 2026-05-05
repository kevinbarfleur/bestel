<script setup lang="ts">
interface Props {
  value: number;
  showLabel?: boolean;
  size?: 'sm' | 'md' | 'lg';
  color?: 'default' | 't0' | 't1' | 't2' | 't3' | 'vaal';
}

withDefaults(defineProps<Props>(), {
  showLabel: true,
  size: 'md',
  color: 'default',
});
</script>

<template>
  <div class="runic-progress" :class="[`runic-progress--${size}`, `runic-progress--${color}`]">
    <div class="runic-progress__track">
      <div class="runic-progress__corner runic-progress__corner--tl"></div>
      <div class="runic-progress__corner runic-progress__corner--tr"></div>
      <div class="runic-progress__corner runic-progress__corner--bl"></div>
      <div class="runic-progress__corner runic-progress__corner--br"></div>

      <div
        class="runic-progress__fill"
        :style="{ width: `calc(${Math.min(100, Math.max(0, value))}% - var(--progress-inset, 4px))` }"
      ></div>

      <span v-if="showLabel" class="runic-progress__label">
        {{ Number.isInteger(value) ? value : value.toFixed(1) }}%
      </span>
    </div>
  </div>
</template>

<style scoped>
.runic-progress { width: 100%; }

.runic-progress__track {
  position: relative;
  width: 100%;
  background: var(--paper-shade);
  border-radius: 4px;
  overflow: hidden;
  border: 1.4px solid var(--ink-faint);
}

.runic-progress__corner {
  position: absolute;
  width: 8px;
  height: 8px;
  pointer-events: none;
  z-index: 3;
}
.runic-progress__corner::before,
.runic-progress__corner::after {
  content: "";
  position: absolute;
  background: rgba(var(--progress-accent-rgb, 175, 96, 37), 0.5);
}
.runic-progress__corner--tl { top: 3px; left: 3px; }
.runic-progress__corner--tr { top: 3px; right: 3px; transform: rotate(90deg); }
.runic-progress__corner--bl { bottom: 3px; left: 3px; transform: rotate(-90deg); }
.runic-progress__corner--br { bottom: 3px; right: 3px; transform: rotate(180deg); }
.runic-progress__corner::before { width: 8px; height: 1px; top: 0; left: 0; }
.runic-progress__corner::after { width: 1px; height: 8px; top: 0; left: 0; }

.runic-progress__fill {
  position: absolute;
  top: var(--progress-inset, 4px);
  left: var(--progress-inset, 4px);
  bottom: var(--progress-inset, 4px);
  background:
    repeating-linear-gradient(
      -45deg,
      transparent 0 5px,
      rgba(255, 255, 255, 0.12) 5px 6px
    ),
    var(--progress-fill, var(--amber));
  border-radius: 2px;
  transition: width 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  opacity: 0.85;
}

.runic-progress__label {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 2;
  font-family: var(--hand-display);
  font-weight: 700;
  color: var(--progress-label-color, var(--ink));
  white-space: nowrap;
  pointer-events: none;
}

.runic-progress--sm { --progress-inset: 3px; }
.runic-progress--sm .runic-progress__track { height: 24px; }
.runic-progress--sm .runic-progress__label { font-size: 0.75rem; }
.runic-progress--sm .runic-progress__corner { width: 6px; height: 6px; }
.runic-progress--sm .runic-progress__corner::before { width: 6px; }
.runic-progress--sm .runic-progress__corner::after { height: 6px; }

.runic-progress--md { --progress-inset: 4px; }
.runic-progress--md .runic-progress__track { height: 32px; }
.runic-progress--md .runic-progress__label { font-size: 0.9375rem; }

.runic-progress--lg { --progress-inset: 5px; }
.runic-progress--lg .runic-progress__track { height: 40px; }
.runic-progress--lg .runic-progress__label { font-size: 1.125rem; }
.runic-progress--lg .runic-progress__corner { width: 10px; height: 10px; }
.runic-progress--lg .runic-progress__corner::before { width: 10px; }
.runic-progress--lg .runic-progress__corner::after { height: 10px; }

.runic-progress--default {
  --progress-fill: var(--amber);
  --progress-accent-rgb: 175, 96, 37;
  --progress-label-color: var(--ink);
}
.runic-progress--t0 {
  --progress-fill: #c9a227;
  --progress-accent-rgb: 201, 162, 39;
  --progress-label-color: var(--ink);
}
.runic-progress--t1 {
  --progress-fill: #7a6a8a;
  --progress-accent-rgb: 122, 106, 138;
  --progress-label-color: var(--paper);
}
.runic-progress--t2 {
  --progress-fill: #5a7080;
  --progress-accent-rgb: 90, 112, 128;
  --progress-label-color: var(--paper);
}
.runic-progress--t3 {
  --progress-fill: var(--ink-soft);
  --progress-accent-rgb: 100, 95, 90;
  --progress-label-color: var(--paper);
}
.runic-progress--vaal {
  --progress-fill: var(--bad);
  --progress-accent-rgb: 164, 72, 72;
  --progress-label-color: var(--paper);
}

@media (max-width: 480px) {
  .runic-progress--md .runic-progress__track { height: 28px; }
  .runic-progress--md .runic-progress__label { font-size: 0.8125rem; }
  .runic-progress--lg .runic-progress__track { height: 34px; }
  .runic-progress--lg .runic-progress__label { font-size: 1rem; }
}
</style>

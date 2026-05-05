<script setup lang="ts">
interface Props {
  value: number | string;
  label?: string;
  color?: 'default' | 't0' | 't1' | 't2' | 't3' | 'vaal';
  size?: 'sm' | 'md' | 'lg';
  icon?: string;
  iconSrc?: string;
}

withDefaults(defineProps<Props>(), {
  label: undefined,
  color: 'default',
  size: 'md',
  icon: undefined,
  iconSrc: undefined,
});
</script>

<template>
  <div
    class="runic-number"
    :class="[`runic-number--${color}`, `runic-number--${size}`]"
  >
    <div class="runic-number__cavity">
      <div class="runic-number__corner runic-number__corner--tl"></div>
      <div class="runic-number__corner runic-number__corner--tr"></div>
      <div class="runic-number__corner runic-number__corner--bl"></div>
      <div class="runic-number__corner runic-number__corner--br"></div>

      <img v-if="iconSrc" :src="iconSrc" alt="" class="runic-number__icon-img" />
      <span v-else-if="icon" class="runic-number__icon">{{ icon }}</span>
      <span class="runic-number__value">{{ value }}</span>
    </div>
    <span v-if="label" class="runic-number__label">{{ label }}</span>
  </div>
</template>

<style scoped>
.runic-number {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
}

.runic-number__cavity {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 60px;
  min-height: 48px;
  padding: 0.5rem 0.875rem;
  border-radius: 4px 7px 5px 6px / 6px 4px 7px 5px;
  background: var(--paper-shade);
  border: 1.4px solid var(--ink-soft);
  transition: border-color 0.18s ease;
}

.runic-number:hover .runic-number__cavity {
  border-color: var(--amber);
}

.runic-number__corner {
  position: absolute;
  width: 10px;
  height: 10px;
  pointer-events: none;
  z-index: 2;
}

.runic-number__corner::before,
.runic-number__corner::after {
  content: "";
  position: absolute;
  background: rgba(var(--accent-rgb, 175, 96, 37), 0.5);
}

.runic-number__corner--tl {
  top: 4px;
  left: 4px;
}
.runic-number__corner--tr {
  top: 4px;
  right: 4px;
  transform: rotate(90deg);
}
.runic-number__corner--bl {
  bottom: 4px;
  left: 4px;
  transform: rotate(-90deg);
}
.runic-number__corner--br {
  bottom: 4px;
  right: 4px;
  transform: rotate(180deg);
}

.runic-number__corner::before {
  width: 10px;
  height: 1px;
  top: 0;
  left: 0;
}
.runic-number__corner::after {
  width: 1px;
  height: 10px;
  top: 0;
  left: 0;
}

.runic-number__value {
  position: relative;
  z-index: 1;
  font-family: var(--hand-display);
  font-weight: 700;
  line-height: 1;
  color: var(--number-color, var(--ink));
  transition: color 0.18s ease;
}

.runic-number__label {
  font-family: var(--label);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--ink-faint);
  transition: color 0.18s ease;
}

.runic-number:hover .runic-number__label {
  color: var(--ink-soft);
}

/* Size Variants */
.runic-number--sm .runic-number__cavity {
  min-width: 50px;
  min-height: 40px;
  padding: 0.375rem 0.625rem;
}
.runic-number--sm .runic-number__value {
  font-size: 1.1rem;
}
.runic-number--sm .runic-number__label {
  font-size: 0.75rem;
}
.runic-number--sm .runic-number__corner {
  width: 8px;
  height: 8px;
}
.runic-number--sm .runic-number__corner::before {
  width: 8px;
}
.runic-number--sm .runic-number__corner::after {
  height: 8px;
}
.runic-number--sm {
  gap: 0.375rem;
}

.runic-number--md .runic-number__value {
  font-size: 1.875rem;
}
.runic-number--md .runic-number__label {
  font-size: 0.8125rem;
}

.runic-number--lg .runic-number__cavity {
  min-width: 75px;
  min-height: 58px;
  padding: 0.625rem 1.125rem;
}
.runic-number--lg .runic-number__value {
  font-size: 2.5rem;
}
.runic-number--lg .runic-number__label {
  font-size: 0.875rem;
}
.runic-number--lg .runic-number__corner {
  width: 12px;
  height: 12px;
}
.runic-number--lg .runic-number__corner::before {
  width: 12px;
}
.runic-number--lg .runic-number__corner::after {
  height: 12px;
}

/* Color Variants */
.runic-number--default {
  --number-color: var(--ink);
  --accent-rgb: 175, 96, 37;
}

.runic-number--t0 {
  --number-color: #c9a227;
  --accent-rgb: 201, 162, 39;
}
.runic-number--t0 .runic-number__cavity { border-color: rgba(201, 162, 39, 0.5); }
.runic-number--t0 .runic-number__label { color: rgba(120, 95, 25, 0.85); }

.runic-number--t1 {
  --number-color: #7a6a8a;
  --accent-rgb: 122, 106, 138;
}
.runic-number--t1 .runic-number__cavity { border-color: rgba(122, 106, 138, 0.5); }
.runic-number--t1 .runic-number__label { color: rgba(85, 70, 100, 0.85); }

.runic-number--t2 {
  --number-color: #5a7080;
  --accent-rgb: 90, 112, 128;
}
.runic-number--t2 .runic-number__cavity { border-color: rgba(90, 112, 128, 0.5); }
.runic-number--t2 .runic-number__label { color: rgba(70, 90, 105, 0.85); }

.runic-number--t3 {
  --number-color: var(--ink-soft);
  --accent-rgb: 100, 95, 90;
}
.runic-number--t3 .runic-number__cavity { border-color: var(--ink-faint); }

.runic-number--vaal {
  --number-color: var(--bad);
  --accent-rgb: 164, 72, 72;
}
.runic-number--vaal .runic-number__cavity { border-color: rgba(164, 72, 72, 0.5); }
.runic-number--vaal .runic-number__label { color: rgba(130, 50, 50, 0.85); }

/* Icon */
.runic-number__icon {
  position: relative;
  z-index: 1;
  font-size: 0.85em;
  margin-right: 0.25em;
  opacity: 0.9;
}

.runic-number__icon-img {
  position: relative;
  z-index: 1;
  width: 1.25em;
  height: 1.25em;
  margin-right: 0.25em;
  object-fit: contain;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.5));
}

/* Responsive */
@media (max-width: 480px) {
  .runic-number--md .runic-number__cavity {
    min-width: 48px;
    min-height: 38px;
    padding: 0.375rem 0.5rem;
  }
  .runic-number--md .runic-number__value {
    font-size: 1.375rem;
  }
  .runic-number--md .runic-number__label {
    font-size: 0.6875rem;
  }
  .runic-number--md .runic-number__corner {
    width: 7px;
    height: 7px;
  }
  .runic-number--md .runic-number__corner::before {
    width: 7px;
  }
  .runic-number--md .runic-number__corner::after {
    height: 7px;
  }

  .runic-number--lg .runic-number__cavity {
    min-width: 58px;
    min-height: 46px;
    padding: 0.5rem 0.75rem;
  }
  .runic-number--lg .runic-number__value {
    font-size: 1.625rem;
  }
  .runic-number--lg .runic-number__label {
    font-size: 0.75rem;
  }
  .runic-number--lg .runic-number__corner {
    width: 9px;
    height: 9px;
  }
  .runic-number--lg .runic-number__corner::before {
    width: 9px;
  }
  .runic-number--lg .runic-number__corner::after {
    height: 9px;
  }
}
</style>

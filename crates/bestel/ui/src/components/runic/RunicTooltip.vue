<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';

interface Props {
  visible: boolean;
  target?: { x: number; y: number; width: number; height: number } | null;
  title?: string;
  preferredSide?: 'right' | 'left' | 'bottom' | 'top';
  width?: number;
  card?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  target: null,
  title: undefined,
  preferredSide: 'right',
  width: 280,
  card: false,
});

const tooltipRef = ref<HTMLElement | null>(null);
const position = ref({ top: 0, left: 0, side: 'right' as 'right' | 'left' | 'bottom' | 'top' });

const compute = () => {
  if (!props.target) return;
  const margin = 12;
  const t = props.target;
  const tooltipHeight = tooltipRef.value?.offsetHeight ?? 80;
  const tooltipWidth = tooltipRef.value?.offsetWidth ?? props.width;
  const vw = window.innerWidth;
  const vh = window.innerHeight;

  const trySide = (side: 'right' | 'left' | 'bottom' | 'top'): { top: number; left: number; ok: boolean } => {
    let top = 0, left = 0;
    if (side === 'right') {
      left = t.x + t.width + margin;
      top = t.y + t.height / 2 - tooltipHeight / 2;
      const ok = left + tooltipWidth <= vw - 8;
      return { top, left, ok };
    }
    if (side === 'left') {
      left = t.x - tooltipWidth - margin;
      top = t.y + t.height / 2 - tooltipHeight / 2;
      return { top, left, ok: left >= 8 };
    }
    if (side === 'bottom') {
      left = t.x + t.width / 2 - tooltipWidth / 2;
      top = t.y + t.height + margin;
      return { top, left, ok: top + tooltipHeight <= vh - 8 };
    }
    left = t.x + t.width / 2 - tooltipWidth / 2;
    top = t.y - tooltipHeight - margin;
    return { top, left, ok: top >= 8 };
  };

  const order: Array<'right' | 'left' | 'bottom' | 'top'> = [
    props.preferredSide,
    ...(['right', 'left', 'bottom', 'top'] as const).filter((s) => s !== props.preferredSide),
  ];

  for (const side of order) {
    const { top, left, ok } = trySide(side);
    if (ok) {
      position.value = {
        top: Math.max(8, Math.min(vh - tooltipHeight - 8, top)),
        left: Math.max(8, Math.min(vw - tooltipWidth - 8, left)),
        side,
      };
      return;
    }
  }
  const fallback = trySide(props.preferredSide);
  position.value = {
    top: Math.max(8, Math.min(vh - tooltipHeight - 8, fallback.top)),
    left: Math.max(8, Math.min(vw - tooltipWidth - 8, fallback.left)),
    side: props.preferredSide,
  };
};

watch(() => [props.visible, props.target], () => {
  if (props.visible) requestAnimationFrame(compute);
}, { deep: true, immediate: true });

const onResize = () => { if (props.visible) compute(); };
onMounted(() => {
  window.addEventListener('resize', onResize);
  window.addEventListener('scroll', onResize, true);
});
onUnmounted(() => {
  window.removeEventListener('resize', onResize);
  window.removeEventListener('scroll', onResize, true);
});

const style = computed(() => {
  // In card mode, let the inner component define width (auto). Otherwise use the
  // explicit width prop so the layout calc above is consistent.
  if (props.card) {
    return {
      top: `${position.value.top}px`,
      left: `${position.value.left}px`,
      width: 'auto',
      maxWidth: '380px',
    };
  }
  return {
    top: `${position.value.top}px`,
    left: `${position.value.left}px`,
    width: `${props.width}px`,
  };
});
</script>

<template>
  <Teleport to="body">
    <Transition name="runic-tooltip">
      <div
        v-if="visible && target"
        ref="tooltipRef"
        class="runic-tooltip"
        :class="{ 'runic-tooltip--card': card }"
        :style="style"
        role="tooltip"
      >
        <div v-if="title && !card" class="runic-tooltip__title">{{ title }}</div>
        <div class="runic-tooltip__body">
          <slot />
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.runic-tooltip {
  position: fixed;
  z-index: 10100;
  pointer-events: none;
  font-family: var(--hand);
  background: var(--paper);
  border: 1px solid var(--paper-line);
  border-radius: 4px;
  box-shadow: 0 6px 18px rgba(40, 35, 30, 0.15);
  padding: 8px 12px;
  color: var(--ink);
  overflow: hidden;
}
.runic-tooltip--card {
  padding: 8px 12px;
}

.runic-tooltip__title {
  font-family: var(--label);
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--ink-soft);
  margin: 0 0 6px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--paper-line);
  white-space: nowrap;
}

.runic-tooltip__body {
  font-family: var(--hand);
  font-size: 13px;
  line-height: 1.5;
  color: var(--ink);
  white-space: pre-wrap;
}

.runic-tooltip-enter-active,
.runic-tooltip-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.runic-tooltip-enter-from,
.runic-tooltip-leave-to {
  opacity: 0;
  transform: scale(0.96);
}
</style>

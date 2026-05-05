import { onBeforeUnmount, onMounted, ref } from 'vue';

export type TooltipKind = 'text' | 'item' | 'node' | 'gem';

interface TooltipState {
  visible: boolean;
  target: { x: number; y: number; width: number; height: number } | null;
  title: string | null;
  content: string;
  kind: TooltipKind;
  side: 'right' | 'left' | 'bottom' | 'top';
}

const state = ref<TooltipState>({
  visible: false,
  target: null,
  title: null,
  content: '',
  kind: 'text',
  side: 'right',
});

let listenersInstalled = false;

const findTooltipRoot = (el: Element | null): HTMLElement | null => {
  let cur: Element | null = el;
  while (cur && cur !== document.body) {
    if (cur instanceof HTMLElement && (cur.dataset.tooltipText || cur.dataset.tooltipKey)) {
      return cur;
    }
    cur = cur.parentElement;
  }
  return null;
};

const showFor = (el: HTMLElement) => {
  const text = el.dataset.tooltipText ?? '';
  if (!text) return;
  const title = el.dataset.tooltipTitle ?? null;
  const side = (el.dataset.tooltipSide as TooltipState['side']) ?? 'right';
  const kind = (el.dataset.tooltipKind as TooltipKind) ?? 'text';
  const rect = el.getBoundingClientRect();
  state.value = {
    visible: true,
    target: { x: rect.left, y: rect.top, width: rect.width, height: rect.height },
    title,
    content: text,
    kind,
    side,
  };
};

const hide = () => {
  state.value = { ...state.value, visible: false };
};

let activeEl: HTMLElement | null = null;
let scheduled = 0;
const HOVER_DELAY_MS = 50;

const onPointerMove = (e: PointerEvent) => {
  const root = findTooltipRoot(e.target as Element);
  if (root === activeEl) return;
  activeEl = root;
  window.clearTimeout(scheduled);
  if (!root) {
    hide();
    return;
  }
  scheduled = window.setTimeout(() => {
    if (activeEl) showFor(activeEl);
  }, HOVER_DELAY_MS);
};

const onFocusIn = (e: FocusEvent) => {
  const root = findTooltipRoot(e.target as Element);
  if (!root) return;
  activeEl = root;
  window.clearTimeout(scheduled);
  showFor(root);
};

const onFocusOut = () => {
  activeEl = null;
  hide();
};

const onScroll = () => hide();

export function useTooltip() {
  onMounted(() => {
    if (listenersInstalled) return;
    listenersInstalled = true;
    document.addEventListener('pointermove', onPointerMove, true);
    document.addEventListener('focusin', onFocusIn, true);
    document.addEventListener('focusout', onFocusOut, true);
    document.addEventListener('scroll', onScroll, true);
    window.addEventListener('resize', onScroll);
  });

  onBeforeUnmount(() => {
    if (!listenersInstalled) return;
    listenersInstalled = false;
    document.removeEventListener('pointermove', onPointerMove, true);
    document.removeEventListener('focusin', onFocusIn, true);
    document.removeEventListener('focusout', onFocusOut, true);
    document.removeEventListener('scroll', onScroll, true);
    window.removeEventListener('resize', onScroll);
  });

  return { state };
}

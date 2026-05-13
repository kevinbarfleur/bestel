import { onBeforeUnmount, onMounted } from 'vue';

export interface ShortcutHandlers {
  onBuildPicker?: () => void;
  onModelPicker?: () => void;
  onSettings?: () => void;
  onEscape?: () => void;
  onResetChat?: () => void;
  onOpenDebug?: () => void;
}

const isEditableTarget = (e: KeyboardEvent): boolean => {
  const t = e.target as HTMLElement | null;
  if (!t) return false;
  const tag = t.tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA') return true;
  if (t.isContentEditable) return true;
  return false;
};

export function useShortcuts(handlers: ShortcutHandlers) {
  const onKey = (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      handlers.onEscape?.();
      return;
    }
    if (!(e.ctrlKey || e.metaKey)) return;
    const k = e.key.toLowerCase();
    if (k === 'b') {
      e.preventDefault();
      handlers.onBuildPicker?.();
    } else if (k === 'p') {
      e.preventDefault();
      handlers.onModelPicker?.();
    } else if (k === 'n' && !isEditableTarget(e)) {
      // Ctrl+N — start a new conversation. Gated by `!isEditableTarget`
      // so typing the letter `n` in the prompt or any input never
      // triggers a chat reset.
      e.preventDefault();
      handlers.onResetChat?.();
    } else if (k === 'd' && e.shiftKey) {
      e.preventDefault();
      handlers.onOpenDebug?.();
    } else if (k === ',') {
      e.preventDefault();
      handlers.onSettings?.();
    }
  };

  onMounted(() => window.addEventListener('keydown', onKey));
  onBeforeUnmount(() => window.removeEventListener('keydown', onKey));
}

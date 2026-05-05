import { defineStore } from 'pinia';
import { ref } from 'vue';

export type ToastVariant = 'info' | 'success' | 'warning' | 'error';

export interface ToastVm {
  id: string;
  variant: ToastVariant;
  title: string;
  body?: string;
  ttl: number;
}

const newId = () => `t_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;

export const useToastsStore = defineStore('toasts', () => {
  const toasts = ref<ToastVm[]>([]);
  const timers = new Map<string, ReturnType<typeof setTimeout>>();

  function dismiss(id: string) {
    toasts.value = toasts.value.filter((t) => t.id !== id);
    const handle = timers.get(id);
    if (handle) {
      clearTimeout(handle);
      timers.delete(id);
    }
  }

  function push(input: { variant?: ToastVariant; title: string; body?: string; ttl?: number }) {
    const t: ToastVm = {
      id: newId(),
      variant: input.variant ?? 'info',
      title: input.title,
      body: input.body,
      ttl: input.ttl ?? 5000,
    };
    toasts.value.push(t);
    if (t.ttl > 0) {
      const handle = setTimeout(() => dismiss(t.id), t.ttl);
      timers.set(t.id, handle);
    }
    return t.id;
  }

  return { toasts, push, dismiss };
});

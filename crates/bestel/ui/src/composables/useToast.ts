import { useToastsStore } from '../stores/toasts';

export function useToast() {
  const store = useToastsStore();
  return {
    push: store.push,
    dismiss: store.dismiss,
  };
}

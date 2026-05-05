import { defineStore } from 'pinia';
import { ref } from 'vue';

export type UiPicker = 'build' | 'model' | 'chat' | null;

export const useUiStore = defineStore('ui', () => {
  const picker = ref<UiPicker>(null);

  const openBuild = () => {
    picker.value = 'build';
  };
  const openModel = () => {
    picker.value = 'model';
  };
  const openChat = () => {
    picker.value = 'chat';
  };
  const close = () => {
    picker.value = null;
  };

  return {
    picker,
    openBuild,
    openModel,
    openChat,
    close,
  };
});

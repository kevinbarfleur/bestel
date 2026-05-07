import { createApp } from 'vue';
import { createPinia } from 'pinia';

import PromptEditorRoot from './components/prompt-editor/PromptEditorRoot.vue';

import './styles/tokens.css';
import './styles/fonts.css';
import './styles/global.css';
import './styles/prompt-editor.css';

try {
  const stored = localStorage.getItem('bestel.theme');
  if (stored === 'dark') {
    document.documentElement.classList.add('theme-dark');
  }
} catch {
  /* localStorage unavailable */
}

createApp(PromptEditorRoot).use(createPinia()).mount('#app');

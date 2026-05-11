import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { listen } from '@tauri-apps/api/event';

import PromptEditorRoot from './components/prompt-editor/PromptEditorRoot.vue';
import { installGlobalAnchorGuard } from './api/tauri';

import './styles/tokens.css';
import './styles/fonts.css';
import './styles/global.css';
import './styles/prompt-editor.css';

// Defense-in-depth anchor guard — see comments in main.ts.
installGlobalAnchorGuard();

function applyTheme(theme: 'light' | 'dark') {
  document.documentElement.classList.toggle('theme-dark', theme === 'dark');
  try {
    localStorage.setItem('bestel.theme', theme);
  } catch {
    /* ignore */
  }
}

try {
  const stored = localStorage.getItem('bestel.theme');
  if (stored === 'dark' || stored === 'light') {
    applyTheme(stored);
  }
} catch {
  /* localStorage unavailable */
}

// Stay in lock-step with the main window: when the user toggles the theme
// over there, the settings store emits `theme:changed` to this window.
listen<{ theme: 'light' | 'dark' }>('theme:changed', (ev) => {
  if (ev.payload?.theme === 'light' || ev.payload?.theme === 'dark') {
    applyTheme(ev.payload.theme);
  }
}).catch(() => {
  /* listener registration failed — degrade gracefully */
});

createApp(PromptEditorRoot).use(createPinia()).mount('#app');

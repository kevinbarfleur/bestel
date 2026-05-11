import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import { router } from './router';
import { installGlobalAnchorGuard } from './api/tauri';

import './styles/tokens.css';
import './styles/fonts.css';
import './styles/sketch.css';
import './styles/global.css';
import './styles/markdown.css';

// Apply persisted theme before mount to avoid a flash of incorrect theme.
try {
  const stored = localStorage.getItem('bestel.theme');
  if (stored === 'dark') {
    document.documentElement.classList.add('theme-dark');
  }
} catch {
  /* localStorage unavailable */
}

// Defense-in-depth: catch every anchor click at the document level and
// route external URLs through `openLink` instead of letting the WebView
// hijack the main frame. Per-component v-html containers also register
// their own handler — this is the safety net for the ones that don't.
installGlobalAnchorGuard();

createApp(App).use(createPinia()).use(router).mount('#app');

import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import { router } from './router';

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

createApp(App).use(createPinia()).use(router).mount('#app');

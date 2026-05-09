import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router';

import ChatView from './views/ChatView.vue';
import ComponentsLab from './views/ComponentsLab.vue';

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'chat', component: ChatView },
  { path: '/components', name: 'components', component: ComponentsLab },
  // Catch-all so any orphaned link / stale hash (e.g. legacy /debug, removed
  // /build-sheets dev preview) sends the user back to the chat instead of a
  // blank screen. Keep it last so explicit matches win.
  { path: '/:pathMatch(.*)*', redirect: '/' },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

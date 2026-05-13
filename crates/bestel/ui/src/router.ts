import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router';

import ChatView from './views/ChatView.vue';

// The components lab (design tokens + Runic components gallery) is only
// reachable in `cargo tauri dev`. The lazy import keeps the lab out of
// the production bundle entirely — it doesn't ship to testers.
const devOnlyRoutes: RouteRecordRaw[] = import.meta.env.DEV
  ? [
      {
        path: '/components',
        name: 'components',
        component: () => import('./views/ComponentsLab.vue'),
      },
    ]
  : [];

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'chat', component: ChatView },
  ...devOnlyRoutes,
  // Catch-all so any orphaned link / stale hash (e.g. legacy /debug, removed
  // /build-sheets dev preview) sends the user back to the chat instead of a
  // blank screen. Keep it last so explicit matches win.
  { path: '/:pathMatch(.*)*', redirect: '/' },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

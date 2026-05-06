import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router';

import ChatView from './views/ChatView.vue';
import ComponentsLab from './views/ComponentsLab.vue';
import DebugView from './views/DebugView.vue';

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'chat', component: ChatView },
  { path: '/components', name: 'components', component: ComponentsLab },
  { path: '/debug', name: 'debug', component: DebugView },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

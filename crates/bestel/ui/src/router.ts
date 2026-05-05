import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router';

import ChatView from './views/ChatView.vue';
import ComponentsLab from './views/ComponentsLab.vue';

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'chat', component: ChatView },
  { path: '/components', name: 'components', component: ComponentsLab },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

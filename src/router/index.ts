import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  {
    path: '/',
    redirect: '/process',
  },
  {
    path: '/process',
    name: 'ProcessManager',
    component: () => import('../views/ProcessManager.vue'),
  },
  {
    path: '/config',
    name: 'ConfigManager',
    component: () => import('../views/ConfigManager.vue'),
  },
  {
    path: '/logs',
    name: 'LogViewer',
    component: () => import('../views/LogViewer.vue'),
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('../views/Settings.vue'),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;


import { createRouter, createWebHistory } from 'vue-router';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', redirect: '/overview' },
    { path: '/overview', component: () => import('./views/v2/Overview2.vue'), meta: { title: '概览' } },
    { path: '/detail', component: () => import('./views/v2/Detail2.vue'), meta: { title: '明细' } },
    { path: '/stats', component: () => import('./views/v2/Stats2.vue'), meta: { title: '统计' } },
    { path: '/budget', component: () => import('./views/v2/Budget2.vue'), meta: { title: '预算' } },
    { path: '/categories', component: () => import('./views/v2/Categories2.vue'), meta: { title: '分类' } },
    { path: '/settings', component: () => import('./views/v2/Settings2.vue'), meta: { title: '设置' } },
  ],
});

export default router;

import { createRouter, createWebHashHistory, createWebHistory } from 'vue-router';

// VITE_HASH=1 的测试构建（file:// 模拟台无法用 web history）走 hash 路由；正常构建不变
const history = import.meta.env.VITE_HASH === '1' ? createWebHashHistory() : createWebHistory();

const router = createRouter({
  history,
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

import { createRouter, createWebHistory } from 'vue-router';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', redirect: '/overview' },
    { path: '/overview', component: () => import('./views/OverviewView.vue'), meta: { title: '概览' } },
    { path: '/categories', component: () => import('./views/CategoriesView.vue'), meta: { title: '分类' } },
    { path: '/detail', component: () => import('./views/DetailView.vue'), meta: { title: '详细' } },
    { path: '/stats', component: () => import('./views/StatsView.vue'), meta: { title: '统计' } },
    { path: '/settings', component: () => import('./views/SettingsView.vue'), meta: { title: '设置' } },
  ],
});

export default router;

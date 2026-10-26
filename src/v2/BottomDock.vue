<script setup lang="ts">
// v2 移动端底部悬浮 Dock（安卓 / 窄窗口）：桌面侧栏的五页导航 + 设置收进 Dock，
// 「记一笔」提成右上悬浮按钮（FAB）。视觉沿用侧栏玻璃材质与强调色 token。
import { computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { icon } from './icons';
import { openQuickAdd } from './ui';

const route = useRoute();
const router = useRouter();

const navs = [
  { path: '/overview', title: '概览', ic: 'overview' },
  { path: '/detail', title: '明细', ic: 'list' },
  { path: '/stats', title: '统计', ic: 'stats' },
  { path: '/budget', title: '预算', ic: 'budget' },
  { path: '/categories', title: '分类', ic: 'cats' },
  { path: '/settings', title: '设置', ic: 'gear' },
];

const activePath = computed(() => '/' + route.path.split('/')[1]);
</script>

<template>
  <nav class="v2-dock">
    <button
      v-for="nav in navs" :key="nav.path"
      class="v2-dock-item" :class="{ active: activePath === nav.path }"
      :title="nav.title" @click="router.push(nav.path)"
    >
      <span class="v2-dock-ic" v-html="icon(nav.ic, 21)" />
      <span class="v2-dock-label">{{ nav.title }}</span>
    </button>
  </nav>
  <button class="v2-fab" title="记一笔" @click="openQuickAdd()">
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round"><path d="M12 5v14M5 12h14" /></svg>
  </button>
</template>

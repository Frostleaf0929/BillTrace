<script setup lang="ts">
// v2 移动端底部导航（M3 navigation bar 构图）：
// 4 个目的地（概览/明细/统计/分类）左右各二，「记一笔」FAB 居中嵌在 Dock 上沿。
// 预算从概览页的预算锚点卡进入，设置在概览页头部齿轮（移动端），不占导航位。
// 激活态 = M3 式胶囊指示器垫在图标后（250ms emphasized 缓动），视觉沿用强调色 token。
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
  { path: '/categories', title: '分类', ic: 'cats' },
];

const activePath = computed(() => '/' + route.path.split('/')[1]);
</script>

<template>
  <nav class="v2-dock">
    <button
      v-for="nav in navs.slice(0, 2)" :key="nav.path"
      class="v2-dock-item" :class="{ active: activePath === nav.path }"
      :title="nav.title" @click="router.push(nav.path)"
    >
      <span class="v2-dock-ic" v-html="icon(nav.ic, 22)" />
      <span class="v2-dock-label">{{ nav.title }}</span>
    </button>
    <div class="v2-dock-slot">
      <button class="v2-fab" title="记一笔" @click="openQuickAdd()">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round"><path d="M12 5v14M5 12h14" /></svg>
      </button>
    </div>
    <button
      v-for="nav in navs.slice(2)" :key="nav.path"
      class="v2-dock-item" :class="{ active: activePath === nav.path }"
      :title="nav.title" @click="router.push(nav.path)"
    >
      <span class="v2-dock-ic" v-html="icon(nav.ic, 22)" />
      <span class="v2-dock-label">{{ nav.title }}</span>
    </button>
  </nav>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { convertFileSrc } from '@tauri-apps/api/core';
import { Odometer, CollectionTag, Memo, TrendCharts, Setting, Moon, Sunny, Minus, Close } from '@element-plus/icons-vue';
import { api } from './api';
import {
  initAppearance, isDark, themeMode, toggleTheme, brightnessOverlayStyle,
  bgEnabled, bgPath, bgFit, bgStamp,
} from './theme';

const route = useRoute();
const router = useRouter();
const appWin = getCurrentWindow();

const collapsed = ref(false);

const navs = [
  { path: '/overview', title: '概览', icon: Odometer },
  { path: '/categories', title: '分类', icon: CollectionTag },
  { path: '/detail', title: '详细', icon: Memo },
  { path: '/stats', title: '统计', icon: TrendCharts },
  { path: '/settings', title: '设置', icon: Setting },
];

const activePath = computed(() => '/' + route.path.split('/')[1]);
const activeIndex = computed(() => navs.findIndex((n) => n.path === activePath.value));
const pageTitle = computed(() => navs.find((n) => n.path === activePath.value)?.title ?? '账痕');

const footerText = computed(() => {
  const cur = isDark.value ? '深色' : '浅色';
  return themeMode.value === 'system' ? `跟随系统 · ${cur}` : `${cur}模式`;
});

const overlayStyle = computed(() => brightnessOverlayStyle());

// 背景图样式
const bgStyle = computed(() => {
  if (!bgEnabled.value || !bgPath.value) return {};
  const url = `url("${convertFileSrc(bgPath.value)}?t=${bgStamp.value}")`;
  const base: Record<string, string> = { backgroundImage: url };
  if (bgFit.value === 'tile') {
    Object.assign(base, { backgroundRepeat: 'repeat', backgroundSize: 'auto' });
  } else if (bgFit.value === 'fill') {
    Object.assign(base, { backgroundRepeat: 'no-repeat', backgroundSize: '100% 100%' });
  } else {
    Object.assign(base, { backgroundRepeat: 'no-repeat', backgroundSize: bgFit.value, backgroundPosition: 'center' });
  }
  return base;
});

// ---------- 滚动换页：滚到底继续下滚 → 下一页；页顶上滚 → 上一页 ----------
const wheelNavEnabled = ref(true);
let wheelCooldown = 0;

function onMainWheel(e: WheelEvent) {
  if (!wheelNavEnabled.value) return;
  const el = e.currentTarget as HTMLElement;
  if (!el) return;
  const now = Date.now();
  if (now < wheelCooldown) return;
  const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 4;
  const atTop = el.scrollTop <= 2;
  let target = -1;
  if (atBottom && e.deltaY > 40) target = activeIndex.value + 1;
  else if (atTop && e.deltaY < -40) target = activeIndex.value - 1;
  if (target < 0 || target >= navs.length || target === activeIndex.value) return;
  wheelCooldown = now + 1000;
  router.push(navs[target].path);
}

// ---------- 缩放防卡顿：窗口调整大小时临时关闭毛玻璃 ----------
let resizeTimer: ReturnType<typeof setTimeout> | undefined;
let unlistenResize: (() => void) | undefined;

async function toggleSidebar() {
  collapsed.value = !collapsed.value;
  try { await api.setSetting('sidebarCollapsed', String(collapsed.value)); } catch { /* 忽略 */ }
}

function footerToggle(e: MouseEvent) {
  toggleTheme({ x: e.clientX || window.innerWidth - 70, y: e.clientY || window.innerHeight - 60 });
}

onMounted(async () => {
  await initAppearance();
  try {
    collapsed.value = (await api.getSetting('sidebarCollapsed')) === 'true';
    wheelNavEnabled.value = (await api.getSetting('wheelNav')) !== 'false';
  } catch { /* 默认值 */ }
  unlistenResize = await appWin.onResized(() => {
    document.documentElement.classList.add('resizing');
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => document.documentElement.classList.remove('resizing'), 200);
  });
});

onUnmounted(() => {
  unlistenResize?.();
});
</script>

<template>
  <div class="zj-root">
    <!-- 自绘标题栏（跟随主题） -->
    <div class="zj-titlebar">
      <div class="tb-title">
        <span style="font-weight: 700; color: var(--zj-text)">账痕</span>
        <span>· {{ pageTitle }}</span>
      </div>
      <div class="tb-drag" data-tauri-drag-region @dblclick="appWin.toggleMaximize()" />
      <div class="tb-btns">
        <button class="tb-btn" title="最小化" @click="appWin.minimize()"><el-icon><Minus /></el-icon></button>
        <button class="tb-btn" title="最大化 / 还原" @click="appWin.toggleMaximize()">
          <svg width="11" height="11" viewBox="0 0 11 11"><rect x="0.5" y="0.5" width="10" height="10" fill="none" stroke="currentColor" /></svg>
        </button>
        <button class="tb-btn close" title="关闭" @click="appWin.close()"><el-icon><Close /></el-icon></button>
      </div>
    </div>

    <!-- 背景图层（设置里可启用） -->
    <div v-if="bgEnabled && bgPath" class="zj-bg-image" :style="bgStyle" />
    <div class="zj-brightness" :style="overlayStyle as any" />

    <div class="zj-layout">
      <aside class="zj-sidebar" :class="{ collapsed }">
        <div class="zj-logo" :title="collapsed ? '展开侧边栏' : '收起侧边栏'" @click="toggleSidebar">
          <div class="logo-dot">
            <svg width="19" height="19" viewBox="0 0 24 24" fill="none">
              <path d="M4.5 16.2 L10 10.5 L13.6 13 L19.5 6.8" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
              <circle cx="19.5" cy="6.8" r="1.7" fill="currentColor" />
              <path d="M4.5 20 H11" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" opacity="0.55" />
            </svg>
          </div>
          <div class="logo-name">账痕</div>
        </div>
        <nav class="zj-nav">
          <div
            v-for="nav in navs"
            :key="nav.path"
            class="zj-nav-item"
            :class="{ active: activePath === nav.path }"
            :title="nav.title"
            @click="router.push(nav.path)"
          >
            <el-icon><component :is="nav.icon" /></el-icon>
            <span class="nav-label">{{ nav.title }}</span>
          </div>
        </nav>
        <div class="zj-sidebar-footer" title="切换深浅色" @click="footerToggle">
          <el-icon><component :is="isDark ? Sunny : Moon" /></el-icon>
          <span class="footer-label">{{ footerText }}</span>
        </div>
      </aside>
      <main class="zj-main" @wheel="onMainWheel">
        <router-view v-slot="{ Component }">
          <transition name="fade-slide" mode="out-in">
            <component :is="Component" @refresh-pending="() => {}" />
          </transition>
        </router-view>
      </main>
    </div>
  </div>
</template>

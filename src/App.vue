<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { convertFileSrc } from '@tauri-apps/api/core';
import { Minus, Close } from '@element-plus/icons-vue';
import { api } from './api';
import {
  initAppearance, isDark, themeMode, toggleTheme, brightnessOverlayStyle,
  bgEnabled, bgPath, bgFit, bgStamp,
} from './theme';
import { icon } from './v2/icons';
import { openQuickAdd, quickVisible, loadUiPrefs } from './v2/ui';
import QuickAdd from './v2/QuickAdd.vue';
import { toasts, kill, toast } from './v2/toast';

const route = useRoute();
const router = useRouter();
const appWin = getCurrentWindow();

const navs = [
  { path: '/overview', title: '概览', ic: 'overview' },
  { path: '/detail', title: '明细', ic: 'list' },
  { path: '/stats', title: '统计', ic: 'stats' },
  { path: '/budget', title: '预算', ic: 'budget' },
  { path: '/categories', title: '分类', ic: 'cats' },
];

const activePath = computed(() => '/' + route.path.split('/')[1]);

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

// ---------- 滚动换页（保留旧版能力） ----------
const wheelNavEnabled = ref(true);
let wheelCooldown = 0;
let wheelAcc = 0;
let wheelAccAt = 0;

function onMainWheel(e: WheelEvent) {
  const t = e.target as HTMLElement | null;
  if (t && t.closest('.el-overlay, .el-dialog, .el-message-box, .el-popper, .el-select-dropdown, .v2-panel, .v2-overlay')) return;
  if (!wheelNavEnabled.value) return;
  const el = e.currentTarget as HTMLElement;
  if (!el) return;
  const now = Date.now();
  if (now < wheelCooldown) return;
  if (now - wheelAccAt > 400) wheelAcc = 0;
  wheelAccAt = now;
  wheelAcc += e.deltaY;
  const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 4;
  const atTop = el.scrollTop <= 2;
  const idx = navs.findIndex((n) => n.path === activePath.value);
  let target = -1;
  if (atBottom && wheelAcc >= 300) target = idx + 1;
  else if (atTop && wheelAcc <= -300) target = idx - 1;
  if (target < 0 || target >= navs.length || target === idx) return;
  wheelAcc = 0;
  wheelCooldown = now + 800;
  router.push(navs[target].path);
}

// ---------- 缩放防卡顿 ----------
let resizeTimer: ReturnType<typeof setTimeout> | undefined;
let unlistenResize: (() => void) | undefined;

function themeIcon(): string {
  return icon(isDark.value ? 'sun' : 'moon', 19);
}
function onThemeToggle(e: MouseEvent): void {
  toggleTheme({ x: e.clientX || window.innerWidth - 70, y: e.clientY || window.innerHeight - 60 });
}
function onQuickSaved(): void {
  toast('已记一笔');
  window.dispatchEvent(new CustomEvent('v2-refresh'));
}

onMounted(async () => {
  await initAppearance();
  await loadUiPrefs();
  try {
    wheelNavEnabled.value = (await api.getSetting('wheelNav')) !== 'false';
  } catch { /* 默认值 */ }
  window.addEventListener('zj-wheel-nav', (ev) => {
    wheelNavEnabled.value = (ev as CustomEvent).detail !== false;
  });
  window.addEventListener('keydown', (ev) => {
    if ((ev.ctrlKey || ev.metaKey) && ev.key.toLowerCase() === 'n') {
      ev.preventDefault();
      openQuickAdd();
    }
  });
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
    <!-- 悬浮窗口按钮（无标题栏） -->
    <div class="tb-btns">
      <button class="tb-btn" title="最小化" @click="appWin.minimize()"><el-icon><Minus /></el-icon></button>
      <button class="tb-btn" title="最大化 / 还原" @click="appWin.toggleMaximize()">
        <svg width="11" height="11" viewBox="0 0 11 11"><rect x="0.5" y="0.5" width="10" height="10" fill="none" stroke="currentColor" /></svg>
      </button>
      <button class="tb-btn close" title="关闭" @click="appWin.close()"><el-icon><Close /></el-icon></button>
    </div>

    <!-- 背景图层（设置里可启用） -->
    <div v-if="bgEnabled && bgPath" class="zj-bg-image" :style="bgStyle" />
    <div class="zj-brightness" :style="overlayStyle as any" />

    <div class="zj-layout">
      <!-- v2 侧栏：上黑下浅两段式 -->
      <aside class="v2-rail">
        <div class="v2-rail-main">
          <img class="v2-logo" src="/app-icon.png" alt="账痕" title="账痕 BillTrace" @click="router.push('/overview')">
          <button class="v2-add" title="记一笔 (Ctrl+N)" @click="openQuickAdd()">
            <svg width="19" height="19" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M12 5v14M5 12h14"/></svg>
          </button>
          <nav style="display:contents">
            <button
              v-for="nav in navs" :key="nav.path"
              class="v2-nav-item" :class="{ active: activePath === nav.path }"
              :title="nav.title" @click="router.push(nav.path)"
              v-html="icon(nav.ic, 20)"
            />
          </nav>
        </div>
        <div class="v2-rail-gap" />
        <div class="v2-rail-foot">
          <button class="v2-rail-btn" :title="themeMode === 'system' ? '切换深浅主题（跟随系统中）' : '切换深浅主题'" @click="onThemeToggle" v-html="themeIcon()" />
          <button class="v2-rail-btn" title="设置" @click="router.push('/settings')" v-html="icon('gear', 19)" />
          <button class="v2-rail-btn exit" title="退出" @click="appWin.close()" v-html="icon('exit', 19)" />
        </div>
      </aside>

      <main class="zj-content">
        <div class="zj-content-drag" data-tauri-drag-region @dblclick="appWin.toggleMaximize()" />
        <div class="zj-content-scroll" @wheel="onMainWheel">
          <router-view v-slot="{ Component }">
            <transition name="fade-slide" mode="out-in">
              <component :is="Component" />
            </transition>
          </router-view>
        </div>
      </main>
    </div>

    <!-- v2 全局层：快速记账 / 遮罩 / Toast -->
    <div class="v2-overlay" :class="{ show: quickVisible }" @click="quickVisible = false" />
    <div class="v2-panel" :class="{ show: quickVisible }">
      <QuickAdd v-if="quickVisible" :visible="quickVisible" @close="quickVisible = false" @saved="onQuickSaved" />
    </div>
    <div class="v2-toasts">
      <div v-for="t in toasts" :key="t.id" class="v2-toast">
        <span>{{ t.msg }}</span>
        <button v-if="t.action" @click="() => { t.onAction?.(); kill(t.id); }">{{ t.action }}</button>
      </div>
    </div>
  </div>
</template>

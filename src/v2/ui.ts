// v2 全局 UI 状态（快速记账面板 / 面板样式）
import { ref, watchEffect } from 'vue';
import { api } from '../api';

export const quickVisible = ref(false);
export const v2PanelOpen = ref(false); // 任一右侧面板打开时为真（隐藏窗口按钮用）
export const panelMode = ref<'float' | 'dock'>('float');
export const railCollapsed = ref(false); // 经典模式侧栏收起（平板模式是固定细条，无此概念）

// 移动端（安卓 APK 或窄窗口）：侧栏换底部悬浮 Dock。
// 安卓 WebView 的 UA 固定含 "Android"，恒为移动布局（横屏 CSS 宽度可能超过 900，不能只看宽度）；
// 桌面端窄窗口（<900，即桌面最小窗宽以下）也走移动布局，方便浏览器预览。
export const isMobile = ref(
  /Android/i.test(navigator.userAgent) || window.matchMedia('(max-width: 899px)').matches,
);
window.matchMedia('(max-width: 899px)').addEventListener('change', (e) => {
  isMobile.value = /Android/i.test(navigator.userAgent) || e.matches;
});
watchEffect(() => {
  document.documentElement.classList.toggle('zj-mobile', isMobile.value);
});

export async function loadUiPrefs(): Promise<void> {
  try {
    const v = await api.getSetting('panelMode');
    if (v === 'dock' || v === 'float') panelMode.value = v;
    // 经典模式侧栏收展沿用 v0.3.0 的设置键
    railCollapsed.value = (await api.getSetting('sidebarCollapsed')) === 'true';
  } catch { /* 默认悬浮 */ }
  applyPanelMode();
}

export async function toggleRail(): Promise<void> {
  railCollapsed.value = !railCollapsed.value;
  try { await api.setSetting('sidebarCollapsed', String(railCollapsed.value)); } catch { /* 忽略 */ }
  // 收展动画期间（~240ms）关掉墙面模糊，收排卡顿（html.rail-anim 见 global.css）
  const root = document.documentElement;
  root.classList.add('rail-anim');
  window.setTimeout(() => root.classList.remove('rail-anim'), 300);
}

export function setPanelMode(v: 'float' | 'dock'): void {
  panelMode.value = v;
  applyPanelMode();
  api.setSetting('panelMode', v).catch(() => { /* 忽略 */ });
}

function applyPanelMode(): void {
  document.documentElement.dataset.panel = panelMode.value;
}

export function openQuickAdd(): void {
  quickVisible.value = true;
}

export function notifyRefresh(): void {
  window.dispatchEvent(new CustomEvent('v2-refresh'));
}

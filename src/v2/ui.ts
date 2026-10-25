// v2 全局 UI 状态（快速记账面板 / 面板样式）
import { ref } from 'vue';
import { api } from '../api';

export const quickVisible = ref(false);
export const v2PanelOpen = ref(false); // 任一右侧面板打开时为真（隐藏窗口按钮用）
export const panelMode = ref<'float' | 'dock'>('float');
export const railCollapsed = ref(false); // 经典模式侧栏收起（平板模式是固定细条，无此概念）

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

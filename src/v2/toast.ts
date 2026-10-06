// v2 轻量 toast（带撤销动作）
import { reactive } from 'vue';

export interface ToastItem {
  id: number;
  msg: string;
  action?: string;
  onAction?: () => void;
}

export const toasts = reactive<ToastItem[]>([]);
let seq = 1;

export function toast(msg: string, action?: string, onAction?: () => void): void {
  const id = seq++;
  toasts.push({ id, msg, action, onAction });
  setTimeout(() => kill(id), 4200);
}

export function kill(id: number): void {
  const i = toasts.findIndex((t) => t.id === id);
  if (i >= 0) toasts.splice(i, 1);
}

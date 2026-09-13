import { ref } from 'vue';
import { api } from './api';

// ---------- 界面模式（跟随系统 / 浅色 / 深色） ----------
export type ThemeMode = 'system' | 'light' | 'dark';
export const themeMode = ref<ThemeMode>('system');
export const isDark = ref(true);

const media = window.matchMedia('(prefers-color-scheme: dark)');

function applyTheme() {
  const dark = themeMode.value === 'dark' || (themeMode.value === 'system' && media.matches);
  isDark.value = dark;
  document.documentElement.classList.toggle('dark', dark);
}

// ---------- 个性化外观 ----------
export interface AccentOption { id: string; name: string; light: string; dark: string }
export const ACCENTS: AccentOption[] = [
  { id: 'aurora', name: '极光蓝', light: '#3e63dd', dark: '#7b9aff' },
  { id: 'mint', name: '薄荷绿', light: '#1f9e6e', dark: '#4cd694' },
  { id: 'sunset', name: '落日橙', light: '#d97435', dark: '#ffa057' },
  { id: 'rose', name: '玫瑰红', light: '#c94f6d', dark: '#ff8ba3' },
  { id: 'violet', name: '雾紫', light: '#8250c4', dark: '#bb8cff' },
  { id: 'slate', name: '石青灰', light: '#4a5a6f', dark: '#93a6bf' },
];
export const accentId = ref('aurora');
export const customAccent = ref(''); // 用户自定义色（#rrggbb），设置后优先于预设
export const glassOn = ref(true);
export const blurPx = ref(24);
export const glassAlpha = ref(70);   // 玻璃强度 30~95（卡片不透明度百分比）
export const brightness = ref(0);    // -100 ~ 100，0 为原始

// 背景图
export const bgEnabled = ref(false);
export const bgPath = ref('');
export const bgFit = ref<'cover' | 'contain' | 'fill' | 'tile'>('cover');
export const bgStamp = ref(Date.now()); // 缓存击穿：换图后立即生效

function currentAccentColor(): string {
  if (customAccent.value) return customAccent.value;
  const acc = ACCENTS.find((a) => a.id === accentId.value) ?? ACCENTS[0];
  return isDark.value ? acc.dark : acc.light;
}

function applyAccent() {
  const c = currentAccentColor();
  const root = document.documentElement;
  root.style.setProperty('--zj-primary', c);
  root.style.setProperty('--zj-logo-from', c);
  root.style.setProperty('--zj-logo-to', shiftHue(c, isDark.value ? -26 : 40));
  root.style.setProperty('--zj-sidebar-active-text', isDark.value ? lighten(c, 0.35) : darken(c, 0.15));
  root.style.setProperty('--zj-sidebar-active', withAlpha(c, isDark.value ? 0.2 : 0.12));
}

function applyGlass() {
  const root = document.documentElement;
  root.classList.toggle('no-glass', !glassOn.value);
  // 只写数值型变量（模糊半径、玻璃强度），颜色由 CSS color-mix 统一计算，
  // 这样 no-glass / resizing 类可以正常覆盖，开关与强度即时生效
  root.style.setProperty('--zj-blur-px', `${blurPx.value}px`);
  root.style.setProperty('--zj-glass-alpha', String(glassAlpha.value));
}

export function brightnessOverlayStyle(): { background: string; opacity: number } {
  if (brightness.value === 0) return { background: 'transparent', opacity: 0 };
  return brightness.value > 0
    ? { background: '#ffffff', opacity: (brightness.value / 100) * 0.6 }
    : { background: '#000814', opacity: (-brightness.value / 100) * 0.65 };
}

function applyAll() {
  applyTheme();
  applyAccent();
  applyGlass();
}

async function loadAll() {
  try {
    const t = await api.getSetting('theme');
    if (t === 'light' || t === 'dark' || t === 'system') themeMode.value = t;
    const a = await api.getSetting('accent');
    if (a && ACCENTS.some((x) => x.id === a)) accentId.value = a;
    const ca = await api.getSetting('accentCustom');
    if (ca) customAccent.value = ca;
    glassOn.value = (await api.getSetting('glassOn')) !== 'false';
    const b = Number(await api.getSetting('blurPx'));
    if (b >= 4 && b <= 40) blurPx.value = b;
    const ga = Number(await api.getSetting('glassAlpha'));
    if (ga >= 30 && ga <= 95) glassAlpha.value = ga;
    // 亮度：新版刻度 -100~100；兼容旧版 80~120（旧 100 = 新 0）
    const br = Number(await api.getSetting('brightness'));
    if (br >= -100 && br <= 100) brightness.value = br > 60 ? br - 100 : br;
    bgEnabled.value = (await api.getSetting('bgEnabled')) === 'true';
    bgPath.value = (await api.getBackground()) ?? '';
    const f = await api.getSetting('bgFit');
    if (f === 'cover' || f === 'contain' || f === 'fill' || f === 'tile') bgFit.value = f;
  } catch { /* 用默认值 */ }
  applyAll();
}

export async function initAppearance() {
  await loadAll();
  media.addEventListener('change', () => {
    if (themeMode.value === 'system') applyAll();
  });
}

export async function setThemeMode(mode: ThemeMode) {
  themeMode.value = mode;
  applyAll();
  try { await api.setSetting('theme', mode); } catch { /* 忽略 */ }
}

export async function setAccent(id: string) {
  accentId.value = id;
  customAccent.value = '';
  applyAccent();
  try {
    await api.setSetting('accent', id);
    await api.setSetting('accentCustom', '');
  } catch { /* 忽略 */ }
}

export async function setCustomAccent(hex: string) {
  const v = hex.trim();
  if (!/^#[0-9a-fA-F]{6}$/.test(v)) return;
  customAccent.value = v;
  accentId.value = 'custom';
  applyAccent();
  try {
    await api.setSetting('accentCustom', v);
  } catch { /* 忽略 */ }
}

export async function setGlass(on: boolean) {
  glassOn.value = on;
  applyGlass();
  try { await api.setSetting('glassOn', String(on)); } catch { /* 忽略 */ }
}

export async function setBlur(px: number) {
  blurPx.value = px;
  applyGlass();
  try { await api.setSetting('blurPx', String(px)); } catch { /* 忽略 */ }
}

export async function setGlassAlpha(v: number) {
  glassAlpha.value = v;
  applyGlass();
  try { await api.setSetting('glassAlpha', String(v)); } catch { /* 忽略 */ }
}

export async function setBrightness(v: number) {
  brightness.value = v;
  try { await api.setSetting('brightness', String(v)); } catch { /* 忽略 */ }
}

// ---------- 背景图 ----------
export async function refreshBackground() {
  bgPath.value = (await api.getBackground().catch(() => null)) ?? '';
  bgStamp.value = Date.now();
}

export async function setBgEnabled(on: boolean) {
  bgEnabled.value = on;
  try { await api.setSetting('bgEnabled', String(on)); } catch { /* 忽略 */ }
}

export async function setBgFit(fit: 'cover' | 'contain' | 'fill' | 'tile') {
  bgFit.value = fit;
  try { await api.setSetting('bgFit', fit); } catch { /* 忽略 */ }
}

/**
 * 主题切换辐射动效：以触发位置为圆心，用「新主题背景色」的圆形遮罩扫过全屏，
 * 中途切换主题，再淡出遮罩。相比 View Transitions 更平滑可控（无截断卡顿）。
 */
export function animateThemeChange(apply: () => void, origin?: { x: number; y: number }) {
  const { innerWidth: w, innerHeight: h } = window;
  const x = origin?.x ?? w - 60;
  const y = origin?.y ?? h - 60;
  const radius = Math.hypot(Math.max(x, w - x), Math.max(y, h - y));

  const overlay = document.createElement('div');
  const darkNext = !isDark.value;
  overlay.style.cssText = [
    'position:fixed', 'inset:0', 'z-index:9999', 'pointer-events:none',
    'background:', darkNext
      ? 'radial-gradient(1200px 750px at 10% -12%, rgba(64,96,210,0.20) 0%, transparent 55%), #0c0f16'
      : 'radial-gradient(1100px 700px at 12% -8%, rgba(62,99,221,0.10) 0%, transparent 55%), #eef0f7',
    `clip-path: circle(0px at ${x}px ${y}px)`,
  ].join(';');
  document.body.appendChild(overlay);

  const expand = overlay.animate(
    [
      { clipPath: `circle(0px at ${x}px ${y}px)` },
      { clipPath: `circle(${radius}px at ${x}px ${y}px)` },
    ],
    { duration: 460, easing: 'cubic-bezier(0.4, 0, 0.2, 1)' }
  );
  expand.onfinish = () => {
    apply();
    const fade = overlay.animate([{ opacity: 1 }, { opacity: 0 }], { duration: 200, easing: 'ease-out' });
    fade.onfinish = () => overlay.remove();
  };
  expand.oncancel = () => {
    apply();
    overlay.remove();
  };
}

export async function setThemeModeAnimated(mode: ThemeMode, origin?: { x: number; y: number }) {
  animateThemeChange(() => {
    themeMode.value = mode;
    applyAll();
  }, origin);
  try { await api.setSetting('theme', mode); } catch { /* 忽略 */ }
}

/** 侧边栏底部按钮：在浅色/深色间切换（带辐射动效） */
export async function toggleTheme(origin?: { x: number; y: number }) {
  await setThemeModeAnimated(isDark.value ? 'light' : 'dark', origin);
}

// ---------- 颜色工具 ----------
function parseHex(c: string): [number, number, number] {
  const s = c.replace('#', '');
  const n = parseInt(s.length === 3 ? s.split('').map((x) => x + x).join('') : s, 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}
function toHex(r: number, g: number, b: number): string {
  const f = (v: number) => Math.max(0, Math.min(255, Math.round(v))).toString(16).padStart(2, '0');
  return `#${f(r)}${f(g)}${f(b)}`;
}
export function lighten(c: string, amt: number): string {
  const [r, g, b] = parseHex(c);
  return toHex(r + (255 - r) * amt, g + (255 - g) * amt, b + (255 - b) * amt);
}
export function darken(c: string, amt: number): string {
  const [r, g, b] = parseHex(c);
  return toHex(r * (1 - amt), g * (1 - amt), b * (1 - amt));
}
export function withAlpha(c: string, a: number): string {
  const [r, g, b] = parseHex(c);
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}
function shiftHue(c: string, deg: number): string {
  const [r, g, b] = parseHex(c).map((v) => v / 255) as [number, number, number];
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  let h = 0;
  const l = (max + min) / 2;
  const d = max - min;
  if (d !== 0) {
    if (max === r) h = ((g - b) / d) % 6;
    else if (max === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
    h *= 60;
    if (h < 0) h += 360;
  }
  h = (h + deg + 360) % 360;
  const s = d === 0 ? 0 : d / (1 - Math.abs(2 * l - 1));
  // HSL → RGB
  const c2 = (1 - Math.abs(2 * l - 1)) * s;
  const x = c2 * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - c2 / 2;
  let rp = 0, gp = 0, bp = 0;
  if (h < 60) [rp, gp, bp] = [c2, x, 0];
  else if (h < 120) [rp, gp, bp] = [x, c2, 0];
  else if (h < 180) [rp, gp, bp] = [0, c2, x];
  else if (h < 240) [rp, gp, bp] = [0, x, c2];
  else if (h < 300) [rp, gp, bp] = [x, 0, c2];
  else [rp, gp, bp] = [c2, 0, x];
  return toHex((rp + m) * 255, (gp + m) * 255, (bp + m) * 255);
}

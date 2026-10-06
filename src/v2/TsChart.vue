<script setup lang="ts">
// v2 概览时间序列图（FusionTime 范式：折线 + 底部导航器 + 十字线气泡）
// 数据由父组件提供：全量月序列 + 窗口内的日粒度补拉函数
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import type { ChartPoint } from '../types';

const props = defineProps<{
  series: ChartPoint[];
  months: { y: number; m: number }[];
  range: [number, number];
  fetchDay: (from: string, to: string) => Promise<ChartPoint[]>;
}>();
const emit = defineEmits<{ range: [r: [number, number]] }>();

const wrap = ref<HTMLElement>();
const width = ref(800);
let ro: ResizeObserver | undefined;
onMounted(() => {
  if (wrap.value && 'ResizeObserver' in window) {
    ro = new ResizeObserver(() => { width.value = wrap.value?.clientWidth || 800; });
    ro.observe(wrap.value);
  }
});
onUnmounted(() => ro?.disconnect());

const range = ref<[number, number]>([...props.range] as [number, number]);
watch(() => props.range, (r) => { range.value = [...r] as [number, number]; });

const all = computed(() => props.series);
const dayCache = new Map<string, ChartPoint[]>();

async function windowData(): Promise<ChartPoint[]> {
  const [i0, i1] = range.value;
  const span = i1 - i0 + 1;
  if (span > 1) return all.value.slice(i0, i1 + 1);
  const meta = props.months?.[i1];
  if (!meta) return all.value.slice(i0, i1 + 1);
  const from = `${meta.y}-${String(meta.m).padStart(2, '0')}-01`;
  const to = `${meta.y}-${String(meta.m).padStart(2, '0')}-31`;
  const key = from + '~' + to;
  const cached = dayCache.get(key);
  if (cached) return cached;
  const rows = await props.fetchDay(from, to);
  dayCache.set(key, rows);
  return rows;
}

const data = ref<ChartPoint[]>([]);
const W = computed(() => width.value || 800);
const H = 240, PAD_L = 50, PAD_T = 16, PAD_B = 26, PAD_R = 16;

async function draw(): Promise<void> {
  data.value = await windowData();
}
watch([range, all], draw, { immediate: true });

const n = computed(() => data.value.length);
const maxV = computed(() => {
  const hi = Math.max(...data.value.map((d) => Math.max(d.expense, d.income)), 1);
  const netMin = Math.min(...data.value.map((d) => d.income - d.expense), 0);
  return hi * 1.12 - netMin; // y 域下探到最小结余（负值完整显示）
});
const minV = computed(() => Math.min(...data.value.map((d) => d.income - d.expense), 0));
const plotW = computed(() => W.value - PAD_L - PAD_R);
const plotH = H - PAD_T - PAD_B;
function xAt(i: number): number { return PAD_L + (n.value === 1 ? plotW.value / 2 : (plotW.value * i) / (n.value - 1)); }
function yAt(v: number): number { return PAD_T + plotH * (1 - (v - minV.value) / Math.max(1e-9, maxV.value - minV.value)); }
function yTick(v: number): string { return v >= 1000 ? `${Math.round(v / 100) / 10}k` : String(Math.round(v)); }
const grid = computed(() => {
  let s = '';
  for (let k = 0; k <= 3; k++) {
    const v = minV.value + ((maxV.value - minV.value) * k) / 3, y = yAt(v);
    s += `<line x1="${PAD_L}" y1="${y.toFixed(1)}" x2="${W.value - PAD_R}" y2="${y.toFixed(1)}" stroke="var(--zj-border)" stroke-width=".75"/>`
      + `<text x="${PAD_L - 8}" y="${(y + 4).toFixed(1)}" font-size="10.5" fill="var(--v2-ink-3)" text-anchor="end">${yTick(v)}</text>`;
  }
  const step = Math.ceil(n.value / 12);
  data.value.forEach((d, i) => {
    if (i % step === 0 || i === n.value - 1)
      s += `<text x="${xAt(i)}" y="${H - 8}" font-size="10.5" fill="var(--v2-ink-3)" text-anchor="middle">${d.label}</text>`;
  });
  return s;
});

// Catmull-Rom → 贝塞尔平滑（点间拟合曲线，代替生硬折线）
function smooth(pts: [number, number][]): string {
  if (pts.length < 2) return pts.length ? `M${pts[0][0].toFixed(1)},${pts[0][1].toFixed(1)}` : '';
  let d = `M${pts[0][0].toFixed(1)},${pts[0][1].toFixed(1)}`;
  for (let i = 0; i < pts.length - 1; i++) {
    const p0 = pts[Math.max(0, i - 1)], p1 = pts[i], p2 = pts[i + 1], p3 = pts[Math.min(pts.length - 1, i + 2)];
    const c1x = p1[0] + (p2[0] - p0[0]) / 6, c1y = p1[1] + (p2[1] - p0[1]) / 6;
    const c2x = p2[0] - (p3[0] - p1[0]) / 6, c2y = p2[1] - (p3[1] - p1[1]) / 6;
    d += ` C${c1x.toFixed(1)},${c1y.toFixed(1)} ${c2x.toFixed(1)},${c2y.toFixed(1)} ${p2[0].toFixed(1)},${p2[1].toFixed(1)}`;
  }
  return d;
}
const smoothOf = (key: 'expense' | 'income') => smooth(data.value.map((d, i) => [xAt(i), yAt(d[key])] as [number, number]));
const body = computed(() => {
  if (!n.value) return '';
  let s = `<path d="${smoothOf('expense')} L${xAt(n.value - 1).toFixed(1)},${PAD_T + plotH} L${xAt(0).toFixed(1)},${PAD_T + plotH} Z" fill="var(--zj-primary)" opacity=".07"/>`;
  s += `<path class="line" d="${smoothOf('expense')}" fill="none" stroke="var(--zj-expense)" stroke-width="2.4" stroke-linejoin="round" stroke-linecap="round"/>`;
  s += `<path class="line" d="${smoothOf('income')}" fill="none" stroke="var(--zj-income)" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"/>`;
  s += `<path class="dotin" d="${smooth(data.value.map((d, i) => [xAt(i), yAt(d.income - d.expense)] as [number, number]))}" fill="none" stroke="var(--v2-ink-3)" stroke-width="1.4" stroke-dasharray="4 4"/>`;
  data.value.forEach((d, i) => {
    s += `<circle cx="${xAt(i)}" cy="${yAt(d.expense)}" r="2.8" fill="var(--zj-expense)"/><circle cx="${xAt(i)}" cy="${yAt(d.income)}" r="2.4" fill="var(--zj-income)"/>`;
  });
  return s;
});

// 导航器底图（全区间支出阴影）
const maxAll = computed(() => Math.max(...all.value.map((m) => m.expense), 1));
const navSvg = computed(() => {
  const NW = 1000, NH = 46;
  const pts: [number, number][] = all.value.map((m, i) => [
    (NW * i) / Math.max(1, all.value.length - 1),
    NH - 3 - ((NH - 8) * m.expense) / maxAll.value,
  ]);
  const ln = smooth(pts);
  let p = `M0,${NH}`;
  if (pts.length) p += ` ${smooth(pts)}`;
  p += ` L${NW},${NH} Z`;
  return `<path d="${p}" fill="var(--zj-primary)" opacity=".26"/><path d="${ln}" fill="none" stroke="var(--zj-primary)" stroke-width="1.2" opacity=".85"/>`;
});

// 选窗布局 + 拖拽（拖拽中只本地重绘，pointerup 才向父级提交）
const winEl = ref<HTMLElement>();
const navEl = ref<HTMLElement>();
const dimL = ref<HTMLElement>();
const dimR = ref<HTMLElement>();
function layoutNav(): void {
  const el = navEl.value;
  if (!el) return;
  const wpx = el.clientWidth;
  const x0 = (range.value[0] / Math.max(1, all.value.length - 1)) * wpx;
  const x1 = (range.value[1] / Math.max(1, all.value.length - 1)) * wpx;
  if (winEl.value) { winEl.value.style.left = x0 + 'px'; winEl.value.style.width = Math.max(14, x1 - x0) + 'px'; }
  if (dimL.value) dimL.value.style.width = x0 + 'px';
  if (dimR.value) dimR.value.style.width = Math.max(0, wpx - x1) + 'px';
}
watch([all, () => width.value], () => setTimeout(layoutNav, 0), { immediate: true });

function setRange(i0: number, i1: number): void {
  range.value = [Math.min(Math.max(0, Math.round(i0)), all.value.length - 1), Math.min(Math.max(Math.round(i1), 0), all.value.length - 1)];
  if (range.value[1] < range.value[0]) range.value[1] = range.value[0];
  draw();
  layoutNav();
}
function onWinDown(e: PointerEvent): void {
  e.preventDefault();
  const sx = e.clientX, r0 = range.value[0], r1 = range.value[1];
  const el = navEl.value;
  if (!el) return;
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  const move = (ev: PointerEvent): void => {
    const di = ((ev.clientX - sx) / el.clientWidth) * (all.value.length - 1);
    let i0 = r0 + di, i1 = r1 + di;
    if (i0 < 0) { i1 -= i0; i0 = 0; }
    if (i1 > all.value.length - 1) { i0 -= i1 - (all.value.length - 1); i1 = all.value.length - 1; }
    setRange(i0, i1);
  };
  const up = (): void => {
    window.removeEventListener('pointermove', move);
    window.removeEventListener('pointerup', up);
    emit('range', [...range.value] as [number, number]);
  };
  window.addEventListener('pointermove', move);
  window.addEventListener('pointerup', up);
}
function onHandleDown(e: PointerEvent, side: 'l' | 'r'): void {
  e.stopPropagation();
  e.preventDefault();
  const sx = e.clientX, r0 = range.value[0], r1 = range.value[1];
  const el = navEl.value;
  if (!el) return;
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  const move = (ev: PointerEvent): void => {
    const di = ((ev.clientX - sx) / el.clientWidth) * (all.value.length - 1);
    if (side === 'l') setRange(Math.min(Math.max(0, r0 + di), range.value[1]), range.value[1]);
    else setRange(range.value[0], Math.max(Math.min(all.value.length - 1, r1 + di), range.value[0]));
  };
  const up = (): void => {
    window.removeEventListener('pointermove', move);
    window.removeEventListener('pointerup', up);
    emit('range', [...range.value] as [number, number]);
  };
  window.addEventListener('pointermove', move);
  window.addEventListener('pointerup', up);
}
function onNavDown(e: PointerEvent): void {
  if ((e.target as HTMLElement).closest('.v2-nav-win')) return;
  const el = navEl.value;
  if (!el) return;
  const f = ((e.clientX - el.getBoundingClientRect().left) / el.clientWidth) * (all.value.length - 1);
  const span = range.value[1] - range.value[0];
  const c = Math.round(f);
  const i0 = Math.min(Math.max(0, c - Math.round(span / 2)), all.value.length - 1 - span);
  setRange(i0, i0 + span);
  emit('range', [...range.value] as [number, number]);
}

// 十字线 + 气泡
const tip = ref<HTMLElement>();
const ch = ref<HTMLElement>();
const cur = ref<{ n: number; xAt: (i: number) => number }>({ n: 0, xAt });
watch(data, () => { cur.value = { n: n.value, xAt }; });
function onMove(ev: MouseEvent): void {
  const plot = wrap.value?.querySelector('.v2-ts-plot') as HTMLElement | null;
  if (!plot || !cur.value.n) return;
  const rect = plot.getBoundingClientRect();
  const rel = ((ev.clientX - rect.left) / rect.width) * W.value;
  let i = Math.round((rel - PAD_L) / ((W.value - PAD_L - PAD_R) / Math.max(1, cur.value.n - 1)));
  i = Math.min(cur.value.n - 1, Math.max(0, i));
  const d = data.value[i];
  if (!d) return;
  if (ch.value) { ch.value.style.display = 'block'; ch.value.style.left = (xAt(i) / W.value) * rect.width + 'px'; }
  if (tip.value) {
    const net = d.income - d.expense;
    tip.value.innerHTML = `<div class="tt-date">${d.label}</div>`
      + `<div class="tt-row"><span class="v2-tt-dot" style="background:var(--zj-expense)"></span>支出<b>¥${d.expense.toLocaleString('zh-CN')}</b></div>`
      + `<div class="tt-row"><span class="v2-tt-dot" style="background:var(--zj-income)"></span>收入<b>¥${d.income.toLocaleString('zh-CN')}</b></div>`
      + `<div class="tt-row"><span class="v2-tt-dot" style="background:${net >= 0 ? '#8cc7a8' : '#e09a9c'}"></span>结余<b>${net >= 0 ? '+' : '-'}¥${Math.abs(net).toLocaleString('zh-CN')}</b></div>`;
    tip.value.style.display = 'block';
    const tw = tip.value.offsetWidth;
    let lx = (xAt(i) / W.value) * rect.width + 12;
    if (lx + tw > rect.width - 8) lx = (xAt(i) / W.value) * rect.width - tw - 12;
    tip.value.style.left = lx + 'px';
    tip.value.style.top = '16px';
  }
}
function onLeave(): void {
  if (tip.value) tip.value.style.display = 'none';
  if (ch.value) ch.value.style.display = 'none';
}

const dateLabel = computed(() => {
  const [i0, i1] = range.value;
  if (i0 === i1) {
    const m = all.value[i1];
    return m ? `${m.label}` : '';
  }
  const a = all.value[i0], b = all.value[i1];
  return a && b ? `${a.label} – ${b.label}` : '';
});

const legendColors = { expense: 'var(--zj-expense)', income: 'var(--zj-income)', net: 'var(--v2-ink-3)' };
</script>

<template>
  <div ref="wrap">
    <div style="display:flex;align-items:center;gap:10px;flex-wrap:wrap;margin-bottom:8px">
      <div class="v2-chart-legend">
        <span><i :style="{ background: legendColors.expense }" />支出</span>
        <span><i :style="{ background: legendColors.income }" />收入</span>
        <span><i :style="{ background: legendColors.net }" />结余</span>
      </div>
      <span class="v2-date-pill num" style="margin-left:auto">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><rect x="3.5" y="5" width="17" height="15.5" rx="2.5"/><path d="M8 3v4M16 3v4M3.5 10.5h17"/></svg>
        {{ dateLabel }}
      </span>
    </div>
    <div class="v2-ts-plot" @mousemove="onMove" @mouseleave="onLeave">
      <svg :viewBox="`0 0 ${W} ${H}`" preserveAspectRatio="none" class="v2-anim">
        <template v-if="n">
          <g v-html="grid" />
          <g v-html="body" />
        </template>
      </svg>
      <div class="v2-crosshair" ref="ch" />
      <div class="v2-tooltip" ref="tip" />
    </div>
    <div class="v2-ts-nav" ref="navEl" @pointerdown="onNavDown">
      <svg viewBox="0 0 1000 46" preserveAspectRatio="none" v-html="navSvg" />
      <div class="v2-nav-dim" ref="dimL" style="left:0" />
      <div class="v2-nav-dim" ref="dimR" style="right:0" />
      <div class="v2-nav-win" ref="winEl" @pointerdown="onWinDown">
        <div class="v2-nav-handle l" @pointerdown="onHandleDown($event, 'l')" />
        <div class="v2-nav-handle r" @pointerdown="onHandleDown($event, 'r')" />
      </div>
    </div>
  </div>
</template>

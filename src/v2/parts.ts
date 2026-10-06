// v2 图表几何助手（lieflat-charts 模板移植，prototype.html 同源）
export const P2R = Math.PI / 180;

export function prnd(a: number, b: number): number {
  let h = ((a * 374761393 + b * 668265263) >>> 0) || 1;
  h ^= h >>> 13;
  h = Math.imul(h, 1274126177) >>> 0;
  h ^= h >>> 16;
  return (h % 10000) / 10000;
}

export function ppol(cx: number, cy: number, r: number, a: number): [number, number] {
  return [cx + r * Math.cos(a * P2R), cy + r * Math.sin(a * P2R)];
}

export function psect(cx: number, cy: number, r0: number, r1: number, a0: number, a1: number): string {
  const p0 = ppol(cx, cy, r1, a0), p1 = ppol(cx, cy, r1, a1), p2 = ppol(cx, cy, r0, a1), p3 = ppol(cx, cy, r0, a0);
  const laf = a1 - a0 > 180 ? 1 : 0;
  return `M${p0[0].toFixed(1)} ${p0[1].toFixed(1)} A${r1} ${r1} 0 ${laf} 1 ${p1[0].toFixed(1)} ${p1[1].toFixed(1)} L${p2[0].toFixed(1)} ${p2[1].toFixed(1)} A${r0} ${r0} 0 ${laf} 0 ${p3[0].toFixed(1)} ${p3[1].toFixed(1)} Z`;
}

export function pblob(x: number, y: number, r: number, seed: number): string {
  const n = Math.max(14, Math.round(r * 1.6));
  const pts: [number, number][] = [];
  for (let t = 0; t < n; t++) {
    const a = (t / n) * Math.PI * 2;
    const w = 1 + 0.055 * Math.sin(a * 2 + seed * 7) + 0.04 * Math.sin(a * 3 + seed * 13) + (prnd(seed + t, 3) - 0.5) * 0.03;
    pts.push([x + Math.cos(a) * r * w, y + Math.sin(a) * r * w]);
  }
  let d = `M${pts[0][0].toFixed(1)} ${pts[0][1].toFixed(1)}`;
  for (let t = 0; t < n; t++) {
    const p = pts[t], q = pts[(t + 1) % n];
    d += ` Q${p[0].toFixed(1)} ${p[1].toFixed(1)} ${((p[0] + q[0]) / 2).toFixed(1)} ${((p[1] + q[1]) / 2).toFixed(1)}`;
  }
  return d + ' Z';
}

export function pTick(v: number): string {
  return v >= 1000 ? `${Math.round(v / 100) / 10}k` : String(Math.round(v));
}

export function money(n: number): string {
  const neg = n < 0;
  const a = Math.abs(n);
  const s = Number.isInteger(a) ? a.toLocaleString('zh-CN') : a.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  return (neg ? '-' : '') + '¥' + s;
}

export function monthLabel(y: number, m: number): string {
  const now = new Date();
  return (y < now.getFullYear() ? `'${String(y).slice(2)}/` : '') + m + '月';
}

// 标签防重叠：按 y 排序后强制最小间距
export function spreadLabels(items: { y: number }[], minGap: number, top?: number, bottom?: number): void {
  const arr = items.slice().sort((a, b) => a.y - b.y);
  for (let i = 1; i < arr.length; i++) if (arr[i].y - arr[i - 1].y < minGap) arr[i].y = arr[i - 1].y + minGap;
  if (bottom != null && arr.length && arr[arr.length - 1].y > bottom) {
    const shift = arr[arr.length - 1].y - bottom;
    arr.forEach((o) => (o.y -= shift));
    for (let i = 1; i < arr.length; i++) if (arr[i].y - arr[i - 1].y < minGap) arr[i].y = arr[i - 1].y + minGap;
  }
  if (top != null && arr.length && arr[0].y < top) {
    const shift = top - arr[0].y;
    arr.forEach((o) => (o.y += shift));
  }
}

export function pnum(v: string, s: string, c = ''): string {
  return `<div class="v2-pnum"><div class="pv">${v}</div><div class="ps">${s}</div>${c ? `<div class="pc">${c}</div>` : ''}</div>`;
}

export function paperHead(h2: string, sub: string): string {
  return `<div class="p-h2">${h2}</div><div class="p-sub">${sub}</div>`;
}

export function paperSrc(t: string): string {
  return `<div class="p-src">${t}</div>`;
}

export function esc(s: string): string {
  return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/"/g, '&quot;');
}

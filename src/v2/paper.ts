// v2 纸面卡构建器（prototype.html 移植；输入真实 Tx 数据，输出 HTML 字符串，由 PaperCard v-html 渲染）
import type { Tx } from '../types';
import {
  prnd, ppol, psect, pblob, pTick, money, monthLabel, pnum, paperHead, paperSrc, esc, P2R,
} from './parts';

export interface WinMonth { y: number; m: number; label: string }
export interface Win {
  label: string;
  months: WinMonth[];
  txs: Tx[];
  days: number;
  fa: string;
  fb: string;
}

const isExp = (t: Tx) => t.tx_type === '支出';
const isInc = (t: Tx) => t.tx_type === '收入';
const dayOf = (t: Tx) => t.tx_time.slice(0, 10);
const catOf = (t: Tx) => t.l1 || '未分类';

/* ① 一页纸（图六式数字卡） */
export function buildLedgerCard(win: Win): string {
  const list = win.txs;
  const ce = list.filter(isExp).reduce((s, t) => s + t.amount, 0);
  const ci = list.filter(isInc).reduce((s, t) => s + t.amount, 0);
  const exs = list.filter(isExp).map((t) => t.amount).sort((x, y) => x - y);
  const med = exs.length ? exs[Math.floor(exs.length / 2)] : 0;
  const net = ci - ce;
  const rate = ci > 0 ? Math.round(net / ci * 100) : 0;
  return paperHead(`一页纸 · ${esc(win.label)}`, `${win.fa} – ${win.fb} · 数字优先，图表其次`)
    + '<div class="v2-pnums">'
    + pnum(money(ce), `日均 ${money(Math.round(ce / Math.max(1, win.days)))}`, `支出 · ${exs.length} 笔`)
    + pnum(money(med), '一半的笔数低于它', '单笔中位数')
    + pnum(money(ci), '区间进账合计', '收入')
    + pnum((net >= 0 ? '+' : '-') + money(Math.abs(net)), '收入 − 支出', '结余')
    + pnum(`${rate}%`, '结余 ÷ 收入', '储蓄率')
    + '</div><div class="p-src">MONTHLY LEDGER · PORCELAIN · 账痕</div>';
}

/* ② DOT HEAT · 区间内的消费作息 */
export function buildDotHeatCard(win: Win): string {
  const wd = ['周一', '周二', '周三', '周四', '周五', '周六', '周日'];
  const heat = ['var(--p-data)', 'var(--p-data2)', 'var(--p-fd)'];
  const weeks = Math.min(26, Math.ceil(win.days / 7));
  const capped = win.days > weeks * 7;
  const byDay: Record<string, number> = {};
  for (const t of win.txs) if (isExp(t)) byDay[dayOf(t)] = (byDay[dayOf(t)] || 0) + t.amount;
  const grid: Record<string, number> = {};
  let max = 0, mi = 0, mj = 0;
  const end = new Date(win.fb + 'T00:00:00');
  for (let off = 0; off < weeks * 7; off++) {
    const d = new Date(end);
    d.setDate(d.getDate() - off);
    const key = d.toISOString().slice(0, 10);
    const dow = d.getDay(), wk = Math.floor(off / 7), row = dow === 0 ? 6 : dow - 1;
    const gk = wk + '-' + row;
    grid[gk] = (grid[gk] || 0) + (byDay[key] || 0);
  }
  for (let wk = 0; wk < weeks; wk++) for (let row = 0; row < 7; row++) {
    const v = grid[wk + '-' + row] || 0;
    if (v > max) { max = v; mi = row; mj = wk; }
  }
  const W = 76 + weeks * 44, H = 262, x0 = 76, y0 = 26;
  let g = '';
  const endD = end;
  for (let off = 0; off < weeks * 7; off++) {
    const d = new Date(endD);
    d.setDate(d.getDate() - off);
    const dow = d.getDay(), wk = Math.floor(off / 7), row = dow === 0 ? 6 : dow - 1;
    const v = grid[wk + '-' + row] || 0, x = x0 + wk * 44, y = y0 + row * 30;
    const delay = (row * 0.05 + wk * 0.015).toFixed(3);
    if (!v) { g += `<circle cx="${x}" cy="${y}" r=".9" fill="var(--p-quiet)" class="ppop" style="animation-delay:${delay}s"/>`; continue; }
    const hero = row === mi && wk === mj;
    g += `<circle cx="${x}" cy="${y}" r="${(1.2 + Math.sqrt(v / 30) * 2.1).toFixed(1)}" fill="${v > max * 0.66 ? heat[0] : v > max * 0.33 ? heat[1] : heat[2]}" class="ppop" style="animation-delay:${delay}s"><title>${esc(win.label)} · ${wd[row]} — ¥${Math.round(v)}</title></circle>`;
    if (hero) g += `<circle cx="${x}" cy="${y}" r="${(1.2 + Math.sqrt(v / 30) * 2.1 + 3.4).toFixed(1)}" fill="none" stroke="var(--p-hero)" stroke-width="1" stroke-dasharray="2 3" class="dotin" style="animation-delay:1s"/>`;
  }
  for (let j = weeks - 1; j >= 0; j -= 4) {
    const d = new Date(endD);
    d.setDate(d.getDate() - j * 7);
    g += `<text x="${x0 + j * 44}" y="${y0 + 6 * 30 + 22}" font-size="7" font-weight="600" fill="var(--p-faint)" text-anchor="middle" class="dotin" style="animation-delay:${j * 0.02}s">${d.getMonth() + 1}/${d.getDate()}</text>`;
  }
  const list = win.txs.filter(isExp);
  const ce = list.reduce((s, t) => s + t.amount, 0);
  let peak = { amount: 0, merchant: '', date: '' };
  for (const t of list) if (t.amount > peak.amount) peak = { amount: t.amount, merchant: t.merchant || '', date: t.tx_time };
  return paperHead(`消费作息 · ${esc(win.label)}`, `点面积 = 当日支出 · 虚线圈 = 峰值日 · 极小点 = 安静的日子${capped ? `（区间较长，仅显示最近 ${weeks} 周）` : ''}`)
    + `<div style="flex:1;display:flex;align-items:center"><svg viewBox="0 0 ${W} ${H}" style="max-height:300px">${g}</svg></div>`
    + `<div class="v2-pnums" style="margin-top:auto">${pnum(money(ce), `日均 ${money(Math.round(ce / Math.max(1, win.days)))}`, '区间支出')}${pnum(money(peak.amount), esc(peak.merchant), `最大单笔 · ${peak.date.slice(5, 10).replace('-', '/')}`)}${pnum(wd[mi], `距区间末第 ${weeks - mj} 周`, '作息最重的一天')}</div>`
    + '<div class="p-src" style="text-align:center;letter-spacing:.12em">DOT AREA = DAILY SPEND · DASHED RING = THE PEAK · TINY DOT = A QUIET DAY</div>';
}

/* ③ TICK DONUT · 支出构成钟摆 */
export function buildDonutCard(win: Win): string {
  const list = win.txs.filter(isExp);
  const total = list.reduce((s, t) => s + t.amount, 0);
  const agg: Record<string, number> = {};
  for (const t of list) agg[catOf(t)] = (agg[catOf(t)] || 0) + t.amount;
  const entries = Object.entries(agg).sort((x, y) => y[1] - x[1]);
  const top = entries.slice(0, 4);
  const shades = ['var(--p-ink)', 'var(--p-data)', 'var(--p-data2)', 'var(--p-fd)'];
  const D = top.map((e, i) => ({ name: e[0], v: Math.round((e[1] / Math.max(1, total)) * 100), shade: shades[i] }));
  let resid = 100 - D.reduce((s, d) => s + d.v, 0);
  if (D.length) D[0].v += resid;
  const others = entries.slice(4).reduce((s, e) => s + e[1], 0);
  if (others > 0) D.push({ name: '其他', v: 0, shade: 'var(--p-fd)' });
  const cx = 200, cy = 150, R0 = 64;
  let k0 = 0, g = '';
  for (const d of D) {
    const si = D.indexOf(d);
    for (let k = 0; k < d.v; k++) {
      const idx = k0 + k, ang = idx * 3.6 - 90;
      const len = 12 + prnd(idx + 1, si + 2) * 6;
      const [x1, y1] = ppol(cx, cy, R0, ang), [x2, y2] = ppol(cx, cy, R0 + len, ang);
      g += `<line x1="${x1.toFixed(1)}" y1="${y1.toFixed(1)}" x2="${x2.toFixed(1)}" y2="${y2.toFixed(1)}" stroke="${d.shade}" stroke-width="1.9" class="dotin" style="animation-delay:${(idx * 0.012).toFixed(3)}s"/>`;
      if (idx % 10 === 0) {
        const [dx, dy] = ppol(cx, cy, R0 - 5, ang);
        g += `<circle cx="${dx.toFixed(1)}" cy="${dy.toFixed(1)}" r=".8" fill="var(--p-faint)" class="dotin" style="animation-delay:${(idx * 0.012).toFixed(3)}s"/>`;
      }
    }
    const mid = (k0 + d.v / 2) * 3.6 - 90;
    const [lx, ly] = ppol(cx, cy, R0 + 40, mid), [gx, gy] = ppol(cx, cy, R0 + 20, mid);
    g += `<line x1="${gx.toFixed(1)}" y1="${gy.toFixed(1)}" x2="${lx.toFixed(1)}" y2="${ly.toFixed(1)}" stroke="var(--p-faint)" stroke-width=".7" stroke-dasharray="1 3" class="dotin" style="animation-delay:${(0.6 + si * 0.1).toFixed(2)}s"/>`;
    const anchor = Math.cos(mid * P2R) > 0.3 ? 'start' : Math.cos(mid * P2R) < -0.3 ? 'end' : 'middle';
    g += `<text x="${lx.toFixed(1)}" y="${(ly + 3).toFixed(1)}" font-size="8" font-weight="800" fill="${d.shade}" text-anchor="${anchor}" letter-spacing=".06em" style="paint-order:stroke;stroke:var(--p-bg);stroke-width:3px;animation-delay:${(0.65 + si * 0.1).toFixed(2)}s" class="dotin">${esc(d.name)} · ${d.v}%</text>`;
    k0 += d.v;
  }
  g += `<text x="${cx}" y="${cy - 2}" font-size="22" font-weight="800" fill="var(--p-ink)" text-anchor="middle" class="dotin" style="animation-delay:.9s">¥${pTick(total)}</text>`;
  g += `<text x="${cx}" y="${cy + 14}" font-size="7" font-weight="600" fill="var(--p-mut)" text-anchor="middle" letter-spacing=".1em" class="dotin" style="animation-delay:.9s">TICKS · 一格 = 1% 支出</text>`;
  g += `<text x="200" y="296" font-size="7" font-weight="600" fill="var(--p-faint)" text-anchor="middle" letter-spacing=".12em" class="dotin" style="animation-delay:1.1s">十二点为零 · 每十格一枚圆点 · 顺时针读</text>`;
  return paperHead(`支出构成，按分类钟摆 · ${esc(win.label)}`, '一格 = 1% 支出 · 颜色深→浅 = 金额降序（深色为主力分类）')
    + `<svg viewBox="0 0 400 306" style="max-height:330px">${g}</svg>`
    + `<div class="p-src" style="text-align:center">${D.map((d) => `${esc(d.name)} ¥${pTick((d.v / 100) * total)}`).join(' · ')}</div>`;
}

/* ④ RUNG WATERFALL · 收支瀑布 */
export function buildWaterfallCard(win: Win): string {
  const list = win.txs;
  const ci = list.filter(isInc).reduce((s, t) => s + t.amount, 0);
  const ce = list.filter(isExp).reduce((s, t) => s + t.amount, 0);
  const net = ci - ce;
  const agg: Record<string, number> = {};
  for (const t of list) if (isExp(t)) agg[catOf(t)] = (agg[catOf(t)] || 0) + t.amount;
  const cats = Object.entries(agg).sort((x, y) => y[1] - x[1]);
  const D: [string, number][] = [['收入', ci]];
  for (const [k, v] of cats.slice(0, 5)) D.push([k, -v]);
  const otherAmt = cats.slice(5).reduce((s, e) => s + e[1], 0);
  if (otherAmt > 0) D.push(['其他', -otherAmt]);
  D.push(['结余', net]);
  const W = Math.max(720, 720), rowsN = D.length, xGap = (W - 116) / rowsN, HW = Math.min(20, xGap * 0.24);
  const unit = (() => {
    const raw = ci / 32;
    for (const s of [100, 200, 250, 500, 1000, 2000, 5000, 10000, 20000, 50000]) if (s >= raw) return s;
    return Math.ceil(raw / 1000) * 1000;
  })();
  const base = 296, step = 208 / Math.max(1, ci / unit);
  const yOf = (k: number) => base - k * step;
  let lv = 0, g = '';
  D.forEach(([name, v], i) => {
    const x = 58 + i * xGap;
    let lo: number, hi: number, isTotal: boolean;
    if (name === '收入') { lo = 0; hi = v; lv = v; isTotal = true; }
    else if (name === '结余') { lo = 0; hi = lv; isTotal = true; }
    else { lo = lv + v; hi = lv; lv = lv + v; isTotal = false; }
    const isNeg = v < 0 && !isTotal;
    const fromR = lo / unit, toR = hi / unit;
    const n = Math.max(1, Math.round(Math.abs(toR - fromR)));
    for (let k = 0; k < n; k++) {
      const y = yOf(fromR + k), w = HW - 1.2 + prnd(k + 1, i + 2) * 2.4;
      const delay = (i * 0.12 + k * 0.014).toFixed(3);
      if (isNeg) g += `<line x1="${(x - w).toFixed(1)}" y1="${y.toFixed(1)}" x2="${(x + w).toFixed(1)}" y2="${y.toFixed(1)}" stroke="var(--p-data2)" stroke-width="1.8" stroke-dasharray="2.5 2.5" class="dotin" style="animation-delay:${delay}s"/>`;
      else g += `<line x1="${(x - w).toFixed(1)}" y1="${y.toFixed(1)}" x2="${(x + w).toFixed(1)}" y2="${y.toFixed(1)}" stroke="var(--p-data)" stroke-width="1.8" class="dotin" style="animation-delay:${delay}s"/>`;
    }
    if (i < D.length - 1) {
      const nx = 58 + (i + 1) * xGap, lvl = (isTotal ? v : isNeg ? lo : hi) / unit;
      g += `<line x1="${(x + HW + 2).toFixed(1)}" y1="${yOf(lvl).toFixed(1)}" x2="${(nx - HW - 2).toFixed(1)}" y2="${yOf(lvl).toFixed(1)}" stroke="var(--p-faint)" stroke-width=".7" stroke-dasharray="2 3" class="dotin" style="animation-delay:${(0.3 + i * 0.12).toFixed(2)}s"/>`;
    }
    const topY = yOf(Math.max(fromR, toR));
    g += `<text x="${x.toFixed(1)}" y="${(topY - 8).toFixed(1)}" font-size="10" font-weight="800" fill="${isNeg ? 'var(--p-data2)' : 'var(--p-ink)'}" text-anchor="middle" class="dotin" style="animation-delay:${(0.4 + i * 0.12).toFixed(2)}s">${isNeg ? '−' : ''}${money(Math.abs(v))}</text>`;
    g += `<text x="${x.toFixed(1)}" y="${base + 18}" font-size="7.5" font-weight="700" fill="var(--p-mut)" text-anchor="middle" letter-spacing=".08em" class="dotin" style="animation-delay:${(i * 0.12).toFixed(2)}s">${name}</text>`;
  });
  g += `<line x1="30" y1="${base + 4}" x2="${W - 30}" y2="${base + 4}" stroke="var(--p-grid)" stroke-width=".8" class="dotin"/>`;
  g += `<text x="${W / 2}" y="344" font-size="7.5" font-weight="600" fill="var(--p-faint)" text-anchor="middle" letter-spacing=".12em" class="dotin" style="animation-delay:1.2s">实线瓦 = 进账 · 虚线瓦 = 扣减 · 一格 = ¥${unit}</text>`;
  return paperHead(`收支瀑布 · ${esc(win.label)}`, '从收入到结余，一步一步：实线瓦进账、虚线瓦扣减，阶梯停在多高，就剩多少')
    + `<svg viewBox="0 0 ${W} 360" style="max-height:440px">${g}</svg>`
    + '<div class="v2-pnums">'
    + pnum(money(ci), '区间收入合计')
    + pnum(money(ce), `区间支出 · ${list.filter(isExp).length} 笔`)
    + pnum((net >= 0 ? '+' : '-') + money(Math.abs(net)), '走到最后一级', '结余')
    + pnum(`${ci > 0 ? Math.round(net / ci * 100) : 0}%`, '结余 ÷ 收入', '储蓄率')
    + '</div>'
    + paperSrc('RUNG WATERFALL · PORCELAIN · 账痕');
}

/* ⑤ STACKED RUNGS · 收入构成 */
export function buildIncomeCard(win: Win): string {
  const byMonth = new Map<string, Record<string, number>>();
  const order: string[] = [];
  for (const m of win.months) {
    const key = `${m.y}-${String(m.m).padStart(2, '0')}`;
    byMonth.set(key, {});
    order.push(key);
  }
  for (const t of win.txs) {
    if (!isInc(t)) continue;
    const key = t.tx_time.slice(0, 7);
    const bucket = byMonth.get(key);
    if (bucket) bucket[catOf(t)] = (bucket[catOf(t)] || 0) + t.amount;
  }
  const srcTotals: Record<string, number> = {};
  for (const key of order) for (const [k, v] of Object.entries(byMonth.get(key) || {})) srcTotals[k] = (srcTotals[k] || 0) + v;
  const sources = Object.entries(srcTotals).sort((x, y) => y[1] - x[1]).slice(0, 3);
  const shades = ['var(--p-data)', 'var(--p-data2)', 'var(--p-fd)'];
  const maxTot = Math.max(...order.map((key) => Object.values(byMonth.get(key) || {}).reduce((s, v) => s + v, 0)), 1);
  const unit = (() => {
    const raw = maxTot / 24;
    for (const s of [100, 200, 250, 500, 1000, 2000, 5000]) if (s >= raw) return s;
    return Math.ceil(raw / 1000) * 1000;
  })();
  const n = order.length;
  const W = Math.max(680, 680);
  const colW = (W - 110) / Math.max(1, n), HW = Math.min(20, colW * 0.3);
  const base = 292, step = 210 / (maxTot / unit), H = 352;
  let g = '';
  order.forEach((key, i) => {
    const segs = byMonth.get(key) || {};
    const m = win.months[i];
    const x = 64 + i * colW;
    let k = 0;
    sources.forEach(([src], si) => {
      const amt = segs[src] || 0;
      const rungs = Math.round(amt / unit);
      for (let k2 = 0; k2 < rungs; k2++) {
        const y = base - (k + k2) * step, w = HW - 1.3 + prnd(k + k2 + 1, i * 3 + si + 2) * 2.6;
        g += `<line x1="${(x - w).toFixed(1)}" y1="${y.toFixed(1)}" x2="${(x + w).toFixed(1)}" y2="${y.toFixed(1)}" stroke="${shades[si]}" stroke-width="1.8" class="dotin" style="animation-delay:${(i * 0.09 + (k + k2) * 0.012).toFixed(3)}s"/>`;
      }
      k += rungs;
    });
    const tot = Object.values(segs).reduce((s, v) => s + v, 0);
    if (tot > 0) g += `<text x="${x.toFixed(1)}" y="${(base - (k - 1) * step - 10).toFixed(1)}" font-size="10" font-weight="800" fill="var(--p-ink)" text-anchor="middle" class="dotin" style="animation-delay:${(0.4 + i * 0.08).toFixed(2)}s">¥${pTick(tot)}</text>`;
    g += `<text x="${x.toFixed(1)}" y="${base + 18}" font-size="7.5" font-weight="700" fill="var(--p-mut)" text-anchor="middle" letter-spacing=".08em" class="dotin" style="animation-delay:${(i * 0.08).toFixed(2)}s">${monthLabel(m.y, m.m)}</text>`;
  });
  g += `<line x1="34" y1="${base + 4}" x2="${W - 30}" y2="${base + 4}" stroke="var(--p-grid)" stroke-width=".8" class="dotin"/>`;
  const ci = win.txs.filter(isInc).reduce((s, t) => s + t.amount, 0);
  let best = { label: '—', total: 0 };
  order.forEach((key, i) => {
    const tot = Object.values(byMonth.get(key) || {}).reduce((s, v) => s + v, 0);
    if (tot > best.total) best = { label: monthLabel(win.months[i].y, win.months[i].m), total: tot };
  });
  const legend = sources.map(([s2], i) => `${esc(s2)}（${['深', '中', '浅'][i]}）`).join(' / ') || '暂无收入';
  return paperHead(`收入构成 · ${esc(win.label)}`, `每根横瓦 = ${money(unit)} 收入 · 颜色深→浅 = 来源金额降序（${legend}）`)
    + `<svg viewBox="0 0 ${W} ${H}" style="max-height:400px">${g}</svg>`
    + '<div class="v2-pnums">'
    + pnum(money(ci), '区间收入合计')
    + pnum(money(Math.round(ci / Math.max(1, n))), `${n} 个月平均`, '月均收入')
    + pnum(money(best.total), best.label, '最高收入月')
    + pnum(`${win.txs.filter(isInc).length}`, '笔进账', '区间笔数')
    + '</div>'
    + paperSrc('STACKED RUNGS · PORCELAIN · 账痕');
}

/* ⑥ BUBBLE ALMANAC · 支出年鉴 */
export function buildAlmanacCard(win: Win): string {
  const rows = win.months.map((m) => {
    const key = `${m.y}-${String(m.m).padStart(2, '0')}`;
    const cats: Record<string, number> = {};
    let total = 0;
    for (const t of win.txs) {
      if (!isExp(t)) continue;
      if (t.tx_time.slice(0, 7) !== key) continue;
      cats[catOf(t)] = (cats[catOf(t)] || 0) + t.amount;
      total += t.amount;
    }
    return { label: monthLabel(m.y, m.m), cats, total };
  });
  const keys = Object.keys(rows.reduce((a2, r) => { for (const k of Object.keys(r.cats)) a2[k] = 1; return a2; }, {} as Record<string, number>));
  const totals: Record<string, number> = {};
  for (const k of keys) totals[k] = rows.reduce((s, r) => s + (r.cats[k] || 0), 0);
  const cols = keys.slice().sort((x, y) => totals[y] - totals[x]).slice(0, 8);
  const n = rows.length;
  const rowGap = Math.min(24, Math.floor(300 / Math.max(1, n - 1)));
  const W = 840, lastRow = 70 + (n - 1) * rowGap, evY = lastRow + 40, H = evY + 26;
  const rowY = (i: number) => 70 + i * rowGap, colX = (j: number) => 150 + j * 84;
  let g = '';
  for (let y = 44; y <= lastRow + 8; y += 6.8)
    g += `<line x1="56" y1="${y}" x2="822" y2="${y}" stroke="var(--p-quiet)" stroke-width=".5" class="dotin" style="animation-delay:${((y - 44) * 0.001).toFixed(2)}s"/>`;
  const vals: { v: number; i: number; j: number; c: string; amount: number }[] = [];
  rows.forEach((r, i) => {
    g += `<line x1="56" y1="${rowY(i)}" x2="822" y2="${rowY(i)}" stroke="var(--p-grid)" stroke-width=".9" class="dotin" style="animation-delay:${i * 0.04}s"/>`;
    g += `<text x="48" y="${rowY(i) + 3}" font-size="9" font-weight="700" fill="var(--p-mut)" text-anchor="end" class="dotin" style="animation-delay:${i * 0.04}s">${r.label}</text>`;
    cols.forEach((c, j) => {
      const v = r.cats[c] || 0;
      if (!v) return;
      vals.push({ v: Math.round(v / 10), i, j, c, amount: v });
    });
  });
  const colPeak: Record<string, number> = {};
  for (const c of cols) {
    let best = 0, bi = -1;
    rows.forEach((r, i) => { const v = r.cats[c] || 0; if (v > best) { best = v; bi = i; } });
    colPeak[c] = bi;
  }
  const mainCat = rows.map((r) => cols.reduce((a2, b2) => ((r.cats[b2] || 0) > (r.cats[a2] || 0) ? b2 : a2)));
  vals.sort((x, y) => y.v - x.v);
  const top3 = vals.slice(0, 3);
  const top1 = vals[0];
  const RAMP = ['var(--p-ramp)', 'var(--p-fd)', 'var(--p-data2)', 'var(--p-data)', 'var(--p-ink)'];
  let dots = '';
  vals.forEach((o, k) => {
    const x = colX(o.j) + (prnd(o.i * 7 + o.j + 2, o.j + 9) - 0.5) * 10, y = rowY(o.i), r = Math.max(2, Math.sqrt(o.v) * 2.5);
    const seed = o.i * 12 + o.j, isTop = top3.includes(o);
    const tier = Math.min(4, Math.floor(o.v / 14));
    const isPeak = colPeak[o.c] === o.i, isMain = mainCat[o.i] === o.c;
    let inner = isPeak
      ? `<path d="${pblob(x, y, r, seed)}" fill="none" stroke="var(--p-data2)" stroke-width="1.2" stroke-dasharray="3 3"/>`
      : `<path d="${pblob(x, y, r, seed)}" fill="${RAMP[tier]}" fill-opacity="${(0.16 + prnd(o.i + 2, o.j + 4) * 0.14).toFixed(2)}" stroke="${isTop ? 'var(--p-hero)' : RAMP[tier]}" stroke-opacity="${isTop ? 0.95 : (0.5 + prnd(o.i + 5, o.j + 7) * 0.38).toFixed(2)}" stroke-width="${isTop ? 1.7 : (0.8 + prnd(o.i + 3, o.j + 11)).toFixed(1)}"/>`;
    if (isMain && !isPeak) {
      const ox = (prnd(seed + 1, 17) - 0.5) * r * 0.3, oy = (prnd(seed + 3, 19) - 0.5) * r * 0.3;
      inner += `<path d="${pblob(x + ox, y + oy, Math.max(1.4, r * 0.17), seed + 29)}" fill="${isTop ? 'var(--p-hero)' : 'var(--p-ink)'}"/>`;
    }
    dots += `<g class="ppop" style="animation-delay:${(0.15 + k * 0.012).toFixed(3)}s">${inner}<title>${esc(o.c)} · ${rows[o.i].label} — ¥${o.amount}${isPeak ? '（区间内该分类峰值）' : ''}</title></g>`;
  });
  if (top1) {
    const lx = colX(top1.j) + (prnd(top1.i * 7 + top1.j + 2, top1.j + 9) - 0.5) * 10;
    const ly = rowY(top1.i) + Math.max(2, Math.sqrt(top1.v) * 2.5) + 12;
    dots += `<text x="${lx.toFixed(1)}" y="${ly.toFixed(1)}" font-size="9" font-weight="800" fill="var(--p-hero)" text-anchor="middle" style="paint-order:stroke;stroke:var(--p-bg);stroke-width:3px;animation-delay:1s" class="dotin">最大一笔 ¥${top1.amount}</text>`;
  }
  g += dots;
  cols.forEach((c, j) => {
    g += `<text x="${colX(j)}" y="34" font-size="7.5" font-weight="700" fill="var(--p-mut)" letter-spacing=".1em" transform="rotate(-32 ${colX(j)} 34)" class="dotin" style="animation-delay:${j * 0.03}s">${esc(c)}</text>`;
  });
  const rentCol = cols.indexOf('居住');
  if (rentCol >= 0 && n > 1) {
    g += `<path d="M${colX(rentCol) + 14} ${rowY(0) + 16} C${colX(rentCol) + 40} ${rowY(0) + 40} ${colX(rentCol) + 60} ${rowY(0) + 44} ${colX(rentCol) + 80} ${rowY(0) + 44}" fill="none" stroke="var(--p-faint)" stroke-width=".7" class="dotin" style="animation-delay:1.1s"/>`;
    g += `<text x="${colX(rentCol) + 86}" y="${rowY(0) + 47}" font-size="7" fill="var(--p-lab)" font-style="italic" class="dotin" style="animation-delay:1.1s">居住等固定支出多在每月 1 日</text>`;
  }
  g += `<line x1="56" y1="${evY - 16}" x2="822" y2="${evY - 16}" stroke="var(--p-faint)" stroke-width=".8" class="dotin" style="animation-delay:1.2s"/>`;
  g += `<text x="56" y="${evY - 24}" font-size="7" font-weight="800" fill="var(--p-mut)" letter-spacing=".16em" class="dotin" style="animation-delay:1.2s">FIXED EVENTS · 每月固定收支</text>`;
  [['1日', '发薪 + 房租'], ['8日', '水电燃气'], ['12日', '话费网费'], ['15日', '理财收益']].forEach((ev, k) => {
    const x = 90 + k * 196;
    g += `<line x1="${x}" y1="${evY - 16}" x2="${x}" y2="${evY - 6}" stroke="var(--p-mut)" stroke-width="1" class="dotin" style="animation-delay:${(1.3 + k * 0.08).toFixed(2)}s"/>`;
    g += `<text x="${x}" y="${evY + 4}" font-size="8" font-weight="800" fill="var(--p-ink)" class="dotin" style="animation-delay:${(1.3 + k * 0.08).toFixed(2)}s">${ev[0]}</text>`;
    g += `<text x="${x}" y="${evY + 15}" font-size="7" fill="var(--p-mut)" class="dotin" style="animation-delay:${(1.3 + k * 0.08).toFixed(2)}s">${ev[1]}</text>`;
  });
  const monthAvg = rows.reduce((s, r) => s + r.total, 0) / Math.max(1, n);
  return paperHead(`支出年鉴 · ${esc(win.label)}`, '气泡面积 = 该月该分类支出 · 深色墨核 = 当月主力分类 · 虚线圈 = 该分类的峰值月')
    + `<div class="p-split" style="grid-template-columns:1fr 225px"><div><svg viewBox="0 0 ${W} ${H}" style="max-height:430px">${g}</svg></div>`
    + `<div style="padding-right:14px"><div class="p-note">每一行是一个月，账本发丝纸纹铺在全部气泡下面——先读纹理，再读数字。墨色越堆越重的列，就是钱最常去的地方；灰色越叠越深的那一格，就是这个月的支柱开销。</div>`
    + `<div class="p-legend">● &nbsp;气泡面积 = 金额（内圈一格 ≈ ¥10）<br>◉ &nbsp;深色墨核 = 当月主力分类<br>◌ &nbsp;虚线圈 = 该分类区间峰值月<br>▮ &nbsp;描边加粗 = 全表 Top3</div>`
    + `<div class="v2-pnums" style="flex-direction:column;border-top:1.5px solid var(--p-ink);border-bottom:1.5px solid var(--p-ink)">`
    + pnum(money(monthAvg), `${n} 个月平均`, '月均支出')
    + pnum(money(rows[n - 1]?.total ?? 0), '区间最后一个月', '最新一个月')
    + '</div></div></div>'
    + paperSrc('BUBBLE ALMANAC · PORCELAIN · 账痕');
}

/* ⑦ RADIAL PATCHWORK · 区间支出叠成一圈 */
export function buildPatchCard(win: Win): string {
  const list = win.txs.filter(isExp);
  const CX = 200, CY = 152;
  let g = '';
  for (let d = 1; d <= 31; d++) {
    const a = -90 + ((d - 1) / 31) * 360;
    const [x1, y1] = ppol(CX, CY, 138, a), [x2, y2] = ppol(CX, CY, d % 5 === 0 ? 144 : 141, a);
    g += `<line x1="${x1.toFixed(1)}" y1="${y1.toFixed(1)}" x2="${x2.toFixed(1)}" y2="${y2.toFixed(1)}" stroke="var(--p-faint)" stroke-width="${d % 5 === 0 ? 1 : 0.5}" class="dotin" style="animation-delay:${(d * 0.006).toFixed(3)}s"/>`;
  }
  [1, 8, 15, 22, 29].forEach((d) => {
    const [x, y] = ppol(CX, CY, 155, -90 + ((d - 1) / 31) * 360);
    g += `<text x="${x.toFixed(1)}" y="${(y + 3).toFixed(1)}" font-size="8" font-weight="700" fill="var(--p-mut)" text-anchor="middle" class="dotin">${d}日</text>`;
  });
  const RAMP = ['var(--p-ramp)', 'var(--p-fd)', 'var(--p-data2)', 'var(--p-data)', 'var(--p-ink)'];
  const ranks = list.map((t) => t.amount).slice().sort((a2, b2) => a2 - b2);
  const pctOf = (v: number) => (ranks.length > 1 ? ranks.indexOf(v) / (ranks.length - 1) : 1);
  list.forEach((t, i) => {
    const day = +t.tx_time.slice(8, 10) || 1;
    const a0 = -90 + ((day - 1) / 31) * 360 + 2, sw = 360 / 31 - 4;
    const r1 = 30 + Math.min(1, pctOf(t.amount)) * 104;
    const tier = Math.min(4, Math.floor(t.amount / 300));
    g += `<path d="${psect(CX, CY, 16, r1, a0, a0 + sw)}" fill="${RAMP[tier]}" fill-opacity="${(0.17 + prnd(i + 1, 6) * 0.15).toFixed(2)}" class="dotin" style="animation-delay:${(0.2 + i * 0.012).toFixed(3)}s"><title>${esc(t.merchant || '未记商家')} · ${t.tx_time.slice(5, 10)} — ¥${t.amount}</title></path>`;
  });
  list.filter((t) => t.amount >= 500).forEach((t, k) => {
    const day = +t.tx_time.slice(8, 10) || 1;
    const a0 = -90 + ((day - 1) / 31) * 360 + 2, sw = 360 / 31 - 4, r1 = 30 + Math.min(1, pctOf(t.amount)) * 104;
    g += `<path d="${psect(CX, CY, 16, r1, a0, a0 + sw)}" fill="none" stroke="var(--p-hero)" stroke-width="1.5" class="dotin" style="animation-delay:${(1.2 + k * 0.15).toFixed(2)}s"><title>${esc(t.merchant || '未记商家')} · ¥${t.amount}（大额）</title></path>`;
  });
  g += `<circle cx="${CX}" cy="${CY}" r="3" fill="var(--p-data)" class="ppop" style="animation-delay:1.3s"/>`;
  const total = list.reduce((s, t) => s + t.amount, 0);
  return paperHead(`支出叠成一圈 · ${esc(win.label)}`, '一瓦 = 一笔交易 · 角度 = 日期（1–31日） · 半径 = 金额排位 · 深框 = ≥¥500 大额')
    + `<div style="display:flex;gap:18px;align-items:center"><div style="flex:1;min-width:0"><svg viewBox="0 0 400 296" style="max-height:420px">${g}`
    + `<text x="200" y="288" font-size="7.5" font-weight="600" fill="var(--p-faint)" text-anchor="middle" letter-spacing=".12em" class="dotin" style="animation-delay:1.4s">OUTLINED = ≥¥500 · 墨色堆积处 = 最常花钱的日期</text></svg></div>`
    + `<div class="v2-pnums" style="flex-direction:column;width:132px;flex:none;margin-right:16px;border-top:1.5px solid var(--p-ink);border-bottom:1.5px solid var(--p-ink)">`
    + pnum(`${list.length}`, '笔交易', '区间支出')
    + pnum(`¥${pTick(total)}`, '合计支出', '叠瓦最厚 = 花钱最密')
    + pnum(`¥${pTick(list.length ? total / list.length : 0)}`, '平均单笔', '中位数见一页纸')
    + '</div></div>'
    + paperSrc('RADIAL PATCHWORK · PORCELAIN · 账痕');
}

/* ⑧ TICK ROWS · 钱流向了谁 */
export function buildRowsCard(win: Win): string {
  const list = win.txs.filter(isExp);
  const agg: Record<string, { n: number; amt: number }> = {};
  for (const t of list) {
    const k = t.merchant || '未记商家';
    agg[k] = agg[k] || { n: 0, amt: 0 };
    agg[k].n++;
    agg[k].amt += t.amount;
  }
  const D = Object.entries(agg).sort((a2, b2) => b2[1].amt - a2[1].amt).slice(0, 7);
  const maxN = Math.max(...D.map((d) => d[1].n), 1);
  const W = 620, rowH = 46, X0 = 170, PX = Math.min(8, (W - 280) / maxN), H = D.length * rowH + 74;
  let g = '';
  D.forEach(([name, o], i) => {
    const y = 18 + i * rowH;
    g += `<text x="${X0 - 12}" y="${y + 4}" font-size="9" font-weight="700" fill="var(--p-lab)" text-anchor="end" letter-spacing=".08em" class="dotin" style="animation-delay:${i * 0.08}s">${esc(name)}</text>`;
    g += `<line x1="${X0}" y1="${y + 10}" x2="${(X0 + maxN * PX).toFixed(1)}" y2="${y + 10}" stroke="var(--p-grid)" stroke-width=".6" class="dotin" style="animation-delay:${i * 0.08}s"/>`;
    for (let k = 0; k < o.n; k++) {
      const x = X0 + k * PX + PX / 2, h = 10 + prnd(k + 1, i + 2) * 7;
      g += `<line x1="${x.toFixed(1)}" y1="${y + 10}" x2="${x.toFixed(1)}" y2="${(y + 10 - h).toFixed(1)}" stroke="var(--p-data)" stroke-width="2" class="dotin" style="animation-delay:${(i * 0.08 + k * 0.012).toFixed(3)}s"/>`;
      if (k % 5 === 4) g += `<circle cx="${x.toFixed(1)}" cy="${y + 15}" r=".9" fill="var(--p-faint)" class="dotin" style="animation-delay:${(i * 0.08 + k * 0.012).toFixed(3)}s"/>`;
    }
    g += `<text x="${(X0 + o.n * PX + 12).toFixed(1)}" y="${y + 5}" font-size="12" font-weight="800" fill="var(--p-ink)" class="dotin" style="animation-delay:${(0.4 + i * 0.08).toFixed(2)}s">¥${pTick(o.amt)}</text>`;
  });
  const top = D[0];
  return paperHead(`钱流向了谁 · ${esc(win.label)}`, '一竖 = 一笔交易 · 每 5 笔一枚圆点 · 右侧数字 = 区间合计')
    + `<svg viewBox="0 0 ${W} ${H}" style="max-height:400px">${g}</svg>`
    + '<div class="v2-pnums">'
    + pnum(`¥${pTick(top ? top[1].amt : 0)}`, `${top ? top[1].n : 0} 笔`, `最常去 · ${top ? esc(top[0]) : '—'}`)
    + pnum(`¥${pTick(list.reduce((s, t) => s + t.amount, 0))}`, `${list.length} 笔`, `${esc(win.label)}支出合计`)
    + '</div>'
    + '<div class="p-src" style="text-align:center;letter-spacing:.12em">ONE TICK = ONE TRANSACTION · DOT MARKS EVERY FIFTH</div>';
}

/* 排行（明细页底部，维度切换；只输出行，卡头由视图渲染） */
export function buildRankCard(rows: { name: string; value: number }[]): string {
  const max = rows.length ? rows[0].value : 1;
  const total = rows.reduce((s, r) => s + r.value, 0);
  const ladder = [1, 0.78, 0.6, 0.46, 0.35, 0.26, 0.19, 0.14];
  const body = rows.slice(0, 8).map((r, i) =>
    `<div class="v2-rank-row" data-rank="${esc(r.name)}"><span class="rk">${i + 1}</span><span class="rk-name">${esc(r.name)}</span>`
    + `<div class="v2-track"><i style="width:${Math.round((r.value / max) * 100)}%;opacity:${ladder[i] ?? 0.1}"></i></div>`
    + `<span class="rk-amt">¥${pTick(r.value)}</span><span class="rk-pct">${total ? Math.round((r.value / total) * 100) : 0}%</span></div>`
  ).join('');
  return body || '<div style="padding:20px;color:var(--v2-ink-3);font-size:13px">区间内暂无数据</div>';
}

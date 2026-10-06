// v2 统计页长图导出：纯 Canvas 按布局坐标复刻页面（不依赖 foreignObject）
// 结构约定：.v2-pagehead（标题/副题/范围胶囊）+ 若干 .v2-paper 卡（p-h2/p-sub/svg/pnums/p-src）
import { api } from '../api';
import { fmtAmount } from '../api';

function cssVars(el: HTMLElement): Record<string, string> {
  const cs = getComputedStyle(el);
  const out: Record<string, string> = {};
  for (const name of Array.from(cs)) {
    if (name.startsWith('--')) out[name] = cs.getPropertyValue(name).trim();
  }
  return out;
}

function resolveVars(text: string, vars: Record<string, string>): string {
  return text.replace(/var\((--[\w-]+)\)/g, (_m, n: string) => vars[n] ?? 'transparent');
}

function roundRectPath(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number): void {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.arcTo(x + w, y, x + w, y + h, r);
  ctx.arcTo(x + w, y + h, x, y + h, r);
  ctx.arcTo(x, y + h, x, y, r);
  ctx.arcTo(x, y, x + w, y, r);
  ctx.closePath();
}

function svgToImage(svgEl: SVGSVGElement, vars: Record<string, string>, fontFamily: string): Promise<HTMLImageElement> {
  const clone = svgEl.cloneNode(true) as SVGSVGElement;
  clone.removeAttribute('class');
  clone.setAttribute('xmlns', 'http://www.w3.org/2000/svg');
  clone.setAttribute('font-family', fontFamily);
  let markup = new XMLSerializer().serializeToString(clone);
  markup = resolveVars(markup, vars);
  const src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(markup);
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error('图表光栅化失败'));
    img.src = src;
  });
}

export async function exportStatsAsPng(root: HTMLElement, name: string): Promise<string> {
  const vars = cssVars(root);
  const bodyFont = getComputedStyle(document.body).fontFamily;
  const dark = document.documentElement.classList.contains('dark');
  const pageBg = dark ? '#0d0d0d' : '#F5F6FA';
  const ink = vars['--v2-ink'] || vars['--zj-text'] || '#222026';
  const ink2 = vars['--v2-ink-2'] || '#6e7080';
  const ink3 = vars['--v2-ink-3'] || '#a2a5b0';
  const surface = vars['--v2-surface'] || '#ffffff';
  const line = vars['--v2-line'] || 'rgba(34,32,38,.1)';
  const scale = 2;

  const W = root.scrollWidth;
  const H = root.scrollHeight;
  const base = root.getBoundingClientRect();

  const canvas = document.createElement('canvas');
  canvas.width = W * scale;
  canvas.height = H * scale;
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('画布不可用');
  ctx.scale(scale, scale);
  ctx.fillStyle = pageBg;
  ctx.fillRect(0, 0, W, H);

  const rel = (r: DOMRect): { x: number; y: number; w: number; h: number } => ({
    x: r.left - base.left, y: r.top - base.top, w: r.width, h: r.height,
  });

  // ── 页头 ──
  const h1 = root.querySelector('.v2-pagehead h1');
  if (h1) {
    const r = rel(h1.getBoundingClientRect());
    ctx.fillStyle = ink;
    ctx.font = '700 28px "Segoe UI Variable", "Segoe UI", "Microsoft YaHei UI", sans-serif';
    ctx.textBaseline = 'alphabetic';
    ctx.fillText(h1.textContent || '统计', r.x, r.y + r.h - 4);
  }
  const sub = root.querySelector('.v2-pagehead .sub');
  if (sub) {
    const r = rel(sub.getBoundingClientRect());
    ctx.fillStyle = ink3;
    ctx.font = '400 13px "Segoe UI", "Microsoft YaHei UI", sans-serif';
    ctx.fillText(sub.textContent || '', r.x, r.y + r.h - 2);
  }
  const pill = root.querySelector('.v2-date-pill');
  if (pill) {
    const r = rel(pill.getBoundingClientRect());
    ctx.strokeStyle = line;
    roundRectPath(ctx, r.x, r.y, r.w, r.h, r.h / 2);
    ctx.stroke();
    ctx.fillStyle = ink2;
    ctx.font = '500 12.5px "Segoe UI", "Microsoft YaHei UI", sans-serif';
    ctx.fillText(pill.textContent || '', r.x + 14, r.y + r.h / 2 + 4.5);
  }

  // ── 逐卡复刻 ──
  for (const card of Array.from(root.querySelectorAll('.v2-card'))) {
    const cr = rel(card.getBoundingClientRect());
    const varsCard = cssVars(card as HTMLElement); // --p-* 定义在 .v2-paper 上，必须按卡取
    if (cr.w < 10 || cr.y + cr.h < 0 || cr.y > H) continue;
    // 卡底（paper 卡用同色 surface，普通卡同）——统一画 surface 底 + 描边
    ctx.fillStyle = surface;
    roundRectPath(ctx, cr.x, cr.y, cr.w, cr.h, 20);
    ctx.fill();
    ctx.strokeStyle = line;
    ctx.stroke();


    // p-h2 / p-sub（纸卡）或 h3（普通卡）
    const h2 = card.querySelector('.p-h2');
    const sub = card.querySelector('.p-sub');
    const h3 = card.querySelector('.v2-card-head h3');
    if (h2) {
      const r = rel(h2.getBoundingClientRect());
      ctx.fillStyle = ink;
      ctx.font = '700 16.5px "Segoe UI", "Microsoft YaHei UI", sans-serif';
      ctx.fillText(h2.textContent || '', r.x, r.y + r.h - 3);
    }
    if (sub) {
      const r = rel(sub.getBoundingClientRect());
      ctx.fillStyle = ink2;
      ctx.font = '400 11.5px "Segoe UI", "Microsoft YaHei UI", sans-serif';
      ctx.fillText(sub.textContent || '', r.x, r.y + r.h - 2);
    }
    if (h3) {
      const r = rel(h3.getBoundingClientRect());
      ctx.fillStyle = ink;
      ctx.font = '600 15px "Segoe UI", "Microsoft YaHei UI", sans-serif';
      ctx.fillText(h3.textContent || '', r.x, r.y + r.h - 3);
    }

    // 主 SVG 图表（原位绘制）
    const svgEl = card.querySelector('svg');
    if (svgEl) {
      const r = rel(svgEl.getBoundingClientRect());
      const img = await svgToImage(svgEl, varsCard, bodyFont);
      ctx.drawImage(img, r.x, r.y, r.w, r.h);
    }

    // pnums 数字行（pv/ps/pc 三层）
    const pnums = card.querySelector('.v2-pnums');
    if (pnums) {
      const pr = rel(pnums.getBoundingClientRect());
      ctx.strokeStyle = ink;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(pr.x, pr.y);
      ctx.lineTo(pr.x + pr.w, pr.y);
      ctx.moveTo(pr.x, pr.y + pr.h);
      ctx.lineTo(pr.x + pr.w, pr.y + pr.h);
      ctx.stroke();
      ctx.lineWidth = 1;
      for (const pn of Array.from(pnums.querySelectorAll('.v2-pnum'))) {
        const r = rel(pn.getBoundingClientRect());
        if (r.x > pr.x) {
          ctx.strokeStyle = line;
          ctx.setLineDash([2, 3]);
          ctx.beginPath();
          ctx.moveTo(r.x, pr.y + 6);
          ctx.lineTo(r.x, pr.y + pr.h - 6);
          ctx.stroke();
          ctx.setLineDash([]);
        }
        const pv = pn.querySelector('.pv'), ps = pn.querySelector('.ps'), pc2 = pn.querySelector('.pc');
        ctx.textAlign = 'left';
        if (pv) { ctx.fillStyle = ink; ctx.font = '800 28px "Segoe UI", "Microsoft YaHei UI", sans-serif'; ctx.fillText(pv.textContent || '', r.x, r.y + 30); }
        if (ps) { ctx.fillStyle = ink; ctx.font = '700 12px "Segoe UI", "Microsoft YaHei UI", sans-serif'; ctx.fillText(ps.textContent || '', r.x, r.y + 48); }
        if (pc2) { ctx.fillStyle = ink2; ctx.font = '400 11px "Segoe UI", "Microsoft YaHei UI", sans-serif'; ctx.fillText(pc2.textContent || '', r.x, r.y + 64); }
      }
    }

    // p-src 页脚小字
    const srcEl = card.querySelector('.p-src') as HTMLElement | null;
    if (srcEl) {
      const r = rel(srcEl.getBoundingClientRect());
      ctx.fillStyle = ink3;
      ctx.font = '500 9.5px "Segoe UI", "Microsoft YaHei UI", sans-serif';
      const center = srcEl.style.textAlign === 'center';
      ctx.fillText(srcEl.textContent || '', center ? cr.x + cr.w / 2 - ctx.measureText(srcEl.textContent || '').width / 2 : r.x, r.y + r.h - 2);
    }
  }

  const url = canvas.toDataURL('image/png');
  const base64 = url.replace(/^data:image\/png;base64,/, '');
  return api.exportImage(name, base64);
}

// 保留：金额格式化兜底（避免未用告警的语义占位）
void fmtAmount;

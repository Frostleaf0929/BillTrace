// v2 长图导出：把页面元素序列化为 SVG foreignObject → 光栅化 PNG → 走 export_image 保存
// 约束：无外部图片依赖；字体/颜色随当前主题（深浅与强调色）内联
import { api } from '../api';

async function collectCss(): Promise<string> {
  let css = '';
  for (const tag of document.querySelectorAll('style')) css += (tag.textContent || '') + '\n';
  for (const link of document.querySelectorAll('link[rel="stylesheet"]')) {
    try {
      const href = (link as HTMLLinkElement).href;
      css += (await (await fetch(href)).text()) + '\n';
    } catch { /* 同源外样式忽略 */ }
  }
  // html.* 选择器在 foreignObject 内不生效，替换为可挂到包裹容器上的类
  css = css
    .replace(/html\.dark/g, '.dark-root')
    .replace(/html\.no-glass/g, '.no-glass-root')
    .replace(/html\.resizing/g, '.resizing-root')
    .replace(/html\[data-panel='dock'\]/g, ".dock-root[data-panel='dock']")
    .replace(/(^|\})\s*:root/g, '$1.export-root,:root');
  return css;
}

function rootVars(): string {
  const cs = getComputedStyle(document.documentElement);
  let out = '';
  for (const name of Array.from(cs)) {
    if (name.startsWith('--')) out += `${name}:${cs.getPropertyValue(name)};`;
  }
  return out;
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error('长图光栅化失败（foreignObject 渲染被拒绝）'));
    img.src = src;
  });
}

export async function exportElementAsPng(el: HTMLElement, name: string): Promise<string> {
  const w = el.offsetWidth;
  const h = el.scrollHeight;
  if (!w || !h) throw new Error('内容为空，无法导出');

  const css = await collectCss();
  const vars = rootVars();
  const dark = document.documentElement.classList.contains('dark');
  const noGlass = document.documentElement.classList.contains('no-glass');
  const pageBg = dark ? '#0d0d0d' : '#F5F6FA';

  const clone = el.cloneNode(true) as HTMLElement;
  clone.style.width = w + 'px';
  clone.style.margin = '0';
  clone.style.boxShadow = 'none';

  const wrapper = document.createElement('div');
  wrapper.setAttribute('xmlns', 'http://www.w3.org/1999/xhtml');
  wrapper.className = 'export-root' + (dark ? ' dark-root' : '') + (noGlass ? ' no-glass-root' : '');
  wrapper.style.cssText =
    `width:${w}px;background:${pageBg};` +
    `font-family:${getComputedStyle(document.body).fontFamily};` +
    `color:${getComputedStyle(document.body).color};`;
  wrapper.appendChild(clone);

  const styleTag = `<style>${css}\n.export-root{${vars}}</style>`;
  const svg =
    `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}">` +
    `<foreignObject width="100%" height="100%">` +
    `<div xmlns="http://www.w3.org/1999/xhtml">${styleTag}${wrapper.outerHTML}</div>` +
    `</foreignObject></svg>`;

  const img = await loadImage('data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svg));
  const scale = 2;
  const canvas = document.createElement('canvas');
  canvas.width = w * scale;
  canvas.height = h * scale;
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('画布不可用');
  ctx.scale(scale, scale);
  ctx.fillStyle = pageBg;
  ctx.fillRect(0, 0, w, h);
  ctx.drawImage(img, 0, 0, w, h);

  const url = canvas.toDataURL('image/png');
  const base64 = url.replace(/^data:image\/png;base64,/, '');
  return api.exportImage(name, base64);
}

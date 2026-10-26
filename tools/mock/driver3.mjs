// 驱动 v3：把弹层的几何与计算样式全部打出来
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const file = join(root, 'dist', 'index.html');
let html = readFileSync(file, 'utf8');

// 替换 dbg 函数实现（定位 driver2 中的 dbg 定义）
html = html.replace(
  /function dbg\(tag\) \{[\s\S]*?\n  \}/,
  `function dbg(tag) {
    var pop = document.querySelector('.v2-vselect-pop');
    var info = 'NOPOP';
    if (pop) {
      var r = pop.getBoundingClientRect();
      var cs = getComputedStyle(pop);
      var parent = pop.parentElement === document.body ? 'BODY' : (pop.parentElement && (pop.parentElement.className || 'TAG:' + pop.parentElement.tagName));
      info = 'rect=' + Math.round(r.left) + ',' + Math.round(r.top) + ' ' + Math.round(r.width) + 'x' + Math.round(r.height)
        + ' display=' + cs.display + ' vis=' + cs.visibility + ' opa=' + cs.opacity + ' z=' + cs.zIndex
        + ' pos=' + cs.position + ' parent=' + parent
        + ' style=' + (pop.getAttribute('style') || 'none').slice(0, 80);
    }
    var d = document.getElementById('dbg') || document.body.appendChild(Object.assign(document.createElement('div'), { id: 'dbg' }));
    d.style.cssText = 'position:fixed;left:8px;top:4px;z-index:99999;background:#fff;color:#000;font-size:11px;padding:3px 8px;border:1px solid #000;max-width:1100px';
    d.textContent = tag + ' :: ' + info + ' :: ERR:' + (window.__pageErrors || []).join('|').slice(0, 150);
  }`,
);

writeFileSync(file, html);
console.log('driver3 OK');

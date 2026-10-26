// 驱动 v5：弹层打开后强制整页重绘（验证无头合成器伪影假设）
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const file = join(root, 'dist', 'index.html');
let html = readFileSync(file, 'utf8');

html = html.replace(
  /function dbg\(tag\) \{[\s\S]*?\n  \}/,
  `function dbg(tag) {
    return H(60).then(function () {
      document.body.style.display = 'none';
      void document.body.offsetHeight;
      document.body.style.display = '';
      return H(60);
    }).then(function () {
      var pop = document.querySelector('.v2-vselect-pop');
      var info = 'NOPOP';
      if (pop) {
        var r = pop.getBoundingClientRect();
        info = 'rect=' + Math.round(r.left) + ',' + Math.round(r.top) + ' ' + Math.round(r.width) + 'x' + Math.round(r.height) + ' opts=' + pop.querySelectorAll('.vs-opt').length;
      }
      var d = document.getElementById('dbg') || document.body.appendChild(Object.assign(document.createElement('div'), { id: 'dbg' }));
      d.style.cssText = 'position:fixed;left:8px;top:4px;z-index:99999;background:#fff;color:#000;font-size:12px;padding:3px 8px;border:1px solid #000';
      d.textContent = tag + ' :: ' + info + ' :: ERR:' + (window.__pageErrors || []).join('|').slice(0, 120);
    });
  }`,
);

writeFileSync(file, html);
console.log('driver5 OK');

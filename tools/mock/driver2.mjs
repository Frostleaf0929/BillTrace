// 驱动 v2：完整指针事件序列 + 禁用弹层动画（绕过无头截图的动画冻结伪影）
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const file = join(root, 'dist', 'index.html');
let html = readFileSync(file, 'utf8');

// 换掉旧驱动块（从最后一个 <script> 到 </body>）
const start = html.lastIndexOf('<script>');
const end = html.lastIndexOf('</body>');
if (start < 0 || end < 0) throw new Error('driver anchors not found');

const driver = `<script>
(function () {
  var style = document.createElement('style');
  style.textContent = '.v2-vselect-pop{animation:none!important}.v2-panel{transition:none!important}';
  document.head.appendChild(style);
  var mode = new URLSearchParams(location.search).get('mode') || 'panel';
  var H = function (ms) { return new Promise(function (r) { setTimeout(r, ms); }); };
  function fullClick(el) {
    if (!el) return;
    var r = el.getBoundingClientRect();
    var opts = { bubbles: true, cancelable: true, clientX: r.left + r.width / 2, clientY: r.top + r.height / 2 };
    el.dispatchEvent(new PointerEvent('pointerdown', opts));
    el.dispatchEvent(new MouseEvent('mousedown', opts));
    el.dispatchEvent(new PointerEvent('pointerup', opts));
    el.dispatchEvent(new MouseEvent('mouseup', opts));
    el.dispatchEvent(new MouseEvent('click', opts));
  }
  function clickText(sel, text) {
    var el = Array.prototype.find.call(document.querySelectorAll(sel), function (b) { return b.textContent.trim() === text; });
    fullClick(el);
  }
  function first(sel) { return document.querySelector(sel); }
  function dbg(tag) {
    var d = document.getElementById('dbg') || document.body.appendChild(Object.assign(document.createElement('div'), { id: 'dbg' }));
    d.style.cssText = 'position:fixed;left:8px;top:4px;z-index:99999;background:#fff;color:#000;font-size:12px;padding:3px 8px;border:1px solid #000';
    var pop = document.querySelector('.v2-vselect-pop');
    d.textContent = tag + ' :: ERR:' + (window.__pageErrors || []).join('|').slice(0, 200) + ' :: POP:' + document.querySelectorAll('.v2-vselect-pop').length + ' :: OPTS:' + (pop ? pop.querySelectorAll('.vs-opt').length : 0);
  }
  run();
  function run() {
    H(1600).then(function () {
      if (mode === 'panel') {
        fullClick(first('.v2-tx-row'));
        return H(800).then(function () {
          fullClick(first('.v2-panel .v2-vselect-trigger'));
          return H(400).then(function () { dbg('PANEL-L1-POP'); });
        });
      }
      if (mode === 'transfer') {
        fullClick(first('.v2-tx-row'));
        return H(800).then(function () {
          clickText('.v2-panel .v2-seg button', '转账');
          return H(400);
        }).then(function () {
          fullClick(first('.v2-panel .v2-vselect-trigger'));
          return H(400).then(function () { dbg('TRANSFER-OUT-POP'); });
        });
      }
      if (mode === 'quickadd') {
        fullClick(first('.v2-add'));
        return H(800).then(function () {
          clickText('.v2-panel .v2-seg button', '转账');
          return H(400);
        }).then(function () {
          fullClick(first('.v2-panel .v2-vselect-trigger'));
          return H(400).then(function () { dbg('QUICKADD-TRANSFER-POP'); });
        });
      }
      if (mode === 'toolbar') {
        fullClick(first('.v2-card .v2-vselect-trigger'));
        return H(400).then(function () { dbg('TOOLBAR-POP'); });
      }
      dbg('IDLE');
    });
  }
})();
</script>
</body>`;

html = html.slice(0, start) + driver + html.slice(end + '</body>'.length);
writeFileSync(file, html);
console.log('driver2 OK');

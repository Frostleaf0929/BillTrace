// 驱动 v4：红底弹层 + 可选隐藏墙面（定位"在 DOM 却不绘制"）
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const file = join(root, 'dist', 'index.html');
let html = readFileSync(file, 'utf8');

html = html.replace(
  "style.textContent = '.v2-vselect-pop{animation:none!important}.v2-panel{transition:none!important}';",
  `style.textContent = '.v2-vselect-pop{animation:none!important;background:red!important;outline:4px solid lime!important;z-index:99998!important}.v2-panel{transition:none!important}'
      + (new URLSearchParams(location.search).get('flat') === '1' ? '.zj-wall{display:none!important}' : '');`,
);

writeFileSync(file, html);
console.log('driver4 OK');

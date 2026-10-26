// 给 dist-mock/index.html 注入 Tauri IPC 模拟 + 测试驱动（file:// 模拟台专用）
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const file = join(root, 'dist', 'index.html');
let html = readFileSync(file, 'utf8');

const mock = `<script>
(function () {
  var CATS = [
    { id: 1, kind: 'expense', parent_id: null, name: '食品饮料', sort: 0 },
    { id: 2, kind: 'expense', parent_id: null, name: '购物支出', sort: 1 },
    { id: 3, kind: 'expense', parent_id: null, name: '行车交通', sort: 2 },
    { id: 4, kind: 'expense', parent_id: null, name: '医疗保健', sort: 3 },
    { id: 5, kind: 'expense', parent_id: null, name: '兴趣爱好', sort: 4 },
    { id: 11, kind: 'expense', parent_id: 1, name: '零食饮料', sort: 0 },
    { id: 12, kind: 'expense', parent_id: 1, name: '早午晚餐', sort: 1 },
    { id: 13, kind: 'expense', parent_id: 2, name: '日用百货', sort: 0 },
    { id: 21, kind: 'income', parent_id: null, name: '工资', sort: 0 },
    { id: 22, kind: 'income', parent_id: null, name: '理财收益', sort: 1 },
    { id: 31, kind: 'account', parent_id: null, name: '工商银行储蓄卡(1882)', sort: 0 },
    { id: 32, kind: 'account', parent_id: null, name: '零钱通', sort: 1 },
    { id: 33, kind: 'account', parent_id: null, name: '微信钱包', sort: 2 },
    { id: 34, kind: 'account', parent_id: null, name: '支付宝余额', sort: 3 },
    { id: 35, kind: 'account', parent_id: null, name: '现金', sort: 4 },
    { id: 36, kind: 'account', parent_id: null, name: '信用卡', sort: 5 }
  ];
  var ACCS = CATS.filter(function (c) { return c.kind === 'account'; })
    .map(function (c, i) { return { name: c.name, group: '账户', base: 1000 + i, net: 0, balance: 1000 + i }; });
  var TXS = [
    { id: 101, tx_type: '支出', tx_time: '2026-07-31 12:00:00', l1: '食品饮料', l2: '零食饮料', l3: null, account_out: '零钱通', account_in: null, currency: null, amount: 18, member: null, merchant: '沙县小吃（南星读商业街店）', project_category: null, project: null, booker: null, remark: null },
    { id: 102, tx_type: '转账', tx_time: '2026-07-31 09:00:00', l1: null, l2: null, l3: null, account_out: '工商银行储蓄卡(1882)', account_in: '零钱通', currency: null, amount: 500, member: null, merchant: '账户转账', project_category: null, project: null, booker: null, remark: null },
    { id: 103, tx_type: '收入', tx_time: '2026-07-29 12:00:00', l1: '工资', l2: null, l3: null, account_out: null, account_in: '工商银行储蓄卡(1882)', currency: null, amount: 15200, member: null, merchant: '十月工资', project_category: null, project: null, booker: null, remark: '工资到账' }
  ];
  window.__mockLog = [];
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } },
    plugins: {},
    transformCallback: function (cb) { return cb; },
    invoke: function (cmd, args) {
      if (cmd === 'plugin:event|listen' || cmd === 'plugin:event|unlisten') return Promise.resolve(1);
      if (cmd === 'get_setting') return Promise.resolve(args && args.key === 'theme' ? 'light' : '');
      if (cmd === 'get_data_dir') return Promise.resolve('C:\\\\mock\\\\data');
      if (cmd === 'get_background') return Promise.resolve(null);
      if (cmd === 'list_categories') return Promise.resolve(args && args.kind ? CATS.filter(function (c) { return c.kind === args.kind; }) : CATS);
      if (cmd === 'account_balances') return Promise.resolve(ACCS);
      if (cmd === 'query_transactions') return Promise.resolve({ total: TXS.length, rows: TXS });
      if (cmd === 'stats_summary') return Promise.resolve({ today_expense: 18, today_income: 0, month_expense: 18, month_income: 15200, year_expense: 18, year_income: 15200, month_transfer: 500, tx_count: 3 });
      if (cmd === 'stats_chart' || cmd === 'stats_pie' || cmd === 'list_pending' || cmd === 'list_rules' || cmd === 'budget_list') return Promise.resolve([]);
      if (cmd === 'category_sums') return Promise.resolve({});
      if (cmd === 'save_transaction' || cmd === 'set_transactions_category') return Promise.resolve(1);
      window.__mockLog.push(cmd);
      return Promise.resolve(null);
    }
  };
})();
</script>`;

const driver = `<script>
(function () {
  var mode = new URLSearchParams(location.search).get('mode') || 'panel';
  var H = function (ms) { return new Promise(function (r) { setTimeout(r, ms); }); };
  function click(el) { if (el) el.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true })); }
  function clickText(sel, text) {
    var el = Array.prototype.find.call(document.querySelectorAll(sel), function (b) { return b.textContent.trim() === text; });
    click(el);
  }
  function first(sel) { return document.querySelector(sel); }
  run();
  function run() {
    H(1600).then(function () {
      if (mode === 'panel') {
        click(first('.v2-tx-row'));
        return H(800).then(function () {
          click(first('.v2-panel .v2-vselect-trigger'));
          return H(400);
        });
      }
      if (mode === 'transfer') {
        click(first('.v2-tx-row'));
        return H(800).then(function () {
          clickText('.v2-panel .v2-seg button', '转账');
          return H(400);
        }).then(function () {
          click(first('.v2-panel .v2-vselect-trigger'));
          return H(400);
        });
      }
      if (mode === 'quickadd') {
        click(first('.v2-add'));
        return H(800).then(function () {
          clickText('.v2-panel .v2-seg button', '转账');
          return H(400);
        }).then(function () {
          click(first('.v2-panel .v2-vselect-trigger'));
          return H(400);
        });
      }
      if (mode === 'toolbar') {
        click(first('.v2-card .v2-vselect-trigger'));
        return H(400);
      }
      if (mode === 'categories') {
        return H(200);
      }
    }).then(function () { document.title = 'MOCK-READY-' + mode; })
      .catch(function (e) { document.title = 'MOCK-ERR-' + e; });
  }
})();
</script>`;

html = html.replace('<script type="module"', mock + '\n  <script type="module"');
html = html.replace('</body>', driver + '\n  </body>');
writeFileSync(file, html);
console.log('patched', file, html.includes('__TAURI_INTERNALS__') && html.includes('MOCK-READY') ? 'OK' : 'MISSING-PARTS');

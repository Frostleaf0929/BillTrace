<script setup lang="ts">
// 明细 v2：筛选 + 日分组流水 + 批操作 + 维度排行
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { api, fmtAmount } from '../../api';
import type { Category, PiePoint, Tx } from '../../types';
import { icon, catColor, catIcon } from '../../v2/icons';
import TxPanel from '../../v2/TxPanel.vue';
import PendingResolveDialog from '../../components/PendingResolveDialog.vue';
import { toast } from '../../v2/toast';
import { buildRankCard } from '../../v2/paper';

const rows = ref<Tx[]>([]);
const total = ref(0);
const sumExpense = ref(0);
const sumIncome = ref(0);
const cats = ref<Category[]>([]);
const accounts = ref<string[]>([]);
const pendingCount = ref(0);
const panelVisible = ref(false);
const panelTx = ref<Tx | null>(null);
const checked = ref<Set<number>>(new Set());
const loading = ref(false);

const seg = ref<'all' | 'expense' | 'income'>('all');
const cat = ref('');
const acc = ref('');
const scope = ref<'month' | 'year' | 'all'>('all');
const q = ref('');
const pendingVisible = ref(false);

// 排行卡（本月口径，从统计页移入）
const rankType = ref<'支出' | '收入'>('支出');
const rankDim = ref<'l1' | 'l2' | 'merchant' | 'account'>('l1');
const rankScope = ref<'month' | 'year' | 'all'>('month');
const rankLabel = ref('本月');
const RANK_DIMS = [['l1', '一级分类'], ['l2', '二级分类'], ['merchant', '商家'], ['account', '账户']] as const;
const rankData = ref<PiePoint[]>([]);
const rankHtml = ref('');

const l1Options = computed(() => [...new Set(cats.value.filter((c) => c.kind === 'expense' && c.parent_id === null).map((c) => c.name))]);
const l2Options = computed(() => cats.value.filter((c) => c.parent_id !== null && c.name === moveL1.value).map((c) => c.name));
const moveL1 = ref('');
const moveL2 = ref('');

const dateRange = computed(() => {
  const now = new Date();
  const p = (n: number) => String(n).padStart(2, '0');
  if (scope.value === 'month') {
    return [`${now.getFullYear()}-${p(now.getMonth() + 1)}-01`, `${now.getFullYear()}-${p(now.getMonth() + 1)}-${p(new Date(now.getFullYear(), now.getMonth() + 1, 0).getDate())}`];
  }
  if (scope.value === 'year') {
    return [`${now.getFullYear()}-01-01`, `${now.getFullYear()}-12-31`];
  }
  return ['', ''];
});

const list = computed(() => rows.value);

const dayGroups = computed(() => {
  const byDay = new Map<string, Tx[]>();
  for (const t of list.value) {
    const d = t.tx_time.slice(0, 10);
    (byDay.get(d) || byDay.set(d, []).get(d)!).push(t);
  }
  return [...byDay.entries()].sort((a, b) => b[0].localeCompare(a[0])).map(([d, r]) => ({
    date: d,
    label: `${+d.slice(5, 7)}月${+d.slice(8, 10)}日 ${WD[new Date(d + 'T00:00:00').getDay()]}`,
    net: r.reduce((s, t) => s + (t.tx_type === '收入' ? t.amount : t.tx_type === '支出' || t.tx_type === '报销' ? -t.amount : 0), 0),
    rows: r,
  }));
});
const WD = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];

async function load(): Promise<void> {
  loading.value = true;
  try {
    const [fa, fb] = dateRange.value;
    const page = await api.queryTransactions({
      tx_type: seg.value === 'all' ? undefined : seg.value === 'expense' ? '支出' : '收入',
      l1: cat.value || undefined,
      keyword: q.value || undefined,
      date_from: fa || undefined,
      date_to: fb || undefined,
      page: 1,
      page_size: 3000,
    });
    rows.value = page.rows;
    total.value = page.total;
    sumExpense.value = page.rows.filter((t) => t.tx_type === '支出' || t.tx_type === '报销').reduce((s, t) => s + t.amount, 0);
    sumIncome.value = page.rows.filter((t) => t.tx_type === '收入').reduce((s, t) => s + t.amount, 0);
  } catch (e) {
    ElMessage.error(String(e));
  } finally {
    loading.value = false;
  }
  try {
    const now = new Date();
    const p = (n: number) => String(n).padStart(2, '0');
    let from: string, to: string, lbl: string;
    if (rankScope.value === 'month') {
      from = `${now.getFullYear()}-${p(now.getMonth() + 1)}-01`;
      to = `${now.getFullYear()}-${p(now.getMonth() + 1)}-${p(new Date(now.getFullYear(), now.getMonth() + 1, 0).getDate())}`;
      lbl = '本月';
    } else if (rankScope.value === 'year') {
      from = `${now.getFullYear()}-01-01`;
      to = `${now.getFullYear()}-12-31`;
      lbl = '今年';
    } else {
      from = '2000-01-01';
      to = '2099-12-31';
      lbl = '全部';
    }
    rankLabel.value = lbl;
    rankData.value = await api.statsPie(rankType.value, from, to, rankDim.value);
    rankHtml.value = buildRankCard(rankData.value);
  } catch { rankHtml.value = ''; }
  try { pendingCount.value = (await api.listPending()).length; } catch { pendingCount.value = 0; }
}

async function loadRefs(): Promise<void> {
  try {
    const [cs, bs] = await Promise.all([api.listCategories(), api.accountBalances()]);
    cats.value = cs;
    accounts.value = bs.map((b) => b.name);
  } catch { /* 忽略 */ }
}

onMounted(() => { load(); loadRefs(); 
  const onRefresh = () => { load(); };
  window.addEventListener('v2-refresh', onRefresh);
  onUnmounted(() => window.removeEventListener('v2-refresh', onRefresh));
});

function openPanel(tx: Tx): void {
  panelTx.value = tx;
  panelVisible.value = true;
}
function toggleCheck(ev: Event, id: number): void {
  ev.stopPropagation();
  const s = new Set(checked.value);
  if (s.has(id)) s.delete(id);
  else s.add(id);
  checked.value = s;
}
async function batchDelete(): Promise<void> {
  const n = checked.value.size;
  try {
    await ElMessageBox.confirm(`确定删除选中的 ${n} 条记录？此操作不可恢复。`, '批量删除', { type: 'warning' });
  } catch { return; }
  try {
    const deleted = await api.deleteTransactions([...checked.value]);
    toast(`已删除 ${deleted} 条`);
    checked.value = new Set();
    await load();
  } catch (e) {
    ElMessage.error(String(e));
  }
}
async function batchMove(): Promise<void> {
  if (!moveL1.value) { ElMessage.warning('请选择一级分类'); return; }
  try {
    const n = await api.setTransactionsCategory([...checked.value], moveL1.value, moveL2.value || null, null);
    toast(`已移动 ${n} 条到「${[moveL1.value, moveL2.value].filter(Boolean).join(' / ')}」`);
    checked.value = new Set();
    await load();
  } catch (e) {
    ElMessage.error(String(e));
  }
}
function openPending(): void {
  pendingVisible.value = true;
}
function onRankClick(e: MouseEvent): void {
  const el = (e.target as HTMLElement).closest('[data-rank]') as HTMLElement | null;
  const name = el?.dataset.rank;
  if (!name) return;
  if (rankDim.value === 'l1') { cat.value = name; load(); }
  else toast(`「${name}」的下钻筛选将在正式版接入（当前支持一级分类）`);
}
</script>

<template>
  <div>
    <div class="v2-pagehead">
      <div>
        <h1>明细</h1>
        <div class="sub">全部交易的浏览与编辑</div>
      </div>
      <div class="v2-head-right">
        <div class="v2-search">
          <span v-html="icon('search', 16)" />
          <input v-model="q" placeholder="搜索商家、备注…  Ctrl+K" @keydown.enter="load">
        </div>
        <button v-if="pendingCount" class="v2-pending-badge" @click="openPending">
          <span v-html="icon('alert', 15)" />{{ pendingCount }} 个待确认
        </button>
      </div>
    </div>

    <div class="v2-grid">
      <div class="v2-card span12">
        <div style="display:flex;align-items:center;gap:10px;flex-wrap:wrap;margin-bottom:6px">
          <div class="v2-seg">
            <button :class="{ on: seg === 'all' }" @click="seg = 'all'; load()">全部</button>
            <button :class="{ on: seg === 'expense' }" @click="seg = 'expense'; load()">支出</button>
            <button :class="{ on: seg === 'income' }" @click="seg = 'income'; load()">收入</button>
          </div>
          <select v-model="cat" class="v2-select" @change="load()">
            <option value="">全部分类</option>
            <option v-for="o in l1Options" :key="o" :value="o">{{ o }}</option>
          </select>
          <select v-model="acc" class="v2-select" @change="load()">
            <option value="">全部账户</option>
            <option v-for="o in accounts" :key="o" :value="o">{{ o }}</option>
          </select>
          <div class="v2-seg" style="margin-left:auto">
            <button :class="{ on: scope === 'month' }" @click="scope = 'month'; load()">本月</button>
            <button :class="{ on: scope === 'year' }" @click="scope = 'year'; load()">今年</button>
            <button :class="{ on: scope === 'all' }" @click="scope = 'all'; load()">全部</button>
          </div>
        </div>
        <div style="border-top:1px solid var(--v2-line);max-height:calc(100vh - 300px);overflow-y:auto;padding-right:4px">
          <div v-for="g in dayGroups" :key="g.date">
            <div class="v2-day-head"><span>{{ g.label }}</span><span class="net">净 {{ g.net < 0 ? '-' : '+' }}{{ fmtAmount(Math.abs(g.net)) }}</span></div>
            <div
              v-for="t in g.rows" :key="t.id" class="v2-tx-row" :class="{ checked: checked.has(t.id) }"
              @click="openPanel(t)"
            >
              <div class="v2-dot" :style="{ background: catColor(t.l1 || '') + '26', color: catColor(t.l1 || '') }" v-html="catIcon(t.l1 || '')" />
              <div style="min-width:0">
                <div class="v2-t-merchant">{{ t.merchant || '—' }}</div>
                <div class="v2-t-path">{{ [t.l1, t.l2, t.l3].filter(Boolean).join(' · ') || '未分类' }}</div>
              </div>
              <span class="v2-t-account">{{ t.account_out || t.account_in || '—' }}</span>
              <div class="v2-t-amount" :class="{ income: t.tx_type === '收入' }">
                {{ t.tx_type === '收入' ? '+' : t.tx_type === '支出' || t.tx_type === '报销' ? '-' : '' }}{{ fmtAmount(t.amount) }}
              </div>
              <span class="v2-row-check" @click="toggleCheck($event, t.id)">
                <svg v-if="checked.has(t.id)" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4"><path d="M5 12l5 5L20 7"/></svg>
              </span>
            </div>
          </div>
          <div v-if="!dayGroups.length" style="padding:28px 20px;text-align:center;color:var(--v2-ink-3);font-size:13px">
            <template v-if="loading">
              <div v-for="i in 6" :key="i" style="height:14px;border-radius:7px;background:var(--v2-surface-2);margin:14px auto;width:" :style="{ width: (88 - i * 9) + '%' }" />
            </template>
            <template v-else>没有符合条件的记录</template>
          </div>
        </div>
        <div style="display:flex;gap:18px;padding:12px 8px 0;color:var(--v2-ink-3);font-size:12.5px" class="num">
          <span>共 {{ total }} 条</span>
          <span style="color:var(--zj-expense)">支出 {{ fmtAmount(sumExpense) }}</span>
          <span style="color:var(--zj-income)">收入 {{ fmtAmount(sumIncome) }}</span>
        </div>
      </div>

      <!-- 维度排行（从统计页移入） -->
      <div class="v2-card span12">
        <div class="v2-card-head">
          <h3>维度排行 · {{ rankLabel }}</h3>
          <div class="spacer" />
          <div class="v2-seg">
            <button :class="{ on: rankScope === 'month' }" @click="rankScope = 'month'; load()">本月</button>
            <button :class="{ on: rankScope === 'year' }" @click="rankScope = 'year'; load()">今年</button>
            <button :class="{ on: rankScope === 'all' }" @click="rankScope = 'all'; load()">全部</button>
          </div>
          <div class="v2-seg">
            <button
              v-for="[v, lbl] in RANK_DIMS" :key="v" :class="{ on: rankDim === v }"
              @click="rankDim = v; load()"
            >{{ lbl }}</button>
          </div>
          <div class="v2-seg">
            <button :class="{ on: rankType === '支出' }" @click="rankType = '支出'; load()">支出</button>
            <button :class="{ on: rankType === '收入' }" @click="rankType = '收入'; load()">收入</button>
          </div>
        </div>
        <div style="color:var(--v2-ink-3);font-size:12px;margin:-8px 0 10px">点击行可按一级分类筛选上方流水 · 最多显示 15 条</div>
        <div v-html="rankHtml" @click="onRankClick" />
      </div>
    </div>

    <!-- 批操作条（仅有勾选时渲染，避免转场闪现） -->
    <Transition name="pop">
      <div v-if="checked.size > 0" class="v2-batch-bar show">
        <b class="num">已选 {{ checked.size }} 项</b>
        <select v-model="moveL1" class="v2-select" style="height:34px">
          <option value="">移动到…</option>
          <option v-for="o in l1Options" :key="o" :value="o">{{ o }}</option>
        </select>
        <select v-if="moveL1 && l2Options.length" v-model="moveL2" class="v2-select" style="height:34px">
          <option value="">二级（可选）</option>
          <option v-for="o in l2Options" :key="o" :value="o">{{ o }}</option>
        </select>
        <button class="bb-btn" :disabled="!moveL1" @click="batchMove">移动</button>
        <button class="bb-btn danger" @click="batchDelete">删除</button>
        <button class="bb-btn" @click="checked = new Set()">取消</button>
      </div>
    </Transition>

    <TxPanel :tx="panelTx" :visible="panelVisible" @close="panelVisible = false" @saved="() => { toast('已保存修改'); load(); }" @deleted="() => { toast('已删除'); load(); }" />
    <PendingResolveDialog v-model:visible="pendingVisible" @resolved="() => { toast('待确认已处理'); load(); }" />
  </div>
</template>

<script lang="ts">
export default { name: 'Detail2' };
</script>

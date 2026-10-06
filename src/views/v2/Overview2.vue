<script setup lang="ts">
// 概览 v2：数字卡 + 时间序列（导航器）+ 预算锚点卡 + 最近记录 + 账户
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { ElMessage, ElMessageBox } from 'element-plus';
import { api } from '../../api';
import type { AccountBalance, ChartPoint, SummaryStats, Tx } from '../../types';
import { icon, catColor, catIcon } from '../../v2/icons';
import { monthLabel } from '../../v2/parts';
import TsChart from '../../v2/TsChart.vue';
import TxPanel from '../../v2/TxPanel.vue';
import { toast } from '../../v2/toast';
import { openQuickAdd } from '../../v2/ui';

function money(n: number): string {
  return '¥' + n.toLocaleString('zh-CN', { maximumFractionDigits: 2 });
}

const router = useRouter();
const summary = ref<SummaryStats | null>(null);
const balances = ref<AccountBalance[]>([]);
const recent = ref<Tx[]>([]);
const trend = ref<ChartPoint[]>([]);
const trendMonths = ref<{ y: number; m: number }[]>([]);
const budgets = ref<{ category: string; amount: number; spent: number }[]>([]);
const panelVisible = ref(false);
const panelTx = ref<Tx | null>(null);

const monthKey = computed(() => {
  const n = new Date();
  return `${n.getFullYear()}-${String(n.getMonth() + 1).padStart(2, '0')}`;
});
const budget = computed(() => budgets.value.find((b) => b.category === '') ?? null);
const budgetPct = computed(() => (budget.value && budget.value.amount > 0 ? Math.min(100, Math.round((budget.value.spent / budget.value.amount) * 100)) : 0));
const budgetLeft = computed(() => (budget.value ? budget.value.amount - budget.value.spent : 0));
const totalAssets = computed(() => balances.value.reduce((s, a) => s + a.balance, 0));

const monthExpense = computed(() => summary.value?.month_expense ?? 0);
const monthIncome = computed(() => summary.value?.month_income ?? 0);

// 时间范围（日/周/月）：卡片数字与环比随之切换
type R = 'day' | 'week' | 'month' | 'year' | 'all';
const ovR = ref<R>('month');
const rangeSums = ref({ exp: 0, inc: 0, prevExp: 0, prevInc: 0 });
const R_LABEL: Record<R, string> = { day: '日', week: '周', month: '月', year: '年', all: '总' };

async function loadRangeSums(): Promise<void> {
  const now = new Date();
  const p = (n: number) => String(n).padStart(2, '0');
  const fmt = (d: Date) => `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
  let curFrom: Date, prevFrom: Date, prevTo: Date;
  if (ovR.value === 'day') {
    curFrom = new Date(now);
    prevFrom = new Date(now); prevFrom.setDate(now.getDate() - 1);
    prevTo = prevFrom;
  } else if (ovR.value === 'week') {
    curFrom = new Date(now); curFrom.setDate(now.getDate() - ((now.getDay() + 6) % 7));
    prevFrom = new Date(curFrom); prevFrom.setDate(curFrom.getDate() - 7);
    prevTo = new Date(curFrom); prevTo.setDate(curFrom.getDate() - 1);
  } else if (ovR.value === 'year') {
    curFrom = new Date(now.getFullYear(), 0, 1);
    prevFrom = new Date(now.getFullYear() - 1, 0, 1);
    prevTo = new Date(now.getFullYear() - 1, 11, 31);
  } else {
    curFrom = new Date(2000, 0, 1);
    prevFrom = new Date(2000, 0, 1);
    prevTo = new Date(2000, 0, 1);
  }
  const q = async (a: Date, b: Date) =>
    (await api.queryTransactions({ date_from: fmt(a), date_to: fmt(b), page: 1, page_size: 99999 })).rows;
  const sum = (rows: Tx[], inc: boolean) =>
    rows.filter((t) => t.tx_type === (inc ? '收入' : '支出')).reduce((s, t) => s + t.amount, 0);
  try {
    const cur = await q(curFrom, now);
    const prev = ovR.value === 'all' ? [] : await q(prevFrom, prevTo);
    rangeSums.value = {
      exp: sum(cur, false), inc: sum(cur, true),
      prevExp: sum(prev, false), prevInc: sum(prev, true),
    };
  } catch { /* 保持旧值 */ }
}
watch(ovR, loadRangeSums);

const cardExp = computed(() => (ovR.value === 'month' ? monthExpense.value : rangeSums.value.exp));
const cardInc = computed(() => (ovR.value === 'month' ? monthIncome.value : rangeSums.value.inc));
const cardNet = computed(() => cardInc.value - cardExp.value);
const expPct = computed(() => {
  const prev = ovR.value === 'month' ? summary.value?.prev_month_expense ?? 0 : rangeSums.value.prevExp;
  return prev > 0 ? Math.round(((cardExp.value - prev) / prev) * 100) : null;
});
const incPct = computed(() => {
  const prev = ovR.value === 'month' ? summary.value?.prev_month_income ?? 0 : rangeSums.value.prevInc;
  return prev > 0 ? Math.round(((cardInc.value - prev) / prev) * 100) : null;
});

const todayStr = (() => {
  const n = new Date();
  return `${n.getFullYear()}-${String(n.getMonth() + 1).padStart(2, '0')}-${String(n.getDate()).padStart(2, '0')}`;
})();
const todayCount = ref(0);

// 最近记录按日分组（最近 3 天 + 其余平铺由行内日期表达）
const recentGroups = computed(() => {
  const byDay = new Map<string, Tx[]>();
  for (const t of recent.value) {
    const d = t.tx_time.slice(0, 10);
    (byDay.get(d) || byDay.set(d, []).get(d)!).push(t);
  }
  return [...byDay.entries()].sort((a, b) => b[0].localeCompare(a[0])).slice(0, 3)
    .map(([d, rows]) => ({
      date: d,
      label: `${+d.slice(5, 7)}月${+d.slice(8, 10)}日 ${WD[new Date(d + 'T00:00:00').getDay()]}`,
      net: rows.reduce((s, t) => s + (t.tx_type === '收入' ? t.amount : t.tx_type === '支出' || t.tx_type === '报销' ? -t.amount : 0), 0),
      rows,
    }));
});
const WD = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];

async function fetchDay(from: string, to: string): Promise<ChartPoint[]> {
  try { return await api.statsChart('day', from, to); } catch { return []; }
}

async function load(): Promise<void> {
  try {
    const [s, bl, page, b] = await Promise.all([
      api.statsSummary(),
      api.budgetList('month', monthKey.value),
      api.queryTransactions({ page: 1, page_size: 12 }),
      api.accountBalances(),
    ]);
    summary.value = s;
    budgets.value = bl;
    recent.value = page.rows;
    balances.value = b;
    todayCount.value = (await api.queryTransactions({ date_from: todayStr, date_to: todayStr, page: 1, page_size: 500 })).rows.length;
  } catch (e) {
    ElMessage.error(String(e));
  }
}

async function loadTrend(): Promise<void> {
  const now = new Date();
  const from = new Date(now.getFullYear(), now.getMonth() - 13, 1);
  const f = (d: Date) => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-01`;
  try {
    const pts = await api.statsChart('month', f(from), f(now));
    // 后端只回有数据的月份（label = 'YYYY-MM'），客户端按最近 14 个月补零对齐
    const byKey = new Map(pts.map((p2) => [p2.label, p2]));
    const arr: ChartPoint[] = [];
    const meta: { y: number; m: number }[] = [];
    for (let i = 0; i < 14; i++) {
      const d = new Date(now.getFullYear(), now.getMonth() - 13 + i, 1);
      const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
      const p2 = byKey.get(key);
      arr.push(p2 ?? { label: monthLabel(d.getFullYear(), d.getMonth() + 1), income: 0, expense: 0 });
      meta.push({ y: d.getFullYear(), m: d.getMonth() + 1 });
    }
    trend.value = arr;
    trendMonths.value = meta;
  } catch { trend.value = []; }
}

const ovTrendRange = ref<[number, number]>([1, 13]);

function openPanel(tx: Tx): void {
  panelTx.value = tx;
  panelVisible.value = true;
}

async function setBase(acc: AccountBalance): Promise<void> {
  const v = await ElMessageBox.prompt(`设置「${acc.name}」的期初基数（当前基数 ${acc.base}）`, '设置基数', {
    inputValue: String(acc.base),
    inputPattern: /^-?\d+(\.\d+)?$/,
    inputErrorMessage: '请输入数字',
  }).catch(() => null);
  if (v === null) return;
  try {
    await api.setAccountBase(acc.name, Number(v.value));
    toast('基数已更新');
    await load();
  } catch (e) {
    ElMessage.error(String(e));
  }
}

async function addAccount(): Promise<void> {
  const res = await ElMessageBox.prompt('输入新账户名称（如：微信钱包、招行储蓄卡）', '新增账户', {
    inputPattern: /\S+/,
    inputErrorMessage: '名称不能为空',
  }).catch(() => null);
  if (res === null) return;
  try {
    await api.saveCategory({ kind: 'account', parent_id: null, name: res.value.trim() });
    toast('账户已添加');
    await load();
  } catch (e) {
    ElMessage.error(String(e));
  }
}

onMounted(() => { load(); loadTrend(); loadRangeSums();
  const onRefresh = () => { load(); loadTrend(); loadRangeSums(); };
  window.addEventListener('v2-refresh', onRefresh);
  onUnmounted(() => window.removeEventListener('v2-refresh', onRefresh));
});

function openQuick(): void { openQuickAdd(); }
defineExpose({ openQuick });
</script>

<template>
  <div>
    <div class="v2-pagehead">
      <div>
        <h1>概览</h1>
        <div class="sub">{{ new Date().getMonth() + 1 }}月{{ new Date().getDate() }}日 · 今天已记 {{ todayCount }} 笔</div>
      </div>
      <div class="v2-head-right">
        <div class="v2-seg">
          <button :class="{ on: ovR === 'day' }" @click="ovR = 'day'">日</button>
          <button :class="{ on: ovR === 'week' }" @click="ovR = 'week'">周</button>
          <button :class="{ on: ovR === 'month' }" @click="ovR = 'month'">月</button>
          <button :class="{ on: ovR === 'year' }" @click="ovR = 'year'">年</button>
          <button :class="{ on: ovR === 'all' }" @click="ovR = 'all'">总</button>
        </div>
        <button class="v2-btn primary" @click="openQuick">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M12 5v14M5 12h14"/></svg>
          记一笔
        </button>
      </div>
    </div>

    <div class="v2-grid">
      <!-- 4 张统计卡 -->
      <div class="v2-card stat span3">
        <div class="v2-chip red" v-html="icon('stats', 19)" />
        <div class="v2-stat-num">{{ money(cardExp) }}<span v-if="expPct !== null" class="v2-trend-pill" :class="expPct > 0 ? 'bad' : 'good'">{{ expPct > 0 ? '↑' : '↓' }} {{ Math.abs(expPct) }}%</span></div>
        <div class="v2-stat-label">{{ R_LABEL[ovR] }}支出</div>
      </div>
      <div class="v2-card stat span3">
        <div class="v2-chip green" v-html="icon('stats', 19)" />
        <div class="v2-stat-num">{{ money(cardInc) }}<span v-if="incPct !== null" class="v2-trend-pill" :class="incPct >= 0 ? 'good' : 'bad'">{{ incPct >= 0 ? '↑' : '↓' }} {{ Math.abs(incPct) }}%</span></div>
        <div class="v2-stat-label">{{ R_LABEL[ovR] }}收入</div>
      </div>
      <div class="v2-card stat span3">
        <div class="v2-chip" v-html="icon('wallet', 19)" />
        <div class="v2-stat-num">{{ money(cardNet) }}</div>
        <div class="v2-stat-label">{{ R_LABEL[ovR] }}结余</div>
      </div>
      <div class="v2-card stat span3">
        <div class="v2-chip amber" v-html="icon('overview', 19)" />
        <div class="v2-stat-num">{{ money(totalAssets) }}</div>
        <div class="v2-stat-label">账户总额 · {{ balances.length }} 个账户</div>
      </div>

      <!-- 收支趋势（导航器） -->
      <div class="v2-card span8">
        <div class="v2-card-head"><h3>收支趋势</h3></div>
        <TsChart
          :series="trend" :months="trendMonths" :range="ovTrendRange" :fetch-day="fetchDay"
          @range="(r) => (ovTrendRange = r)"
        />
      </div>

      <!-- 预算锚点卡 -->
      <div class="v2-card span4 v2-anchor-card">
        <div style="display:flex;align-items:center;gap:10px;margin-bottom:14px">
          <span style="opacity:.75" v-html="icon('budget', 18)" /><b style="font-size:14px">本月预算执行</b>
        </div>
        <template v-if="budget">
          <div class="ink2" style="font-size:12.5px">剩余</div>
          <div class="num" style="font-size:26px;font-weight:700;margin:2px 0 12px">{{ money(budgetLeft) }}</div>
          <div class="v2-track"><i :class="{ over: budgetLeft < 0 }" :style="{ width: budgetPct + '%' }" /></div>
          <div style="display:flex;justify-content:space-between;font-size:11.5px;margin-top:8px" class="ink2 num">
            <span>已用 {{ money(budget.spent) }} · {{ budgetPct }}%</span><span>预算 {{ money(budget.amount) }}</span>
          </div>
        </template>
        <div v-else class="ink2" style="font-size:12.5px;line-height:1.8;padding:6px 0 10px">
          本月还没有设置总预算——去「预算」页设定一个月度总额，这里会显示执行进度。
        </div>
        <button class="v2-btn v2-anchor-btn" @click="router.push('/budget')">查看预算 <span v-html="icon('arrow', 15)" /></button>
      </div>

      <!-- 最近记录 -->
      <div class="v2-card span8">
        <div class="v2-card-head">
          <h3>最近记录</h3>
          <div class="spacer" />
          <button class="v2-link-btn" @click="router.push('/detail')">查看全部 <span v-html="icon('arrow', 14)" /></button>
        </div>
        <div v-for="g in recentGroups" :key="g.date">
          <div class="v2-day-head"><span>{{ g.label }}</span><span class="net">净 {{ g.net < 0 ? '-' : '+' }}{{ money(Math.abs(g.net)) }}</span></div>
          <div v-for="t in g.rows" :key="t.id" class="v2-tx-row" @click="openPanel(t)">
            <div class="v2-dot" :style="{ background: catColor(t.l1 || '') + '26', color: catColor(t.l1 || '') }" v-html="catIcon(t.l1 || '')" />
            <div style="min-width:0">
              <div class="v2-t-merchant">{{ t.merchant || '—' }}</div>
              <div class="v2-t-path">{{ [t.l1, t.l2].filter(Boolean).join(' · ') || '未分类' }}</div>
            </div>
            <span class="v2-t-account">{{ t.account_out || t.account_in || '—' }}</span>
            <div class="v2-t-amount" :class="{ income: t.tx_type === '收入' }">
              {{ t.tx_type === '收入' ? '+' : t.tx_type === '支出' || t.tx_type === '报销' ? '-' : '' }}{{ money(t.amount) }}
            </div>
          </div>
        </div>
        <div v-if="!recent.length" style="padding:28px;text-align:center;color:var(--v2-ink-3);font-size:13px">
          还没有账单——点右上角「记一笔」，或到设置里导入微信 / 随手记账单
        </div>
      </div>

      <!-- 账户 -->
      <div class="v2-card span4">
        <div class="v2-card-head">
          <h3>账户</h3>
          <div class="spacer" />
          <button class="v2-icon-btn" title="添加账户" @click="addAccount" v-html="icon('overview', 16)" />
        </div>
        <div style="max-height:578px;overflow-y:auto">
        <div v-for="a in balances.slice(0, 10)" :key="a.name" class="v2-tx-row" style="cursor:pointer" @dblclick="setBase(a)" :title="'双击设置基数'">
          <div class="v2-dot" style="background:var(--v2-accent-tint);color:var(--v2-accent)" v-html="icon('wallet', 16)" />
          <div style="flex:1;min-width:0">
            <div style="font-weight:600;font-size:13.5px">{{ a.name }}</div>
            <div class="v2-track" style="margin-top:6px"><i :style="{ width: Math.max(2, Math.round((a.balance / Math.max(1, totalAssets)) * 100)) + '%' }" /></div>
          </div>
          <div class="v2-t-amount" style="font-size:13.5px" :style="{ color: a.balance < 0 ? 'var(--v2-expense)' : 'var(--v2-ink)' }">{{ money(a.balance) }}</div>
        </div>
        <div class="v2-tx-row" style="color:var(--v2-ink-3);justify-content:center;font-size:12.5px" @click="addAccount">＋ 添加账户（双击行可设基数）</div>
        </div>
      </div>
    </div>

    <TxPanel :tx="panelTx" :visible="panelVisible" @close="panelVisible = false" @saved="() => { toast('已保存修改'); load(); loadTrend(); }" @deleted="() => { toast('已删除'); load(); loadTrend(); }" />
  </div>
</template>

<script lang="ts">
export default { name: 'Overview2' };
</script>

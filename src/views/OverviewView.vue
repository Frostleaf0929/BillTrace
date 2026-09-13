<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import { UploadFilled, EditPen, Coin, Money, Wallet, Plus, Delete, CollectionTag } from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { api, fmtAmount } from '../api';
import type { AccountBalance, SummaryStats, Tx } from '../types';
import ImportBillDialog from '../components/ImportBillDialog.vue';
import TxEditDialog from '../components/TxEditDialog.vue';
import AccountBaseDialog from '../components/AccountBaseDialog.vue';
import PageSub from '../components/PageSub.vue';

const router = useRouter();
const summary = ref<SummaryStats | null>(null);
const recent = ref<Tx[]>([]);
const balances = ref<AccountBalance[]>([]);
const accountOrder = ref<string[]>([]);
const importVisible = ref(false);
const editVisible = ref(false);
const editing = ref<Tx | null>(null);
const baseVisible = ref(false);
const baseAccount = ref<{ name: string; base: number } | null>(null);

// 三张概览卡：支出 / 收入 / 账户总额，共享一个「年 / 月 / 日」范围（右下角切换，持久化）
type Range = 'year' | 'month' | 'day';
const ovRange = ref<Range>('day');
const RANGE_LABEL: Record<Range, string> = { day: '日', month: '月', year: '年' };

const RECENT_COLUMNS = [
  { key: 'tx_time', label: '日期', width: 160 },
  { key: 'tx_type', label: '类型', width: 84 },
  { key: 'cat', label: '分类', width: 190 },
  { key: 'merchant', label: '商家', width: 110 },
  { key: 'remark', label: '备注' },
  { key: 'amount', label: '金额', width: 140 },
];

function cellOf(row: Tx, key: string): string {
  switch (key) {
    case 'tx_time': return row.tx_time;
    case 'tx_type': return row.tx_type;
    case 'cat': return [row.l1, row.l2, row.l3].filter(Boolean).join(' / ') || '—';
    case 'merchant': return row.merchant || '—';
    case 'remark': return row.remark || '—';
    default: return '—';
  }
}

function openEdit(tx: Tx | null) {
  editing.value = tx;
  editVisible.value = true;
}

async function loadRangeSetting() {
  try {
    const v = await api.getSetting('ovCardRange');
    if (v === 'day' || v === 'month' || v === 'year') ovRange.value = v;
  } catch { /* 默认日 */ }
}

async function setOvRange(v: Range) {
  ovRange.value = v;
  try { await api.setSetting('ovCardRange', v); } catch { /* 忽略 */ }
}

const card1Value = () =>
  ovRange.value === 'day' ? summary.value?.today_expense ?? 0
    : ovRange.value === 'month' ? summary.value?.month_expense ?? 0
    : summary.value?.year_expense ?? 0;
const card2Value = () =>
  ovRange.value === 'day' ? summary.value?.today_income ?? 0
    : ovRange.value === 'month' ? summary.value?.month_income ?? 0
    : summary.value?.year_income ?? 0;
const card3Value = () => balances.value.reduce((s, a) => s + a.balance, 0);

// 账户排序持久化
function sortedBalances(): AccountBalance[] {
  const idx = (n: string) => {
    const i = accountOrder.value.indexOf(n);
    return i === -1 ? 9999 : i;
  };
  return [...balances.value].sort((a, b) => idx(a.name) - idx(b.name));
}

async function persistOrder() {
  try { await api.setSetting('accountOrder', JSON.stringify(accountOrder.value)); } catch { /* 忽略 */ }
}

// ---- 指针拖拽排序（按住拖动，实时换位，松手保存） ----
const dragName = ref('');
let dragStartX = 0;
let dragStartY = 0;
let dragArmed = false;

function onChipPointerDown(e: PointerEvent, name: string) {
  if (e.button !== 0) return;
  const el = e.currentTarget as HTMLElement;
  // 避免与删除按钮冲突
  if ((e.target as HTMLElement).closest('button')) return;
  dragName.value = name;
  dragStartX = e.clientX;
  dragStartY = e.clientY;
  dragArmed = false;
  el.setPointerCapture(e.pointerId);
}

function onChipPointerMove(e: PointerEvent) {
  if (!dragName.value) return;
  if (!dragArmed) {
    if (Math.hypot(e.clientX - dragStartX, e.clientY - dragStartY) < 6) return;
    dragArmed = true;
  }
  // 指针下的另一个账户块 → 实时交换位置（TransitionGroup 弹簧补位）
  const el = document.elementFromPoint(e.clientX, e.clientY)?.closest('[data-acc]') as HTMLElement | null;
  const targetName = el?.dataset.acc;
  if (!targetName || targetName === dragName.value) return;
  const list = sortedBalances().map((a) => a.name);
  const from = list.indexOf(dragName.value);
  const to = list.indexOf(targetName);
  if (from < 0 || to < 0) return;
  list.splice(to, 0, list.splice(from, 1)[0]);
  accountOrder.value = list;
}

function onChipPointerUp() {
  if (dragArmed && dragName.value) persistOrder();
  dragName.value = '';
  dragArmed = false;
}

async function addAccount() {
  const { value } = await ElMessageBox.prompt('输入新账户名称（如：微信钱包、招行储蓄卡）', '新增账户', {
    inputPattern: /\S+/,
    inputErrorMessage: '名称不能为空',
  });
  try {
    await api.saveCategory({ kind: 'account', parent_id: null, name: value.trim() });
    ElMessage.success('账户已添加（在「分类 → 账户分类」中可归入子级）');
    await load();
  } catch (e) {
    ElMessage.error(String(e));
  }
}

async function removeAccount(acc: AccountBalance) {
  await ElMessageBox.confirm(`删除账户「${acc.name}」？其账目记录保留，仅移除账户本身。`, '删除账户', { type: 'warning' });
  try {
    const cats = await api.listCategories('account');
    const node = cats.find((c) => c.name === acc.name);
    if (node) {
      await api.deleteCategory(node.id);
    } else {
      ElMessage.warning('未在分类中找到该账户，请到「分类 → 账户分类」中删除');
      return;
    }
    accountOrder.value = accountOrder.value.filter((n) => n !== acc.name);
    await persistOrder();
    ElMessage.success('已删除');
    await load();
  } catch (e) {
    ElMessage.error(String(e));
  }
}

async function load() {
  summary.value = await api.statsSummary();
  const page = await api.queryTransactions({ page: 1, page_size: 10 });
  recent.value = page.rows;
  balances.value = await api.accountBalances();
  if (accountOrder.value.length === 0) {
    try {
      const raw = await api.getSetting('accountOrder');
      if (raw) accountOrder.value = JSON.parse(raw);
    } catch { /* 默认排序 */ }
  }
}

onMounted(async () => {
  await loadRangeSetting();
  load();
});

function setBase(acc: AccountBalance) {
  baseAccount.value = { name: acc.name, base: acc.base };
  baseVisible.value = true;
}

function colorOf(tx: Tx): string {
  if (tx.tx_type === '支出' || tx.tx_type === '报销') return 'zj-amount-expense';
  if (tx.tx_type === '收入') return 'zj-amount-income';
  return 'zj-amount-transfer';
}

function signOf(tx: Tx): string {
  if (tx.tx_type === '支出' || tx.tx_type === '报销') return '-';
  if (tx.tx_type === '收入') return '+';
  return '';
}
</script>

<template>
  <div>
    <h1 class="zj-page-title">概览</h1>
    <PageSub page="overview" fallback="今天的你花了多少钱？" />

    <div class="zj-row" style="margin-bottom: 14px">
      <div class="zj-card ov-card" style="flex: 1">
        <div class="ov-card-head">
          <span class="ov-card-title"><el-icon><Coin /></el-icon> 支出</span>
        </div>
        <div class="zj-amount-expense zj-num ov-card-num">¥ {{ fmtAmount(card1Value()) }}</div>
        <div class="ov-card-foot">
          <span class="ov-card-sub">{{ RANGE_LABEL[ovRange] }}支出</span>
          <el-segmented
            :model-value="ovRange"
            :options="[{ label: '日', value: 'day' }, { label: '月', value: 'month' }, { label: '年', value: 'year' }]"
            size="small"
            @change="(v: any) => setOvRange(v)"
          />
        </div>
      </div>
      <div class="zj-card ov-card" style="flex: 1">
        <div class="ov-card-head">
          <span class="ov-card-title"><el-icon><Money /></el-icon> 收入</span>
        </div>
        <div class="zj-amount-income zj-num ov-card-num">¥ {{ fmtAmount(card2Value()) }}</div>
        <div class="ov-card-foot">
          <span class="ov-card-sub">{{ RANGE_LABEL[ovRange] }}收入</span>
          <el-segmented
            :model-value="ovRange"
            :options="[{ label: '日', value: 'day' }, { label: '月', value: 'month' }, { label: '年', value: 'year' }]"
            size="small"
            @change="(v: any) => setOvRange(v)"
          />
        </div>
      </div>
      <div class="zj-card ov-card" style="flex: 1">
        <div class="ov-card-head">
          <span class="ov-card-title"><el-icon><Wallet /></el-icon> 账户总额</span>
        </div>
        <div class="zj-num ov-card-num" :style="{ color: card3Value() < 0 ? 'var(--zj-expense)' : 'var(--zj-text)' }">
          ¥ {{ fmtAmount(card3Value()) }}
        </div>
        <div class="ov-card-foot">
          <span class="ov-card-sub">全部账户余额合计</span>
          <el-segmented
            :model-value="ovRange"
            :options="[{ label: '日', value: 'day' }, { label: '月', value: 'month' }, { label: '年', value: 'year' }]"
            size="small"
            @change="(v: any) => setOvRange(v)"
          />
        </div>
      </div>
      <div class="zj-card ov-actions" style="flex: 1">
        <el-button type="primary" @click="importVisible = true">
          <el-icon style="margin-right: 6px"><UploadFilled /></el-icon> 导入账单
        </el-button>
        <el-button @click="openEdit(null)">
          <el-icon style="margin-right: 6px"><EditPen /></el-icon> 手动记账
        </el-button>
        <el-button text type="primary" @click="router.push('/stats')">查看统计 →</el-button>
      </div>
    </div>

    <!-- 账户余额框架：拖拽排序 / 增删 / 双击设基数 -->
    <div class="zj-card" style="margin-bottom: 14px">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px">
        <span style="font-weight: 700; font-size: 15px">账户余额</span>
        <div style="display: flex; align-items: center; gap: 8px">
          <span style="color: var(--zj-text-sub); font-size: 12px">按住拖动排序 · 双击设基数</span>
          <el-button size="small" type="primary" plain @click="addAccount">
            <el-icon style="margin-right: 4px"><Plus /></el-icon> 账户
          </el-button>
          <el-button size="small" @click="router.push({ path: '/categories', query: { kind: 'account' } })">
            <el-icon style="margin-right: 4px"><CollectionTag /></el-icon> 账户分类
          </el-button>
        </div>
      </div>
      <TransitionGroup v-if="balances.length" tag="div" name="chip" class="acc-wrap">
        <div
          v-for="acc in sortedBalances()"
          :key="acc.name"
          :data-acc="acc.name"
          class="acc-chip"
          :class="{ dragging: dragName === acc.name }"
          :title="acc.group"
          @pointerdown="(e: PointerEvent) => onChipPointerDown(e, acc.name)"
          @pointermove="onChipPointerMove"
          @pointerup="onChipPointerUp"
          @pointercancel="onChipPointerUp"
          @dblclick="setBase(acc)"
        >
          <span class="zj-num" style="font-weight: 600" :style="{ color: acc.balance < 0 ? 'var(--zj-expense)' : 'var(--zj-text)' }">
            ¥ {{ fmtAmount(acc.balance) }}
          </span>
          <span style="color: var(--zj-text-sub); font-size: 12.5px">{{ acc.name }}</span>
          <el-button class="acc-del" text size="small" type="danger" title="删除账户" @click.stop="removeAccount(acc)">
            <el-icon><Delete /></el-icon>
          </el-button>
        </div>
      </TransitionGroup>
      <div v-else style="color: var(--zj-text-sub); font-size: 13px; padding: 8px 0">
        暂无账户 —— 点击右上角「+ 账户」新建，或导入账单后自动出现
      </div>
    </div>

    <div class="zj-card">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px">
        <span style="font-weight: 700; font-size: 15px">最近记录</span>
        <el-button text type="primary" @click="router.push('/detail')">全部 →</el-button>
      </div>

      <el-table :data="recent" size="default" style="width: 100%; cursor: pointer" highlight-current-row @row-click="(row: Tx) => openEdit(row)">
        <el-table-column
          v-for="col in RECENT_COLUMNS"
          :key="col.key"
          :label="col.label"
          :width="col.width"
          :min-width="col.width ? undefined : 140"
          :align="col.key === 'amount' ? 'right' : undefined"
          :show-overflow-tooltip="col.key === 'remark' || col.key === 'merchant' || col.key === 'cat'"
        >
          <template #default="{ row }">
            <template v-if="col.key === 'tx_type'">
              <el-tag size="small" :type="row.tx_type === '支出' ? 'danger' : row.tx_type === '收入' ? 'success' : 'warning'" effect="light">
                {{ row.tx_type }}
              </el-tag>
            </template>
            <template v-else-if="col.key === 'amount'">
              <span :class="colorOf(row)"><span class="zj-num">{{ signOf(row) }}{{ fmtAmount(row.amount) }}</span><span class="cur-unit"> {{ row.currency || '' }}</span></span>
            </template>
            <template v-else>{{ cellOf(row, col.key) }}</template>
          </template>
        </el-table-column>
      </el-table>
      <div v-if="recent.length === 0" style="text-align: center; color: var(--zj-text-sub); padding: 30px 0">
        还没有账单，点击上方「导入账单」开始吧
      </div>
    </div>

    <ImportBillDialog v-model:visible="importVisible" @imported="load" />
    <TxEditDialog v-model:visible="editVisible" :tx="editing" @saved="load" />
    <AccountBaseDialog v-model:visible="baseVisible" :account="baseAccount" @saved="load" />
  </div>
</template>

<style scoped>
.ov-card {
  display: flex;
  flex-direction: column;
  transition: transform 0.3s cubic-bezier(0.34, 1.5, 0.5, 1), box-shadow 0.25s ease, border-color 0.2s ease;
}

/* 鼠标聚焦：弹性上浮 + 阴影加深 + 主色描边 */
.ov-card:hover {
  transform: translateY(-4px) scale(1.012);
  box-shadow: var(--zj-shadow-hover), var(--zj-inset), 0 0 0 1px var(--zj-primary);
}
.ov-card-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.ov-card-title {
  color: var(--zj-text-sub);
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 6px;
}
.ov-card-num {
  font-size: 26px;
  margin-top: 10px;
}
.ov-card-foot {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: auto;
  padding-top: 10px;
}
.ov-card-sub {
  color: var(--zj-text-sub);
  font-size: 12px;
}
/* 年/月/日切换字号缩小 */
.ov-card-foot :deep(.el-segmented) {
  --el-segmented-font-size: 10px;
  --el-segmented-item-selected-bg-color: var(--zj-primary);
}
.ov-card-foot :deep(.el-segmented__group) {
  padding: 1px;
}
.ov-card-foot :deep(.el-segmented__item) {
  padding: 0 6px;
  line-height: 17px;
}
.ov-actions {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 10px;
}
.ov-actions .el-button,
.ov-actions .el-button + .el-button {
  margin-left: 0;
  width: 100%;
}

.acc-wrap {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  padding: 6px 0 2px;
}

.acc-chip {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px 8px 14px;
  border-radius: 12px;
  background: var(--zj-card-hover);
  border: 1px solid var(--zj-border);
  cursor: grab;
  touch-action: none;
  user-select: none;
  transition: box-shadow 0.2s ease, border-color 0.2s ease;
}

.acc-chip:hover {
  border-color: var(--zj-primary);
}

.acc-chip.dragging {
  cursor: grabbing;
  opacity: 0.55;
  box-shadow: 0 10px 26px rgba(26, 31, 51, 0.2);
}

.acc-del {
  opacity: 0;
  transition: opacity 0.15s ease;
}

.acc-chip:hover .acc-del {
  opacity: 1;
}

/* 拖拽排序的弹簧位移动画（TransitionGroup FLIP） */
.chip-move {
  transition: transform 0.4s cubic-bezier(0.34, 1.45, 0.5, 1);
}
.chip-enter-active {
  transition: all 0.35s cubic-bezier(0.34, 1.4, 0.44, 1);
}
.chip-leave-active {
  display: none;
}

.cur-unit {
  margin-left: 5px;
  font-size: 11px;
  color: var(--zj-text-sub);
}
</style>

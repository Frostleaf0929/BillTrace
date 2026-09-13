<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus, Search, Bell } from '@element-plus/icons-vue';
import { api, fmtAmount } from '../api';
import type { Category, Tx, TxFilter } from '../types';
import TxEditDialog from '../components/TxEditDialog.vue';
import PendingResolveDialog from '../components/PendingResolveDialog.vue';
import PageSub from '../components/PageSub.vue';

const emit = defineEmits<{ (e: 'refresh-pending'): void }>();

const filter = reactive<TxFilter>({
  tx_type: undefined,
  l1: undefined,
  l2: undefined,
  merchant: undefined,
  keyword: undefined,
  date_from: undefined,
  date_to: undefined,
  page: 1,
  page_size: 50,
});

const rows = ref<Tx[]>([]);
const total = ref(0);
const loading = ref(false);
const categories = ref<Category[]>([]);
const pendingCount = ref(0);
const pendingSeen = ref(0);
const pendingUrgent = ref(false);

const editVisible = ref(false);
const pendingVisible = ref(false);
const pendingRef = ref<InstanceType<typeof PendingResolveDialog>>();
const editing = ref<Tx | null>(null);

const l1Options = ref<Category[]>([]);
const l2Options = ref<Category[]>([]);

// 多选批量操作
const selected = ref<Tx[]>([]);
const selectionTableRef = ref();
const moveDialogVisible = ref(false);
const moveL1 = ref('');
const moveL2 = ref<string | null>(null);
const moveL3 = ref<string | null>(null);

const moveL2Options = computed(() => {
  const p = categories.value.find((c) => c.name === moveL1.value && c.kind !== 'account' && c.parent_id === null);
  return p ? categories.value.filter((c) => c.parent_id === p.id) : [];
});
const moveL3Options = computed(() => {
  const p = categories.value.find((c) => c.name === moveL2.value && c.parent_id !== null);
  return p ? categories.value.filter((c) => c.parent_id === p.id) : [];
});

async function load() {
  loading.value = true;
  try {
    const page = await api.queryTransactions(filter);
    rows.value = page.rows;
    total.value = page.total;
  } catch (e) {
    ElMessage.error(String(e));
  } finally {
    loading.value = false;
  }
  pendingCount.value = (await api.listPending()).length;
  pendingUrgent.value = pendingCount.value > pendingSeen.value;
  selected.value = [];
}

onMounted(async () => {
  const all = await api.listCategories();
  categories.value = all;
  l1Options.value = all.filter((c) => (c.kind === 'expense' || c.kind === 'income') && c.parent_id === null);
  try {
    pendingSeen.value = Number(await api.getSetting('pendingSeenCount')) || 0;
  } catch { /* 默认 0 */ }
  load();
});

function onL1Change() {
  filter.l2 = undefined;
  const parent = categories.value.find((c) => c.name === filter.l1);
  l2Options.value = parent ? categories.value.filter((c) => c.parent_id === parent.id) : [];
  filter.page = 1;
  load();
}

function resetFilter() {
  Object.assign(filter, {
    tx_type: undefined, l1: undefined, l2: undefined, merchant: undefined,
    keyword: undefined, date_from: undefined, date_to: undefined, page: 1,
  });
  l2Options.value = [];
  load();
}

function openEdit(tx: Tx | null) {
  editing.value = tx;
  editVisible.value = true;
}

function openPending() {
  pendingVisible.value = true;
  pendingRef.value?.load();
}

function onPendingOpened() {
  pendingSeen.value = pendingCount.value;
  pendingUrgent.value = false;
  api.setSetting('pendingSeenCount', String(pendingSeen.value)).catch(() => {});
}

function onResolved() {
  load();
  emit('refresh-pending');
}

async function remove(tx: Tx) {
  await ElMessageBox.confirm(`删除这条 ${tx.tx_time} 的记录（${fmtAmount(tx.amount)}）？`, '删除', { type: 'warning' });
  await api.deleteTransaction(tx.id);
  ElMessage.success('已删除');
  load();
}

function onSelectionChange(rowsSel: Tx[]) {
  selected.value = rowsSel;
}

async function batchDelete() {
  const n = selected.value.length;
  await ElMessageBox.confirm(`确定删除选中的 ${n} 条记录？此操作不可恢复。`, '批量删除', { type: 'warning' });
  const ids = selected.value.map((r) => r.id);
  const deleted = await api.deleteTransactions(ids);
  ElMessage.success(`已删除 ${deleted} 条`);
  load();
}

function openMove() {
  moveL1.value = '';
  moveL2.value = null;
  moveL3.value = null;
  moveDialogVisible.value = true;
}

async function doMove() {
  if (!moveL1.value) {
    ElMessage.warning('请选择一级分类');
    return;
  }
  const ids = selected.value.map((r) => r.id);
  const n = await api.setTransactionsCategory(ids, moveL1.value, moveL2.value, moveL3.value);
  ElMessage.success(`已移动 ${n} 条到「${[moveL1.value, moveL2.value, moveL3.value].filter(Boolean).join(' / ')}」`);
  moveDialogVisible.value = false;
  load();
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
    <h1 class="zj-page-title">详细</h1>
    <PageSub page="detail" fallback="全部账目明细 · 双击任意一行即可编辑，支持多选批量操作" />

    <div
      class="zj-card"
      :style="{
        marginBottom: '14px',
        display: 'flex',
        alignItems: 'center',
        gap: '12px',
        border: pendingUrgent ? '1px solid var(--zj-expense)' : '1px solid var(--zj-border)',
      }"
    >
      <el-icon :color="pendingUrgent ? 'var(--zj-expense)' : 'var(--zj-text-sub)'"><Bell /></el-icon>
      <span>有商家记录待归类</span>
      <el-button :type="pendingUrgent ? 'danger' : 'default'" size="small" @click="openPending">去归类</el-button>
      <transition name="pop">
        <span v-if="pendingCount > 0" class="pending-count" :class="{ urgent: pendingUrgent }">{{ pendingCount }}</span>
      </transition>
    </div>

    <div class="zj-card" style="margin-bottom: 14px">
      <div class="zj-toolbar">
        <el-select v-model="filter.tx_type" placeholder="类型" clearable style="width: 110px" @change="filter.page = 1; load()">
          <el-option v-for="t in ['支出', '收入', '转账', '报销', '代付', '余额变更', '债权变更']" :key="t" :label="t" :value="t" />
        </el-select>
        <el-select v-model="filter.l1" placeholder="一级分类" clearable filterable style="width: 140px" @change="onL1Change">
          <el-option v-for="c in l1Options" :key="c.id" :label="c.name" :value="c.name" />
        </el-select>
        <el-select v-model="filter.l2" placeholder="二级分类" clearable filterable style="width: 140px" :disabled="!filter.l1" @change="filter.page = 1; load()">
          <el-option v-for="c in l2Options" :key="c.id" :label="c.name" :value="c.name" />
        </el-select>
        <el-input v-model="filter.merchant" placeholder="商家" clearable style="width: 140px" @change="filter.page = 1; load()" />
        <el-date-picker v-model="filter.date_from" type="date" placeholder="开始日期" value-format="YYYY-MM-DD" style="width: 140px" @change="filter.page = 1; load()" />
        <el-date-picker v-model="filter.date_to" type="date" placeholder="结束日期" value-format="YYYY-MM-DD" style="width: 140px" @change="filter.page = 1; load()" />
        <el-input v-model="filter.keyword" placeholder="关键词搜索" clearable style="width: 180px" @change="filter.page = 1; load()">
          <template #prefix><el-icon><Search /></el-icon></template>
        </el-input>
        <el-button @click="resetFilter">重置</el-button>
        <div style="flex: 1" />
        <el-button type="primary" @click="openEdit(null)">
          <el-icon style="margin-right: 6px"><Plus /></el-icon> 记一笔
        </el-button>
      </div>
    </div>

    <!-- 批量操作条 -->
    <transition name="pop">
      <div v-if="selected.length > 0" class="zj-card" style="margin-bottom: 14px; display: flex; align-items: center; gap: 12px">
        <span>已选 <b class="zj-num">{{ selected.length }}</b> 条</span>
        <el-button type="primary" size="small" @click="openMove">移动到分类…</el-button>
        <el-button type="danger" size="small" @click="batchDelete">批量删除</el-button>
        <el-button text size="small" @click="selectionTableRef?.clearSelection()">取消选择</el-button>
      </div>
    </transition>

    <div class="zj-card">
      <el-table
        ref="selectionTableRef"
        :data="rows"
        v-loading="loading"
        size="small"
        height="calc(100vh - 330px)"
        style="width: 100%"
        @row-dbl-click="(row: Tx) => openEdit(row)"
        @selection-change="onSelectionChange"
      >
        <el-table-column type="selection" width="42" />
        <el-table-column label="日期" width="160">
          <template #default="{ row }">{{ row.tx_time }}</template>
        </el-table-column>
        <el-table-column label="类型" width="80">
          <template #default="{ row }">
            <el-tag size="small" :type="row.tx_type === '支出' ? 'danger' : row.tx_type === '收入' ? 'success' : 'warning'" effect="light">
              {{ row.tx_type }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="分类" min-width="150" show-overflow-tooltip>
          <template #default="{ row }">{{ [row.l1, row.l2, row.l3].filter(Boolean).join(' / ') || '—' }}</template>
        </el-table-column>
        <el-table-column label="账户" min-width="110" show-overflow-tooltip>
          <template #default="{ row }">{{ row.account_out || row.account_in || '—' }}</template>
        </el-table-column>
        <el-table-column label="商家" min-width="110" show-overflow-tooltip>
          <template #default="{ row }">{{ row.merchant || '—' }}</template>
        </el-table-column>
        <el-table-column label="备注" min-width="170" show-overflow-tooltip>
          <template #default="{ row }">{{ row.remark || '—' }}</template>
        </el-table-column>
        <el-table-column label="金额" width="140" align="right" fixed="right">
          <template #default="{ row }">
            <span :class="colorOf(row)"><span class="zj-num">{{ signOf(row) }}{{ fmtAmount(row.amount) }}</span><span class="cur-unit"> {{ row.currency }}</span></span>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="130" align="center" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="openEdit(row)">编辑</el-button>
            <el-button size="small" type="danger" plain @click="remove(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div style="display: flex; justify-content: flex-end; margin-top: 12px">
        <el-pagination
          v-model:current-page="filter.page"
          :page-size="filter.page_size"
          :total="total"
          layout="total, prev, pager, next, sizes"
          :page-sizes="[50, 100, 200, 500]"
          @current-change="load"
          @size-change="(s: number) => { filter.page_size = s; filter.page = 1; load(); }"
        />
      </div>
    </div>

    <!-- 批量移动分类 -->
    <el-dialog v-model="moveDialogVisible" title="移动到分类" width="420px" align-center destroy-on-close>
      <div style="display: flex; flex-direction: column; gap: 12px">
        <el-select v-model="moveL1" placeholder="一级分类（必选）" filterable @change="moveL2 = null; moveL3 = null">
          <el-option v-for="c in l1Options" :key="c.id" :label="c.name" :value="c.name" />
        </el-select>
        <el-select v-model="moveL2" placeholder="二级分类（可选）" filterable clearable :disabled="!moveL1" @change="moveL3 = null">
          <el-option v-for="c in moveL2Options" :key="c.id" :label="c.name" :value="c.name" />
        </el-select>
        <el-select v-model="moveL3" placeholder="小级分类（可选）" filterable clearable :disabled="!moveL2">
          <el-option v-for="c in moveL3Options" :key="c.id" :label="c.name" :value="c.name" />
        </el-select>
      </div>
      <template #footer>
        <el-button @click="moveDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="doMove">移动 {{ selected.length }} 条</el-button>
      </template>
    </el-dialog>

    <TxEditDialog v-model:visible="editVisible" :tx="editing" @saved="load" />
    <PendingResolveDialog ref="pendingRef" v-model:visible="pendingVisible" @resolved="onResolved" @opened="onPendingOpened" />
  </div>
</template>

<style scoped>
.cur-unit {
  margin-left: 5px;
  font-size: 11px;
  color: var(--zj-text-sub);
}

.pending-count {
  min-width: 22px;
  padding: 1px 8px;
  border-radius: 999px;
  font-size: 12px;
  text-align: center;
  background: var(--zj-sidebar-hover);
  color: var(--zj-text-sub);
}

.pending-count.urgent {
  background: var(--zj-expense);
  color: #fff;
}

:deep(.el-table .el-button) {
  min-width: 48px;
  padding-left: 10px;
  padding-right: 10px;
}
</style>

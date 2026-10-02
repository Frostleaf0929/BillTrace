<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { api, fmtAmount } from '../api';
import PageSub from '../components/PageSub.vue';
import type { BudgetStatus, Category } from '../types';

// ── 周期：月 / 年，前后翻页 ──
const period = ref<'month' | 'year'>('month');
const now = new Date();
const monthKey = ref(`${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`);
const yearKey = ref(String(now.getFullYear()));
const key = computed(() => (period.value === 'month' ? monthKey.value : yearKey.value));
const keyLabel = computed(() => (period.value === 'month' ? `${monthKey.value} 月` : `${yearKey.value} 年`));

const items = ref<BudgetStatus[]>([]);
const cats = ref<Category[]>([]); // 支出一级分类候选

async function load() {
  try {
    items.value = await api.budgetList(period.value, key.value);
  } catch (e) {
    ElMessage.error(String(e));
  }
}

function shift(delta: number) {
  if (period.value === 'month') {
    const [y, m] = monthKey.value.split('-').map(Number);
    const d = new Date(y, m - 1 + delta, 1);
    monthKey.value = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
  } else {
    yearKey.value = String(Number(yearKey.value) + delta);
  }
  load();
}

function onPeriodChange() {
  load();
}

onMounted(async () => {
  try {
    const all = await api.listCategories('expense');
    cats.value = all.filter((c) => c.parent_id === null);
  } catch { /* 分类加载失败不阻塞预算展示 */ }
  load();
});

// ── 展示与编辑 ──
const total = computed(() => items.value.find((i) => i.category === ''));
const withBudget = computed(() => items.value.filter((i) => i.category !== ''));
const unbudgeted = computed(() => cats.value.filter((c) => !items.value.some((i) => i.category === c.name)));

function ratio(i: BudgetStatus): number {
  return i.amount > 0 ? i.spent / i.amount : 0;
}
function barColor(i: BudgetStatus): string {
  const r = ratio(i);
  return r > 1 ? 'var(--zj-expense)' : r > 0.8 ? 'var(--zj-transfer)' : 'var(--zj-primary)';
}

async function editBudget(category: string) {
  const current = items.value.find((i) => i.category === category)?.amount;
  const label = category === '' ? '总预算' : `「${category}」预算`;
  const res = await ElMessageBox.prompt(
    `${keyLabel.value} ${label}：输入金额（输入 0 表示取消该预算）`,
    '设置预算',
    { inputValue: current ? String(current) : '', inputPattern: /^\d+(\.\d+)?$/, inputErrorMessage: '请输入不小于 0 的金额' }
  ).catch(() => null);
  if (res === null) return; // 用户取消
  try {
    await api.budgetSet(period.value, key.value, category, Number(res.value));
    ElMessage.success(Number(res.value) > 0 ? '预算已保存' : '预算已取消');
    await load();
  } catch (e) {
    ElMessage.error(String(e));
  }
}
</script>

<template>
  <div>
    <h1 class="zj-page-title">预算</h1>
    <PageSub page="budget" fallback="给总支出和各分类设上限，执行进度实时联动账目数据" />

    <!-- 超支醒目提示 -->
    <el-alert
      v-if="total && total.spent > total.amount"
      type="error"
      :closable="false"
      show-icon
      style="margin-bottom: 14px"
      :title="`${keyLabel}总预算已超支：已用 ¥ ${fmtAmount(total.spent)} / ¥ ${fmtAmount(total.amount)}，超出 ¥ ${fmtAmount(total.spent - total.amount)}`"
    />

    <!-- 周期切换 -->
    <div class="zj-card" style="margin-bottom: 14px; display: flex; align-items: center; gap: 12px">
      <el-radio-group v-model="period" @change="onPeriodChange">
        <el-radio-button value="month">按月</el-radio-button>
        <el-radio-button value="year">按年</el-radio-button>
      </el-radio-group>
      <el-button circle size="small" @click="shift(-1)">‹</el-button>
      <span style="font-weight: 600; min-width: 110px; text-align: center" class="zj-num">{{ keyLabel }}</span>
      <el-button circle size="small" @click="shift(1)">›</el-button>
      <el-date-picker
        v-if="period === 'month'"
        :model-value="monthKey"
        type="month"
        value-format="YYYY-MM"
        :clearable="false"
        style="width: 130px"
        placeholder="跳到任意月"
        @update:model-value="(v: string) => { monthKey = v; load(); }"
      />
      <el-date-picker
        v-else
        :model-value="yearKey"
        type="year"
        value-format="YYYY"
        :clearable="false"
        style="width: 110px"
        placeholder="跳到任意年"
        @update:model-value="(v: string) => { yearKey = v; load(); }"
      />
      <div style="flex: 1" />
      <el-button type="primary" plain size="small" @click="editBudget('')">设置总预算</el-button>
    </div>

    <!-- 总预算执行卡 -->
    <div v-if="total" class="zj-card" style="margin-bottom: 14px">
      <div style="display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 10px">
        <span style="font-weight: 600">总预算 · {{ keyLabel }}</span>
        <el-button text size="small" @click="editBudget('')">修改</el-button>
      </div>
      <div style="display: flex; align-items: baseline; gap: 14px; margin-bottom: 10px">
        <span :style="{ color: barColor(total) }" style="font-size: 30px; font-weight: 700" class="zj-num">¥ {{ fmtAmount(total.spent) }}</span>
        <span style="color: var(--zj-text-sub)" class="zj-num">/ 预算 ¥ {{ fmtAmount(total.amount) }}</span>
        <span class="zj-num" style="margin-left: auto; color: var(--zj-text-sub)">
          {{ total.spent > total.amount ? '已超支' : '剩余' }}
          <b :style="{ color: total.spent > total.amount ? 'var(--zj-expense)' : 'var(--zj-income)' }">¥ {{ fmtAmount(Math.max(total.amount - total.spent, 0)) }}</b>
        </span>
      </div>
      <div class="budget-track">
        <i :style="{ width: Math.min(ratio(total), 1) * 100 + '%', background: barColor(total) }" />
      </div>
      <div class="budget-pct zj-num">已使用 {{ (ratio(total) * 100).toFixed(1) }}%</div>
    </div>
    <div v-else class="zj-card" style="margin-bottom: 14px; display: flex; align-items: center; gap: 12px">
      <span style="color: var(--zj-text-sub)">{{ keyLabel }}还没有总预算——设一个，超支前心里有数。</span>
      <el-button size="small" type="primary" @click="editBudget('')">设置总预算</el-button>
    </div>

    <!-- 分类预算 -->
    <div class="zj-card">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px">
        <span style="font-weight: 600">分类预算</span>
        <el-select
          v-if="unbudgeted.length"
          placeholder="给分类加预算…"
          style="width: 200px"
          value-key="name"
          @change="(name: string) => { if (name) editBudget(name); }"
        >
          <el-option v-for="c in unbudgeted" :key="c.id" :label="c.name" :value="c.name" />
        </el-select>
      </div>

      <div v-if="withBudget.length" class="budget-list">
        <div v-for="i in withBudget" :key="i.category" class="budget-row" @click="editBudget(i.category)">
          <span class="budget-cat" :title="i.category">{{ i.category }}</span>
          <span class="budget-track" style="flex: 1">
            <i :style="{ width: Math.min(ratio(i), 1) * 100 + '%', background: barColor(i) }" />
          </span>
          <span class="budget-pct zj-num">{{ (ratio(i) * 100).toFixed(0) }}%</span>
          <span class="budget-amt zj-num">
            ¥ {{ fmtAmount(i.spent) }} <span style="color: var(--zj-text-sub)">/ {{ fmtAmount(i.amount) }}</span>
          </span>
        </div>
      </div>
      <div v-else class="budget-empty">
        还没有分类预算。点右上角下拉给常用分类（如 食品饮料、行车交通）各设一个上限。
      </div>
    </div>
  </div>
</template>

<style scoped>
.budget-track {
  height: 8px;
  border-radius: 999px;
  background: var(--zj-sidebar-hover);
  overflow: hidden;
}
.budget-track i {
  display: block;
  height: 100%;
  border-radius: 999px;
  transition: width 0.25s var(--zj-ease);
}
.budget-pct {
  margin-top: 6px;
  font-size: 12px;
  color: var(--zj-text-sub);
  text-align: right;
}
.budget-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.budget-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 8px 10px;
  border-radius: 10px;
  cursor: pointer;
  transition: background 0.18s var(--zj-ease);
}
.budget-row:hover {
  background: var(--zj-sidebar-hover);
}
.budget-cat {
  width: 150px;
  font-size: 13px;
  color: var(--zj-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: none;
}
.budget-amt {
  width: 190px;
  text-align: right;
  font-size: 13px;
  color: var(--zj-text);
  flex: none;
}
.budget-empty {
  height: 140px;
  display: grid;
  place-items: center;
  color: var(--zj-text-sub);
  font-size: 13.5px;
  border: 1px dashed var(--zj-border);
  border-radius: 12px;
  padding: 0 24px;
  text-align: center;
}
</style>

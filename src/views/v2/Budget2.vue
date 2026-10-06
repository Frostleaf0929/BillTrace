<script setup lang="ts">
// 预算 v2：月度信封式管理
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { ElMessageBox } from 'element-plus';
import { api } from '../../api';
import type { BudgetStatus, Category } from '../../types';
import { icon, catColor, catIcon } from '../../v2/icons';
import { toast } from '../../v2/toast';

const periodKey = ref('');
const items = ref<BudgetStatus[]>([]);
const expCats = ref<Category[]>([]);

const p = (n: number) => String(n).padStart(2, '0');
function initKey(): void {
  const n = new Date();
  periodKey.value = `${n.getFullYear()}-${p(n.getMonth() + 1)}`;
}
function shift(d: number): void {
  const [y, m] = periodKey.value.split('-').map(Number);
  const nm = m + d;
  const ny = nm < 1 ? y - 1 : nm > 12 ? y + 1 : y;
  periodKey.value = `${ny}-${p((nm < 1 ? 12 : nm > 12 ? 1 : nm))}`;
}
const title = computed(() => periodKey.value.replace('-', ' 年 ') + ' 月');

const totalItem = computed(() => items.value.find((b) => b.category === '') ?? null);
const catItems = computed(() => items.value.filter((b) => b.category !== ''));
const totalAmt = computed(() => totalItem.value?.amount ?? 0);
const spentAll = computed(() => catItems.value.reduce((s, b) => s + b.spent, 0));
const over = computed(() => spentAll.value > totalAmt.value && totalAmt.value > 0);
const totalPct = computed(() => (totalAmt.value > 0 ? Math.min(100, Math.round((spentAll.value / totalAmt.value) * 100)) : 0));

function trackClass(pct: number): string {
  return pct >= 100 ? 'red' : pct >= 70 ? 'amber' : '';
}

async function load(): Promise<void> {
  try {
    items.value = await api.budgetList('month', periodKey.value);
  } catch (e) {
    toast(`预算加载失败：${e}`);
  }
}
async function loadCats(): Promise<void> {
  try { expCats.value = (await api.listCategories('expense')).filter((c) => c.parent_id === null); } catch { /* 忽略 */ }
}

async function setBudget(category: string): Promise<void> {
  const cur = items.value.find((b) => b.category === category)?.amount ?? 0;
  try {
    const res = await ElMessageBox.prompt(
      category === '' ? '设置本月总预算金额' : `设置「${category}」的月度预算金额`,
      '预算金额',
      { inputValue: cur ? String(cur) : '', inputPattern: /^\d+(\.\d+)?$/, inputErrorMessage: '请输入数字' },
    ).catch(() => null);
    if (res === null) return;
    await api.budgetSet('month', periodKey.value, category, Number(res.value));
    toast('预算已保存');
    await load();
  } catch (e) {
    toast(`保存失败：${e}`);
  }
}

onMounted(() => { initKey(); load(); loadCats(); 
  const onRefresh = () => { load(); };
  window.addEventListener('v2-refresh', onRefresh);
  onUnmounted(() => window.removeEventListener('v2-refresh', onRefresh));
});
</script>

<template>
  <div>
    <div class="v2-pagehead">
      <div>
        <h1>预算</h1>
        <div class="sub">按月信封式管理，超支提前预警</div>
      </div>
      <div class="v2-head-right">
        <button class="v2-icon-btn" style="width:40px;height:40px;border:1px solid var(--v2-line)" @click="shift(-1)">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><path d="M14.5 6l-6 6 6 6"/></svg>
        </button>
        <span class="v2-date-pill" style="height:40px">{{ title }}</span>
        <button class="v2-icon-btn" style="width:40px;height:40px;border:1px solid var(--v2-line)" @click="shift(1)">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><path d="M9.5 6l6 6-6 6"/></svg>
        </button>
      </div>
    </div>

    <div class="v2-grid">
      <div v-if="over" class="v2-card span12" style="background:var(--v2-warn-tint);border-color:transparent;display:flex;align-items:center;gap:10px;padding:14px 20px">
        <span style="color:var(--v2-warn)" v-html="icon('alert', 18)" />
        <b style="font-size:13.5px">本月总预算已超支 {{ money(spentAll - totalAmt) }}，建议到明细页查看大额分类</b>
      </div>

      <div class="v2-card span5 v2-anchor-card">
        <div style="display:flex;align-items:center;gap:10px;margin-bottom:14px">
          <span style="opacity:.75" v-html="icon('budget', 18)" /><b style="font-size:14px">{{ title }}总预算</b>
        </div>
        <div style="display:flex;gap:24px;margin:6px 0 16px">
          <div><div style="font-size:11.5px" class="ink2">预算</div><div class="num" style="font-size:22px;font-weight:700">{{ money(totalAmt) }}</div></div>
          <div><div style="font-size:11.5px" class="ink2">已用</div><div class="num" style="font-size:22px;font-weight:700">{{ money(spentAll) }}</div></div>
          <div><div style="font-size:11.5px" class="ink2">剩余</div><div class="num" style="font-size:22px;font-weight:700" :style="{ color: over ? 'var(--v2-expense)' : 'var(--v2-accent-soft)' }">{{ money(totalAmt - spentAll) }}</div></div>
        </div>
        <div class="v2-track"><i :class="{ over: over }" :style="{ width: totalPct + '%' }" /></div>
        <div style="display:flex;justify-content:space-between;margin-top:14px;padding-top:12px;border-top:1px solid rgba(255,255,255,.1);font-size:12px" class="ink2">
          <span>{{ totalPct }}% 已使用</span>
          <span>{{ catItems.length }} 个分类预算</span>
        </div>
        <button class="v2-btn v2-anchor-btn" @click="setBudget('')">设定总预算</button>
      </div>

      <div class="v2-card span7">
        <div class="v2-card-head">
          <h3>分类预算</h3>
          <div class="spacer" />
          <button class="v2-btn ghost" style="height:32px" @click="setBudget('')">＋ 添加 / 修改</button>
        </div>
        <div v-for="b in catItems" :key="b.category" class="v2-bud-row">
          <div class="dot" :style="{ background: catColor(b.category) + '26', color: catColor(b.category) }" v-html="catIcon(b.category, 16)" />
          <div class="bud-main">
            <div class="v2-bud-top">
              <span class="v2-bud-name">{{ b.category }}</span>
              <span class="v2-bud-nums">已用 <b>{{ money(b.spent) }}</b> / {{ money(b.amount) }}</span>
            </div>
            <div class="v2-track">
              <i :class="trackClass(Math.round((b.spent / Math.max(1, b.amount)) * 100))" :style="{ width: Math.min(100, Math.round((b.spent / Math.max(1, b.amount)) * 100)) + '%' }" />
            </div>
          </div>
          <div class="v2-bud-left" :class="{ over: b.spent > b.amount }">
            {{ b.spent > b.amount ? '超支 ' + money(b.spent - b.amount) : '剩 ' + money(b.amount - b.spent) }}
          </div>
          <button class="v2-icon-btn" title="编辑" @click="setBudget(b.category)" v-html="icon('edit', 15)" />
        </div>
        <div v-if="!catItems.length" style="padding:26px;text-align:center;color:var(--v2-ink-3);font-size:13px">
          还没有分类预算——点右上角「添加 / 修改」，先设一个总预算，再给大类分配额度
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
function money(n: number): string {
  return '¥' + n.toLocaleString('zh-CN', { maximumFractionDigits: 2 });
}
export default { name: 'Budget2' };
</script>

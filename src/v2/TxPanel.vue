<script setup lang="ts">
// v2 交易详情滑出面板（编辑/删除）
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { api } from '../api';
import type { Category, Tx } from '../types';
import { v2PanelOpen, attachSheetDrag } from './ui';
import VSelect from './VSelect.vue';

const props = defineProps<{ tx: Tx | null; visible: boolean }>();
const emit = defineEmits<{ close: []; saved: []; deleted: [id: number] }>();

const amount = ref('');
const merchant = ref('');
const txType = ref('支出');
const l1 = ref('');
const l2 = ref('');
const account = ref('');     // 支出=转出账户 / 收入=转入账户
const accountFrom = ref(''); // 转账-转出
const accountTo = ref('');   // 转账-转入
const txDate = ref('');
const remark = ref('');
const cats = ref<Category[]>([]);
const accounts = ref<string[]>([]);
const saving = ref(false);
const editAmt = ref(false);

const l1Options = computed(() => [...new Set(cats.value.filter((c) => c.kind !== 'account' && c.parent_id === null).map((c) => c.name))]);
const parentCat = computed(() => cats.value.find((c) => c.parent_id === null && c.name === l1.value));
const l2Options = computed(() => cats.value.filter((c) => c.parent_id !== null && c.parent_id === parentCat.value?.id).map((c) => c.name));
// VSelect 选项表
const l1Opts = computed(() => [{ value: '', label: '未分类' }, ...l1Options.value.map((o) => ({ value: o, label: o }))]);
const l2Opts = computed(() => [{ value: '', label: '二级（可选）' }, ...l2Options.value.map((o) => ({ value: o, label: o }))]);
const accountOpts = computed(() => accounts.value.map((o) => ({ value: o, label: o })));
const transferInOpts = computed(() => accounts.value.filter((o) => o !== accountFrom.value).map((o) => ({ value: o, label: o })));
const subAcc = computed(() => (txType.value === '转账' ? `${accountFrom.value || '…'} → ${accountTo.value || '…'}` : account.value));

watch(() => props.visible, (v) => { v2PanelOpen.value = v; });

// 移动端：把手下拉关闭 + 返回键关面板（守卫见 ui.ts）
// Vue 3.5 模板 ref 绑定在本工程不生效（档案 §坑5 再现），改用选择器直查：
// 同一时刻只有一个视图挂载，全局仅存在一个 TxPanel 的 aside
const onForceClose = (): void => { if (props.visible) emit('close'); };
onMounted(() => {
  const el = document.querySelector('aside.v2-panel');
  if (el) attachSheetDrag(el as HTMLElement, () => props.visible, () => emit('close'));
  window.addEventListener('v2-close-panel', onForceClose);
});
onBeforeUnmount(() => {
  window.removeEventListener('v2-close-panel', onForceClose);
});
watch(() => props.visible, async (v) => {
  if (!v || !props.tx) return;
  const t = props.tx;
  amount.value = String(t.amount);
  merchant.value = t.merchant || '';
  txType.value = t.tx_type;
  l1.value = t.l1 || '';
  l2.value = t.l2 || '';
  account.value = t.account_out || t.account_in || '';
  accountFrom.value = t.account_out || '';
  accountTo.value = t.account_in || '';
  txDate.value = t.tx_time.slice(0, 10);
  remark.value = t.remark || '';
  await refreshCats();
  try {
    const bs = await api.accountBalances();
    accounts.value = bs.map((b) => b.name);
    if (account.value && !accounts.value.includes(account.value)) accounts.value.push(account.value);
  } catch { /* 忽略 */ }
});

async function addCat(level: 1 | 2): Promise<void> {
    const kind = txType.value === '收入' ? 'income' : 'expense';
    if (level === 1) {
      const res = await ElMessageBox.prompt('新增一级分类名称', '新增一级', { inputPattern: /\S+/, inputErrorMessage: '名称不能为空' }).catch(() => null);
      if (!res) return;
      await api.saveCategory({ kind, parent_id: null, name: res.value.trim() });
      await refreshCats();
      l1.value = res.value.trim();
      l2.value = '';
      return;
    }
    const parent = cats.value.find((c) => c.parent_id === null && c.name === l1.value);
    if (!parent) { ElMessage.warning('请先选择一级分类'); return; }
    const res = await ElMessageBox.prompt(`在「${l1.value}」下新增二级分类`, '新增二级', { inputPattern: /\S+/, inputErrorMessage: '名称不能为空' }).catch(() => null);
    if (!res) return;
    await api.saveCategory({ kind, parent_id: parent.id, name: res.value.trim() });
    await refreshCats();
    l2.value = res.value.trim();
  }

function dblAmount(): void {
  editAmt.value = true;
  nextTick(() => {
    const el = document.getElementById('v2-tx-amt') as HTMLInputElement | null;
    el?.focus();
    el?.select();
  });
}

async function refreshCats(): Promise<void> {
  try { cats.value = await api.listCategories(); } catch { /* 忽略 */ }
}

async function save(): Promise<void> {
  if (!props.tx || saving.value) return;
  const isT = txType.value === '转账';
  if (isT && (!accountFrom.value || !accountTo.value)) {
    ElMessage.warning('转账需要选择转出与转入账户');
    return;
  }
  if (isT && accountFrom.value === accountTo.value) {
    ElMessage.warning('转出与转入不能是同一个账户');
    return;
  }
  saving.value = true;
  try {
    const amt = Math.abs(parseFloat(amount.value) || props.tx.amount);
    await api.saveTransaction({
      ...props.tx,
      tx_type: txType.value,
      tx_time: `${txDate.value} 12:00:00`,
      l1: isT ? null : l1.value || null,
      l2: isT ? null : l2.value || null,
      account_out: isT ? (accountFrom.value || null) : txType.value === '收入' ? null : (account.value || null),
      account_in: isT ? (accountTo.value || null) : txType.value === '支出' ? null : (account.value || null),
      amount: amt,
      merchant: merchant.value,
      remark: remark.value || null,
    });
    emit('saved');
    emit('close');
  } catch (e) {
    alert(`保存失败：${e}`);
  } finally {
    saving.value = false;
  }
}

async function del(): Promise<void> {
  if (!props.tx) return;
  try {
    await ElMessageBox.confirm(`删除这条 ${props.tx.tx_time.slice(0, 10)} 的记录（${props.tx.amount}）？`, '删除', { type: 'warning' });
  } catch {
    return;
  }
  try {
    await api.deleteTransaction(props.tx.id);
    emit('deleted', props.tx.id);
    emit('close');
  } catch (e) {
    alert(`删除失败：${e}`);
  }
}
</script>

<template>
  <!-- 遮罩 + 右侧滑出面板（与 QuickAdd 同一 .v2-panel 体系） -->
  <div class="v2-overlay" :class="{ show: visible }" @click="emit('close')" />
  <aside class="v2-panel" :class="{ show: visible }">
    <button type="button" class="v2-sheet-handle" title="下拉收起" aria-label="下拉收起" />
    <div style="display:flex;align-items:center;gap:10px">
      <h2 style="font-size:19px;margin:0">编辑记录</h2>
    </div>
    <div class="panel-sub">{{ txDate }} · {{ subAcc }}</div>
    <div class="v2-field-label">金额（双击数字可改）</div>
    <div style="display:flex;align-items:baseline;gap:10px;margin:4px 0 2px">
      <input
        v-if="editAmt" id="v2-tx-amt" v-model="amount" class="v2-text-input num"
        style="font-size:22px;font-weight:700;height:44px" type="text" inputmode="decimal"
        @keydown.enter="editAmt = false" @blur="editAmt = false"
      >
      <span
        v-else class="num" style="font-size:34px;font-weight:700;cursor:text"
        :style="{ color: txType === '收入' ? 'var(--v2-income)' : 'var(--v2-ink)' }"
        title="双击修改金额" @dblclick="dblAmount"
      >
        {{ txType === '收入' ? '+' : '-' }}{{ amount || '0' }}
      </span>
    </div>
    <div class="v2-field-label">商家</div>
    <input v-model="merchant" class="v2-text-input">
    <div class="v2-field-label">类型</div>
    <div class="v2-seg">
      <button v-for="t in ['支出', '收入', '转账']" :key="t" :class="{ on: txType === t }" @click="txType = t">{{ t }}</button>
    </div>
    <div v-if="txType !== '转账'" class="v2-field-label" style="display:flex;align-items:center">
      分类
      <span style="flex:1" />
      <button class="v2-link-btn" style="font-size:12px" @click="addCat(1)">＋一级</button>
      <button class="v2-link-btn" style="font-size:12px;margin-left:8px" @click="addCat(2)">＋二级</button>
    </div>
    <div v-if="txType !== '转账'" style="display:flex;gap:8px">
      <VSelect v-model="l1" :options="l1Opts" block style="flex:1" @change="l2 = ''" />
      <VSelect v-model="l2" :options="l2Opts" block style="flex:1" />
    </div>
    <template v-if="txType !== '转账'">
      <div class="v2-field-label">账户（{{ txType === '收入' ? '转入' : '转出' }}）</div>
      <VSelect v-model="account" :options="accountOpts" block />
    </template>
    <template v-else>
      <div class="v2-field-label">转出账户</div>
      <VSelect v-model="accountFrom" :options="accountOpts" block />
      <div class="v2-field-label">转入账户</div>
      <VSelect v-model="accountTo" :options="transferInOpts" block />
    </template>
    <div class="v2-field-label">日期</div>
    <input v-model="txDate" class="v2-text-input num" type="date">
    <div class="v2-field-label">备注</div>
    <input v-model="remark" class="v2-text-input">
    <div style="flex:1;min-height:12px" />
    <div style="display:flex;gap:10px;margin-top:20px">
      <button class="v2-btn primary" style="flex:1;justify-content:center" :disabled="saving" @click="save">保存</button>
      <button class="v2-btn ghost" style="padding:0 14px" title="关闭" @click="emit('close')">✕</button>
      <button class="v2-btn danger" style="padding:0 14px" title="删除" @click="del">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><path d="M4.5 7h15M9.5 7V5.2A1.2 1.2 0 0 1 10.7 4h2.6a1.2 1.2 0 0 1 1.2 1.2V7M6.5 7l.9 12a1.5 1.5 0 0 0 1.5 1.4h6.2a1.5 1.5 0 0 0 1.5-1.4l.9-12"/></svg>
      </button>
    </div>
  </aside>
</template>

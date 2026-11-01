<script setup lang="ts">
// v2 快速记账面板（右侧滑入）
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { api } from '../api';
import type { Category, AccountBalance } from '../types';
import { catColor, catIcon } from './icons';
import { v2PanelOpen } from './ui';
import { toast } from './toast';
import VSelect from './VSelect.vue';
import { ElMessageBox } from 'element-plus';

// visible 由 App.vue 的 v-if 控制（挂载即打开），组件内不再依赖它
defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: []; saved: [] }>();

const type = ref<'expense' | 'income' | 'transfer'>('expense');
const amount = ref('');
const l1 = ref('');
const l2 = ref('');
const account = ref('');
const accountTo = ref('');
const date = ref('');
const note = ref('');
const saving = ref(false);

const expCats = ref<Category[]>([]);
const incCats = ref<Category[]>([]);
const accounts = ref<AccountBalance[]>([]);

const catChips = computed(() => {
  const cats = type.value === 'expense' ? expCats.value : incCats.value;
  const l1s = [...new Set(cats.filter((c) => c.parent_id === null).map((c) => c.name))];
  return l1s.slice(0, 9);
});
// 二级分类按 parent_id 过滤（按名称相等过滤 + 列表只装了一级分类 = 二级永远为空的老坑）
const l2Options = computed(() => {
  const cats = type.value === 'expense' ? expCats.value : incCats.value;
  const parent = cats.find((c) => c.parent_id === null && c.name === l1.value);
  if (!parent) return [];
  return cats.filter((c) => c.parent_id === parent.id).map((c) => c.name);
});
// VSelect 选项表
const l2SelOpts = computed(() => [{ value: '', label: '不选' }, ...l2Options.value.map((o) => ({ value: o, label: o }))]);
const accountOpts = computed(() => accounts.value.map((a) => ({ value: a.name, label: a.name })));
const transferInOpts = computed(() => accounts.value.filter((a) => a.name !== account.value).map((a) => ({ value: a.name, label: a.name })));

async function loadRefs(): Promise<void> {
  try {
    const [e, i, b] = await Promise.all([
      api.listCategories('expense'),
      api.listCategories('income'),
      api.accountBalances(),
    ]);
    expCats.value = e; // 全量（一级 + 二级），一级/二级在使用处各自过滤
    incCats.value = i;
    accounts.value = b;
  } catch { /* 忽略 */ }
}

// 关键：本组件由 v-if 挂载（挂载时 visible 已经是 true），watch(visible) 永远不会触发——
// 曾经导致打开时分类/账户/日期全空（用户反馈"根本没有选择的地方"）。改为挂载即初始化。
function onForceClose(): void {
  emit('close');
}
onMounted(async () => {
  v2PanelOpen.value = true;
  // 移动端返回键关面板：全局关闭事件 → close（守卫见 ui.ts）
  window.addEventListener('v2-close-panel', onForceClose);
  await loadRefs();
  type.value = 'expense';
  amount.value = '';
  l1.value = expCats.value[0]?.name ?? '';
  l2.value = '';
  account.value = accounts.value[0]?.name ?? '';
  accountTo.value = '';
  const now = new Date();
  date.value = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
  note.value = '';
  setTimeout(() => (document.getElementById('v2-qa-amount') as HTMLInputElement | null)?.focus(), 240);
});
onUnmounted(() => {
  v2PanelOpen.value = false;
  window.removeEventListener('v2-close-panel', onForceClose);
});

async function addL1(): Promise<void> {
  const kind = type.value === 'income' ? 'income' : 'expense';
  const res = await ElMessageBox.prompt('新增一级分类名称', '新增一级', { inputPattern: /\S+/, inputErrorMessage: '名称不能为空' }).catch(() => null);
  if (!res) return;
  await api.saveCategory({ kind, parent_id: null, name: res.value.trim() });
  await loadRefs();
  l1.value = res.value.trim();
}

async function save(keep: boolean): Promise<void> {
  const amt = Math.abs(parseFloat(amount.value) || 0);
  if (!amt) return;
  if (type.value === 'transfer') {
    if (!account.value || !accountTo.value) { toast('转账需要选择转出与转入账户'); return; }
    if (account.value === accountTo.value) { toast('转出与转入不能是同一个账户'); return; }
  }
  if (saving.value) return;
  saving.value = true;
  try {
    const tx = {
      tx_type: type.value === 'expense' ? '支出' : type.value === 'income' ? '收入' : '转账',
      tx_time: `${date.value} 12:00:00`,
      l1: type.value === 'transfer' ? null : l1.value || null,
      l2: type.value === 'transfer' ? null : l2.value || null,
      l3: null,
      account_out: type.value === 'income' ? null : account.value || null,
      account_in: type.value === 'expense' ? null : type.value === 'transfer' ? accountTo.value || null : account.value || null,
      currency: null,
      amount: amt,
      member: null,
      merchant: type.value === 'transfer' ? '账户转账' : '手动记入',
      project_category: null,
      project: null,
      booker: null,
      remark: note.value || null,
    };
    await api.saveTransaction({ id: 0, ...tx });
    emit('saved');
    if (keep) {
      amount.value = '';
      (document.getElementById('v2-qa-amount') as HTMLInputElement | null)?.focus();
    } else {
      emit('close');
    }
  } catch (e) {
    alert(`保存失败：${e}`);
  } finally {
    saving.value = false;
  }
}

function onKey(e: KeyboardEvent): void {
  if (e.key === 'Enter') save(false);
}
</script>

<template>
  <div>
    <h2>记一笔</h2>
    <div class="panel-sub">金额、分类、账户——三步完成</div>
    <input id="v2-qa-amount" v-model="amount" class="v2-amount-input" type="text" inputmode="decimal" placeholder="0.00" @keydown="onKey">
    <div class="v2-field-label">类型</div>
    <div class="v2-seg">
      <button :class="{ on: type === 'expense' }" @click="type = 'expense'">支出</button>
      <button :class="{ on: type === 'income' }" @click="type = 'income'">收入</button>
      <button :class="{ on: type === 'transfer' }" @click="type = 'transfer'">转账</button>
    </div>
    <template v-if="type !== 'transfer'">
      <div class="v2-field-label">分类</div>
      <div class="v2-chip-grid">
        <button
          v-for="c in catChips" :key="c" class="v2-cat-chip" :class="{ on: l1 === c }"
          @click="l1 = c; l2 = ''"
        >
          <i :style="{ background: catColor(c) }" /><span v-html="catIcon(c, 13)" />{{ c }}
        </button>
      </div>
      <div v-if="l2Options.length" class="v2-field-label" style="display:flex;align-items:center">
        二级分类（可选）
        <span style="flex:1" />
        <button class="v2-link-btn" style="font-size:12px" @click="addL1">＋新建一级</button>
      </div>
      <VSelect v-if="l2Options.length" v-model="l2" :options="l2SelOpts" block />
    </template>
    <template v-else>
      <div class="v2-two-col">
        <div>
          <div class="v2-field-label">转出账户</div>
          <VSelect v-model="account" :options="accountOpts" block placeholder="选择转出" />
        </div>
        <div>
          <div class="v2-field-label">转入账户</div>
          <VSelect v-model="accountTo" :options="transferInOpts" block placeholder="选择转入" />
        </div>
      </div>
    </template>
    <template v-if="type !== 'transfer'">
      <div class="v2-field-label">账户（{{ type === 'income' ? '转入' : '转出' }}）</div>
      <VSelect v-model="account" :options="accountOpts" block />
    </template>
    <div class="v2-field-label">日期</div>
    <input v-model="date" class="v2-text-input num" type="date">
    <div class="v2-field-label">备注（可选）</div>
    <input v-model="note" class="v2-text-input" placeholder="填写备注">
    <div style="display:flex;gap:10px;margin-top:24px">
      <button class="v2-btn primary" style="flex:1;justify-content:center" :disabled="saving" @click="save(false)">保存</button>
      <button class="v2-btn ghost" :disabled="saving" @click="save(true)">保存并继续</button>
      <button class="v2-btn ghost" style="padding:0 14px" @click="emit('close')">✕</button>
    </div>
  </div>
</template>

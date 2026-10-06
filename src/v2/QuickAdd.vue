<script setup lang="ts">
// v2 快速记账面板（右侧滑入）
import { computed, ref, watch } from 'vue';
import { api } from '../api';
import type { Category, AccountBalance } from '../types';
import { catColor, catIcon } from './icons';

const props = defineProps<{ visible: boolean }>();
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
  const l1s = [...new Set(cats.map((c) => c.name))];
  return l1s.slice(0, 9);
});
const l2Options = computed(() => {
  const cats = type.value === 'expense' ? expCats.value : incCats.value;
  return cats.filter((c) => c.parent_id !== null && c.name === l1.value).map((c) => c.name);
});

async function loadRefs(): Promise<void> {
  try {
    const [e, i, b] = await Promise.all([
      api.listCategories('expense'),
      api.listCategories('income'),
      api.accountBalances(),
    ]);
    expCats.value = e.filter((c) => c.parent_id === null);
    incCats.value = i.filter((c) => c.parent_id === null);
    accounts.value = b;
  } catch { /* 忽略 */ }
}

watch(() => props.visible, async (v) => {
  if (!v) return;
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

async function save(keep: boolean): Promise<void> {
  const amt = Math.abs(parseFloat(amount.value) || 0);
  if (!amt) return;
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
      <div v-if="l2Options.length" class="v2-field-label">二级分类（可选）</div>
      <select v-model="l2" class="v2-select" style="width:100%">
        <option value="">不选</option>
        <option v-for="o in l2Options" :key="o" :value="o">{{ o }}</option>
      </select>
    </template>
    <template v-else>
      <div class="v2-field-label">从哪个账户转出</div>
      <div class="v2-chip-grid">
        <button v-for="a in accounts" :key="a.name" class="v2-cat-chip" :class="{ on: account === a.name }" @click="account = a.name">{{ a.name }}</button>
      </div>
      <div class="v2-field-label">转入哪个账户</div>
      <div class="v2-chip-grid">
        <button v-for="a in accounts.filter((x) => x.name !== account)" :key="a.name" class="v2-cat-chip" :class="{ on: accountTo === a.name }" @click="accountTo = a.name">{{ a.name }}</button>
      </div>
    </template>
    <div class="v2-field-label">{{ type === 'transfer' ? '转出账户' : '账户' }}</div>
    <div class="v2-chip-grid">
      <button v-for="a in accounts" :key="a.name" class="v2-cat-chip" :class="{ on: account === a.name }" @click="account = a.name">{{ a.name }}</button>
    </div>
    <div class="v2-field-label">日期与备注</div>
    <div style="display:flex;gap:10px">
      <input v-model="date" class="v2-text-input" type="date" style="flex:none;width:170px">
      <input v-model="note" class="v2-text-input" placeholder="备注（可选）">
    </div>
    <div style="display:flex;gap:10px;margin-top:24px">
      <button class="v2-btn primary" style="flex:1;justify-content:center" :disabled="saving" @click="save(false)">保存</button>
      <button class="v2-btn ghost" :disabled="saving" @click="save(true)">保存并继续</button>
      <button class="v2-btn ghost" style="padding:0 14px" @click="emit('close')">✕</button>
    </div>
  </div>
</template>

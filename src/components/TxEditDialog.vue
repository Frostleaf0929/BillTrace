<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { api } from '../api';
import type { Category, Tx } from '../types';

const props = defineProps<{ visible: boolean; tx: Tx | null }>();
const emit = defineEmits<{ (e: 'update:visible', v: boolean): void; (e: 'saved'): void }>();

const visible = computed({
  get: () => props.visible,
  set: (v) => emit('update:visible', v),
});

const expenseCats = ref<Category[]>([]);
const incomeCats = ref<Category[]>([]);
const accounts = ref<Category[]>([]);

const form = reactive<Tx>({
  id: 0,
  tx_type: '支出',
  tx_time: '',
  l1: null,
  l2: null,
  l3: null,
  account_out: null,
  account_in: null,
  currency: 'CNY',
  amount: 0,
  member: null,
  merchant: null,
  project_category: null,
  project: null,
  booker: null,
  remark: null,
});

const catSource = computed(() =>
  form.tx_type === '收入' ? incomeCats.value : expenseCats.value
);
const l1List = computed(() => catSource.value.filter((c) => c.parent_id === null));
const l2List = computed(() =>
  catSource.value.filter((c) => c.parent_id !== null && c.parent_id === l1IdByName.value[form.l1 ?? ''])
);
const l2IdByName = computed<Record<string, number>>(() => {
  const m: Record<string, number> = {};
  for (const c of l2List.value) m[c.name] = c.id;
  return m;
});
const l3List = computed(() =>
  catSource.value.filter((c) => c.parent_id !== null && c.parent_id === l2IdByName.value[form.l2 ?? ''])
);
const l1IdByName = computed<Record<string, number>>(() => {
  const m: Record<string, number> = {};
  for (const c of catSource.value.filter((x) => x.parent_id === null)) m[c.name] = c.id;
  return m;
});
const accountL1 = computed(() => accounts.value.filter((c) => c.parent_id === null));

watch(
  () => props.visible,
  async (v) => {
    if (!v) return;
    if (accounts.value.length === 0) {
      const all = await api.listCategories();
      expenseCats.value = all.filter((c) => c.kind === 'expense');
      incomeCats.value = all.filter((c) => c.kind === 'income');
      accounts.value = all.filter((c) => c.kind === 'account');
    }
    Object.assign(form, props.tx ?? {
      id: 0, tx_type: '支出', tx_time: nowLocal(), l1: null, l2: null, l3: null,
      account_out: null, account_in: null, currency: 'CNY', amount: 0,
      member: null, merchant: null, project_category: null, project: null,
      booker: null, remark: null,
    });
  }
);

function nowLocal(): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}

async function save() {
  if (!form.tx_time || !form.amount) {
    ElMessage.warning('请至少填写日期与金额');
    return;
  }
  await api.saveTransaction({ ...form, amount: Number(form.amount) });
  ElMessage.success(form.id ? '已保存修改' : '已新增记录');
  visible.value = false;
  emit('saved');
}
</script>

<template>
  <el-dialog v-model="visible" :title="form.id ? '编辑记录' : '手动记账'" width="640px" align-center destroy-on-close>
    <el-form label-width="82px" label-position="left">
      <el-row :gutter="12">
        <el-col :span="12">
          <el-form-item label="交易类型">
            <el-select v-model="form.tx_type" style="width: 100%">
              <el-option v-for="t in ['支出', '收入', '转账', '报销', '代付', '余额变更', '债权变更']" :key="t" :label="t" :value="t" />
            </el-select>
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item label="日期时间">
            <el-date-picker v-model="form.tx_time" type="datetime" value-format="YYYY-MM-DD HH:mm:ss" style="width: 100%" />
          </el-form-item>
        </el-col>
        <template v-if="form.tx_type === '支出' || form.tx_type === '收入'">
          <el-col :span="12">
            <el-form-item label="一级分类">
              <el-select v-model="form.l1" clearable style="width: 100%" @change="form.l2 = null">
                <el-option v-for="c in l1List" :key="c.id" :label="c.name" :value="c.name" />
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="二级分类">
              <el-select v-model="form.l2" clearable style="width: 100%" :disabled="!form.l1" @change="form.l3 = null">
                <el-option v-for="c in l2List" :key="c.id" :label="c.name" :value="c.name" />
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="12" v-if="l3List.length > 0">
            <el-form-item label="小级分类">
              <el-select v-model="form.l3" clearable style="width: 100%" :disabled="!form.l2">
                <el-option v-for="c in l3List" :key="c.id" :label="c.name" :value="c.name" />
              </el-select>
            </el-form-item>
          </el-col>
        </template>
        <el-col :span="12" v-if="form.tx_type !== '收入'">
          <el-form-item label="支出账户">
            <el-select v-model="form.account_out" filterable clearable allow-create style="width: 100%">
              <el-option-group v-for="g in accountL1" :key="g.id" :label="g.name">
                <el-option v-for="c in accounts.filter((x) => x.parent_id === g.id)" :key="c.id" :label="c.name" :value="c.name" />
              </el-option-group>
            </el-select>
          </el-form-item>
        </el-col>
        <el-col :span="12" v-else>
          <el-form-item label="转入账户">
            <el-select v-model="form.account_in" filterable clearable allow-create style="width: 100%">
              <el-option-group v-for="g in accountL1" :key="g.id" :label="g.name">
                <el-option v-for="c in accounts.filter((x) => x.parent_id === g.id)" :key="c.id" :label="c.name" :value="c.name" />
              </el-option-group>
            </el-select>
          </el-form-item>
        </el-col>
        <el-col :span="12" v-if="form.tx_type === '转账' || form.tx_type === '代付' || form.tx_type === '报销'">
          <el-form-item label="转入账户">
            <el-select v-model="form.account_in" filterable clearable allow-create style="width: 100%">
              <el-option-group v-for="g in accountL1" :key="g.id" :label="g.name">
                <el-option v-for="c in accounts.filter((x) => x.parent_id === g.id)" :key="c.id" :label="c.name" :value="c.name" />
              </el-option-group>
            </el-select>
          </el-form-item>
        </el-col>
        <el-col :span="8">
          <el-form-item label="金额">
            <el-input-number v-model="form.amount" :min="0" :precision="2" :controls="false" style="width: 100%" />
          </el-form-item>
        </el-col>
        <el-col :span="8">
          <el-form-item label="币种">
            <el-select v-model="form.currency" style="width: 100%" allow-create filterable>
              <el-option v-for="c in ['CNY', 'USD', 'EUR', 'JPY', 'HKD', 'TWD', 'GBP']" :key="c" :label="c" :value="c" />
            </el-select>
          </el-form-item>
        </el-col>
        <el-col :span="8">
          <el-form-item label="商家">
            <el-input v-model="form.merchant" placeholder="商家/交易对方" />
          </el-form-item>
        </el-col>
        <el-col :span="8">
          <el-form-item label="成员">
            <el-input v-model="form.member" />
          </el-form-item>
        </el-col>
        <el-col :span="8">
          <el-form-item label="项目分类">
            <el-input v-model="form.project_category" />
          </el-form-item>
        </el-col>
        <el-col :span="8">
          <el-form-item label="项目">
            <el-input v-model="form.project" />
          </el-form-item>
        </el-col>
        <el-col :span="24">
          <el-form-item label="备注">
            <el-input v-model="form.remark" type="textarea" :rows="2" />
          </el-form-item>
        </el-col>
      </el-row>
    </el-form>
    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" @click="save">保存</el-button>
    </template>
  </el-dialog>
</template>

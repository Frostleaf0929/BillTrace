<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { ElMessage } from 'element-plus';
import { api } from '../api';
import type { Category, PendingRow, MerchantAssign } from '../types';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ (e: 'update:visible', v: boolean): void; (e: 'resolved'): void; (e: 'opened'): void }>();

const visible = computed({
  get: () => props.visible,
  set: (v) => emit('update:visible', v),
});

interface Group {
  merchant: string;
  count: number;
  l1: string;
  l2: string | null;
  action: 'assign' | 'skip';
}

const groups = ref<Group[]>([]);
const expenseCats = ref<Category[]>([]);
const incomeCats = ref<Category[]>([]);

const l1List = computed(() => [...expenseCats.value, ...incomeCats.value].filter((c) => c.parent_id === null));
const l2ByL1 = computed(() => {
  const m: Record<string, Category[]> = {};
  for (const c of [...expenseCats.value, ...incomeCats.value].filter((x) => x.parent_id !== null)) {
    const parentName = [...expenseCats.value, ...incomeCats.value].find((p) => p.id === c.parent_id)?.name ?? '';
    (m[parentName] ||= []).push(c);
  }
  return m;
});

onMounted(async () => {
  const all = await api.listCategories();
  expenseCats.value = all.filter((c) => c.kind === 'expense');
  incomeCats.value = all.filter((c) => c.kind === 'income');
});

async function load() {
  const pending = await api.listPending();
  const byMerchant = new Map<string, number>();
  for (const p of pending) byMerchant.set(p.tx.merchant ?? '(空)', (byMerchant.get(p.tx.merchant ?? '(空)') ?? 0) + 1);
  groups.value = [...byMerchant.entries()]
    .sort((a, b) => b[1] - a[1])
    .map(([merchant, count]) => ({
      merchant,
      count,
      l1: merchantHasIncome(pending, merchant) ? '其他收入' : '',
      l2: null,
      action: 'assign' as const,
    }));
}

function merchantHasIncome(pending: PendingRow[], merchant: string): boolean {
  return pending.some((p) => p.tx.merchant === merchant && p.tx.tx_type === '收入');
}

async function confirmAll() {
  const assigns: MerchantAssign[] = [];
  const skip: string[] = [];
  for (const g of groups.value) {
    if (g.action === 'skip') {
      skip.push(g.merchant);
    } else if (!g.l1) {
      ElMessage.warning(`「${g.merchant}」请选择一级分类（必选），或选择跳过`);
      return;
    } else {
      assigns.push({ merchant: g.merchant, l1: g.l1, l2: g.l2 || null });
    }
  }
  const [confirmed, skipped] = await api.resolvePending(assigns, skip);
  ElMessage.success(`已归类 ${confirmed} 条，跳过 ${skipped} 条；规则已自动沉淀`);
  visible.value = false;
  emit('resolved');
}

defineExpose({ load });
</script>

<template>
  <el-dialog v-model="visible" title="新商家归类" width="760px" align-center destroy-on-close @open="(() => { load(); emit('opened'); })">
    <el-alert type="info" :closable="false" show-icon style="margin-bottom: 12px"
      title="以下商家是首次出现且没有匹配的自动分类规则。一级分类为必选，二级可选；确认后将自动写入规则，下次导入同类商家自动分类。" />
    <el-table :data="groups" size="small" max-height="420">
      <el-table-column label="商家" prop="merchant" min-width="160" show-overflow-tooltip />
      <el-table-column label="笔数" prop="count" width="60" />
      <el-table-column label="处理" width="90">
        <template #default="{ row }">
          <el-radio-group v-model="row.action" size="small">
            <el-radio-button value="assign">归类</el-radio-button>
            <el-radio-button value="skip">跳过</el-radio-button>
          </el-radio-group>
        </template>
      </el-table-column>
      <el-table-column label="一级分类（必选）" min-width="140">
        <template #default="{ row }">
          <el-select v-model="row.l1" filterable :disabled="row.action === 'skip'" placeholder="选择一级分类" @change="row.l2 = null">
            <el-option v-for="c in l1List" :key="c.id" :label="c.name" :value="c.name" />
          </el-select>
        </template>
      </el-table-column>
      <el-table-column label="二级分类（可选）" min-width="140">
        <template #default="{ row }">
          <el-select v-model="row.l2" filterable clearable :disabled="row.action === 'skip' || !row.l1" placeholder="可不选">
            <el-option v-for="c in l2ByL1[row.l1] ?? []" :key="c.id" :label="c.name" :value="c.name" />
          </el-select>
        </template>
      </el-table-column>
    </el-table>
    <template #footer>
      <el-button @click="visible = false">稍后处理</el-button>
      <el-button type="primary" @click="confirmAll">确认并沉淀为规则</el-button>
    </template>
  </el-dialog>
</template>

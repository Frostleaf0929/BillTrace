<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
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
  loading.value = true;
  try {
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
    page.value = 1;
    // 恢复上次未确认的草稿（中途关掉弹窗不丢已选的分类）
    try {
      const draft = JSON.parse(localStorage.getItem(DRAFT_KEY) ?? '{}') as Record<string, Pick<Group, 'action' | 'l1' | 'l2'>>;
      for (const g of groups.value) {
        const d = draft[g.merchant];
        if (d) {
          g.action = d.action;
          g.l1 = d.l1;
          g.l2 = d.l2;
        }
      }
    } catch { /* 无草稿 */ }
  } catch (e) {
    ElMessage.error(String(e));
  } finally {
    loading.value = false;
  }
}

// 分页渲染：几百个商家时整表渲染会卡（用户验收反馈），每页 15 行
const loading = ref(false);
const page = ref(1);
const PAGE_SIZE = 15;
const pagedGroups = computed(() => groups.value.slice((page.value - 1) * PAGE_SIZE, page.value * PAGE_SIZE));

// 草稿缓存：只要做过的选择就实时落盘，确认成功后才清除
const DRAFT_KEY = 'zj.pendingDraft';

function saveDraft() {
  const draft: Record<string, Pick<Group, 'action' | 'l1' | 'l2'>> = {};
  for (const g of groups.value) {
    if (g.action === 'skip' || g.l1) draft[g.merchant] = { action: g.action, l1: g.l1, l2: g.l2 };
  }
  try {
    localStorage.setItem(DRAFT_KEY, JSON.stringify(draft));
  } catch { /* 忽略 */ }
}

watch(groups, saveDraft, { deep: true });

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
  try {
    const [confirmed, skipped] = await api.resolvePending(assigns, skip);
    ElMessage.success(
      skipped > 0
        ? `已归类 ${confirmed} 条；${skipped} 条跳过记录已归入「统计未确认」（可到详细页修改）`
        : `已归类 ${confirmed} 条；规则已自动沉淀`
    );
    try {
      localStorage.removeItem(DRAFT_KEY);
    } catch { /* 忽略 */ }
    visible.value = false;
    emit('resolved');
  } catch (e) {
    ElMessage.error(String(e));
  }
}

defineExpose({ load });
</script>

<template>
  <el-dialog v-model="visible" title="新商家归类" width="760px" align-center destroy-on-close @open="(() => { load(); emit('opened'); })">
    <el-alert type="info" :closable="false" show-icon style="margin-bottom: 12px"
      title="以下商家是首次出现且没有匹配的自动分类规则。一级分类为必选，二级可选；确认后将自动写入规则，下次导入同类商家自动分类。" />
    <el-table v-loading="loading" :data="pagedGroups" size="small" max-height="420">
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
    <el-pagination
      v-if="groups.length > PAGE_SIZE"
      v-model:current-page="page"
      layout="prev, pager, next, total"
      :total="groups.length"
      :page-size="PAGE_SIZE"
      style="margin-top: 10px; justify-content: flex-end"
    />
    <template #footer>
      <el-button @click="visible = false">稍后处理</el-button>
      <el-button type="primary" @click="confirmAll">确认并沉淀为规则</el-button>
    </template>
  </el-dialog>
</template>

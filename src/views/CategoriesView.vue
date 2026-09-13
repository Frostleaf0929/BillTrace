<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus, Delete, Edit, Coin, UploadFilled, Share } from '@element-plus/icons-vue';
import { api, fmtAmount } from '../api';
import type { Category } from '../types';
import PresetImportDialog from '../components/PresetImportDialog.vue';
import AccountBaseDialog from '../components/AccountBaseDialog.vue';
import PageSub from '../components/PageSub.vue';

interface CatNode extends Category {
  children: CatNode[];
  level: number; // 1/2/3
  sum?: number;      // 支出/收入：本月合计
  balance?: number;  // 账户：余额
}

const kind = ref<'expense' | 'income' | 'account'>('expense');
const all = ref<Category[]>([]);
const sums = ref<Record<string, number>>({});
const balances = ref<Record<string, number>>({});
const showSums = ref(false);
const presetVisible = ref(false);
const baseVisible = ref(false);
const baseAccount = ref<{ name: string; base: number } | null>(null);

const kindLabel: Record<string, string> = { expense: '支出', income: '收入', account: '账户' };
const route = useRoute();
const now = new Date();
const monthPrefix = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;

async function load() {
  all.value = await api.listCategories();
  if (showSums.value) {
    if (kind.value !== 'account') {
      sums.value = await api.categorySums(kind.value, monthPrefix);
    } else {
      const list = await api.accountBalances();
      const m: Record<string, number> = {};
      for (const a of list) m[a.name] = a.balance;
      balances.value = m;
    }
  }
}

async function loadSumsSetting() {
  try {
    showSums.value = (await api.getSetting('categoryShowSums')) === 'true';
  } catch { /* 默认关 */ }
  // 支持从其他页面带参数直达（如概览「账户分类」按钮）
  const qk = route.query.kind;
  if (qk === 'account' || qk === 'expense' || qk === 'income') {
    kind.value = qk;
  }
  await load();
}

onMounted(loadSumsSetting);

async function toggleSums(v: boolean | string | number) {
  showSums.value = !!v;
  try { await api.setSetting('categoryShowSums', String(showSums.value)); } catch { /* 忽略 */ }
  await load();
}

// 递归构建树，深度 ≤ 3
function buildTree(nodes: Category[], parent: number | null, level: number): CatNode[] {
  return nodes
    .filter((c) => c.parent_id === parent)
    .sort((a, b) => a.sort - b.sort || a.id - b.id)
    .map((c) => {
      const children = level < 3 ? buildTree(nodes, c.id, level + 1) : [];
      return {
        ...c,
        children,
        level,
        sum: level === 1 && kind.value !== 'account' ? sums.value[c.name] : undefined,
        balance: kind.value === 'account' && (c.parent_id !== null || children.length === 0)
          ? balances.value[c.name] ?? 0
          : undefined,
      };
    });
}

const tree = computed<CatNode[]>(() => buildTree(all.value.filter((c) => c.kind === kind.value), null, 1));

function nameClass(level: number): string {
  return level === 1 ? 'cat-name-l1' : level === 2 ? 'cat-name-l2' : 'cat-name-l3';
}

async function addChild(node: CatNode) {
  const label = node.level === 1 ? '二级' : '三级（小级）';
  const { value } = await ElMessageBox.prompt(`在「${node.name}」下新增${label}分类`, '添加分类', {
    inputPattern: /\S+/,
    inputErrorMessage: '名称不能为空',
  });
  try {
    await api.saveCategory({ kind: node.kind, parent_id: node.id, name: value.trim() });
    ElMessage.success('已添加');
  } catch (e) {
    ElMessage.error(String(e));
  }
  load();
}

async function addRoot() {
  const { value } = await ElMessageBox.prompt(`新增${kindLabel[kind.value]}一级分类`, '添加分类', {
    inputPattern: /\S+/,
    inputErrorMessage: '名称不能为空',
  });
  await api.saveCategory({ kind: kind.value, parent_id: null, name: value.trim() });
  ElMessage.success('已添加');
  load();
}

async function rename(node: CatNode) {
  const { value } = await ElMessageBox.prompt('重命名', '重命名', { inputValue: node.name, inputPattern: /\S+/ });
  await api.saveCategory({ id: node.id, kind: node.kind, parent_id: node.parent_id, name: value.trim() });
  load();
}

async function remove(node: CatNode) {
  await ElMessageBox.confirm(`确定删除「${node.name}」？其下子分类需先处理。`, '删除分类', { type: 'warning' });
  try {
    await api.deleteCategory(node.id);
    ElMessage.success('已删除');
    load();
  } catch (e) {
    ElMessage.error(String(e));
  }
}

function editBalance(node: CatNode) {
  baseAccount.value = { name: node.name, base: balances.value[node.name] ?? 0 };
  baseVisible.value = true;
}

async function exportPreset(format: 'md' | 'txt') {
  try {
    await api.exportCategories(format);
    ElMessage.success(`分类体系已导出为 ${format.toUpperCase()}`);
  } catch (e) {
    if (!String(e).includes('取消')) ElMessage.error(String(e));
  }
}

async function onDrop(dragging: any, drop: any, position: 'before' | 'after' | 'inner') {
  const dragged = all.value.find((c) => c.id === dragging.data?.id);
  if (!dragged) return;
  let parentId: number | null = null;
  let sort = 0;
  if (position === 'inner') {
    parentId = drop.data.id;
    sort = drop.data.children?.length ?? 0;
  } else {
    parentId = drop.data.parent_id;
    sort = drop.data.sort + (position === 'after' ? 1 : -1);
  }
  try {
    await api.moveCategory(dragged.id, parentId, Math.max(0, sort));
    ElMessage.success('已移动');
  } catch (e) {
    ElMessage.error(String(e));
  }
  load();
}
</script>

<template>
  <div>
    <h1 class="zj-page-title">分类</h1>
    <PageSub page="categories" fallback="支出 / 收入 / 账户三级体系，拖拽移动层级，点击行可展开收起" />

    <div class="zj-toolbar" style="justify-content: space-between; margin-bottom: 16px">
      <el-radio-group v-model="kind" @change="load">
        <el-radio-button value="expense">支出分类</el-radio-button>
        <el-radio-button value="income">收入分类</el-radio-button>
        <el-radio-button value="account">账户分类</el-radio-button>
      </el-radio-group>
      <div class="zj-toolbar" style="gap: 12px">
        <div class="sum-switch" :title="kind === 'account' ? '显示/隐藏账户余额' : '显示/隐藏本月合计'">
          <el-icon style="margin-right: 6px"><Coin /></el-icon>
          <span>{{ kind === 'account' ? '显示余额' : '显示总额' }}</span>
          <el-switch :model-value="showSums" size="small" style="margin-left: 8px" @change="toggleSums" />
        </div>
        <el-dropdown @command="(c: any) => (c === 'import' ? (presetVisible = true) : exportPreset(c))">
          <el-button type="primary" plain>
            <el-icon style="margin-right: 6px"><UploadFilled /></el-icon> 导入/出预设
            <el-icon style="margin-left: 6px"><Share style="transform: rotate(90deg)" /></el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="import">从 md / txt 导入…</el-dropdown-item>
              <el-dropdown-item command="md" divided>导出为 Markdown</el-dropdown-item>
              <el-dropdown-item command="txt">导出为 txt</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <el-button type="primary" @click="addRoot">
          <el-icon style="margin-right: 6px"><Plus /></el-icon> 新增一级
        </el-button>
      </div>
    </div>

    <div class="zj-card">
      <el-tree
        class="cat-tree"
        :data="tree"
        node-key="id"
        draggable
        default-expand-all
        @node-drop="(d: any, nd: any, pos: any) => onDrop(d, nd, pos)"
      >
        <template #default="{ data, node }">
          <div class="cat-row">
            <span :class="nameClass(data.level ?? node.level)">{{ data.name }}</span>

            <span v-if="showSums && data.sum !== undefined" class="cat-sum">
              本月 <span class="zj-amount-expense zj-num">{{ fmtAmount(data.sum) }}</span>
            </span>
            <span v-else-if="showSums && data.balance !== undefined" class="cat-sum">
              余额
              <span class="zj-num" :style="{ color: data.balance < 0 ? 'var(--zj-expense)' : 'var(--zj-text)' }">
                ¥ {{ fmtAmount(data.balance) }}
              </span>
            </span>

            <span class="cat-actions" @click.stop>
              <el-button v-if="data.level < 3" text size="small" type="primary" :title="data.level === 1 ? '添加二级' : '添加小级'" @click.stop="addChild(data)">
                <el-icon><Plus /></el-icon>
              </el-button>
              <el-button v-if="kind === 'account' && data.parent_id !== null" text size="small" title="设定基数" @click.stop="editBalance(data)">
                <el-icon><Coin /></el-icon>
              </el-button>
              <el-button text size="small" title="重命名" @click.stop="rename(data)"><el-icon><Edit /></el-icon></el-button>
              <el-button text size="small" type="danger" title="删除" @click.stop="remove(data)"><el-icon><Delete /></el-icon></el-button>
            </span>
          </div>
        </template>
      </el-tree>
    </div>

    <PresetImportDialog v-model:visible="presetVisible" @applied="load" />
    <AccountBaseDialog v-model:visible="baseVisible" :account="baseAccount" @saved="load" />
  </div>
</template>

<style scoped>
.cat-row {
  display: flex;
  align-items: center;
  width: 100%;
  padding-right: 8px;
}

.sum-switch {
  display: inline-flex;
  align-items: center;
  height: 32px;
  padding: 0 12px;
  background: var(--zj-card);
  border: 1px solid var(--zj-border);
  border-radius: 8px;
  color: var(--zj-text-sub);
  font-size: 13px;
  box-sizing: border-box;
}
</style>

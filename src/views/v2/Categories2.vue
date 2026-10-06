<script setup lang="ts">
// 分类 v2：三级树 + 预设导入导出
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { ElMessageBox } from 'element-plus';
import { api } from '../../api';
import type { Category } from '../../types';
import { icon, catColor, catIcon } from '../../v2/icons';
import PresetImportDialog from '../../components/PresetImportDialog.vue';
import { toast } from '../../v2/toast';

type Kind = 'expense' | 'income' | 'account';
const kind = ref<Kind>('expense');
const all = ref<Category[]>([]);
const sums = ref<Record<string, number>>({});
const openIds = ref<Set<number>>(new Set());
const presetVisible = ref(false);

const KINDS: [Kind, string][] = [['expense', '支出'], ['income', '收入'], ['account', '账户']];

interface Node {
  cat: Category;
  children: Node[];
  count: number;
}

const tree = computed<Node[]>(() => {
  const list = all.value.filter((c) => c.kind === kind.value);
  const roots = list.filter((c) => c.parent_id === null);
  const build = (cat: Category): Node => {
    const children = list.filter((c) => c.parent_id === cat.id).map(build);
    const count = (sums.value[cat.name] || 0) + children.reduce((s, ch) => s + ch.count, 0);
    return { cat, children, count };
  };
  return roots.map(build);
});

function toggle(id: number): void {
  const s = new Set(openIds.value);
  if (s.has(id)) s.delete(id);
  else s.add(id);
  openIds.value = s;
}

async function load(): Promise<void> {
  try {
    all.value = await api.listCategories();
    if (kind.value === 'expense') {
      const n = new Date();
      const key = `${n.getFullYear()}-${String(n.getMonth() + 1).padStart(2, '0')}`;
      sums.value = await api.categorySums('expense', key);
    } else {
      sums.value = {};
    }
  } catch (e) {
    toast(`分类加载失败：${e}`);
  }
}

async function add(parent: Node | null): Promise<void> {
  try {
    const res = await ElMessageBox.prompt(
      parent ? `在「${parent.cat.name}」下新增${kind.value === 'account' ? '账户' : '子分类'}` : `新增一级${kind.value === 'account' ? '账户' : '分类'}`,
      '新增',
      { inputPattern: /\S+/, inputErrorMessage: '名称不能为空' },
    ).catch(() => null);
    if (res === null) return;
    await api.saveCategory({ kind: kind.value, parent_id: parent ? parent.cat.id : null, name: res.value.trim() });
    if (parent) openIds.value = new Set(openIds.value).add(parent.cat.id);
    toast('已添加');
    await load();
  } catch (e) {
    toast(`添加失败：${e}`);
  }
}

async function rename(node: Node): Promise<void> {
  try {
    const res = await ElMessageBox.prompt('修改名称', '重命名', { inputValue: node.cat.name, inputPattern: /\S+/ }).catch(() => null);
    if (res === null) return;
    await api.saveCategory({ id: node.cat.id, kind: node.cat.kind, parent_id: node.cat.parent_id, name: res.value.trim() });
    toast('已重命名');
    await load();
  } catch (e) {
    toast(`重命名失败：${e}`);
  }
}

async function remove(node: Node): Promise<void> {
  try {
    await ElMessageBox.confirm(
      `删除「${node.cat.name}」？${node.children.length ? `其下 ${node.children.length} 个子级也会一并处理（子级上移）。` : ''}已有账单的分类名保留在账单里。`,
      '删除分类',
      { type: 'warning' },
    );
  } catch { return; }
  try {
    await api.deleteCategory(node.cat.id);
    toast('已删除');
    await load();
  } catch (e) {
    toast(`删除失败：${e}`);
  }
}

async function exportPreset(format: string): Promise<void> {
  try {
    await api.exportCategories(format);
    toast(`已导出（${format.toUpperCase()}）——文件在数据目录`);
  } catch (e) {
    toast(`导出失败：${e}`);
  }
}

onMounted(load);
onMounted(() => {
  const onRefresh = () => load();
  window.addEventListener('v2-refresh', onRefresh);
  onUnmounted(() => window.removeEventListener('v2-refresh', onRefresh));
});
</script>

<template>
  <div>
    <div class="v2-pagehead">
      <div>
        <h1>分类</h1>
        <div class="sub">三级分类体系：一级 → 二级 → 小级 · 本月使用次数随卡片显示</div>
      </div>
      <div class="v2-head-right">
        <div class="v2-seg">
          <button v-for="[k, lbl] in KINDS" :key="k" :class="{ on: kind === k }" @click="kind = k; load()">{{ lbl }}</button>
        </div>
        <button class="v2-btn primary" @click="add(null)">＋ 新增一级</button>
      </div>
    </div>

    <div class="v2-grid">
      <div class="v2-card span8">
        <div v-for="node in tree" :key="node.cat.id">
          <div class="v2-tree-row" @click="toggle(node.cat.id)">
            <span class="tw" :class="{ open: openIds.has(node.cat.id) && node.children.length }" v-html="icon('chevR', 14)" />
            <div class="v2-dot" style="width:32px;height:32px;border-radius:10px;background-color:transparent" :style="{ color: catColor(node.cat.name) }" v-html="catIcon(node.cat.name, 17)" />
            <span class="t-name">{{ node.cat.name }}</span>
            <span class="v2-lvl-tag">一级</span>
            <span class="t-count">{{ node.count }} 笔</span>
            <div class="t-actions" @click.stop>
              <button class="v2-icon-btn" title="添加子级" @click="add(node)">＋</button>
              <button class="v2-icon-btn" title="重命名" @click="rename(node)" v-html="icon('edit', 14)" />
              <button class="v2-icon-btn" title="删除" @click="remove(node)" v-html="icon('trash', 14)" />
            </div>
          </div>
          <template v-if="openIds.has(node.cat.id)">
            <div v-for="ch in node.children" :key="ch.cat.id">
              <div class="v2-tree-row" style="padding-left:46px" @click="toggle(ch.cat.id)">
                <span class="tw" :class="{ open: openIds.has(ch.cat.id) && ch.children.length }" v-html="icon('chevR', 14)" />
                <span style="width:6px;height:6px;border-radius:99px" :style="{ background: catColor(node.cat.name) }" />
                <span class="t-name" style="font-weight:500">{{ ch.cat.name }}</span>
                <span class="v2-lvl-tag">二级</span>
                <span class="t-count">{{ ch.count }} 笔</span>
                <div class="t-actions" @click.stop>
                  <button class="v2-icon-btn" title="添加小级" @click="add(ch)">＋</button>
                  <button class="v2-icon-btn" title="重命名" @click="rename(ch)" v-html="icon('edit', 14)" />
                  <button class="v2-icon-btn" title="删除" @click="remove(ch)" v-html="icon('trash', 14)" />
                </div>
              </div>
              <div v-for="gch in ch.children" :key="gch.cat.id">
                <div class="v2-tree-row" style="padding-left:82px">
                  <span style="width:5px;height:5px;border-radius:99px;background:var(--v2-ink-3);opacity:.5" />
                  <span class="t-name" style="font-weight:500;font-size:13px">{{ gch.cat.name }}</span>
                  <span class="v2-lvl-tag">小级</span>
                  <span class="t-count">{{ gch.count }} 笔</span>
                  <div class="t-actions" @click.stop>
                    <button class="v2-icon-btn" title="重命名" @click="rename(gch)" v-html="icon('edit', 14)" />
                    <button class="v2-icon-btn" title="删除" @click="remove(gch)" v-html="icon('trash', 14)" />
                  </div>
                </div>
              </div>
            </div>
          </template>
        </div>
        <div v-if="!tree.length" style="padding:30px;text-align:center;color:var(--v2-ink-3);font-size:13px">
          该类型下暂无分类——点右上角「新增一级」创建
        </div>
      </div>

      <div class="v2-card span4">
        <div class="v2-card-head"><h3>预设与导入</h3></div>
        <p style="color:var(--v2-ink-2);font-size:13px;line-height:1.7">
          支持 md / txt 分类预设：先预览、确认后写入三级结构；导出同样生成可回导的预设文件。
        </p>
        <div style="display:flex;gap:10px;margin-top:16px;flex-wrap:wrap">
          <button class="v2-btn primary" @click="presetVisible = true">导入预设</button>
          <button class="v2-btn ghost" @click="exportPreset('md')">导出 md</button>
          <button class="v2-btn ghost" @click="exportPreset('txt')">导出 txt</button>
        </div>
        <div style="margin-top:20px;padding:14px;background:var(--v2-surface-2);border-radius:14px;font-size:12.5px;color:var(--v2-ink-2);line-height:1.8">
          <b style="color:var(--v2-ink)">拖拽移动</b><br>
          树行拖拽改层级 / 排序已在旧版实现，本页下一批接入。
        </div>
      </div>
    </div>

    <PresetImportDialog v-model:visible="presetVisible" @applied="load" />
  </div>
</template>

<script lang="ts">
export default { name: 'Categories2' };
</script>

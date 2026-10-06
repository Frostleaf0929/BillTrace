<script setup lang="ts">
// 分类 v2：三级树 + 预设导入导出
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { ElMessageBox } from 'element-plus';
import { api } from '../../api';
import type { Category } from '../../types';
import { icon, catColor, catIcon } from '../../v2/icons';
import PresetImportDialog from '../../components/PresetImportDialog.vue';
import type { Rule } from '../../types';
import { toast } from '../../v2/toast';

type Kind = 'expense' | 'income' | 'account';
const kind = ref<Kind>('expense');
const all = ref<Category[]>([]);
const sums = ref<Record<string, number>>({});
const openIds = ref<Set<number>>(new Set());
const presetVisible = ref(false);
const rulesList = ref<Rule[]>([]);

async function loadRules(): Promise<void> {
  try { rulesList.value = await api.listRules(); } catch { rulesList.value = []; }
}
async function toggleRule(r: Rule): Promise<void> {
  try {
    await api.saveRule({ id: r.id, keyword: r.keyword, kind: r.kind, l1: r.l1, l2: r.l2, priority: r.priority, enabled: !r.enabled });
    await loadRules();
  } catch (e) {
    toast(`规则更新失败：${e}`);
  }
}
async function editRule(r: Rule): Promise<void> {
  try {
    const res = await ElMessageBox.prompt('修改规则映射（格式：关键词 | 一级分类 | 二级分类可选）', '编辑规则', {
      inputValue: `${r.keyword} | ${r.l1} | ${r.l2 ?? ''}`,
    }).catch(() => null);
    if (!res) return;
    const [kw, l1, l2] = res.value.split('|').map((x) => x.trim());
    if (!kw || !l1) { toast('格式不对：至少需要 关键词 和 一级分类'); return; }
    await api.saveRule({ id: r.id, keyword: kw, kind: r.kind, l1, l2: l2 || null, priority: r.priority, enabled: r.enabled });
    await loadRules();
    toast('规则已更新');
  } catch (e) {
    toast(`规则更新失败：${e}`);
  }
}
async function delRule(r: Rule): Promise<void> {
  try {
    await ElMessageBox.confirm(`删除规则「${r.keyword} → ${r.l1}」？`, '删除规则', { type: 'warning' });
  } catch { return; }
  try {
    await api.deleteRule(r.id);
    await loadRules();
    toast('规则已删除');
  } catch (e) {
    toast(`删除失败：${e}`);
  }
}

const KINDS: [Kind, string][] = [['expense', '支出'], ['income', '收入'], ['account', '账户']];

interface Node {
  cat: Category;
  children: Node[];
  count: number;
  depth: number;
}

const tree = computed<Node[]>(() => {
  const list = all.value.filter((c) => c.kind === kind.value);
  const roots = list.filter((c) => c.parent_id === null);
  const build = (cat: Category, depth: number): Node => {
    const children = list.filter((c) => c.parent_id === cat.id).map((c) => build(c, depth + 1));
    const count = (sums.value[cat.name] || 0) + children.reduce((s, ch) => s + ch.count, 0);
    return { cat, children, count, depth };
  };
  return roots.map((r) => build(r, 0));
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

// 把已启用的规则应用到「先导入的账单」：待确认队列按商家归类，已入库未分类的补分类
const applying = ref(false);
async function applyRules(): Promise<void> {
  if (applying.value) return;
  applying.value = true;
  try {
    const rules = (await api.listRules()).filter((r) => r.enabled);
    if (!rules.length) { toast('还没有已启用的规则——先导入规则预设'); return; }
    const matchRule = (m: string, remark: string, kind: string) =>
      rules.find((r) => r.kind === kind && (m.includes(r.keyword) || remark.includes(r.keyword)));
    const pending = await api.listPending();
    const assigns: { merchant: string; l1: string; l2: string | null }[] = [];
    const skip: string[] = [];
    const matched = new Map<string, { l1: string; l2: string | null }>();
    for (const p of pending) {
      const m = p.tx.merchant || '';
      if (matched.has(m)) continue;
      const rule = matchRule(m, p.tx.remark || '', p.tx.tx_type === '收入' ? 'income' : 'expense');
      if (rule) matched.set(m, { l1: rule.l1, l2: rule.l2 });
    }
    for (const p of pending) {
      const m = p.tx.merchant || '';
      const hit = matched.get(m);
      if (hit) { if (!assigns.some((a) => a.merchant === m)) assigns.push({ merchant: m, l1: hit.l1, l2: hit.l2 }); }
      else if (!skip.includes(m)) skip.push(m);
    }
    const [assigned] = await api.resolvePending(assigns, skip);
    const pendingTxIds = new Set(pending.map((p2) => p2.tx.id));
    const page = await api.queryTransactions({ page: 1, page_size: 99999 });
    const batches = new Map<string, number[]>();
    let unmatched = 0;
    for (const t of page.rows) {
      if (t.l1 || pendingTxIds.has(t.id)) continue;
      const m = t.merchant || '';
      const rule = matchRule(m, t.remark || '', t.tx_type === '收入' ? 'income' : 'expense');
      if (!rule) { unmatched++; continue; }
      const key = rule.l1 + '|' + (rule.l2 || '');
      const arr = batches.get(key) || [];
      arr.push(t.id);
      batches.set(key, arr);
    }
    let moved = 0;
    for (const [key, ids] of batches) {
      const [l1, l2] = key.split('|');
      await api.setTransactionsCategory(ids, l1, l2 || null, null);
      moved += ids.length;
    }
    toast(`规则应用完成：待确认归类 ${assigned} 个商家 · 历史账单补分类 ${moved} 笔${skip.length ? ` · ${skip.length} 个商家仍待确认` : ''}${unmatched ? ` · ${unmatched} 笔无匹配规则` : ''}`);
    await load();
  } catch (e) {
    toast(`规则应用失败：${e}`);
  } finally {
    applying.value = false;
  }
}

onMounted(() => { load(); loadRules(); });

// ---------- 拖拽移动（HTML5 DnD：上/内/下 三段判定） ----------
const dragId = ref<number | null>(null);
const dragOver = ref<{ id: number; pos: 'before' | 'after' | 'inner' } | null>(null);
function findNode(id: number): Node | null {
  const walk = (list: Node[]): Node | null => {
    for (const n of list) {
      if (n.cat.id === id) return n;
      const hit = walk(n.children);
      if (hit) return hit;
    }
    return null;
  };
  return walk(tree.value);
}
function descendants(node: Node): number[] {
  const out: number[] = [];
  const walk = (n: Node): void => { out.push(n.cat.id); n.children.forEach(walk); };
  walk(node);
  return out;
}
function onDragStart(e: DragEvent, node: Node): void {
  dragId.value = node.cat.id;
  e.dataTransfer?.setData('text/plain', String(node.cat.id));
}
function onDragOver(e: DragEvent, node: Node): void {
  if (dragId.value == null) return;
  const dragged = findNode(dragId.value);
  if (!dragged || descendants(dragged).includes(node.cat.id)) { dragOver.value = null; return; }
  const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const rel = (e.clientY - r.top) / r.height;
  const canInner = node.depth < 2 && node.cat.id !== dragged.cat.id;
  let pos: 'before' | 'after' | 'inner' = rel < 0.35 ? 'before' : rel > 0.65 ? 'after' : 'inner';
  if (pos === 'inner' && !canInner) pos = rel < 0.5 ? 'before' : 'after';
  dragOver.value = { id: node.cat.id, pos };
  e.preventDefault();
}
function onDrop(e: DragEvent, node: Node): void {
  e.preventDefault();
  const over = dragOver.value;
  dragOver.value = null;
  const id = dragId.value;
  dragId.value = null;
  if (!over || id == null) return;
  const dragged = findNode(id);
  if (!dragged || dragged.cat.id === node.cat.id) return;
  let parentId: number | null;
  let sort: number;
  if (over.pos === 'inner') { parentId = node.cat.id; sort = node.children.length; }
  else { parentId = node.cat.parent_id; sort = node.cat.sort + (over.pos === 'after' ? 1 : -1); }
  api.moveCategory(id, parentId, Math.max(0, sort))
    .then(() => { toast('已移动'); load(); })
    .catch((e2) => toast(`移动失败：${e2}`));
}
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
          <div class="v2-tree-row" :class="{ 'drag-over-before': dragOver?.id === node.cat.id && dragOver?.pos === 'before', 'drag-over-inner': dragOver?.id === node.cat.id && dragOver?.pos === 'inner', 'drag-over-after': dragOver?.id === node.cat.id && dragOver?.pos === 'after' }" draggable="true" @dragstart="onDragStart($event, node)" @dragover="onDragOver($event, node)" @drop="onDrop($event, node)" @dragend="dragOver = null; dragId = null" @click="toggle(node.cat.id)">
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
              <div class="v2-tree-row" style="padding-left:46px" :class="{ 'drag-over-before': dragOver?.id === ch.cat.id && dragOver?.pos === 'before', 'drag-over-inner': dragOver?.id === ch.cat.id && dragOver?.pos === 'inner', 'drag-over-after': dragOver?.id === ch.cat.id && dragOver?.pos === 'after' }" draggable="true" @dragstart="onDragStart($event, ch)" @dragover="onDragOver($event, ch)" @drop="onDrop($event, ch)" @dragend="dragOver = null; dragId = null" @click="toggle(ch.cat.id)">
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
                <div class="v2-tree-row" style="padding-left:82px" :class="{ 'drag-over-before': dragOver?.id === gch.cat.id && dragOver?.pos === 'before', 'drag-over-after': dragOver?.id === gch.cat.id && dragOver?.pos === 'after' }" draggable="true" @dragstart="onDragStart($event, gch)" @dragover="onDragOver($event, gch)" @drop="onDrop($event, gch)" @dragend="dragOver = null; dragId = null">
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
          <button class="v2-btn ghost" :disabled="applying" @click="applyRules">{{ applying ? '应用中…' : '把规则应用到已有账单' }}</button>
          <button class="v2-btn ghost" @click="exportPreset('md')">导出 md</button>
          <button class="v2-btn ghost" @click="exportPreset('txt')">导出 txt</button>
        </div>
        <div style="margin-top:20px;padding:14px;background:var(--v2-surface-2);border-radius:14px;font-size:12.5px;color:var(--v2-ink-2);line-height:1.8">
          <b style="color:var(--v2-ink)">先导账单后导规则？</b><br>
          点「把规则应用到已有账单」：待确认队列按规则自动归类，已入库的未分类账单批量补分类——顺序无关。<br><br>
          <b style="color:var(--v2-ink)">拖拽移动</b><br>
          树行拖拽改层级 / 排序已支持（拖到目标行的上 / 中 / 下分别表示排前面 / 变子级 / 排后面）。
        </div>
      </div>
      <div class="v2-card span4">
        <div class="v2-card-head">
          <h3>已学习规则 · {{ rulesList.length }}</h3>
          <div class="spacer" />
        </div>
        <div style="color:var(--v2-ink-3);font-size:12px;margin:-8px 0 10px">导入与待确认归类时自动沉淀 · 关用后不再自动分类 · 最多显示 30 条</div>
        <div style="max-height:330px;overflow-y:auto">
          <div v-for="r in rulesList.slice(0, 30)" :key="r.id" class="v2-bud-row">
            <div class="bud-main">
              <div class="v2-bud-top">
                <span class="v2-bud-name">{{ r.keyword }}</span>
                <span class="v2-pill" :class="r.source === 'learned' ? 'warnp' : 'grayp'">{{ r.source === 'learned' ? '学习' : '预设' }}</span>
                <span class="v2-bud-nums">{{ r.l1 }}{{ r.l2 ? ' · ' + r.l2 : '' }}</span>
              </div>
              <div style="font-size:11px;color:var(--v2-ink-3)">{{ r.kind === 'income' ? '收入' : '支出' }}规则 · 优先级 {{ r.priority }}</div>
            </div>
            <button class="v2-toggle" :class="{ on: r.enabled }" title="启用 / 停用" @click="toggleRule(r)" />
            <button class="v2-icon-btn" title="编辑" @click="editRule(r)" v-html="icon('edit', 14)" />
            <button class="v2-icon-btn" title="删除" @click="delRule(r)" v-html="icon('trash', 14)" />
          </div>
          <div v-if="!rulesList.length" style="padding:20px;text-align:center;color:var(--v2-ink-3);font-size:12.5px">
            还没有沉淀规则——待确认归类或「应用到已有账单」后会出现在这里
          </div>
        </div>
      </div>
    </div>

    <PresetImportDialog v-model:visible="presetVisible" @applied="load" />
  </div>
</template>

<script lang="ts">
export default { name: 'Categories2' };
</script>

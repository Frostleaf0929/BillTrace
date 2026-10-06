<script setup lang="ts">
// 统计 v2：分析汇报页（范围联动全页，Porcelain 纸卡）
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { ElMessage } from 'element-plus';
import { api } from '../../api';
import type { ChartPoint, Tx } from '../../types';
import { icon } from '../../v2/icons';
import { monthLabel } from '../../v2/parts';
import PaperCard from '../../v2/PaperCard.vue';
import {
  buildLedgerCard, buildDotHeatCard, buildDonutCard, buildWaterfallCard,
  buildIncomeCard, buildAlmanacCard, buildPatchCard, buildRowsCard,
  type Win, type WinMonth,
} from '../../v2/paper';
import { toast } from '../../v2/toast';
import { exportElementAsPng } from '../../v2/exportImage';

const all = ref<ChartPoint[]>([]);
const months = ref<{ y: number; m: number }[]>([]);
const txs = ref<Tx[]>([]);
const loading = ref(false);
const statRange = ref<[number, number]>([2, 13]);
const cards = ref({ ledger: '', heat: '', donut: '', fall: '', income: '', almanac: '', patch: '', rows: '' });

const TABS: { lbl: string; span: number; all?: boolean }[] = [
  { lbl: '近1月', span: 1 }, { lbl: '3月', span: 3 }, { lbl: '6月', span: 6 }, { lbl: '1年', span: 12 }, { lbl: '全部', span: 0, all: true },
];

const win = computed<Win | null>(() => {
  if (!all.value.length) return null;
  const [i0, i1] = statRange.value;
  const winM: WinMonth[] = all.value.slice(i0, i1 + 1).map((p, k) => {
    const idx = i0 + k;
    const meta = months.value[idx];
    return { y: meta?.y ?? new Date().getFullYear(), m: meta?.m ?? new Date().getMonth() + 1, label: p.label };
  });
  const first = winM[0], last = winM[winM.length - 1];
  if (!first || !last) return null;
  const fa = `${first.y}-${String(first.m).padStart(2, '0')}-01`;
  const fb = `${last.y}-${String(last.m).padStart(2, '0')}-${String(new Date(last.y, last.m, 0).getDate()).padStart(2, '0')}`;
  const days = Math.round((new Date(fb + 'T00:00:00').getTime() - new Date(fa + 'T00:00:00').getTime()) / 86400000) + 1;
  return { label: rangeLabel(i0, i1), months: winM, txs: txs.value, days, fa, fb };
});

function rangeLabel(i0: number, i1: number, isAll = false): string {
  const span = i1 - i0 + 1;
  if (span === 1) return '本月';
  if (isAll) return `全部 · ${span} 个月`;
  return `近 ${span} 个月`;
}
// 页签按序列实际长度动态钳位（序列不足 12 个月时"1年"自动等于"全部"）
function tabWindow(t: { span: number; all?: boolean }): [number, number] {
  const len = all.value.length;
  const i1 = len - 1;
  const i0 = t.all ? 0 : Math.max(0, i1 - (t.span - 1));
  return [i0, i1];
}
function isActive(t: { span: number; all?: boolean }): boolean {
  const [i0, i1] = tabWindow(t);
  return statRange.value[0] === i0 && statRange.value[1] === i1;
}
const exporting = ref(false);
async function exportPage(): Promise<void> {
  if (exporting.value) return;
  exporting.value = true;
  try {
    const el = document.querySelector('#page-stats > div') as HTMLElement;
    const d = new Date();
    const name = `账痕-统计-${d.getFullYear()}${String(d.getMonth() + 1).padStart(2, '0')}${String(d.getDate()).padStart(2, '0')}.png`;
    const path = await exportElementAsPng(el, name);
    toast(`长图已导出：${path}`);
  } catch (e) {
    toast(`导出失败：${e}`);
  } finally {
    exporting.value = false;
  }
}

function rebuild(): void {
  const w = win.value;
  if (!w) return;
  cards.value = {
    ledger: buildLedgerCard(w),
    heat: buildDotHeatCard(w),
    donut: buildDonutCard(w),
    fall: buildWaterfallCard(w),
    income: buildIncomeCard(w),
    almanac: buildAlmanacCard(w),
    patch: buildPatchCard(w),
    rows: buildRowsCard(w),
  };
}

async function load(): Promise<void> {
  loading.value = true;
  try {
    const now = new Date();
    const series = await api.statsChart('month', '2000-01-01', `${now.getFullYear()}-12-31`);
    // 后端只返回有数据的月份（GROUP BY 无补零），label = 'YYYY-MM'。
    // 客户端补零成连续月序列：首末数据月之间逐月填 0，年月直接从 label 解析（不再按位置推算）。
    const byKey = new Map(series.map((p) => [p.label, p]));
    const keysWithData = [...byKey.keys()]
      .filter((k) => { const p = byKey.get(k)!; return p.expense > 0 || p.income > 0; })
      .sort();
    if (!keysWithData.length) {
      all.value = [];
      months.value = [];
      txs.value = [];
      cards.value = { ledger: '', heat: '', donut: '', fall: '', income: '', almanac: '', patch: '', rows: '' };
      return;
    }
    const firstKey = keysWithData[0];
    const lastKey = keysWithData[keysWithData.length - 1];
    let y = Number(firstKey.slice(0, 4)), m = Number(firstKey.slice(5, 7));
    const endY = Number(lastKey.slice(0, 4)), endM = Number(lastKey.slice(5, 7));
    const filled: ChartPoint[] = [];
    const meta: { y: number; m: number }[] = [];
    while (y < endY || (y === endY && m <= endM)) {
      const key = `${y}-${String(m).padStart(2, '0')}`;
      const p = byKey.get(key);
      filled.push(p ?? { label: monthLabel(y, m), income: 0, expense: 0 });
      meta.push({ y, m });
      m++; if (m > 12) { m = 1; y++; }
    }
    all.value = filled;
    months.value = meta;
    const i1 = filled.length - 1;
    if (statRange.value[1] > i1 || statRange.value[0] > i1) statRange.value = [Math.max(0, i1 - 11), i1];
    await reloadTxs();
    rebuild();
  } catch (e) {
    ElMessage.error(String(e));
  } finally {
    loading.value = false;
  }
}

// 按当前范围重拉交易明细（切范围必须重拉，否则所有卡片仍是旧数据）
async function reloadTxs(): Promise<void> {
  const w = win.value;
  if (!w) { txs.value = []; return; }
  txs.value = (await api.queryTransactions({ date_from: w.fa, date_to: w.fb, page: 1, page_size: 99999 })).rows;
}

async function applyRange(t: { lbl: string; span: number; all?: boolean }): Promise<void> {
  const [i0, i1] = tabWindow(t);
  statRange.value = [i0, i1];
  loading.value = true;
  try {
    await reloadTxs();
    rebuild();
  } catch (e) {
    ElMessage.error(String(e));
  } finally {
    loading.value = false;
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
        <h1>统计</h1>
        <div class="sub">分析与图表汇报 · 所有卡片跟随右上角的时间范围</div>
      </div>
      <div class="v2-head-right">
        <div class="v2-range-tabs">
          <button
            v-for="t in TABS" :key="t.lbl" :class="{ on: isActive(t) }"
            @click="applyRange(t)"
          >{{ t.lbl }}</button>
        </div>
        <span v-if="win" class="v2-date-pill num">
          <span v-html="icon('cal', 13)" />{{ win.fa }} – {{ win.fb }}
        </span>
        <button class="v2-btn primary" style="height:40px" :disabled="exporting" @click="exportPage">
          <span v-html="icon('dl', 15)" />导出本页长图
        </button>
      </div>
    </div>

    <div class="v2-grid">
      <PaperCard id="card-ledger" span="span12" :html="cards.ledger" />
      <PaperCard span="span7" :html="cards.heat" />
      <PaperCard span="span5" :html="cards.donut" />
      <PaperCard span="span12" :html="cards.fall" />
      <PaperCard span="span12" :html="cards.income" />
      <PaperCard span="span12" :html="cards.almanac" />
      <PaperCard span="span7" :html="cards.patch" />
      <PaperCard span="span5" :html="cards.rows" />
    </div>
  </div>
</template>

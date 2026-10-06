<script setup lang="ts">
// 统计 v2：分析汇报页（范围联动全页，Porcelain 纸卡）
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { ElMessage } from 'element-plus';
import { api } from '../../api';
import type { ChartPoint, Tx } from '../../types';
import { icon } from '../../v2/icons';
import PaperCard from '../../v2/PaperCard.vue';
import {
  buildLedgerCard, buildDotHeatCard, buildDonutCard, buildWaterfallCard,
  buildIncomeCard, buildAlmanacCard, buildPatchCard, buildRowsCard,
  type Win, type WinMonth,
} from '../../v2/paper';
import { toast } from '../../v2/toast';

const all = ref<ChartPoint[]>([]);
const months = ref<{ y: number; m: number }[]>([]);
const txs = ref<Tx[]>([]);
const loading = ref(false);
const statRange = ref<[number, number]>([2, 13]);
const cards = ref({ ledger: '', heat: '', donut: '', fall: '', income: '', almanac: '', patch: '', rows: '' });

const TABS: [string, [number, number]][] = [
  ['近1月', [13, 13]], ['3月', [11, 13]], ['6月', [8, 13]], ['1年', [2, 13]], ['全部', [0, 13]],
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

function rangeLabel(i0: number, i1: number): string {
  const span = i1 - i0 + 1;
  if (span === 1) return '本月';
  if (i0 === 2 && i1 === 13) return '近1年';
  if (i0 === 11) return '近3月';
  if (i0 === 8) return '近6月';
  if (i0 === 0) return `全部 · ${span} 个月`;
  return `${span} 个月`;
}
function setRange(i0: number, i1: number): void {
  statRange.value = [i0, i1];
  rebuild();
}
function exportPage(): void {
  toast('原型演示：正式版将把整页渲染为一张长图 PNG 保存（含全部卡片）');
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
    // 只保留有数据的月份（从首条记录所在月开始）
    const firstIdx = series.findIndex((p) => p.expense > 0 || p.income > 0);
    const trimmed = firstIdx >= 0 ? series.slice(firstIdx) : series.slice(-14);
    all.value = trimmed;
    months.value = trimmed.map((_, k) => {
      const d = new Date(now.getFullYear(), now.getMonth() - (trimmed.length - 1 - k), 1);
      return { y: d.getFullYear(), m: d.getMonth() + 1 };
    });
    const i1 = trimmed.length - 1;
    if (statRange.value[1] > i1 || statRange.value[0] > i1) statRange.value = [Math.max(0, i1 - 11), i1];
    const [i0, i1r] = statRange.value;
    const m0 = months.value[i0], m1 = months.value[i1r];
    if (m0 && m1) {
      const fa = `${m0.y}-${String(m0.m).padStart(2, '0')}-01`;
      const fb = `${m1.y}-${String(m1.m).padStart(2, '0')}-31`;
      txs.value = (await api.queryTransactions({ date_from: fa, date_to: fb, page: 1, page_size: 99999 })).rows;
    }
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
            v-for="[lbl, r] in TABS" :key="lbl" :class="{ on: statRange[0] === r[0] && statRange[1] === r[1] }"
            @click="setRange(r[0], r[1])"
          >{{ lbl }}</button>
        </div>
        <span v-if="win" class="v2-date-pill num">
          <span v-html="icon('cal', 13)" />{{ win.fa }} – {{ win.fb }}
        </span>
        <button class="v2-btn primary" style="height:40px" @click="exportPage">
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

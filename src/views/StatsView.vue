<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { api, fmtAmount } from '../api';
import ChartCard from '../components/ChartCard.vue';
import PageSub from '../components/PageSub.vue';
import { accentLadder, chartBase, chartToken, SOFT_PALETTE, verticalFade } from '../lib/chartTheme';
import type { ChartPoint, PiePoint } from '../types';
import * as echarts from 'echarts';

// ── 第 1 层：筛选 ──
type Dim = 'year' | 'month' | 'day';
const dimension = ref<Dim>('month'); // 年 / 月 / 日：决定时间轴粒度，并联动默认时间区间
const pieType = ref<'支出' | '收入'>('支出');
const groupBy = ref('l1');
const chartKind = ref<'pie' | 'bar'>('pie');

const GROUP_OPTIONS = [
  { value: 'l1', label: '一级分类' },
  { value: 'l2', label: '二级分类' },
  { value: 'merchant', label: '商家' },
  { value: 'account', label: '账户' },
  { value: 'project', label: '项目分类' },
  { value: 'member', label: '成员' },
];
const groupLabel = computed(() => GROUP_OPTIONS.find((o) => o.value === groupBy.value)?.label ?? '一级分类');

const dateFrom = ref('');
const dateTo = ref('');
const barData = ref<ChartPoint[]>([]);
const prevBarData = ref<ChartPoint[]>([]); // 上期同期（幽灵柱）
const pieData = ref<PiePoint[]>([]);
const loadError = ref('');

function shiftDate(d: Date, days: number): string {
  const nd = new Date(d.getTime() + days * 86400000);
  return `${nd.getFullYear()}-${p(nd.getMonth() + 1)}-${p(nd.getDate())}`;
}

async function load() {
  if (!dateFrom.value || !dateTo.value) return;
  loadError.value = '';
  try {
    barData.value = await api.statsChart(dimension.value, dateFrom.value, dateTo.value);
    pieData.value = await api.statsPie(pieType.value, dateFrom.value, dateTo.value, groupBy.value);
    // 上期同期：区间整体向前平移一个等长周期（POS 幽灵柱对比）
    const from = new Date(dateFrom.value);
    const span = Math.max(1, Math.round((new Date(dateTo.value).getTime() - from.getTime()) / 86400000) + 1);
    prevBarData.value = await api.statsChart(dimension.value, shiftDate(from, -span), shiftDate(from, -1));
  } catch (e) {
    loadError.value = String(e);
    barData.value = [];
    prevBarData.value = [];
    pieData.value = [];
    ElMessage.error(`统计数据加载失败：${e}`);
  }
}

const trendRef = ref<InstanceType<typeof ChartCard>>();
const structRef = ref<InstanceType<typeof ChartCard>>();

function p(n: number) { return String(n).padStart(2, '0'); }

function setDefaultRange() {
  const now = new Date();
  if (dimension.value === 'day') {
    const start = new Date(now);
    start.setDate(now.getDate() - 29);
    dateFrom.value = `${start.getFullYear()}-${p(start.getMonth() + 1)}-${p(start.getDate())}`;
    dateTo.value = `${now.getFullYear()}-${p(now.getMonth() + 1)}-${p(now.getDate())}`;
  } else if (dimension.value === 'month') {
    dateFrom.value = `${now.getFullYear()}-01-01`;
    dateTo.value = `${now.getFullYear()}-${p(now.getMonth() + 1)}-${new Date(now.getFullYear(), now.getMonth() + 1, 0).getDate()}`;
  } else {
    dateFrom.value = `${now.getFullYear() - 4}-01-01`;
    dateTo.value = `${now.getFullYear()}-12-31`;
  }
}

function onDimensionChange() {
  setDefaultRange();
}

onMounted(() => {
  setDefaultRange();
  load();
});

watch([dimension, dateFrom, dateTo, groupBy, pieType], load);

// ── 第 2 层：收支趋势柱状图（POS 范式：上期斜纹幽灵柱 + 峰值炭黑高亮，图例自绘）──
const trendOption = computed<echarts.EChartsOption>(() => {
  const peakIdx = barData.value.reduce((mi, d, i, arr) => (d.expense > arr[mi].expense ? i : mi), 0);
  // 幽灵柱与当期柱按索引对齐；上期条数不足时补零
  const ghost = barData.value.map((_, i) => Math.round((prevBarData.value[i]?.expense ?? 0) * 100) / 100);
  return {
    ...chartBase(),
    grid: { left: 4, right: 12, top: 14, bottom: barData.value.length > 40 ? 56 : 0, containLabel: true },
    dataZoom: [
      ...(barData.value.length > 40 ? [{ type: 'slider', height: 16, bottom: 6, borderColor: 'transparent' }] : []),
      { type: 'inside' }, // 滚轮缩放（FusionTime 范式）
    ],
    tooltip: {
      ...chartBase().tooltip,
      trigger: 'axis',
      valueFormatter: (v) => `¥ ${fmtAmount(Number(v))}`,
    },
    xAxis: {
      ...chartBase().xAxis,
      type: 'category',
      data: barData.value.map((d) => d.label),
      axisLabel: {
        color: chartToken('--zj-text-sub', '#71768a'),
        fontSize: 11,
        rotate: barData.value.length > 12 ? 40 : 0,
      },
    },
    yAxis: {
      ...chartBase().yAxis,
      type: 'value',
      axisLabel: {
        color: chartToken('--zj-text-sub', '#71768a'),
        fontSize: 11,
        formatter: (v: number) => (Math.abs(v) >= 10000 ? `${(v / 10000).toFixed(1)}万` : String(v)),
      },
    },
    series: [
      {
        name: '上期支出', type: 'bar', barWidth: '62%', barMaxWidth: 26, barGap: '-100%', barMinHeight: 1, z: 1,
        itemStyle: {
          color: 'rgba(135,127,193,0.22)',
          borderRadius: [5, 5, 0, 0],
          decal: { symbol: 'rect', dashArrayX: [2, 0], dashArrayY: [2, 4], rotation: Math.PI / 6 },
        },
        data: ghost,
      },
      {
        name: '支出', type: 'bar', barWidth: '62%', barMaxWidth: 26, barMinHeight: 1, z: 2,
        itemStyle: { color: verticalFade(chartToken('--zj-expense', '#d4849b')), borderRadius: [5, 5, 0, 0] },
        data: barData.value.map((d, i) => ({
          value: Math.round(d.expense * 100) / 100,
          itemStyle: i === peakIdx && barData.value.length > 1
            ? { color: '#222026', borderRadius: [5, 5, 0, 0] }
            : undefined,
        })),
      },
      {
        name: '收入', type: 'bar', barWidth: '62%', barMaxWidth: 26, barMinHeight: 1, z: 2,
        itemStyle: { color: verticalFade(chartToken('--zj-income', '#6fae9c')), borderRadius: [5, 5, 0, 0] },
        data: barData.value.map((d) => Math.round(d.income * 100) / 100),
      },
    ],
  } as echarts.EChartsOption;
});

// ── 第 3 层：结构图（按维度）；排行条形用强调色阶梯，饼图用柔和色板，图例自绘 ──
const legendItems = computed(() =>
  pieData.value.slice(0, 12).map((d, i) => ({
    name: d.name,
    value: d.value,
    color: chartKind.value === 'pie' ? SOFT_PALETTE[i % SOFT_PALETTE.length] : accentLadder(i),
  }))
);

const structOption = computed<echarts.EChartsOption>(() => {
  const data = pieData.value.map((d) => ({ name: d.name, value: Math.round(d.value * 100) / 100 }));
  if (chartKind.value === 'pie') {
    return {
      ...chartBase(),
      color: SOFT_PALETTE,
      tooltip: {
        ...chartBase().tooltip,
        trigger: 'item',
        formatter: '{b}<br/>¥{c}（{d}%）',
      },
      series: [
        {
          name: pieType.value,
          type: 'pie',
          radius: ['50%', '76%'],
          center: ['50%', '50%'],
          itemStyle: {
            borderRadius: 8,
            borderColor: chartToken('--zj-card-solid', '#ffffff'),
            borderWidth: 2,
          },
          label: {
            show: true,
            position: 'inside',
            formatter: '{d}%',
            color: '#fff',
            fontSize: 11,
            fontWeight: 600,
          },
          labelLayout: { hideOverlap: true },
          data,
        },
      ],
    } as echarts.EChartsOption;
  }
  const top = data.slice(0, 12).reverse();
  const rank = (name: string) => pieData.value.findIndex((d) => d.name === name);
  return {
    ...chartBase(),
    tooltip: { ...chartBase().tooltip, trigger: 'item', valueFormatter: (v) => `¥ ${fmtAmount(Number(v))}` },
    grid: { left: 4, right: 30, top: 6, bottom: 0, containLabel: true },
    xAxis: {
      ...chartBase().xAxis,
      type: 'value',
      axisLabel: {
        color: chartToken('--zj-text-sub', '#575d6c'),
        fontSize: 11,
        formatter: (v: number) => (Math.abs(v) >= 10000 ? `${(v / 10000).toFixed(1)}万` : String(v)),
      },
      splitLine: { lineStyle: { color: chartToken('--zj-border', 'rgba(34,37,46,0.1)') } },
    },
    yAxis: {
      type: 'category',
      data: top.map((d) => d.name),
      axisLabel: { color: chartToken('--zj-text', '#21242d'), fontSize: 12, width: 108, overflow: 'truncate' },
    },
    series: [
      {
        name: pieType.value,
        type: 'bar',
        barWidth: '62%',
        itemStyle: { borderRadius: [0, 5, 5, 0] },
        data: top.map((d) => ({ value: d.value, itemStyle: { color: accentLadder(rank(d.name)) } })),
      },
    ],
  } as echarts.EChartsOption;
});

async function exportBoth() {
  await trendRef.value?.exportPng();
  await structRef.value?.exportPng();
}
</script>

<template>
  <div>
    <h1 class="zj-page-title">统计</h1>
    <PageSub page="stats" fallback="时间趋势看节奏，结构图按分类 / 商家 / 账户等维度自由切分" />

    <!-- 第 1 层：筛选 -->
    <div class="zj-card" style="margin-bottom: 14px">
      <div class="zj-toolbar" style="row-gap: 12px">
        <el-radio-group v-model="dimension" @change="onDimensionChange">
          <el-radio-button value="year">年</el-radio-button>
          <el-radio-button value="month">月</el-radio-button>
          <el-radio-button value="day">日</el-radio-button>
        </el-radio-group>

        <el-date-picker v-model="dateFrom" type="date" value-format="YYYY-MM-DD" placeholder="开始" style="width: 140px" :clearable="false" />
        <span style="color: var(--zj-text-sub)">至</span>
        <el-date-picker v-model="dateTo" type="date" value-format="YYYY-MM-DD" placeholder="结束" style="width: 140px" :clearable="false" />

        <el-divider direction="vertical" />

        <el-radio-group v-model="pieType">
          <el-radio-button value="支出">支出</el-radio-button>
          <el-radio-button value="收入">收入</el-radio-button>
        </el-radio-group>

        <span style="color: var(--zj-text-sub); font-size: 13px">按</span>
        <el-select v-model="groupBy" style="width: 120px">
          <el-option v-for="o in GROUP_OPTIONS" :key="o.value" :label="o.label" :value="o.value" />
        </el-select>

        <el-radio-group v-model="chartKind">
          <el-radio-button value="pie">饼图</el-radio-button>
          <el-radio-button value="bar">条形</el-radio-button>
        </el-radio-group>

        <div style="flex: 1" />
        <el-button text type="primary" @click="exportBoth">导出图表 PNG</el-button>
      </div>
    </div>

    <!-- 第 2 层：收支趋势（柱状图） -->
    <div class="zj-card" style="margin-bottom: 14px">
      <div style="display: flex; align-items: baseline; justify-content: space-between; gap: 12px; margin-bottom: 6px">
        <span style="font-weight: 600">收支趋势 · {{ { year: '按年', month: '按月', day: '按日' }[dimension] }}（{{ dateFrom }} ~ {{ dateTo }}）</span>
        <span class="chart-legend">
          <span class="chart-legend-item"><i class="dot" style="background: rgba(135,127,193,0.4)" />上期支出</span>
          <span class="chart-legend-item"><i class="dot" style="background: var(--zj-expense)" />支出</span>
          <span class="chart-legend-item"><i class="dot" style="background: var(--zj-income)" />收入</span>
        </span>
      </div>
      <template v-if="barData.length > 0">
        <ChartCard ref="trendRef" name="收支趋势" :option="trendOption" height="340px" />
      </template>
      <div v-else class="chart-empty">
        {{ loadError ? '数据加载失败，请检查后重试' : '所选区间内暂无收支数据' }}
      </div>
    </div>

    <!-- 第 3 层：结构（饼图 / 条形） -->
    <div class="zj-card">
      <div style="display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 6px">
        <span style="font-weight: 600">{{ pieType }} · 按{{ groupLabel }}{{ chartKind === 'pie' ? '占比' : '排行' }}</span>
        <span style="color: var(--zj-text-sub); font-size: 12px" class="zj-num">
          合计
          <span :class="pieType === '支出' ? 'zj-amount-expense' : 'zj-amount-income'">¥ {{ fmtAmount(pieData.reduce((s, d) => s + d.value, 0)) }}</span>
        </span>
      </div>
      <div v-if="legendItems.length" class="chart-legend" style="margin-bottom: 4px">
        <span v-for="it in legendItems" :key="it.name" class="chart-legend-item" :title="`${it.name}：¥ ${fmtAmount(it.value)}`">
          <i class="dot" :style="{ background: it.color }" />{{ it.name }}
        </span>
      </div>
      <template v-if="pieData.length > 0">
        <ChartCard ref="structRef" :name="`${pieType}结构`" :option="structOption" height="360px" />
      </template>
      <div v-else class="chart-empty">
        {{ loadError ? '数据加载失败，请检查后重试' : '所选区间内暂无该类型数据' }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.chart-empty {
  height: 300px;
  display: grid;
  place-items: center;
  color: var(--zj-text-sub);
  font-size: 13.5px;
  border: 1px dashed var(--zj-border);
  border-radius: 12px;
}

/* 自绘图例（SPEC §9.1）：8px 圆点 + 小号灰字，flex wrap */
.chart-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 14px;
}
.chart-legend-item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 11.5px;
  color: var(--zj-text-sub);
}
.chart-legend-item .dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex: none;
}
</style>

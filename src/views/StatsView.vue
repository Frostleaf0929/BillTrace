<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { Calendar } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import { api, fmtAmount } from '../api';
import ChartCard from '../components/ChartCard.vue';
import PageSub from '../components/PageSub.vue';
import { accentLadder, chartBase, chartToken, SOFT_PALETTE } from '../lib/chartTheme';
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
const trendData = ref<ChartPoint[]>([]);   // 趋势卡：按快捷区间独立加载
const trendFrom = ref('');
const trendTo = ref('');
const trendRange = ref<'30d' | '1y' | 'all'>('1y'); // 快捷区间页签
const trendType = ref<'line' | 'column'>('line');   // 线 / 柱形态（图一 / 图二）
const pieData = ref<PiePoint[]>([]);
const loadError = ref('');

const trendRef = ref<InstanceType<typeof ChartCard>>();
const structRef = ref<InstanceType<typeof ChartCard>>();

function p(n: number) { return String(n).padStart(2, '0'); }
function fdate(d: Date): string {
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

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

// 结构图（饼/条）数据：跟随工具栏的日期区间与维度
async function load() {
  if (!dateFrom.value || !dateTo.value) return;
  loadError.value = '';
  try {
    pieData.value = await api.statsPie(pieType.value, dateFrom.value, dateTo.value, groupBy.value);
  } catch (e) {
    pieData.value = [];
    ElMessage.error(`统计数据加载失败：${e}`);
  }
}

// 趋势卡：FusionTime 式时间序列——快捷区间决定窗口与粒度
async function loadTrend() {
  const now = new Date();
  let from: Date;
  let dim: string;
  if (trendRange.value === '30d') {
    from = new Date(now.getFullYear(), now.getMonth(), now.getDate() - 29);
    dim = 'day';
  } else if (trendRange.value === '1y') {
    from = new Date(now.getFullYear(), now.getMonth(), now.getDate() - 364);
    dim = 'month';
  } else {
    from = new Date(2000, 0, 1);
    dim = 'month';
  }
  trendFrom.value = fdate(from);
  trendTo.value = fdate(now);
  try {
    trendData.value = await api.statsChart(dim, trendFrom.value, trendTo.value);
  } catch (e) {
    trendData.value = [];
    ElMessage.error(`趋势数据加载失败：${e}`);
  }
}

onMounted(() => {
  setDefaultRange();
  load();
  loadTrend();
});

watch([dimension, dateFrom, dateTo, groupBy, pieType], load);
watch(trendRange, loadTrend);

// ── 第 2 层：收支趋势（FusionTime 时间序列：线=图一 / 柱=图二，底部导航器缩放）──
const trendLegend = computed(() =>
  trendType.value === 'line'
    ? [
        { name: '支出', color: chartToken('--zj-expense', '#d4849b') },
        { name: '收入', color: chartToken('--zj-income', '#6fae9c') },
        { name: '净结余', color: chartToken('--zj-primary', '#877FC1') },
      ]
    : [
        { name: '支出', color: chartToken('--zj-expense', '#d4849b') },
        { name: '收入', color: chartToken('--zj-income', '#6fae9c') },
      ]
);

const trendOption = computed<echarts.EChartsOption>(() => {
  const expense = trendData.value.map((d) => Math.round(d.expense * 100) / 100);
  const income = trendData.value.map((d) => Math.round(d.income * 100) / 100);
  const net = trendData.value.map((d) => Math.round((d.income - d.expense) * 100) / 100);
  const labels = trendData.value.map((d) => d.label);
  const lineSeries: echarts.SeriesOption[] = [
    {
      name: '支出', type: 'line', smooth: true, symbol: 'none', sampling: 'lttb', z: 3,
      lineStyle: { width: 2.5, color: chartToken('--zj-expense', '#d4849b') },
      itemStyle: { color: chartToken('--zj-expense', '#d4849b') },
      data: expense,
    },
    {
      name: '收入', type: 'line', smooth: true, symbol: 'none', sampling: 'lttb', z: 3,
      lineStyle: { width: 2.5, color: chartToken('--zj-income', '#6fae9c') },
      itemStyle: { color: chartToken('--zj-income', '#6fae9c') },
      data: income,
    },
    {
      name: '净结余', type: 'line', smooth: true, symbol: 'none', sampling: 'lttb', z: 2,
      lineStyle: { width: 2, type: 'dashed', color: chartToken('--zj-primary', '#877FC1') },
      itemStyle: { color: chartToken('--zj-primary', '#877FC1') },
      areaStyle: { color: 'rgba(135,127,193,0.08)' },
      data: net,
    },
  ];
  const columnSeries: echarts.SeriesOption[] = [
    {
      name: '支出', type: 'bar', barWidth: '62%', barMaxWidth: 26,
      itemStyle: { color: chartToken('--zj-expense', '#d4849b'), borderRadius: [5, 5, 0, 0] },
      data: expense,
    },
    {
      name: '收入', type: 'bar', barWidth: '62%', barMaxWidth: 26,
      itemStyle: { color: chartToken('--zj-income', '#6fae9c'), borderRadius: [5, 5, 0, 0] },
      data: income,
    },
  ];
  return {
    ...chartBase(),
    color: [chartToken('--zj-expense', '#d4849b'), chartToken('--zj-income', '#6fae9c'), chartToken('--zj-primary', '#877FC1')],
    grid: { left: 4, right: 12, top: 16, bottom: 104, containLabel: true },
    // 底部导航器（图一的标志结构）：数据阴影 mini 图 + 拖拽缩放 + 滚轮
    dataZoom: [
      { type: 'inside' },
      {
        type: 'slider', height: 64, bottom: 10, showDataShadow: true,
        borderColor: 'transparent', backgroundColor: 'rgba(135,127,193,0.05)',
        fillerColor: 'rgba(135,127,193,0.10)',
        dataBackground: {
          lineStyle: { color: 'rgba(135,127,193,0.45)' },
          areaStyle: { color: 'rgba(135,127,193,0.12)' },
        },
        selectedDataBackground: {
          lineStyle: { color: '#877FC1' },
          areaStyle: { color: 'rgba(135,127,193,0.22)' },
        },
        handleStyle: { color: '#877FC1' },
        moveHandleStyle: { color: '#877FC1' },
        emphasis: { handleStyle: { color: '#6f68b5' }, moveHandleStyle: { color: '#6f68b5' } },
        textStyle: { color: 'rgba(113,118,138,0.8)', fontSize: 10 },
      },
    ],
    tooltip: {
      ...chartBase().tooltip,
      trigger: 'axis',
      axisPointer: { type: 'line', lineStyle: { color: 'rgba(135,127,193,0.6)', width: 1.2 } },
      valueFormatter: (v) => `¥ ${fmtAmount(Number(v))}`,
    },
    xAxis: {
      ...chartBase().xAxis,
      type: 'category',
      boundaryGap: trendType.value === 'column',
      data: labels,
      axisLabel: { color: chartToken('--zj-text-sub', '#71768a'), fontSize: 11, hideOverlap: true },
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
    series: trendType.value === 'line' ? lineSeries : columnSeries,
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
      <div style="display: flex; align-items: center; gap: 12px; flex-wrap: wrap; margin-bottom: 6px">
        <span style="font-weight: 600">收支趋势</span>
        <el-radio-group v-model="trendRange" size="small" @change="loadTrend">
          <el-radio-button value="30d">近30天</el-radio-button>
          <el-radio-button value="1y">近1年</el-radio-button>
          <el-radio-button value="all">全部</el-radio-button>
        </el-radio-group>
        <el-radio-group v-model="trendType" size="small">
          <el-radio-button value="line">线</el-radio-button>
          <el-radio-button value="column">柱</el-radio-button>
        </el-radio-group>
        <div style="flex: 1" />
        <span class="chart-legend">
          <span v-for="it in trendLegend" :key="it.name" class="chart-legend-item">
            <i class="dot" :style="{ background: it.color }" />{{ it.name }}
          </span>
        </span>
        <span class="zj-num trend-range"><el-icon style="margin-right: 4px"><Calendar /></el-icon>{{ trendFrom }} - {{ trendTo }}</span>
      </div>
      <template v-if="trendData.length > 0">
        <ChartCard ref="trendRef" name="收支趋势" :option="trendOption" height="400px" />
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
.trend-range {
  display: inline-flex;
  align-items: center;
  padding: 6px 12px;
  border-radius: 999px;
  border: 1px solid var(--zj-border);
  background: var(--zj-card-hover);
  font-size: 12px;
  color: var(--zj-text);
}
</style>

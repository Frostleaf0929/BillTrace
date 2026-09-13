<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { api, fmtAmount } from '../api';
import ChartCard from '../components/ChartCard.vue';
import PageSub from '../components/PageSub.vue';
import type { ChartPoint, PiePoint } from '../types';
import * as echarts from 'echarts';

const PALETTE = ['#5b8ff9', '#61c0bf', '#65779b', '#f2b04c', '#9d7bde', '#6fb7e8', '#d983a6', '#7fbf7f', '#e8a87c', '#a3aecd'];

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
const pieData = ref<PiePoint[]>([]);
const loadError = ref('');

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

async function load() {
  if (!dateFrom.value || !dateTo.value) return;
  loadError.value = '';
  try {
    barData.value = await api.statsChart(dimension.value, dateFrom.value, dateTo.value);
    pieData.value = await api.statsPie(pieType.value, dateFrom.value, dateTo.value, groupBy.value);
  } catch (e) {
    loadError.value = String(e);
    barData.value = [];
    pieData.value = [];
    ElMessage.error(`统计数据加载失败：${e}`);
  }
}

onMounted(() => {
  setDefaultRange();
  load();
});

watch([dimension, dateFrom, dateTo, groupBy, pieType], load);

// ── 第 2 层：收支趋势柱状图 ──
const trendOption = computed<echarts.EChartsOption>(() => ({
  backgroundColor: 'transparent',
  tooltip: { trigger: 'axis' },
  legend: { data: ['支出', '收入'], top: 0, itemGap: 24, textStyle: { color: '#9aa0af' } },
  grid: { left: 8, right: 20, top: 44, bottom: barData.value.length > 40 ? 66 : 10, containLabel: true },
  dataZoom: barData.value.length > 40 ? [{ type: 'slider', height: 18, bottom: 10 }] : [],
  xAxis: {
    type: 'category',
    data: barData.value.map((d) => d.label),
    axisLabel: { color: '#9aa0af', rotate: barData.value.length > 12 ? 40 : 0 },
  },
  yAxis: {
    type: 'value',
    axisLabel: { color: '#9aa0af' },
    splitLine: { lineStyle: { opacity: 0.12, type: 'dashed' } },
  },
  series: [
    {
      name: '支出', type: 'bar', barMaxWidth: 22, barMinHeight: 1,
      itemStyle: { color: '#5b8ff9', borderRadius: [4, 4, 0, 0] },
      data: barData.value.map((d) => Math.round(d.expense * 100) / 100),
    },
    {
      name: '收入', type: 'bar', barMaxWidth: 22, barMinHeight: 1,
      itemStyle: { color: '#61c0bf', borderRadius: [4, 4, 0, 0] },
      data: barData.value.map((d) => Math.round(d.income * 100) / 100),
    },
  ],
}));

// ── 第 3 层：结构图（按维度） ──
const structOption = computed<echarts.EChartsOption>(() => {
  const data = pieData.value.map((d) => ({ name: d.name, value: Math.round(d.value * 100) / 100 }));
  if (chartKind.value === 'pie') {
    return {
      backgroundColor: 'transparent',
      color: PALETTE,
      tooltip: { trigger: 'item', formatter: '{b}<br/>¥{c}（{d}%）' },
      legend: { type: 'scroll', orient: 'vertical', right: 6, top: 'middle', textStyle: { color: '#9aa0af' } },
      series: [
        {
          name: pieType.value,
          type: 'pie',
          radius: ['36%', '58%'],
          center: ['42%', '50%'],
          itemStyle: { borderRadius: 6, borderColor: 'transparent', borderWidth: 2 },
          label: { show: false },
          data,
        },
      ],
    } as echarts.EChartsOption;
  }
  const top = data.slice(0, 12).reverse();
  return {
    backgroundColor: 'transparent',
    tooltip: { trigger: 'item', formatter: '{b}<br/>¥{c}' },
    grid: { left: 120, right: 44, top: 16, bottom: 30 },
    xAxis: { type: 'value', axisLabel: { color: '#9aa0af' }, splitLine: { lineStyle: { opacity: 0.15 } } },
    yAxis: { type: 'category', data: top.map((d) => d.name), axisLabel: { color: '#9aa0af', width: 100, overflow: 'truncate' } },
    series: [
      {
        name: pieType.value,
        type: 'bar',
        barMaxWidth: 18,
        itemStyle: { color: '#5b8ff9', borderRadius: [0, 5, 5, 0] },
        data: top.map((d) => d.value),
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
      <div style="font-weight: 600; margin-bottom: 6px">收支趋势 · {{ { year: '按年', month: '按月', day: '按日' }[dimension] }}（{{ dateFrom }} ~ {{ dateTo }}）</div>
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
</style>

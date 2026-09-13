<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue';
import * as echarts from 'echarts';
import { ElMessage } from 'element-plus';
import { api } from '../api';

const props = defineProps<{ option: echarts.EChartsOption; height?: string; name?: string }>();
const el = ref<HTMLDivElement>();
const exporting = ref(false);
let chart: echarts.ECharts | null = null;
let ro: ResizeObserver | null = null;

function render() {
  if (el.value && chart) chart.setOption(props.option, true);
}

onMounted(() => {
  if (el.value) {
    chart = echarts.init(el.value);
    chart.setOption(props.option);
    ro = new ResizeObserver(() => chart?.resize());
    ro.observe(el.value);
  }
});

watch(() => props.option, render, { deep: true });

onBeforeUnmount(() => {
  ro?.disconnect();
  chart?.dispose();
});

async function exportPng() {
  if (!chart || exporting.value) return;
  exporting.value = true;
  try {
    const bg = getComputedStyle(document.body).getPropertyValue('--zj-card-solid').trim() || '#ffffff';
    const url: string = chart.getDataURL({ pixelRatio: 2, backgroundColor: bg });
    const base64 = url.replace(/^data:image\/png;base64,/, '');
    const path = await api.exportImage(props.name || '图表', base64);
    ElMessage.success({ message: `图表已导出：${path}`, duration: 5000 });
  } catch (e) {
    if (!String(e).includes('取消')) ElMessage.error(String(e));
  } finally {
    exporting.value = false;
  }
}

defineExpose({ exportPng, chart });
</script>

<template>
  <div style="position: relative">
    <div ref="el" :style="{ width: '100%', height: height || '320px' }" />
    <el-button size="small" :loading="exporting" circle style="position: absolute; right: 4px; top: 0" title="导出 PNG（可选择保存位置）" @click="exportPng">⤓</el-button>
  </div>
</template>

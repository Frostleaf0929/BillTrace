//! 图表主题桥（DESIGN-SPEC §9）：ECharts canvas 读不到 CSS 变量，
//! 渲染前从 computedStyle 实时取 --zj-* token；全部图表共享同一套
//! 公共底座（透明背景 / 关动画 / 关内置图例 / 统一坐标轴与 tooltip）。
//! 组件里不允许出现裸色值——要色就进这里。

import * as echarts from 'echarts';

/** 读单个 CSS 变量，读不到用 fallback */
export function chartToken(name: string, fallback: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback;
}

/** Top-N 系列强调色阶梯：同一强调色按透明度分阶，视觉上"同一个家族"（SPEC §9.2） */
const LADDER = [1, 0.72, 0.54, 0.4, 0.3, 0.22, 0.16, 0.12];

export function accentLadder(rank: number): string {
  const accent = chartToken('--zj-primary', '#5570d6');
  const a = LADDER[Math.min(Math.max(rank, 0), LADDER.length - 1)];
  return withAlpha(accent, a);
}

/** #rrggbb → rgba(r,g,b,a) */
export function withAlpha(color: string, a: number): string {
  const hex = color.replace('#', '');
  const n = hex.length === 3 ? hex.split('').map((c) => c + c).join('') : hex;
  const r = parseInt(n.slice(0, 2), 16);
  const g = parseInt(n.slice(2, 4), 16);
  const b = parseInt(n.slice(4, 6), 16);
  return `rgba(${r},${g},${b},${a})`;
}

/** 竖向渐变柱：顶部淡出到底部实色，让单色柱也有层次（用户验收反馈） */
export function verticalFade(color: string, topAlpha = 0.45): echarts.graphic.LinearGradient {
  return new echarts.graphic.LinearGradient(0, 0, 0, 1, [
    { offset: 0, color: withAlpha(color, topAlpha) },
    { offset: 1, color: color },
  ]);
}

/** 饼图等需要强区分场景的 10 色柔和去饱和色板（全应用唯一一处系列色定义） */
export const SOFT_PALETTE = ['#7b84ec', '#6fb59a', '#c9a06a', '#c98ba0', '#6ba3c9', '#9d8fc9', '#c98a8a', '#8fb37e', '#c9b06a', '#7aa3b5'];

/** 公共底座：所有 ECharts option 先展开它，再覆盖各自的 series/xAxis 等 */
export function chartBase(): echarts.EChartsOption {
  const label = chartToken('--zj-text-sub', '#575d6c');
  const split = chartToken('--zj-border', 'rgba(34,37,46,0.1)');
  const tipBg = chartToken('--zj-card-solid', '#ffffff');
  const tipText = chartToken('--zj-text', '#21242d');
  return {
    backgroundColor: 'transparent',
    animation: false, // 数据随筛选刷新，动画只会造成"反复生长"（SPEC §9.1）
    legend: { show: false }, // 内置图例窄卡会压绘图区，改自绘 HTML（SPEC §9.1）
    grid: { left: 4, right: 12, top: 14, bottom: 0, containLabel: true },
    tooltip: {
      backgroundColor: tipBg,
      borderColor: split,
      textStyle: { color: tipText, fontSize: 12 },
      extraCssText: 'box-shadow: 0 8px 24px rgba(0,0,0,.14); border-radius: 10px; font-variant-numeric: tabular-nums;',
    },
    xAxis: {
      axisTick: { show: false },
      axisLine: { lineStyle: { color: split } },
      axisLabel: { color: label, fontSize: 11 },
    },
    yAxis: { splitLine: { lineStyle: { color: split } } },
  } as echarts.EChartsOption;
}

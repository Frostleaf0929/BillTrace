<script setup lang="ts">
// Porcelain 纸面卡外壳：v-html 渲染构建器输出
defineProps<{ html: string; span?: string; padLeft?: boolean }>();
</script>

<template>
  <div class="v2-card v2-paper" :class="[span, { 'pad-left': padLeft }]" style="min-height:120px">
    <template v-if="html">
      <div style="position:relative" v-html="html" />
    </template>
    <div v-else style="padding:40px;text-align:center;color:var(--v2-ink-3);font-size:13px">加载中…</div>
  </div>
</template>

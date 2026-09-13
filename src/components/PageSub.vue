<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { api } from '../api';

const props = defineProps<{ page: string; fallback: string }>();
const text = ref(props.fallback);
const editing = ref(false);
const draft = ref('');

onMounted(async () => {
  try {
    const v = await api.getSetting(`sub_${props.page}`);
    if (v) text.value = v;
  } catch { /* 用默认文案 */ }
});

function start() {
  draft.value = text.value;
  editing.value = true;
  requestAnimationFrame(() => {
    (document.querySelector('.zj-sub-input input') as HTMLInputElement | null)?.focus();
  });
}

async function save() {
  editing.value = false;
  const v = draft.value.trim();
  if (!v || v === text.value) return;
  text.value = v;
  try { await api.setSetting(`sub_${props.page}`, v); } catch { /* 忽略 */ }
}
</script>

<template>
  <div class="sub-wrap">
    <p v-if="!editing" class="zj-page-sub" title="点击修改这句话" @click="start">{{ text }}</p>
    <el-input
      v-else
      v-model="draft"
      class="zj-sub-input"
      size="small"
      style="width: 320px; margin-bottom: 14px"
      @keydown.enter="save"
      @keydown.esc="editing = false"
      @blur="save"
    />
  </div>
</template>

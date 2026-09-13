<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { api } from '../api';

const props = defineProps<{ visible: boolean; account: { name: string; base: number } | null }>();
const emit = defineEmits<{ (e: 'update:visible', v: boolean): void; (e: 'saved'): void }>();

const visible = computed({
  get: () => props.visible,
  set: (v) => emit('update:visible', v),
});

const amount = ref<number>(0);

watch(
  () => props.visible,
  (v) => {
    if (v && props.account) amount.value = props.account.base;
  }
);

async function save() {
  if (!props.account) return;
  if (Number.isNaN(amount.value)) {
    ElMessage.warning('请输入有效数字');
    return;
  }
  await api.setAccountBase(props.account.name, Number(amount.value));
  ElMessage.success(`已更新「${props.account.name}」基数`);
  visible.value = false;
  emit('saved');
}
</script>

<template>
  <el-dialog v-model="visible" title="设定账户基数" width="380px" align-center destroy-on-close>
    <div style="margin-bottom: 8px">
      <div style="font-weight: 600; margin-bottom: 10px">{{ account?.name }}</div>
      <el-input-number v-model="amount" :precision="2" :controls="false" style="width: 100%" size="large" placeholder="基数金额" />
      <div style="color: var(--zj-text-sub); font-size: 12px; margin-top: 8px">余额会随之自动更新</div>
    </div>
    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" @click="save">保存</el-button>
    </template>
  </el-dialog>
</template>

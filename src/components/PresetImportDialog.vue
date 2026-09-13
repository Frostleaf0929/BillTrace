<script setup lang="ts">
import { computed, ref } from 'vue';
import { ElMessage } from 'element-plus';
import { api } from '../api';
import type { PresetPreview } from '../types';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ (e: 'update:visible', v: boolean): void; (e: 'applied'): void }>();

const visible = computed({
  get: () => props.visible,
  set: (v) => emit('update:visible', v),
});

const parsing = ref(false);
const applying = ref(false);
const format = ref('');
const preview = ref<PresetPreview | null>(null);

async function pickAndParse() {
  const { open } = await import('@tauri-apps/plugin-dialog');
  const picked = await open({
    multiple: false,
    filters: [{ name: '分类预设', extensions: ['md', 'txt', 'markdown'] }],
  });
  if (!picked || Array.isArray(picked)) return;
  parsing.value = true;
  preview.value = null;
  try {
    const [fmt, p] = await api.presetParse(picked);
    format.value = fmt;
    preview.value = p;
  } catch (e) {
    ElMessage.error(String(e));
  } finally {
    parsing.value = false;
  }
}

async function apply() {
  if (!preview.value) return;
  applying.value = true;
  try {
    const [cats, rules] = await api.presetApply(preview.value);
    ElMessage.success(`已写入 ${cats} 条分类、${rules} 条规则`);
    visible.value = false;
    emit('applied');
  } catch (e) {
    ElMessage.error(String(e));
  } finally {
    applying.value = false;
  }
}
</script>

<template>
  <el-dialog v-model="visible" title="导入分类预设" width="680px" align-center destroy-on-close>
    <div style="text-align: center; padding: 6px 0">
      <el-button type="primary" :loading="parsing" @click="pickAndParse">选择 md / txt 文件</el-button>
      <div style="color: var(--zj-text-sub); font-size: 12px; margin-top: 10px; line-height: 1.9">
        支持 Markdown（如《分类记录_调整.md》：一级/二级目录、映射表格、自动分类规则表）<br />
        与 txt（缩进层级或「支出/食品饮料/早午晚餐」斜杠路径）。导入前先预览确认。
      </div>
    </div>

    <template v-if="preview">
      <el-divider>解析结果（{{ format }}）</el-divider>
      <el-alert v-for="(w, i) in preview.warnings" :key="i" type="warning" :title="w" :closable="false" show-icon style="margin-bottom: 8px" />
      <el-descriptions :column="3" size="small" border>
        <el-descriptions-item label="分类条目">{{ preview.categories.length }}</el-descriptions-item>
        <el-descriptions-item label="规则条目">{{ preview.rules.length }}</el-descriptions-item>
        <el-descriptions-item label="一级分类数">{{ new Set(preview.categories.map((c) => c.kind + '|' + c.l1)).size }}</el-descriptions-item>
      </el-descriptions>
      <el-table :data="preview.categories.slice(0, 200)" size="small" max-height="260" style="margin-top: 10px">
        <el-table-column label="体系" width="90">
          <template #default="{ row }">{{ { expense: '支出', income: '收入', account: '账户' }[row.kind as string] ?? row.kind }}</template>
        </el-table-column>
        <el-table-column label="一级" prop="l1" />
        <el-table-column label="二级" prop="l2" />
      </el-table>
      <el-table v-if="preview.rules.length" :data="preview.rules.slice(0, 100)" size="small" max-height="200" style="margin-top: 10px">
        <el-table-column label="关键词" prop="keyword" />
        <el-table-column label="→ 一级" prop="l1" width="130" />
        <el-table-column label="→ 二级" prop="l2" width="130" />
      </el-table>
    </template>

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" :disabled="!preview" :loading="applying" @click="apply">确认写入</el-button>
    </template>
  </el-dialog>
</template>

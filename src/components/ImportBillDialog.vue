<script setup lang="ts">
import { computed, ref } from 'vue';
import { ElMessage } from 'element-plus';
import { UploadFilled } from '@element-plus/icons-vue';
import { api } from '../api';
import type { ImportReport } from '../types';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ (e: 'update:visible', v: boolean): void; (e: 'imported'): void }>();

const visible = computed({
  get: () => props.visible,
  set: (v) => emit('update:visible', v),
});

const importing = ref(false);
const report = ref<ImportReport | null>(null);
const hasPending = ref(false);

async function pickAndImport() {
  const { open } = await import('@tauri-apps/plugin-dialog');
  const picked = await open({
    multiple: false,
    filters: [{ name: 'Excel 账单', extensions: ['xlsx', 'xls'] }],
  });
  if (!picked || Array.isArray(picked)) return;
  importing.value = true;
  report.value = null;
  try {
    report.value = await api.importBill(picked);
    hasPending.value = (report.value?.pending ?? 0) > 0;
    ElMessage.success(`导入完成：新增 ${report.value?.inserted} 条，重复跳过 ${report.value?.duplicates} 条`);
    emit('imported');
  } catch (e) {
    ElMessage.error(String(e));
  } finally {
    importing.value = false;
  }
}
</script>

<template>
  <el-dialog v-model="visible" title="导入账单" width="620px" align-center destroy-on-close>
    <div style="text-align: center; padding: 10px 0 4px">
      <el-button type="primary" size="large" :loading="importing" @click="pickAndImport">
        <el-icon style="margin-right: 6px"><UploadFilled /></el-icon>
        选择 Excel 文件导入
      </el-button>
      <div style="color: var(--zj-text-sub); font-size: 12px; margin-top: 12px; line-height: 1.9">
        支持：年度账本 Excel（支出 / 收入 / 转账等多 Sheet 自动识别）<br />
        微信支付官方导出的账单流水 xlsx —— 自动识别格式、自动查重、按规则分类
      </div>
    </div>

    <template v-if="report">
      <el-divider />
      <el-descriptions :column="2" size="small" border>
        <el-descriptions-item label="识别格式">{{ report.detected_format === 'wechat' ? '微信支付流水' : '年度账本' }}</el-descriptions-item>
        <el-descriptions-item label="解析行数">{{ report.total_rows }}</el-descriptions-item>
        <el-descriptions-item label="新增"><span class="zj-amount-income">{{ report.inserted }}</span></el-descriptions-item>
        <el-descriptions-item label="重复跳过">{{ report.duplicates }}</el-descriptions-item>
        <el-descriptions-item label="无法解析">{{ report.skipped }}</el-descriptions-item>
        <el-descriptions-item label="待确认商家">
          <span :class="report.pending > 0 ? 'zj-amount-expense' : ''">{{ report.pending }} 条 / {{ report.pending_merchants.length }} 个商家</span>
        </el-descriptions-item>
      </el-descriptions>
      <el-table v-if="report.sheet_stats.length" :data="report.sheet_stats" size="small" style="margin-top: 10px">
        <el-table-column prop="sheet" label="Sheet" />
        <el-table-column prop="rows" label="解析" width="90" />
        <el-table-column prop="inserted" label="新增" width="90" />
        <el-table-column prop="duplicates" label="重复" width="90" />
      </el-table>
      <el-alert
        v-if="hasPending"
        type="warning"
        :closable="false"
        show-icon
        style="margin-top: 10px"
        title="有新商家等待归类"
        description="请到「详细」页顶部的待确认队列完成分类，归类后会自动沉淀为规则，下次自动分类。"
      />
    </template>
  </el-dialog>
</template>

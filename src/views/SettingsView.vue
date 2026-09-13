<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { api } from '../api';
import {
  setThemeModeAnimated, themeMode, type ThemeMode,
  ACCENTS, accentId, setAccent, customAccent, setCustomAccent,
  glassOn, setGlass, blurPx, setBlur, glassAlpha, setGlassAlpha, brightness, setBrightness,
  bgEnabled, setBgEnabled, bgFit, setBgFit, refreshBackground,
} from '../theme';
import type { TxFilter } from '../types';
import ImportBillDialog from '../components/ImportBillDialog.vue';
import PageSub from '../components/PageSub.vue';

const recentCount = ref(10);
const customHex = ref(customAccent.value || '#3e63dd');
const wheelNav = ref(true);

const BG_FIT_OPTIONS = [
  { value: 'cover', label: '填充（裁剪铺满）' },
  { value: 'contain', label: '包含（完整显示）' },
  { value: 'fill', label: '拉伸（变形铺满）' },
  { value: 'tile', label: '平铺' },
];

const newMerchantPrompt = ref(true);
const fallbackCategory = ref('统计未确认');
const dataDir = ref('');
const importVisible = ref(false);

const emptyFilter: TxFilter = {};

onMounted(async () => {
  const n = Number(await api.getSetting('overviewRecentCount').catch(() => '10'));
  if (n >= 5 && n <= 20) recentCount.value = n;
  dataDir.value = await api.getDataDir();
  newMerchantPrompt.value = (await api.getSetting('newMerchantPrompt')) === 'true';
  fallbackCategory.value = await api.getSetting('fallbackCategory');
  wheelNav.value = (await api.getSetting('wheelNav')) !== 'false';
});

watch(newMerchantPrompt, async (v) => {
  await api.setSetting('newMerchantPrompt', String(v));
});

function onModeChange(mode: ThemeMode, e: MouseEvent) {
  setThemeModeAnimated(mode, { x: e.clientX, y: e.clientY });
}

async function toggleWheelNav(v: boolean | string | number) {
  wheelNav.value = !!v;
  try { await api.setSetting('wheelNav', String(wheelNav.value)); } catch { /* 忽略 */ }
  ElMessage.success(wheelNav.value ? '滚动换页已开启' : '滚动换页已关闭');
}

async function saveRecentCount() {
  recentCount.value = Math.round(recentCount.value);
  await api.setSetting('overviewRecentCount', String(recentCount.value));
  ElMessage.success('概览最近记录条数已更新');
}

async function saveFallback() {
  await api.setSetting('fallbackCategory', fallbackCategory.value);
  ElMessage.success('已保存');
}

async function applyCustomColor() {
  if (!/^#[0-9a-fA-F]{6}$/.test(customHex.value)) {
    ElMessage.warning('请输入有效的颜色值，如 #3E63DD');
    return;
  }
  await setCustomAccent(customHex.value);
  ElMessage.success('已应用自定义颜色');
}

async function pickBackground() {
  const { open } = await import('@tauri-apps/plugin-dialog');
  const picked = await open({ multiple: false, filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif'] }] });
  if (!picked || Array.isArray(picked)) return;
  try {
    await api.importBackground(picked);
    await refreshBackground();
    if (!bgEnabled.value) await setBgEnabled(true);
    ElMessage.success('背景图已设置');
  } catch (e) {
    ElMessage.error(String(e));
  }
}

async function removeBackground() {
  await api.clearBackground();
  await refreshBackground();
  ElMessage.success('背景图已移除');
}

async function clearTx() {
  await ElMessageBox.confirm('清空全部账目记录（交易与待确认队列）？分类、规则、账户基数保留。此操作不可恢复！', '清除账目数据', { type: 'error', confirmButtonText: '清空账目' });
  await api.clearData('tx');
  ElMessage.success('账目数据已清空');
}

async function clearAll() {
  await ElMessageBox.confirm('清除全部数据（账目 + 分类 + 规则 + 账户基数），恢复到初始状态？此操作不可恢复！建议先「立即备份」。', '清除全部数据', { type: 'error', confirmButtonText: '全部清除' });
  await api.clearData('all');
  ElMessage.success('已恢复初始状态');
}

async function doExport(format: 'xlsx' | 'csv') {
  try {
    const n = await api.exportData(format, emptyFilter);
    ElMessage.success(`已导出 ${n} 条记录`);
  } catch (e) {
    if (String(e).includes('取消')) return;
    ElMessage.error(String(e));
  }
}

async function doExportCategories(format: 'md' | 'txt') {
  try {
    await api.exportCategories(format);
    ElMessage.success(`分类体系已导出为 ${format.toUpperCase()}`);
  } catch (e) {
    if (String(e).includes('取消')) return;
    ElMessage.error(String(e));
  }
}

async function doBackup() {
  const path = await api.backupNow();
  ElMessage.success({ message: `备份完成：${path}`, duration: 5000 });
}

async function openDataDir() {
  await api.revealInExplorer(dataDir.value);
}

async function openGithub() {
  const { openUrl } = await import('@tauri-apps/plugin-opener');
  await openUrl('https://github.com/Frostleaf0929');
}
</script>

<template>
  <div>
    <h1 class="zj-page-title">设置</h1>
    <PageSub page="settings" fallback="导入行为、外观个性化、规则管理与数据备份" />

    <div class="zj-row" style="margin-bottom: 14px">
      <div class="zj-card" style="flex: 1">
        <div style="font-weight: 600; margin-bottom: 14px">界面</div>
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px">
          <div>
            <div>界面模式</div>
            <div style="color: var(--zj-text-sub); font-size: 12px">以触发位置为圆心辐射切换主题</div>
          </div>
          <div style="display: flex; gap: 8px">
            <el-button
              v-for="m in [{ v: 'system', t: '跟随系统' }, { v: 'light', t: '浅色' }, { v: 'dark', t: '深色' }]"
              :key="m.v"
              :type="themeMode === m.v ? 'primary' : 'default'"
              @click="onModeChange(m.v as ThemeMode, $event)"
            >{{ m.t }}</el-button>
          </div>
        </div>
        <div style="display: flex; justify-content: space-between; align-items: center">
          <div>
            <div>滚动换页</div>
            <div style="color: var(--zj-text-sub); font-size: 12px">页面滚到底继续下滚切换下一页，页顶上滚返回上一页</div>
          </div>
          <el-switch :model-value="wheelNav" @change="toggleWheelNav" />
        </div>
      </div>
      <div class="zj-card" style="flex: 1">
        <div style="font-weight: 600; margin-bottom: 14px">概览</div>
        <div style="display: flex; justify-content: space-between; align-items: center">
          <div>
            <div>最近记录条数</div>
            <div style="color: var(--zj-text-sub); font-size: 12px">概览页「最近记录」表格显示 5 ~ 20 条</div>
          </div>
          <div style="display: flex; align-items: center; gap: 10px">
            <el-slider v-model="recentCount" :min="5" :max="20" :step="1" style="width: 160px" @change="saveRecentCount" />
            <span class="zj-num" style="width: 26px; text-align: right">{{ recentCount }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="zj-row" style="margin-bottom: 14px">
      <div class="zj-card" style="flex: 1">
        <div style="font-weight: 600; margin-bottom: 14px">个性化 · 颜色</div>
        <div style="margin-bottom: 16px">
          <div style="margin-bottom: 8px">颜色方案</div>
          <div style="display: flex; gap: 10px; flex-wrap: wrap">
            <div
              v-for="acc in ACCENTS"
              :key="acc.id"
              class="accent-chip"
              :class="{ active: accentId === acc.id && !customAccent }"
              @click="setAccent(acc.id)"
            >
              <span class="accent-dot" :style="{ background: acc.light }" />
              <span>{{ acc.name }}</span>
            </div>
            <div class="accent-chip" :class="{ active: !!customAccent }">
              <el-color-picker v-model="customHex" size="small" :predefine="ACCENTS.map((a) => a.light)" @change="applyCustomColor" />
              <el-input
                v-model="customHex"
                size="small"
                style="width: 104px"
                placeholder="#3E63DD"
                @keydown.enter="applyCustomColor"
              />
              <el-button text size="small" type="primary" @click="applyCustomColor">应用</el-button>
            </div>
          </div>
        </div>
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px">
          <div>
            <div>毛玻璃效果</div>
            <div style="color: var(--zj-text-sub); font-size: 12px">关闭后卡片与侧边栏变为实底，更省电</div>
          </div>
          <el-switch :model-value="glassOn" @change="(v: any) => setGlass(!!v)" />
        </div>
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px">
          <div>
            <div>毛玻璃强度</div>
            <div style="color: var(--zj-text-sub); font-size: 12px">玻璃面板的不透明度，越低越通透</div>
          </div>
          <div style="display: flex; align-items: center; gap: 10px">
            <el-slider v-model="glassAlpha" :min="30" :max="95" :step="5" :disabled="!glassOn" style="width: 160px" @change="(v: any) => setGlassAlpha(Number(v))" />
            <span class="zj-num" style="width: 42px; text-align: right">{{ glassAlpha }}%</span>
          </div>
        </div>
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px">
          <div>
            <div>模糊强度</div>
            <div style="color: var(--zj-text-sub); font-size: 12px">毛玻璃的模糊半径</div>
          </div>
          <div style="display: flex; align-items: center; gap: 10px">
            <el-slider v-model="blurPx" :min="4" :max="40" :step="2" :disabled="!glassOn" style="width: 160px" @change="(v: any) => setBlur(Number(v))" />
            <span class="zj-num" style="width: 42px; text-align: right">{{ blurPx }}px</span>
          </div>
        </div>
        <div style="display: flex; justify-content: space-between; align-items: center">
          <div>
            <div>背景亮度</div>
            <div style="color: var(--zj-text-sub); font-size: 12px">负值调暗、正值调亮，0 为原始亮度</div>
          </div>
          <div style="display: flex; align-items: center; gap: 10px">
            <el-slider v-model="brightness" :min="-100" :max="100" :step="5" style="width: 160px" @change="(v: any) => setBrightness(Number(v))" />
            <span class="zj-num" style="width: 42px; text-align: right">{{ brightness }}</span>
          </div>
        </div>
      </div>
      <div class="zj-card" style="flex: 1">
        <div style="font-weight: 600; margin-bottom: 14px">个性化 · 背景图</div>
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px">
          <div>
            <div>启用背景图</div>
            <div style="color: var(--zj-text-sub); font-size: 12px">覆盖默认渐变背景，卡片毛玻璃作用于其上</div>
          </div>
          <el-switch :model-value="bgEnabled" @change="(v: any) => setBgEnabled(!!v)" />
        </div>
        <div style="display: flex; gap: 10px; margin-bottom: 12px">
          <el-button @click="pickBackground">选择图片…</el-button>
          <el-button @click="removeBackground">移除背景图</el-button>
        </div>
        <div style="display: flex; justify-content: space-between; align-items: center">
          <div>
            <div>适配方式</div>
            <div style="color: var(--zj-text-sub); font-size: 12px">填充 / 包含 / 拉伸 / 平铺</div>
          </div>
          <el-select :model-value="bgFit" style="width: 190px" @change="(v: any) => setBgFit(v)">
            <el-option v-for="o in BG_FIT_OPTIONS" :key="o.value" :label="o.label" :value="o.value" />
          </el-select>
        </div>
      </div>
    </div>

    <div class="zj-row" style="margin-bottom: 14px">
      <div class="zj-card" style="flex: 1">
        <div style="font-weight: 600; margin-bottom: 14px">导入行为</div>
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px">
          <div>
            <div>新商家弹窗归类</div>
            <div style="color: var(--zj-text-sub); font-size: 12px">导入时遇到没有规则的商家，先弹窗让你选分类，确认后自动沉淀为预设</div>
          </div>
          <el-switch v-model="newMerchantPrompt" />
        </div>
        <div style="display: flex; justify-content: space-between; align-items: center">
          <div>
            <div>兜底分类（未匹配商家且未弹窗时）</div>
            <div style="color: var(--zj-text-sub); font-size: 12px">无法自动归类的记录统一进入此一级分类</div>
          </div>
          <div style="display: flex; gap: 8px">
            <el-input v-model="fallbackCategory" style="width: 140px" />
            <el-button @click="saveFallback">保存</el-button>
          </div>
        </div>
      </div>
      <div class="zj-card" style="flex: 1">
        <div style="font-weight: 600; margin-bottom: 14px">数据</div>
        <div style="display: flex; flex-wrap: wrap; gap: 10px">
          <el-button type="primary" @click="importVisible = true">导入账单</el-button>
          <el-button @click="doExport('xlsx')">导出 Excel</el-button>
          <el-button @click="doExport('csv')">导出 CSV</el-button>
          <el-dropdown style="margin-left: 0" @command="(c: any) => doExportCategories(c)">
            <el-button>导出分类</el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="md">导出为 Markdown</el-dropdown-item>
                <el-dropdown-item command="txt">导出为 txt</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
          <el-button @click="doBackup">立即备份</el-button>
          <el-button text @click="openDataDir">打开数据目录 →</el-button>
        </div>
        <div class="zj-num" style="color: var(--zj-text-sub); font-size: 12px; margin-top: 12px; word-break: break-all">
          当前数据文件夹：{{ dataDir }}
        </div>
      </div>
    </div>

    <div class="zj-card" style="margin-bottom: 14px">
      <div style="font-weight: 600; margin-bottom: 10px">清除数据</div>
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px">
        <div>
          <div>清空账目记录</div>
          <div style="color: var(--zj-text-sub); font-size: 12px">删除全部交易与待确认队列，保留分类、规则、账户基数</div>
        </div>
        <el-button type="danger" plain @click="clearTx">清空账目</el-button>
      </div>
      <div style="display: flex; justify-content: space-between; align-items: center">
        <div>
          <div>清除全部数据</div>
          <div style="color: var(--zj-text-sub); font-size: 12px">账目、分类、规则、账户基数全部清除，恢复初始状态</div>
        </div>
        <el-button type="danger" @click="clearAll">全部清除</el-button>
      </div>
    </div>

    <div class="zj-card" style="margin-bottom: 14px; display: flex; justify-content: space-between; align-items: center">
      <div>
        <div style="font-weight: 600">作者</div>
        <div style="color: var(--zj-text-sub); font-size: 12px; margin-top: 4px">Frostleaf0929 · GitHub 主页</div>
      </div>
      <el-button type="primary" plain @click="openGithub">
        <svg width="15" height="15" viewBox="0 0 16 16" fill="currentColor" style="margin-right: 6px">
          <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z" />
        </svg>
        Frostleaf0929
      </el-button>
    </div>

    <ImportBillDialog v-model:visible="importVisible" />
  </div>
</template>

<style scoped>
.accent-chip {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-height: 32px;
  padding: 0 12px;
  border-radius: 10px;
  border: 1px solid var(--zj-border);
  background: var(--zj-card-hover);
  cursor: pointer;
  font-size: 13px;
  color: var(--zj-text);
  transition: box-shadow 0.2s ease, border-color 0.2s ease;
}

.accent-chip:hover {
  box-shadow: 0 6px 16px rgba(26, 31, 51, 0.12);
}

.accent-chip.active {
  border-color: var(--zj-primary);
  box-shadow: 0 0 0 1px var(--zj-primary);
}

.accent-dot {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.08);
}
</style>

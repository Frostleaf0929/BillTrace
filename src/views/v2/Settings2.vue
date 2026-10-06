<script setup lang="ts">
// 设置 v2：外观 / 数据 / 关于
import { onMounted, ref } from 'vue';
import { ElMessageBox } from 'element-plus';
import { api } from '../../api';
import {
  ACCENTS, accentId, customAccent, glassOn, blurPx, glassAlpha, brightness,
  setAccent, setCustomAccent, setGlass, setBlur, setGlassAlpha, setBrightness,
  themeMode, setThemeMode, bgEnabled, bgPath, bgFit, setBgEnabled, setBgFit, refreshBackground,
} from '../../theme';
import { icon } from '../../v2/icons';
import ImportBillDialog from '../../components/ImportBillDialog.vue';
import { openUrl } from '@tauri-apps/plugin-opener';
import { toast } from '../../v2/toast';

const dataDir = ref('');
const newMerchantPrompt = ref(true);
const fallbackCategory = ref('');
const wheelNav = ref(true);
const importVisible = ref(false);

const THEME_OPTS: ['light' | 'dark' | 'system', string][] = [['light', '浅色'], ['dark', '深色'], ['system', '跟随系统']];

onMounted(async () => {
  try {
    dataDir.value = await api.getDataDir();
    newMerchantPrompt.value = (await api.getSetting('newMerchantPrompt')) === 'true';
    fallbackCategory.value = await api.getSetting('fallbackCategory');
    wheelNav.value = (await api.getSetting('wheelNav')) !== 'false';
  } catch { /* 默认值 */ }
});

async function pickBg(): Promise<void> {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const picked = await open({ multiple: false, filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp'] }] });
    if (typeof picked === 'string') {
      await api.importBackground(picked);
      await refreshBackground();
      await setBgEnabled(true);
      toast('背景已更新');
    }
  } catch (e) {
    toast(`设置失败：${e}`);
  }
}
async function clearBg(): Promise<void> {
  await api.clearBackground();
  await setBgEnabled(false);
  toast('背景已清除');
}
async function toggleNewMerchant(v: boolean): Promise<void> {
  newMerchantPrompt.value = v;
  await api.setSetting('newMerchantPrompt', String(v));
}
async function toggleWheel(v: boolean): Promise<void> {
  wheelNav.value = v;
  await api.setSetting('wheelNav', String(v));
  window.dispatchEvent(new CustomEvent('zj-wheel-nav', { detail: v }));
}
async function setFallback(): Promise<void> {
  await api.setSetting('fallbackCategory', fallbackCategory.value);
  toast('兜底分类已保存');
}
async function doExport(format: string): Promise<void> {
  try {
    const n = await api.exportData(format, {});
    toast(`已导出 ${n} 条（${format.toUpperCase()}）——文件在数据目录`);
  } catch (e) {
    toast(`导出失败：${e}`);
  }
}
async function doBackup(): Promise<void> {
  try {
    const path = await api.backupNow();
    toast(`已备份：${path}`);
  } catch (e) {
    toast(`备份失败：${e}`);
  }
}
function openGithub(): void {
  openUrl('https://github.com/Frostleaf0929').catch(() => { /* 忽略 */ });
}
async function doClear(scope: 'tx' | 'all'): Promise<void> {
  const msg = scope === 'tx'
    ? '清空全部账目记录（交易与待确认队列）？分类、规则、账户基数保留。此操作不可恢复！'
    : '清除全部数据（账目 + 分类 + 规则 + 账户基数），恢复到初始状态？此操作不可恢复！建议先「一键备份」。';
  try {
    await ElMessageBox.confirm(msg, scope === 'tx' ? '清除账目数据' : '清除全部数据', { type: 'error' });
  } catch { return; }
  try {
    await api.clearData(scope);
    toast('已清除');
  } catch (e) {
    toast(`清除失败：${e}`);
  }
}
</script>

<template>
  <div>
    <div class="v2-pagehead">
      <div>
        <h1>设置</h1>
        <div class="sub">外观与数据管理</div>
      </div>
    </div>

    <div class="v2-grid">
      <div class="v2-card span7">
        <div class="v2-card-head"><h3>外观</h3></div>
        <div class="v2-set-row">
          <div>
            <div class="set-label">界面主题</div>
            <div class="set-desc">跟随系统时自动切换深浅</div>
          </div>
          <div class="v2-seg" style="margin-left:auto">
            <button v-for="[v, lbl] in THEME_OPTS" :key="v" :class="{ on: themeMode === v }" @click="setThemeMode(v)">{{ lbl }}</button>
          </div>
        </div>
        <div class="v2-set-row">
          <div>
            <div class="set-label">强调色</div>
            <div class="set-desc">{{ customAccent ? '自定义 ' + customAccent : '当前：' + (ACCENTS.find((a) => a.id === accentId)?.name ?? '自定义') }}</div>
          </div>
          <div class="v2-swatches" style="margin-left:auto">
            <button
              v-for="a in ACCENTS" :key="a.id" class="v2-swatch" :class="{ on: accentId === a.id && !customAccent }"
              :style="{ background: a.light }" :title="a.name" @click="setAccent(a.id)"
            />
            <input
              type="color" :value="customAccent || (ACCENTS.find((a) => a.id === accentId)?.light ?? '#877FC1')"
              style="width:26px;height:26px;border:0;background:none;cursor:pointer;padding:0"
              title="自定义强调色"
              @change="setCustomAccent(($event.target as HTMLInputElement).value)"
            >
          </div>
        </div>
        <div class="v2-set-row">
          <div>
            <div class="set-label">毛玻璃</div>
            <div class="set-desc">卡片半透明 + 背景模糊</div>
          </div>
          <button class="v2-toggle" :class="{ on: glassOn }" style="margin-left:auto" @click="setGlass(!glassOn)" />
        </div>
        <div class="v2-set-row">
          <div><div class="set-label">模糊半径</div><div class="set-desc">{{ blurPx }}px</div></div>
          <input type="range" min="4" max="40" :value="blurPx" style="margin-left:auto;width:180px" @input="setBlur(Number(($event.target as HTMLInputElement).value))">
        </div>
        <div class="v2-set-row">
          <div><div class="set-label">玻璃强度</div><div class="set-desc">{{ glassAlpha }}%（卡片不透明度）</div></div>
          <input type="range" min="30" max="95" :value="glassAlpha" style="margin-left:auto;width:180px" @input="setGlassAlpha(Number(($event.target as HTMLInputElement).value))">
        </div>
        <div class="v2-set-row">
          <div><div class="set-label">屏幕亮度</div><div class="set-desc">{{ brightness > 0 ? '+' : '' }}{{ brightness }}</div></div>
          <input type="range" min="-100" max="100" :value="brightness" style="margin-left:auto;width:180px" @input="setBrightness(Number(($event.target as HTMLInputElement).value))">
        </div>
        <div class="v2-set-row">
          <div>
            <div class="set-label">背景图</div>
            <div class="set-desc">{{ bgEnabled && bgPath ? '已启用' : '未启用' }}</div>
          </div>
          <div style="margin-left:auto;display:flex;gap:8px">
            <select :value="bgFit" class="v2-select" @change="setBgFit(($event.target as HTMLSelectElement).value as any)">
              <option value="cover">铺满</option>
              <option value="contain">包含</option>
              <option value="fill">拉伸</option>
              <option value="tile">平铺</option>
            </select>
            <button class="v2-btn ghost" style="height:34px" @click="pickBg">选择图片</button>
            <button v-if="bgEnabled" class="v2-btn ghost" style="height:34px" @click="clearBg">清除</button>
          </div>
        </div>
      </div>

      <div class="v2-card span5">
        <div class="v2-card-head"><h3>数据</h3></div>
        <div style="display:flex;flex-direction:column;gap:10px">
          <button class="v2-btn ghost" style="justify-content:flex-start" @click="importVisible = true">
            <span v-html="icon('up', 15)" />导入账单（微信 / 随手记）
          </button>
          <button class="v2-btn ghost" style="justify-content:flex-start" @click="doExport('xlsx')">导出数据（xlsx）</button>
          <button class="v2-btn ghost" style="justify-content:flex-start" @click="doExport('csv')">导出数据（csv）</button>
          <button class="v2-btn ghost" style="justify-content:flex-start" @click="doExport('md')">导出数据（md）</button>
          <button class="v2-btn ghost" style="justify-content:flex-start" @click="doBackup">一键备份</button>
          <button class="v2-btn danger" style="justify-content:flex-start" @click="doClear('tx')">清空账目记录…</button>
          <button class="v2-btn danger" style="justify-content:flex-start" @click="doClear('all')">清除全部数据…</button>
        </div>
        <div style="margin-top:18px;padding-top:14px;border-top:1px solid var(--v2-line);font-size:12px;color:var(--v2-ink-3);line-height:1.8;word-break:break-all">
          数据目录：{{ dataDir || '…' }}
        </div>
      </div>

      <div class="v2-card span7">
        <div class="v2-card-head"><h3>行为</h3></div>
        <div class="v2-set-row">
          <div>
            <div class="set-label">滚动换页</div>
            <div class="set-desc">页底继续滚动时切到下一页</div>
          </div>
          <button class="v2-toggle" :class="{ on: wheelNav }" style="margin-left:auto" @click="toggleWheel(!wheelNav)" />
        </div>
        <div class="v2-set-row">
          <div>
            <div class="set-label">新商家归类弹窗</div>
            <div class="set-desc">导入后弹出未识别商家的归类向导</div>
          </div>
          <button class="v2-toggle" :class="{ on: newMerchantPrompt }" style="margin-left:auto" @click="toggleNewMerchant(!newMerchantPrompt)" />
        </div>
        <div class="v2-set-row">
          <div>
            <div class="set-label">兜底分类</div>
            <div class="set-desc">跳过归类的账单进入此分类</div>
          </div>
          <input v-model="fallbackCategory" class="v2-text-input" style="width:160px;margin-left:auto" @change="setFallback">
        </div>
      </div>

      <div class="v2-card span5">
        <div class="v2-card-head"><h3>关于</h3></div>
        <div style="color:var(--v2-ink-2);font-size:13px;line-height:1.9">
          账痕 BillTrace · v0.4.0<br>
          绿色便携的本地账单管理 · 数据 100% 本地存储
        </div>
        <div style="margin-top:14px;padding-top:12px;border-top:1px solid var(--v2-line);display:flex;align-items:center;gap:10px">
          <span style="font-size:12px;color:var(--v2-ink-3)">作者</span>
          <button class="v2-link-btn" style="font-size:12.5px" @click="openGithub">Frostleaf0929 · GitHub 主页</button>
        </div>
      </div>
    </div>

    <ImportBillDialog v-model:visible="importVisible" @imported="() => toast('导入完成')" />
  </div>
</template>

<script lang="ts">
export default { name: 'Settings2' };
</script>

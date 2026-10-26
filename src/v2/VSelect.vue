<script setup lang="ts">
// v2 自绘下拉选择：原生 <select> 的弹层无法定制样式（方角白板、无滚动上限、箭头恒朝下），
// 全站统一换用本组件——弹层为实底圆角卡、max-height 320 滚动、箭头随开合方向翻转、键盘可用。
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';

// 多根组件（弹层 Teleport 到 body）：属性不自动透传，手动挂到根节点，保证外部 style/class 生效
defineOptions({ name: 'VSelect', inheritAttrs: false });

const props = defineProps<{
  modelValue: string;
  options: { value: string; label: string }[];
  placeholder?: string;
  block?: boolean; // 面板等表单内占满整行
}>();
const emit = defineEmits<{ change: [value: string]; 'update:modelValue': [value: string] }>();

const open = ref(false);
const up = ref(false); // 弹层是否向上展开
const activeIdx = ref(-1);
const triggerRef = ref<HTMLElement | null>(null);
const popRef = ref<HTMLElement | null>(null);
const popStyle = ref<Record<string, string>>({});

const selectedLabel = computed(() => props.options.find((o) => o.value === props.modelValue)?.label ?? '');

function place(): void {
  const el = triggerRef.value;
  if (!el) return;
  const r = el.getBoundingClientRect();
  const count = Math.max(props.options.length, 1);
  const h = Math.min(count * 34 + 10, 320);
  const m = 8;
  let top = r.bottom + 6;
  let isUp = false;
  if (top + h > window.innerHeight - m && r.top - 6 - h > m) {
    isUp = true;
    top = r.top - 6 - Math.min(h, r.top - m);
  }
  const width = Math.max(r.width, 200);
  let left = r.left;
  if (left + width > window.innerWidth - m) left = window.innerWidth - m - width;
  up.value = isUp;
  popStyle.value = { left: `${Math.round(left)}px`, top: `${Math.round(top)}px`, width: `${Math.round(width)}px` };
}

async function openPop(): Promise<void> {
  open.value = true;
  activeIdx.value = Math.max(props.options.findIndex((o) => o.value === props.modelValue), 0);
  place();
  await nextTick();
  (popRef.value?.querySelector('.vs-opt.on') as HTMLElement | null)?.scrollIntoView({ block: 'nearest' });
}

function toggle(): void {
  if (open.value) close();
  else openPop();
}

function close(): void {
  open.value = false;
}

function pick(value: string): void {
  close();
  if (value !== props.modelValue) {
    emit('update:modelValue', value);
    emit('change', value);
  }
}

function onKey(e: KeyboardEvent): void {
  if (!open.value) {
    if (e.key === 'ArrowDown' || e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      openPop();
    }
    return;
  }
  if (e.key === 'Escape') { e.preventDefault(); close(); }
  else if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
    e.preventDefault();
    const d = e.key === 'ArrowDown' ? 1 : -1;
    const n = props.options.length;
    if (n) activeIdx.value = (activeIdx.value + d + n) % n;
  }
  else if (e.key === 'Enter') {
    e.preventDefault();
    const o = props.options[activeIdx.value];
    if (o) pick(o.value);
  }
}

function onDocPointer(e: PointerEvent): void {
  const t = e.target as HTMLElement | null;
  if (!t) return;
  if (t.closest('.v2-vselect') === triggerRef.value?.closest('.v2-vselect')) return;
  if (popRef.value?.contains(t)) return;
  close();
}
function onScroll(): void {
  if (open.value) close();
}

onMounted(() => {
  document.addEventListener('pointerdown', onDocPointer, true);
  window.addEventListener('scroll', onScroll, true);
  window.addEventListener('resize', onScroll);
});
onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', onDocPointer, true);
  window.removeEventListener('scroll', onScroll, true);
  window.removeEventListener('resize', onScroll);
});
</script>

<template>
  <div class="v2-vselect" :class="{ block }" v-bind="$attrs">
    <button
      type="button" ref="triggerRef" class="v2-vselect-trigger" :class="{ open, up }"
      :title="selectedLabel || placeholder || ''" @click="toggle" @keydown="onKey"
    >
      <span class="vs-label" :class="{ ph: !selectedLabel }">{{ selectedLabel || placeholder || '请选择' }}</span>
      <svg class="vs-chev" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6" /></svg>
    </button>
    <Teleport to="body">
      <div v-if="open" ref="popRef" class="v2-vselect-pop" :style="popStyle" role="listbox">
        <button
          v-for="(o, i) in options" :key="o.value" type="button" role="option"
          class="vs-opt" :class="{ on: o.value === modelValue, act: i === activeIdx }"
          :aria-selected="o.value === modelValue"
          @click="pick(o.value)" @mousemove="activeIdx = i"
        >{{ o.label }}</button>
      </div>
    </Teleport>
  </div>
</template>

# 真应用无头测试台（Tauri IPC 模拟）

用法：让构建产物在无头 Edge 里以"假后端"完整运行，可复现/验证面板、下拉、数据加载等
纯静态截图测不了的问题（本工具曾抓到 QuickAdd 因 v-if 挂载导致 watch 永不触发、
打开即全空的真 bug）。

## 步骤（仓库根目录执行）

```bash
VITE_HASH=1 npx vite build --base=./ --outDir "../.zcode/tmp-mock/dist" --emptyOutDir
cd ../.zcode/tmp-mock
node ../../zhangji/tools/mock/patch.mjs        # 注入 Tauri 假 IPC（假分类/账户/流水）
node ../../zhangji/tools/mock/driver2.mjs      # 注入驱动（模式见下）+ 禁动画
node ../../zhangji/tools/mock/driver3.mjs      # 可选：弹层几何/计算样式调试输出
node ../../zhangji/tools/mock/driver4.mjs      # 可选：红底弹层 + ?flat=1 隐藏墙面
node ../../zhangji/tools/mock/driver5.mjs      # 可选：打开后强制整页重绘
"/c/Program Files (x86)/Microsoft/Edge/Application/msedge.exe" --headless=new --disable-gpu \
  --run-all-compositor-stages-before-draw --no-first-run --allow-file-access-from-files \
  --user-data-dir="<临时目录>" --window-size=1440,900 --virtual-time-budget=20000 \
  --screenshot="out.png" \
  "file:///<绝对路径>/tmp-mock/dist/index.html?mode=panel#/detail"
```

## 驱动模式（?mode=）

- `panel`：点第一条流水 → 打开编辑面板 → 展开一级分类弹层
- `transfer`：编辑面板切"转账" → 展开转出账户弹层
- `quickadd`：点侧栏"记一笔" → 切"转账" → 展开转出弹层
- `toolbar`：明细工具栏分类下拉展开

## 已知坑

- 弹层（晚创建的 fixed 图层）在 `--virtual-time-budget` 截图里可能不绘制——几何/样式/DOM
  都正确（driver3 可证），属无头合成器伪影，真机不受影响；以 dbg 读数（OPTS 数量）为准。
- `--timeout=真实毫秒` 不等驱动定时器，验证驱动必须用 virtual-time。
- 完成后 `dist/` 与 `u*` 目录可直接删除。

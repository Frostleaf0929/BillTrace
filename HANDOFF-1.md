# HANDOFF-1.md —— 交接文档（v0.3.0 UI 重设计阶段）

> 生成时间：2026-10-03 ｜ 交接原因：UI 重设计多轮未达预期，用户要求换会话继续
> 前置阅读：`AGENTS.md`（根目录）+ 本文件 + `logs\2026-10-03-图表与UI方向调研\LOG.md`
> 当前 Git：main @ 3e7f1e7 之后（f4ea1cd），已全部推送 GitHub；版本号 0.3.0（三处一致）

## 一、项目现状一句话

账痕 BillTrace v0.3.0：功能全部可用（记账/导入/分类/统计/预算/三级预设），UI 正处于"Zona Pro 风格改造"中途——色板已换血、组件已胶囊化，但**概览页布局与图表仍未达到用户给的参考标准**，这是下一会话的核心任务。

## 二、用户的参考（必须完整保留，改 UI 前先看）

### 设计规范（用户钦定，按优先级）
1. **主参考：Retail POS（Zona Pro）** https://dribbble.com/shots/27083535-Retail-POS-Web-UI-UX-Design
   - 精确色板：主紫 `#877FC1`、淡紫 `#DBD3F5`、炭黑 `#222026`、灰白底 `#F5F6FA`
   - 字体 Zona Pro（付费）；免费替代 Poppins **无中文字形，用户已决定暂不换字体**
   - 标志元素：柱状图"上期斜纹幽灵柱 + 当期实色 + 峰值深紫高亮"、深色锚点卡、图标芯片、白色胶囊 % 徽章、细竖条图标导航
2. **浅色辅助**：Finova https://dribbble.com/shots/26390511-Finova-Finance-Dashboard-UI（"轨道填充式"柱：每根柱后有全高浅灰圆角轨道；大数字统计卡；黑卡锚点）
3. **深色辅助**：Folia https://dribbble.com/shots/27262894-Folia-Finance-Dashboard-Design（**中性纯黑背景、卡片无渐变**、青柠强调色语义卡、当月高亮）
4. **图表（用户钦定，必做）**：FusionCharts 官网 "High-performance time-series charts" 区块的 Time-series with **Line（必须有）** 和 **Columns（必须有）**、Multivariate / Stock / Annotations（可选）。核心结构 = 主图 + **底部导航器（数据阴影拖拽条）** + 快捷区间页签 + 📅 日期范围胶囊 + 十字准线。

### 用户对概览页布局的划分（手绘标注，逐条）
- 主区第一卡 = **图表（月份显示，图四 Columns 双系列堆叠）** ← 已实现（近12月堆叠柱，见用户最新截图，形态正确）
- 账户余额卡保留；最近记录表保留；**预算提醒卡彻底删除**（预算只在预算页）← 已实现
- 右栏从上到下：**时间调节（日月年）→ 支出卡 → 收入卡 → 账户总额卡 → 操作按钮** ← 已实现但**用户截图显示右栏未渲染**（见坑 #1，需先排查）
- **设置按钮挪到侧栏底部**（月亮切换旁）← 已实现
- 深色模式 = **Folia 中性纯黑、无渐变、卡片无染色** ← 已实现
- **全局禁止渐变**（用户原话：渐变就没有好看的，除了毛玻璃加渐变）← 已实现（body 平涂）
- 浅色模式参考前三张图（Zona Pro）：注意浅色下可以有黑卡，但**不允许出现设计语言不统一的混搭**

### 用户明确否定过的做法（不要再犯）
- ❌ 分组并列柱（支出/收入并排）→ 要堆叠
- ❌ 饼图/环形图 → 彻底删除，已删
- ❌ 幽灵柱实验方案（统计页）→ 已废弃，用户否了
- ❌ 深色模式语义渐变染色卡 → 已废弃
- ❌ 概览两栏挪位（把操作卡挪进窄侧栏）→ 已撤销，用户要求按钮留在原位
- ❌ 任何"这里改点颜色那里改点什么"的零敲碎打 → 用户要的是**整体风格统一大改**

## 三、踩过的坑（血泪教训，按杀伤力排序）

1. **模板手术连锁失败**：对 OverviewView.vue 的多轮 python 正则手术互相踩踏——根因是 f965888 版本的模板里**根容器在最近记录卡后就闭合，三个对话框是根级兄弟节点**（Vue 3 多根合法），每次重组都把根闭合卷进网格导致右栏错位。教训：改模板结构前先用 `git show <commit>:file` 确认真实结构，不要凭记忆。
2. **div 深度计数器不可靠**：`<TransitionGroup tag="div">` 的闭合不是 `</div>`、多行属性的自闭合 `<div ... />`、行内 `<template v-if>` 都会让行级计数错位。f965888 原模板（渲染正常）用计数器测也是 -4。**结论：以 vue 编译器（npm run build）为唯一结构裁判**。
3. **vite 热更陈旧**：一个 .vue 文件一天被改十几轮后，HMR 会进入半新半旧状态——用户截图与代码对不上。**大改后必须完全重启 dev**。
4. **火绒实时防护**：会误杀新打包的未签名 exe（两次）；干扰 .git 对象写入（曾致对象库损坏，已用"临时克隆拷 pack"修复）。信任区已覆盖整个工作区目录。
5. **Git Bash 显示中文乱码**：文件名/内容显示乱但文件正常，用 PowerShell 验证。
6. **端口 1420 冲突**：账痕 dev 已改 **1430**（vite.config + tauri.conf），1420 留给了用户同时运行的拾刻（TallyMoment）热更。
7. **git 全局代理指向 127.0.0.1:7897**：代理软件没开时 push 会失败；gh CLI 不走该代理。
8. **cargo 网络**：需 `CARGO_HTTP_CHECK_REVOKE=false` + rsproxy 镜像（完整配方见 `环境搭建日志.md` 第 7 节）。
9. **dev 运行时跑 cargo test**：`target\debug\zhangji.exe` 被锁，尾部报 os error 5——与代码无关，核心结果在报错前已打出。
10. **token 消耗**：本会话大量 token 消耗在模板手术的反复失败上。新会话做 UI 时：**先写完整的目标模板再动手，不做增量正则手术**。

## 四、未完成的计划（按优先级）

### P0｜概览页收尾（下一会话第一件事）
1. **排查右栏不渲染**：代码里右栏存在（OverviewView.vue 的 ov-col 第二个），构建通过，但用户截图缺失——先完全重启 dev 复现；仍缺失则用 Vue Devtools/浏览器检查 DOM。
2. 概览图表卡按图四细化：堆叠柱已做，补"当月高亮 + 悬浮气泡"（Zona Pro 样式）。
3. 统计卡数值为 0 时的空态美化（用户截图里全是 ¥0.00，观感差）。

### P1｜其余页面按 POS 语言过一遍
- 分类页（树行样式）、详细页（表格/横幅）、设置页（已大体达标）逐页截图圈点。
- 深色模式全面走查（纯黑背景下所有页面对比度）。

### P2｜图表可选扩展
- FusionTime 可选三件：Multivariate（双轴）、Stock、Annotations——用户说"其余可有"。
- 统计页导航器的堆叠数据阴影（当前只画第一个系列轮廓）。

### P3｜功能借鉴（用户看着挑，来自 Actual/Maybe 调研）
- 周期性交易 Schedules、交易对账标记、CSV 导入向导、净值视图、交易标签、商户管理页——完整清单见 `logs\2026-10-03-图表与UI方向调研\LOG.md`。

### P4｜发布
- 全部验收后：确认版本号 0.3.0 → 打 tag → GitHub Actions 首跑自动发版 → Release 挂安装包。
- 打包前记得：`npm run tauri build -- --no-bundle` 出绿色版；NSIS 走 CI。

## 五、新会话开场白建议

「读取 zhangji/AGENTS.md 和 zhangji/HANDOFF-1.md，继续账痕 v0.3.0 的 UI 重设计。先读 HANDOFF 第二节的用户参考和第三节的坑，不要重复已否定的方案。第一件事：重启 dev 后排查概览右栏渲染。」

## 六、本次会话改动过的文件清单（全部已推送）

- `src/views/OverviewView.vue`（多轮：统计卡/锚点卡/图表/布局，当前为"主区图表+右栏堆叠"结构）
- `src/views/StatsView.vue`（FusionTime 趋势图 + 排行 + 色板）
- `src/views/DetailView.vue`、`SettingsView.vue`、`BudgetView.vue`（样式/交互修复）
- `src/lib/chartTheme.ts`（新增：图表 token 桥）
- `src/styles/global.css`、`src/App.vue`、`src/theme.ts`（16px 外框、去标题栏、色板、分段胶囊、设置下沉）
- `src-tauri/src/{stats.rs, models.rs, commands.rs}`（SummaryStats 上期对比字段）
- `src-tauri/tauri.conf.json`、`vite.config.ts`（0.3.0、窗口 1080×700、dev 端口 1430）
- `.github/workflows/release.yml`（产物名对齐）
- 工作区：`WORKLOG.md`、`logs\2026-10-0X-*\LOG.md`（全部过程档案）

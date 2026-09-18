# 账痕 · BillTrace

<p align="center">
  <img src="app-icon.png" width="120" alt="账痕图标" />
</p>

<p align="center">
  <strong>绿色免安装的个人账单管理桌面应用</strong><br/>
  导入即分类 · 指纹查重 · 三级分类体系 · 账户余额 · 毛玻璃深浅主题<br/>
  Tauri 2 · Vue 3 · Rust · SQLite
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-Windows%2010%2F11-blue" alt="platform" />
  <img src="https://img.shields.io/badge/framework-Tauri%202%20%2B%20Vue%203-24C8DB" alt="framework" />
  <img src="https://img.shields.io/badge/backend-Rust-DEA584" alt="backend" />
  <img src="https://img.shields.io/badge/license-MIT-green" alt="license" />
</p>

---

## 这是什么

**账痕**是一款 Windows 桌面端绿色单文件记账应用（约 8MB，双击即用，无需安装）。
把你的年度账本 Excel、微信支付流水直接拖进来，它会按照你自定义的**分类体系与商家规则**自动归类、自动查重，并用毛玻璃质感的界面帮你把每一笔钱的"痕迹"看清楚。

- 数据 100% 本地存储（SQLite），不上传任何服务器；
- 整个程序目录拷到 U 盘即可带走，数据随目录迁移；
- 支持从 Markdown / txt 导入你自己的分类体系预设。

## 核心功能

| 模块 | 能力 |
|------|------|
| 导入 | 随手记格式年度账本 Excel（支出/收入/转账/报销/代付/余额变更/债权变更 多 Sheet 自动识别）、微信支付官方流水 xlsx、手动记账；全量指纹查重，重复数据自动跳过 |
| 分类 | 支出 / 收入 / 账户三级体系（一级 → 二级 → 小级），拖拽移动层级；md / txt 分类预设导入（先预览后写入）与导出 |
| 自动分类 | 商家关键词规则引擎（优先级排序）；新商家弹窗归类后自动沉淀为规则，越用越聪明 |
| 概览 | 支出 / 收入 / 账户总额三卡（年/月/日切换联动）；账户余额卡片（余额 = 基数 + 交易净额），支持拖拽排序、增删、双击设基数；最近记录单击即改 |
| 详细 | 全部账目筛选 / 搜索 / 双击编辑 / 多选批量移动分类与删除；新商家待确认队列 |
| 统计 | 年 / 月 / 日趋势柱状图；按一级/二级分类、商家、账户、项目、成员聚合的饼图与条形排行；图表可导出 PNG |
| 个性化 | 深浅主题（跟随系统/浅色/深色，辐射动效切换）、6 种颜色预设 + 自定义色值、毛玻璃开关 / 强度 / 模糊半径、背景亮度、自定义背景图（填充/包含/拉伸/平铺） |
| 数据 | xlsx / csv 导出（与年度账本格式对齐）、分类体系导出 md / txt、一键备份、清除数据（自动先备份）、右缘 & 滚动换页 |

## 界面

深浅双主题，毛玻璃卡片 + 轻拟物阴影 + Swiss 排版，支持自定义背景图与颜色方案：

### 概览

支出 / 收入 / 账户总额三卡（年月日联动）、可拖拽排序的账户余额、单击即改的最近记录：

![概览](https://cdn.jsdelivr.net/gh/Frostleaf0929/BillTrace@main/docs/screenshots/overview.jpg)

### 分类

三级分类体系（一级 → 二级 → 小级），拖拽移动、点击行展开、md/txt 预设导入：

![分类](https://cdn.jsdelivr.net/gh/Frostleaf0929/BillTrace@main/docs/screenshots/categories.jpg)

### 详细 · 新商家归类

导入时遇到新商家弹窗归类，确认后自动沉淀为规则：

![详细](https://cdn.jsdelivr.net/gh/Frostleaf0929/BillTrace@main/docs/screenshots/detail-pending.jpg)

### 设置 · 个性化

背景图 + 毛玻璃强度 + 颜色方案，全部实时生效：

![设置](https://cdn.jsdelivr.net/gh/Frostleaf0929/BillTrace@main/docs/screenshots/settings.jpg)

## 开发

```bash
# 环境要求：Node.js ≥ 18、Rust stable (MSVC)、WebView2（Win10/11 自带）
npm install            # 安装前端依赖
npm run tauri dev      # 开发调试
npm run tauri build    # 打包（产物 src-tauri/target/release/zhangji.exe）
```

真实数据回归测试（检测到本地数据文件则运行，否则自动跳过）：

```bash
cd src-tauri && cargo test --test real_data -- --nocapture
```

## 项目结构

```
zhangji/
├── src/                        # Vue 3 前端
│   ├── views/                  # 概览 / 分类 / 详细 / 统计 / 设置 五页面
│   ├── components/             # 编辑弹窗、导入、预设、图表等组件
│   ├── theme.ts                # 主题与个性化外观管理
│   └── styles/global.css       # 全局样式（毛玻璃 / 拟物 / 动效）
└── src-tauri/                  # Rust 后端
    └── src/
        ├── importer.rs         # Excel/微信流水解析 + 规则引擎 + 查重 + 待确认队列
        ├── presets.rs          # md / txt 分类预设解析
        ├── rules.rs            # 商家关键词规则引擎
        ├── stats.rs            # 统计聚合（趋势 / 维度 / 账户余额）
        ├── exporter.rs         # xlsx / csv / 分类导出
        └── db.rs               # SQLite 初始化与迁移
```

## 隐私与安全

- 所有数据仅存放在程序同级 `data/` 目录（SQLite），无任何网络上传；
- 「清除数据」执行前会自动备份到 `data/backups/before-clear-*`；
- 深度安全扫描：0 发现（607 个依赖项无已知漏洞）。

## 致谢

- [Tauri](https://tauri.app/) · [Vue 3](https://vuejs.org/) · [Element Plus](https://element-plus.org/) · [ECharts](https://echarts.apache.org/) · [calamine](https://github.com/tafia/calamine) · [rusqlite](https://github.com/rusqlite/rusqlite)
- 界面风格借鉴 [Planshit/Tai](https://github.com/Planshit/Tai)
- 本项目为 **Vibe Coding** 实践，由 [ZCode](https://www.zcode.com) 智能体驱动，编码模型 **GLM**（Z.ai）完成全部开发

## 作者

**[Frostleaf0929](https://github.com/Frostleaf0929)**

## License

[MIT](./LICENSE)

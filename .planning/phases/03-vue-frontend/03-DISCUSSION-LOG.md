# Phase 3: Vue Frontend - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-13
**Phase:** 03-vue-frontend
**Areas discussed:** 布局样式, 视频导入方式, 种子展示样式, 空状态处理, 操作反馈策略, 批处理并发设置, 进度展示结构, 输出目录设置

---

## 布局样式

| Option | Description | Selected |
|--------|-------------|----------|
| 固定左右分栏 | 左右面板固定，中间可拖拽分隔条调整比例，默认 50/50 | ✓ |
| 标签页切换 | 种子管理和视频队列分标签页显示 | |
| 可折叠面板 | 左右面板可折叠/展开 | |

**User's choice:** 固定左右分栏 (推荐)
**Notes:** 双面板同时可见符合「拖入→选种子→处理」操作流，减少切换成本。

---

## 视频导入方式

| Option | Description | Selected |
|--------|-------------|----------|
| HTML5 拖拽区 | 明显拖拽热区 + 添加文件按钮调用文件选择器 | ✓ |
| 仅文件选择器 | 传统文件选择对话框 | |
| 系统托盘拖入 | 依赖操作系统原生拖放 | |

**User's choice:** HTML5 拖拽区 (推荐)
**Notes:** 拖入后立即刷新队列列表（import 命令返回已包含完整队列）。

---

## 种子展示样式

| Option | Description | Selected |
|--------|-------------|----------|
| 紧凑卡片 | 每种子一张卡片，hover 显示操作按钮，显示别名/操作摘要/创建时间 | ✓ |
| 列表/表格 | 传统表格行，始终可见操作按钮 | |
| 图标网格 | 大图标 + 种子名，点击展开详情 | |

**User's choice:** 紧凑卡片 (推荐)
**Notes:** 减少视觉噪声，hover 显示操作按钮；删除/重命名始终可访问。

---

## 空状态处理

| Option | Description | Selected |
|--------|-------------|----------|
| 引导式空状态 | 图标 + 引导文案 + 按钮直接触发操作 | ✓ |
| 静默空状态 | 仅显示提示文字，无操作按钮 | |
| 自动填充 | 首次启动自动生成示例种子 | |

**User's choice:** 引导式空状态 (推荐)
**Notes:** 种子空状态按钮触发 `generate_seed`；队列空状态按钮触发文件选择器。

---

## 操作反馈策略

| Option | Description | Selected |
|--------|-------------|----------|
| 分级确认 | 不可逆操作（删除种子、清空队列）用 NModal 确认；普通操作静默执行 | ✓ |
| 全部确认 | 所有操作都需要确认对话框 | |
| 全部静默 | 所有操作直接执行，无确认 | |

**User's choice:** 分级确认 (推荐)
**Notes:** 操作结果使用 Naive UI 的 `useMessage()` / `useNotification()` 轻量提示。

---

## 批处理并发设置

| Option | Description | Selected |
|--------|-------------|----------|
| 轻量选择器 | 开始按钮旁放 NSelect 下拉，选项 1/2/3/4，默认 1，持久化偏好 | ✓ |
| 滑块控制 | 滑块拖动选择并发数 | |
| 固定默认 | 不使用 UI 设置，始终默认 1 并发 | |

**User's choice:** 轻量选择器 (推荐)
**Notes:** 偏好通过 tauri-plugin-store 持久化。渐进式暴露复杂度。

---

## 进度展示结构

| Option | Description | Selected |
|--------|-------------|----------|
| 全局+逐文件 | 顶部横幅（已处理 N/总计 M）+ 队列中每个文件独立进度条 | ✓ |
| 仅全局 | 单一进度条显示整体进度 | |
| 仅逐文件 | 每个文件独立进度条，无汇总信息 | |

**User's choice:** 全局+逐文件 (推荐)
**Notes:** Phase 3 预设 UI 结构（占位），Phase 4 接入 `batch-progress` 实时事件流。

---

## 输出目录设置

| Option | Description | Selected |
|--------|-------------|----------|
| 独立设置行 | 始终显示当前路径 + 更改按钮（调用文件对话框） | ✓ |
| 埋在设置面板 | 输出目录放在独立设置页面 | |
| 处理时选择 | 每次开始处理时弹出选择 | |

**User's choice:** 独立设置行 (推荐)
**Notes:** 默认 `~/Videos/sandwich-output/`。处理控制区固定在右侧面板底部。

---

## Claude's Discretion

- Naive UI 组件选择和布局细节（NLayout, NGrid, NGi, NCard, NButton, NModal, NSelect, NProgress, NTag, NPopconfirm, useMessage, useNotification 等）
- Vue 组件文件拆分和命名
- Pinia store 结构——建议 seed store + queue store + batch store，但具体拆分由规划决定
- TypeScript 类型定义与 Rust 模型的对应（`#[serde(rename_all = "camelCase")]` 保证字段名一致）
- Composable 设计（`useSeed`, `useQueue`, `useBatch` 封装 `invoke()` 调用）
- 拖拽区域的 CSS 样式和视觉反馈
- 卡片 hover 效果的细节
- i18n 翻译 key 命名和组织

## Deferred Ideas

无——讨论全程在 Phase 3 范围内。

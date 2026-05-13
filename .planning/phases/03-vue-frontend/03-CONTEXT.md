# Phase 3: Vue Frontend - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning

<domain>
## Phase Boundary

使用 Naive UI 组件库和 Pinia 状态管理，构建暗色主题双面板界面。左侧管理种子（生成/列表/重命名/删除/复制），右侧管理视频队列（拖拽导入/元数据展示/移除/清空）和批处理（选择种子/输出目录/并发设置/启动处理）。通过 12 个 Tauri IPC 命令与 Phase 2 的 Rust 后端交互。

**不在范围内：** 实时进度（Phase 4）、处理完成摘要（Phase 4）、取消响应式反馈（Phase 4）、种子导出/导入（v2）。
</domain>

<decisions>
## Implementation Decisions

### 布局
- **D-01:** 固定左右分栏布局，中间可拖拽分隔条调整比例。默认 50/50。
- **D-02:** 右侧面板内部分为上下区域：上为拖拽导入区 + 视频队列列表，下为处理控制区（种子选择/输出目录/并发设置/开始按钮）。

### 视频导入
- **D-03:** HTML5 拖拽区——在视频队列上方设明显拖拽热区，接收视频文件拖入。同时提供「添加文件」按钮调用文件选择器（IMPORT-02）。
- **D-04:** 导入后立即刷新队列列表（调用 `list_seeds` / `get_queue` 已在 import 命令返回中包含完整队列）。

### 种子展示
- **D-05:** 紧凑卡片布局。每个种子一张卡片，显示：别名、操作摘要（如「波纹+抽帧+GOP」）、创建时间、操作按钮（重命名/复制/删除）。
- **D-06:** 操作按钮仅在 hover 时显示（减少视觉噪声），删除/重命名始终可访问。

### 空状态
- **D-07:** 引导式空状态。种子列表为空时显示图标 + 「生成第一个种子」按钮；队列为空时显示图标 + 「拖入视频或点击添加」引导文案。
- **D-08:** 空状态按钮直接触发对应操作（种子：调用 `generate_seed`；队列：调用文件选择器）。

### 操作反馈
- **D-09:** 分级确认——不可逆操作（删除种子、清空队列）使用 NModal 确认对话框；普通操作（生成种子、复制、重命名、导入、开始处理）静默执行。
- **D-10:** 操作结果使用 Naive UI 的 `useMessage()` / `useNotification()` 轻量提示（成功/失败）。

### 批处理控制
- **D-11:** 并发数设置——「开始处理」按钮旁放 NSelect 下拉，选项 1/2/3/4，默认 1。偏好通过 tauri-plugin-store 持久化（D-09）。
- **D-12:** 输出目录——独立设置行，始终显示当前路径 + 「更改」按钮（调用文件对话框）。默认 `~/Videos/sandwich-output/`（OUTPUT-01）。

### 进度展示（预设结构）
- **D-13:** 批处理进行中时，顶部显示横幅（已处理 N/总计 M），队列中每个文件显示独立进度条（占位，Phase 4 接入实时事件）。为 Phase 4 的 `batch-progress` 事件预留 UI 结构。

### Claude's Discretion
- Naive UI 组件选择和布局细节（NLayout, NGrid, NGi, NCard, NButton, NModal, NSelect, NProgress, NTag, NPopconfirm, useMessage, useNotification 等）
- Vue 组件文件拆分和命名
- Pinia store 结构——建议 seed store + queue store + batch store，但具体拆分由规划决定
- TypeScript 类型定义与 Rust 模型的对应（`#[serde(rename_all = "camelCase")]` 保证字段名一致）
- Composable 设计（`useSeed`, `useQueue`, `useBatch` 封装 `invoke()` 调用）
- 拖拽区域的 CSS 样式和视觉反馈
- 卡片 hover 效果的细节
- i18n 翻译 key 命名和组织
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Docs
- `.planning/ROADMAP.md` — Phase 3 目标、成功标准、依赖（Phase 2）
- `.planning/REQUIREMENTS.md` — UI-01（双面板布局）、UI-02（暗色主题）需求定义
- `.planning/PROJECT.md` — 技术栈、约束、关键决策
- `CLAUDE.md` — 完整技术栈版本表、IPC 模式（Pattern 1-3）、Naive UI 推荐组件列表

### Prior Phase Context
- `.planning/phases/01-foundation/01-CONTEXT.md` — Phase 1 锁定决策：Dark theme 配置（D-35）、i18n 设置（D-36）、Naive UI 选择（D-37）、Pinia 模式（D-38）
- `.planning/phases/02-rust-backend/02-CONTEXT.md` — Phase 2 锁定决策：种子生成策略（D-01~D-04）、持久化策略（D-05~D-07）、批处理模型（D-08~D-11）、视频导入限制（D-12~D-15）、输出管理（D-16）

### Phase 2 Rust Models (IPC contract)
- `src-tauri/src/models/seed.rs` — Seed, Operation, OperationType 类型定义
- `src-tauri/src/models/video.rs` — VideoEntry, VideoMetadata, VideoStatus 类型定义
- `src-tauri/src/models/batch.rs` — BatchConfig, BatchProgress, BatchResult 类型定义
- `src-tauri/src/state.rs` — AppState 结构（Mutex 包裹的 seeds, queue, batch_state）

### IPC Commands Reference
- `src-tauri/src/commands/seed.rs` — generate_seed, rename_seed, delete_seed, copy_seed, list_seeds
- `src-tauri/src/commands/queue.rs` — get_queue, remove_from_queue, clear_queue
- `src-tauri/src/commands/import.rs` — import_video
- `src-tauri/src/commands/batch.rs` — start_batch, cancel_batch, get_batch_status
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/stores/ffmpeg.ts` — Pinia Composition API store 模板（ref + computed + actions 模式）
- `src/composables/useFfmpeg.ts` — Composable 封装 `invoke()` + 事件监听的模式
- `src/types/ffmpeg.ts` — TypeScript 接口模板（camelCase 字段匹配 Rust serde）
- `src/App.vue` — NConfigProvider darkTheme 已全局配置，i18n locale 联动已就绪
- `src/components/FFmpegStatus.vue` — Naive UI 卡片 + 状态驱动渲染模板
- `src/components/FFmpegDownload.vue` — 进度条 + 事件监听模板

### Established Patterns
- **Store 模式：** `defineStore('name', () => { ... Composition API ... })` —— Pinia Composition API
- **Composable 模式：** `export function useXxx()` 封装 Tauri `invoke()` 和事件监听
- **IPC 调用：** 通过 composable 间接调用，不在组件中直接 `invoke()`
- **类型定义：** `src/types/` 目录，接口命名与 Rust 结构体对应，字段 camelCase
- **Naive UI：** 组件按需导入（tree-shakeable），无全局注册
- **图标：** lucide-vue-next，NIcon 包裹
- **样式：** 内联 scoped style + Tailwind 工具类（`bg-[#101014]`）

### Integration Points
- **Phase 2 → Phase 3:** 12 个 IPC 命令通过 `invoke()` 调用，Rust 结构体 `#[serde(rename_all = "camelCase")]` 保证前后端字段名一致
- **Phase 3 → Phase 4:** 批处理进度事件（`batch-progress`）的 UI 结构在 Phase 3 预设，Phase 4 接入实时事件流
- **事件监听：** `app.emit("event-name", payload)` 在 Rust 端，前端通过 Tauri `listen()` 在 composable 中监听
</code_context>

<specifics>
## Specific Ideas

- 种子卡片 hover 展示完整操作链详情（tooltip 或展开面板），平衡紧凑性与可理解性
- 拖拽区在用户拖文件悬停时高亮边框动画，提供即时视觉反馈
- 处理控制区固定在右侧面板底部（sticky），不随队列滚动
- 种子被选中时卡片高亮（border/background change），视觉区分选中/未选中状态
</specifics>

<deferred>
## Deferred Ideas

无——讨论全程在 Phase 3 范围内。
</deferred>

---

*Phase: 03-vue-frontend*
*Context gathered: 2026-05-13*

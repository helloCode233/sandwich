# Phase 6: 增强指纹修改 - Context

**Gathered:** 2026-05-16
**Status:** Ready for planning

<domain>
## Phase Boundary

在 v1.1 生产加固基础上，系统性增强指纹修改能力：新增 12+ 种 FFmpeg 操作类型（色彩处理/噪声纹理/几何微调/混合叠加四大类）、种子生成智能化升级（5-12 步 + 三档强度预设 + 70% 视频覆盖率保证）、种子管理增强（单文件 JSON 导出/导入）、v2 三项延迟项落地（拖拽排序/缩略图预览/处理日志历史面板）。

**不在范围内：** SEED-COMPLEX-01（不同视频用不同种子）暂缓、GPU 编码器手动选择（Phase 5 已延迟）、代码签名/商店上架、种子手动编辑、视频剪辑功能。
</domain>

<decisions>
## Implementation Decisions

### 新增操作类型
- **D-01:** 四类新操作全要：色彩处理（色相旋转/饱和度/亮度/色彩平衡等）、噪声纹理（颗粒噪点/模糊/锐化等）、几何微调（旋转 ≤1°/缩放 99%-101%/翻转）、混合叠加（半透明纯色/渐变/水印混合，opacity ≤0.15）。
- **D-02:** 每类至少 3 个具体操作变体，总计 12+ 新操作。结合现有 7 种，操作类型总量达到 19+。
- **D-03:** 安全约束从硬编码 clamp 改为三档可调——保守（参数偏安全下限、5-7 步）、标准（当前行为居中、6-9 步）、激进（参数接近上限、8-12 步）。用户一键选择全局强度，不暴露单个操作参数。
- **D-04:** 新增操作加入统一随机池，与现有 7 种操作一起随机抽取。用户不分类选择。
- **D-05:** 纯 FFmpeg 内置滤镜实现（geq, hue, eq, curves, noise, atadenoise, rotate, scale, transpose, overlay, colorbalance, colorchannelmixer 等），不依赖第三方滤镜库，不定制编译 FFmpeg。

### 种子智能化升级
- **D-06:** 操作步数从 3-7 步扩展到 5-12 步。步数是种子复杂度的重要随机维度。
- **D-07:** 全局三档强度预设——保守: 参数偏安全下限 + 5-7 步；标准: 参数居中 + 6-9 步；激进: 参数接近上限 + 8-12 步。种子模型新增 `strength_tier` 字段。
- **D-08:** 操作链顺序保持纯随机（不按管道阶段排序）。智能排序 ROI 不高——随机性本身就是指纹修改的核心。
- **D-09:** 操作链覆盖视频 ≥70% 时长。每个操作分配随机 start_frame/duration_frames（而非全部默认 0=全视频），最后校验累计覆盖率，不满足则重新随机化。FrameDrop 保持原有时间段属性。

### 种子管理增强
- **D-10:** 种子导出/导入——单文件 JSON 格式，包含完整字段（id/alias/operations/created_at/strength_tier）。
- **D-11:** UI 入口——种子卡片 hover 时在右下角显示导出/导入小图标按钮（在现有重命名/复制/删除按钮区域）。
- **D-12:** 导入时重新生成 UUID 和 created_at 时间戳，作为新种子加入列表。不匹配已有 ID（避免覆盖风险）。
- **D-13:** SEED-COMPLEX-01「不同视频使用不同种子」暂缓到后续阶段。当前多种子批处理（Phase 5）已满足主要需求。

### v2 延迟项落地
- **D-14:** PROD-01 拖拽排序——HTML5 drag-and-drop 在队列列表中支持拖拽重排。新顺序持久化到 store，批处理按列表顺序从上到下执行。
- **D-15:** PROD-02 缩略图预览——导入时通过 ffmpeg（`-ss 1 -vframes 1`）提取首帧缩略图，以 base64 编码存储在 VideoEntry 中并持久化。队列列表每行显示缩略图。
- **D-16:** PROD-03 处理日志历史——UI 内嵌日志面板（非独立 Modal），支持按日期/文件名/种子名搜索过滤，显示每次处理详情（时间/耗时/MD5 前后对比/成功失败状态/输出路径）。日志持久化到 store 中。

### 新操作权重分配
- **D-17:** 按操作大类分配随机权重（总池约 19+ 种操作）：
  - 数学叠加类（旧 3 种）：~15%
  - 色彩处理类（新）：~20%
  - 噪声纹理类（新）：~15%
  - 几何微调类（新）：~15%
  - 混合叠加类（新）：~10%
  - 其余旧类别（像素平移/抽帧/GOP/元数据/音频/重封装，6 种）：共 ~25%
  - 大类内部子操作均分。精确数值由规划阶段确定。

### UI 布局
- **D-18:** 增量扩展现有双面板布局——左面板种子列表 + 强度选择器 + 导出/导入按钮；右面板队列列表 + 缩略图 + 拖拽手柄 + 日志 Tab。不引入第三面板或抽屉式侧栏。

### 存量种子兼容
- **D-19:** 启动时自动迁移所有存量种子到新格式——补充 `strength_tier: "standard"` 默认字段。旧种子操作不变（保留原 7 种类型和参数）。迁移后的种子标记为「已升级」，用户可正常使用或删除重建。
- **D-20:** 新生成的种子（Phase 6 起）同时包含新旧操作类型，格式自带 strength_tier 字段。新格式向后兼容——旧版应用读取新种子时忽略未知字段（serde `#[serde(default)]`）。

### Claude's Discretion
- 每类新操作的具体 FFmpeg 滤镜选择和参数范围定义
- 各大类内部子操作类型的权重细化数值
- 三档强度下各操作参数的具体插值/映射规则
- 覆盖率 ≥70% 的校验算法和重试策略
- HTML5 drag-and-drop 排序的具体实现（draggable/v-model 联动）
- 缩略图 base64 分辨率/文件大小限制策略
- 日志面板的 UI 细节（Tab 位置、过滤控件、统计摘要）
- i18n 新增 key（强度档位、新操作名称、导出/导入、缩略图、日志等）
- 自动迁移脚本的具体实现逻辑
- Seed 模型扩展字段（`strength_tier: "conservative" | "standard" | "aggressive"`）
- VideoEntry 模型扩展字段（`thumbnail_base64: Option<String>`, `order_index: u32`）
- OperationType 枚举扩展（新增 12+ 个变体）
- 权重配置是否可通过配置文件调整
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Foundation
- `.planning/PROJECT.md` — 核心价值、技术栈约束、关键决策
- `.planning/REQUIREMENTS.md` — v2 需求定义（PROD-01~03, SEED-EXPORT-01~02）
- `.planning/ROADMAP.md` — Phase 6 目标和依赖关系

### Prior Phase Context
- `.planning/phases/05-production-hardening/05-CONTEXT.md` — Phase 5 锁定决策：GPU 自动检测（D-04/05）、多种子批处理（D-09~D-12）、MD5 校验（D-13~D-15）
- `.planning/phases/03-vue-frontend/03-CONTEXT.md` — Phase 3 锁定决策：双面板布局（D-01/02）、种子卡片展示（D-05/06）、状态管理（Pinia Composition API）
- `.planning/phases/02-rust-backend/02-CONTEXT.md` — Phase 2 锁定决策：种子生成策略（D-01~D-04）、安全约束（SEED-04）、批处理并发（D-08~D-11）

### FFmpeg Core (Phase 2)
- `src-tauri/src/ffmpeg/filters.rs` — 现有 7 种操作的 FFmpeg 滤镜链构建，按 OperationType 分发。Phase 6 新增操作需在此模块扩展
- `src-tauri/src/ffmpeg/executor.rs` — 单文件处理流水线：滤镜合并（vf/af 链）、GPU 编码器注入、进度事件、取消支持。新增操作通过 FilterKind 机制自动融合
- `src-tauri/src/ffmpeg/probe.rs` — ffprobe 元数据提取（用于缩略图生成参考）

### Rust Models (Phase 2)
- `src-tauri/src/models/seed.rs` — Seed, Operation, OperationType 类型定义。Phase 6 需扩展 OperationType 枚举（12+ 变体）+ Seed 加 strength_tier 字段
- `src-tauri/src/models/video.rs` — VideoEntry 结构。Phase 6 需加 thumbnail_base64 + order_index
- `src-tauri/src/models/batch.rs` — BatchResult, PerFileProgress 类型。可能需扩展日志相关字段

### Seed Generation (Phase 2)
- `src-tauri/src/commands/seed.rs` — 种子生成/CRUD + 加权随机 + 安全约束。Phase 6 核心改动点

### Frontend State (Phase 3)
- `src/stores/seed.ts` — 种子 Pinia store。需扩展 strength_tier + 导出导入方法
- `src/stores/batch.ts` — 批处理 store。日志面板状态管理
- `src/stores/queue.ts` — 队列 store。需支持拖拽排序持久化

### Frontend UI (Phase 3-4)
- `src/components/seed/SeedCard.vue` — 种子卡片。需增加强度标识 + 导出/导入按钮
- `src/components/batch/BatchControls.vue` — 批处理控制。强度选择器位置候选
- `src/components/batch/BatchSummary.vue` — 批处理摘要。日志面板参考布局
- `src/components/batch/BatchBanner.vue` — 批处理横幅

### Type Definitions
- `src/types/batch.ts` — BatchResult/PerFileProgress TypeScript 类型。日志和缩略图相关扩展
- `src/types/seed.ts` — Seed 接口定义

### i18n
- `src/locales/zh-CN.json` — 中文翻译，需新增 Phase 6 所有 key
- `src/locales/en.json` — 英文翻译，同步新增
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **FFmpeg executor (`executor.rs:38-205`):** 单文件处理流水线已完整——滤镜合并（FilterKind 机制）、GPU 编码器注入、进度事件发射、取消支持。新增操作通过 `build_filter_args_separated()` 返回的 `FilterKind::VideoFilter`/`AudioFilter`/`Other` 自动融合到命令中，无需改动 executor 核心逻辑。
- **Filter dispatch (`filters.rs:123-133`):** `build_filter_args()` 和 `build_filter_args_separated()` 通过 match OperationType 分发。新增操作只需在 OperationType 枚举加变体 + 实现对应的 builder 函数 + 在 match 分支中添加。
- **Seed generation (`commands/seed.rs:34-73`):** `generate_seed()` 使用 `pick_operation_type()` 加权随机 + `generate_operation()` 参数生成。新操作：扩展权重表 + 添加参数生成分支。
- **BatchControls (`BatchControls.vue`):** 已有种子选择器（NSelect multiple）、并发设置、输出目录。强度档位选择器可自然放置在此区域。
- **SeedCard (`SeedCard.vue`):** 已有 hover 操作按钮（重命名/复制/删除）。导出/导入按钮在此扩展。
- **Queue components:** `ImportZone.vue` + `QueueList.vue` 已有拖拽导入（文件拖入）。HTML5 拖拽排序在同一区域实现，缩略图在 QueueList 行内添加。
- **i18n 模式:** `zh-CN.json` + `en.json` 按键值对，前端通过 `$t()` / `useI18n()` 引用。

### Established Patterns
- **IPC 模式:** Rust `#[tauri::command]` → 前端 `invoke()`。进度/状态通过事件流（`batch-file-progress`, `batch-progress`, `seeds-updated`）推送。
- **Pinia Composition API:** `defineStore('name', () => { ref + computed + actions })` 模式。Phase 6 所有新状态沿用。
- **Naive UI 组件树:** NSelect/NButton/NIcon/NCard/NProgress/NTag/NModal/NPopconfirm/NTabs/NScrollbar 已建立使用模式。
- **ffmpeg-sidecar CLI 执行:** `FfmpegCommand::new_with_path()` → `.args()` → `.spawn()` → `.iter()` 消费进度。缩略图提取为单次短命令。
- **serde 序列化:** `#[serde(rename_all = "camelCase")]` 保证前后端字段名一致。新字段需加 `#[serde(default)]` 保证向后兼容。

### Integration Points
- **Seed 模型 → 前后端类型:** Rust `Seed` struct 通过 `serde` JSON 序列化传递到 TypeScript `Seed` interface。新增字段需两端同步。
- **OperationType 枚举:** 在 Rust 和 TypeScript 两侧都需扩展。Rust 侧增 match 分支；TS 侧增 union 类型。
- **executor → new filters:** 现有 `FilterKind` 三元组（VideoFilter/AudioFilter/Other）已支持任意滤镜合并。新操作只需实现 builder，executor 自动处理冲突消解（-c copy vs filtering）。
- **store → UI:** 种子 store 的 `selectedSeedIds` 已支持多选（Phase 5）。强度档位是种子级属性，不影响选择逻辑。
- **Progress events:** `PerFileProgress` 已包含 `seed_alias` 字段（Phase 5）。Phase 6 沿用此结构。
</code_context>

<specifics>
## Specific Ideas

- 用户强调操作链覆盖视频 ≥70% 时长——这是硬性约束，重新随机直到满足
- 强度分三档（保守/标准/激进），不是连续滑块——用户偏好简单选择
- 操作顺序不排序——用户确认随机性 > 管道优化
- 缩略图在导入时提取（首帧 base64），不是按需——用户偏好即时展示
- 日志面板是内嵌 Tab（不是独立 Modal 或抽屉）——用户偏好增量扩展
- 存量种子启动时自动迁移——不区分新旧版本，无用户操作
- 滤镜纯内置——不依赖第三方，保持 ffmpeg-sidecar auto_download 兼容
</specifics>

<deferred>
## Deferred Ideas

- SEED-COMPLEX-01（不同视频使用不同种子）→ 后续阶段。当前多种子批处理已满足主需求
- 连续强度滑块（1-10）→ 当前选择三档预设，后续可根据用户反馈升级
- 操作链管道阶段智能排序 → 当前保持随机，后续按需添加
- 多帧预览条（hover 时显示多个时间点缩略图）→ 当前首帧即可
- 日志统计仪表板（成功率/最常用种子等）→ 后续迭代
- 权重配置 UI（用户自定义各类权重）→ 后续阶段
- GPU 编码器手动选择界面（Phase 5 D-06）→ 后续
- 代码签名/商店上架 → 后续独立阶段

</deferred>

---

*Phase: 06-增强指纹修改*
*Context gathered: 2026-05-16*

# Phase 07: Audio, Crop, Metadata & Duration - Context

**Gathered:** 2026-05-18
**Status:** Ready for planning

<domain>
## Phase Boundary

在 Phase 6（20 种操作类型 + 三档强度 + 种子导出导入 + v2 延迟项）基础上，四维度增强指纹修改：

1. **音频操作增强** — 新增 ~5 种音频操作类型（重采样、独立音量、音调偏移、EQ、声道操作），替换/拆分现有 AudioTweak
2. **轻微裁切作为默认** — 每个种子强制包含裁切操作（四边随机不对称 0.5%-3%，缩回原尺寸），同时加入随机池
3. **元数据精细化** — 写入假元数据 + 按类别选择性擦除 + 保留现有全擦除 = 3 种元数据操作
4. **时长修改** — 视频变速（setpts+atempo 同步）+ 首尾微裁剪

外加 **FrameDrop 升级为默认操作** — 恢复真正丢帧（framestep），每个种子强制包含 + 跟随强度档位。

**不在范围内：** SEED-COMPLEX-01（不同视频用不同种子，Phase 6 已延迟）、GPU 编码器手动选择、代码签名/商店上架。
</domain>

<decisions>
## Implementation Decisions

### 音频操作 (Audio Operations)
- **D-01:** ~5 个新音频操作类型：Resample（重采样）、Volume（独立音量）、Pitch（音调偏移）、EQ（均衡器）、Channel（声道操作）。替换/拆分现有 AudioTweak 的三个子效果（volume/tempo/echo）。
- **D-02:** 保守安全约束：Pitch ±2 semitones, speed 0.97-1.03x, volume ±3dB, resample 22050-48000 Hz。纯 FFmpeg 内置滤镜实现（aresample, volume, rubberband/asetrate, equalizer, channelmap 等）。
- **D-03:** 重采样使用随机范围 22050-48000 Hz（非固定标准采样率）。

### 裁切默认操作 (Crop — Default Operation)
- **D-04:** 裁切是每个种子的强制操作 — 每个种子必定包含一个裁切 + 随机池中也可被额外抽到（双重保障）。裁切不计入操作步数。
- **D-05:** 每边 0.5%-3%，四边随机不对称（上下左右各自独立随机值）。
- **D-06:** 裁切后缩放回原始分辨率（crop + scale filter chain，等比拉伸）。
- **D-07:** 跟随三档强度：保守 0.5-1.5%，标准 1-2.5%，激进 2-3.5%。
- **D-08:** 独立 OperationType 变体 — 有自己的 filter builder 和参数生成逻辑。

### 元数据操作 (Metadata Operations)
- **D-09:** 3 个元数据操作类型：MetadataWrite（写入假元数据字段）、MetadataSelectiveErase（按类别选择性擦除）、保留现有 MetadataErase（全擦除）。
- **D-10:** 假元数据覆盖常用字段：creation_time, title, author, comment, copyright, encoder。
- **D-11:** creation_time 在原时间 ±30 天范围内随机偏移；title/author/comment 等文本字段从预定义词库中随机选取。
- **D-12:** 选择性擦除按类别：时间类（creation_time, modify_date）、设备类（camera, make, model）、描述类（title, comment, author, copyright）。随机选 1-3 类擦除。
- **D-13:** 元数据操作不跟随强度档位 — 各档行为一致。

### 时长修改 (Duration Modification)
- **D-14:** 2 个操作类型：VideoSpeed（setpts 视频变速 + atempo 音频同步）和 TrimEdges（trim 过滤器裁剪首尾）。
- **D-15:** VideoSpeed 范围 0.95-1.05x（±5% 速度变化）。
- **D-16:** TrimEdges 随机选择裁剪头部、尾部或两端同时，裁剪 1-30 帧。

### FrameDrop 升级为默认操作
- **D-17:** FrameDrop 恢复真正丢帧（framestep 每隔 N 帧丢弃 1 帧），替换当前 setpts 微时序抖动方案。
- **D-18:** 丢弃间隔 30-50 帧（每 30-50 帧丢 1 帧）。
- **D-19:** FrameDrop 与裁切同为默认操作 — 每个种子强制包含 + 随机池中可额外抽到。跟随三档强度（保守 40-50 帧间隔，标准 30-45 帧，激进 25-35 帧）。

### Claude's Discretion
- 新音频 OperationType 变体的具体名称和数量
- 每个新音频操作的 FFmpeg 滤镜选择和参数映射
- 假元数据词库的具体内容
- 元数据类别分配的字段映射
- 裁切 filter builder 的 crop+scale 组合实现
- VideoSpeed 的 setpts/atempo 同步机制
- FrameDrop 的 framestep 实现细节
- 随机池权重重新分配（纳入所有新操作类型）
- Step count 调整（默认操作不占步数时，总步数下限处理）
- i18n 新增 key（新操作名称、裁切、元数据字段等）
- OperationType 枚举扩展（Rust + TypeScript 双侧同步）
- 存量种子迁移逻辑（Phase 6 D-19 已建立迁移模式）
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Foundation
- `.planning/PROJECT.md` — 核心价值、技术栈约束、关键决策
- `.planning/REQUIREMENTS.md` — v2 需求定义
- `.planning/ROADMAP.md` — Phase 7 目标和依赖关系

### Prior Phase Context (locked decisions)
- `.planning/phases/06-/06-CONTEXT.md` — Phase 6 锁定决策：统一随机池（D-04）、三档强度（D-03/D-07）、纯内置滤镜（D-05）、覆盖率 ≥70%（D-09）、UI 增量扩展（D-18）、存量种子迁移模式（D-19/D-20）、权重分配（D-17）
- `.planning/phases/05-production-hardening/05-CONTEXT.md` — Phase 5 锁定决策：GPU 自动检测（D-04/05）、多种子批处理（D-09~D-12）、MD5 校验（D-13~D-15）
- `.planning/phases/03-vue-frontend/03-CONTEXT.md` — Phase 3 锁定决策：双面板布局（D-01/02）、Pinia Composition API 状态管理模式

### Core Source Files (Phase 6 execution output)
- `src-tauri/src/models/seed.rs:95-174` — Operation, OperationType (20 variants), StrengthTier 类型定义。Phase 7 需新增 10+ 变体
- `src-tauri/src/ffmpeg/filters.rs:1-492` — 现有 20 种 filter builder + FilterKind 分发 + build_filter_args/build_filter_args_separated。Phase 7 核心扩展点
- `src-tauri/src/commands/seed.rs:1-500` — 种子生成：pick_operation_type 加权随机 + generate_operation 参数生成 + generate_seed 覆盖率验证。Phase 7 核心改动点
- `src/types/seed.ts:1-44` — TypeScript 类型定义（Seed, Operation, OperationType union）

### FFmpeg Execution
- `src-tauri/src/ffmpeg/executor.rs` — 单文件处理流水线：滤镜合并（vf/af 链）、GPU 编码器注入、进度事件、取消支持
- `src-tauri/src/ffmpeg/probe.rs` — ffprobe 元数据提取

### Frontend
- `src/stores/seed.ts` — 种子 Pinia store
- `src/stores/batch.ts` — 批处理 store
- `src/components/seed/SeedCard.vue` — 种子卡片（操作类型展示需扩展）
- `src/locales/zh-CN.json` — 中文翻译，需新增 Phase 7 所有 key
- `src/locales/en.json` — 英文翻译，同步新增
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Filter dispatch (`filters.rs:352-492`):** `build_filter_args()` 和 `build_filter_args_separated()` 通过 match OperationType 分发。新增操作只需：OperationType 加变体 → 实现 builder 函数 → 在 match 分支添加。VideoFilter/AudioFilter/Other 三元组已支持任意滤镜合并。
- **Seed generation (`commands/seed.rs:13-45`):** `pick_operation_type()` 使用 1000-bucket 权重方案。新操作：扩展 match 分支 + 重新分配权重。`generate_operation()` 为每个 OperationType 生成参数（含 tier-driven 范围）。
- **Strength tier pattern:** 每个操作的参数生成段都按 Conservative/Standard/Aggressive 三分支定义范围（`seed.rs:218-490`）。新操作遵循相同模式。
- **Seed generation flow:** `generate_seed()` → 随机步数 → loop pick_operation_type + generate_operation → 覆盖率验证。默认操作（crop/FrameDrop）需在循环前预注入。
- **i18n 模式:** `zh-CN.json` + `en.json` 按键值对，前端通过 `$t()` 引用。

### Established Patterns
- **IPC 模式:** Rust `#[tauri::command]` → 前端 `invoke()`。进度/状态通过事件流推送。
- **Pinia Composition API:** `defineStore('name', () => { ref + computed + actions })` 模式。
- **Naive UI 组件树:** NSelect/NButton/NIcon/NCard/NProgress/NTag/NModal 已建立使用模式。
- **serde 序列化:** `#[serde(rename_all = "camelCase")]` 保证前后端字段名一致。新字段需 `#[serde(default)]` 保证向后兼容。

### Integration Points
- **OperationType 枚举:** Rust 和 TypeScript 双侧需同步扩展。Rust 侧增 match 分支（filter dispatch + seed generation）；TS 侧增 union 类型。
- **executor → filters:** 现有 FilterKind 三元组已支持任意滤镜合并。新音频操作返回 AudioFilter，新视频操作返回 VideoFilter。
- **种子生成 → 默认操作:** 当前 `generate_seed()` 纯随机抽取。需加 pre-inject 逻辑（在随机循环前插入 crop + FrameDrop）。
- **存量种子:** Phase 6 D-19 已建立自动迁移模式。Phase 7 新增字段需 `#[serde(default)]` + 启动时迁移。
</code_context>

<specifics>
## Specific Ideas

- 裁切是"默认"操作 — 每个种子必定有，不是概率性的。同时也可以被随机抽到第二个裁切
- FrameDrop 恢复真正的丢帧行为 — Phase 6 的 setpts 抖动方案被用户否决
- 音频操作维持保守范围 — 用户不希望明显的音质变化
- 元数据不跟随强度档位 — 与大多数其他操作不同，用户明确不要
- 裁切后缩回原分辨率 — 用户不希望输出视频尺寸变化
- 所有新操作纯 FFmpeg 内置滤镜 — 保持 Phase 6 D-05 约束
</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope. All prior deferred items (SEED-COMPLEX-01, GPU encoder manual selector, code signing, etc.) remain deferred from earlier phases.
</deferred>

---

*Phase: 07-audio-crop-meta*
*Context gathered: 2026-05-18*

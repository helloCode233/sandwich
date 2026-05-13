# Phase 2: Rust Backend - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning

## Phase Boundary

所有领域操作——种子生成、视频导入+元数据提取、视频队列管理、FFmpeg 批处理（含失败隔离和取消）——通过类型化 Tauri IPC 命令暴露，由 Rust 管理权威状态。

## Implementation Decisions

### 种子生成策略

- **D-01:** 纯随机生成，用户不可编辑操作链参数。操作链对用户透明但不可修改，仅显示摘要（如「波纹+抽帧+GOP」）。后续支持种子复制再随机化。
- **D-02:** 7 种操作类型加权随机——数学叠加类（波纹/条纹/同心圆）权重最高（~30%），其余类型均匀分配。原因：数学叠加对指纹修改最有效且视觉副作用最小。
- **D-03:** 操作步数 3-7 步随机。步数是关键随机维度之一，跨度提供足够多样性。
- **D-04:** 自动别名默认使用时间戳格式。用户可后续手动重命名（SEED-05）。

### 状态持久化

- **D-05:** 全部持久化——种子列表、视频队列（含 ffprobe 元数据）、输出目录偏好、并发数偏好——通过 tauri-plugin-store 持久化。重启后恢复到关闭前状态。
- **D-06:** 队列中视频路径检查有效性。已移动/删除的文件标记为失效（保留元数据，提示用户文件不可用）。
- **D-07:** 崩溃恢复：应用重启后检测到上次处理未完成，队列标记为「待处理」状态，已完成输出文件保留。用户手动重新开始批处理。

### 批处理

- **D-08:** 用户可选并发数，范围 1-4，默认 1 个并发。默认 1 的原因：单个 FFmpeg 进程占用 ~500MB-1GB 内存，1 并发确保所有机器稳定运行。
- **D-09:** 并发偏好持久化。用户调整后记住选择。
- **D-10:** 取消行为：终止所有 FFmpeg 进程，清理当前正在写入的不完整输出文件。已完成文件保留。队列回到待处理状态。
- **D-11:** 单文件失败隔离——一个文件处理失败自动跳过，其余文件继续处理。

### 视频导入

- **D-12:** 支持格式：mp4, mov, avi, mkv, webm, flv, wmv（FFmpeg 原生支持的用户常用格式）。通过扩展名过滤 + ffprobe 最终验证。
- **D-13:** 无文件大小硬性上限。仅检查磁盘可用空间是否足够输出。
- **D-14:** 导入时 ffprobe 校验文件有效性（可读取视频流）。无效文件拒绝导入并提示具体错误。
- **D-15:** 允许同一文件路径重复导入队列。用户可能想用不同种子处理同一源文件多次。

### 输出管理

- **D-16:** 命名冲突自动添加数字后缀。若 `{原文件名}_{种子别名}.mp4` 已存在，变为 `{原文件名}_{种子别名}-1.mp4`、`-2.mp4` 等。静默处理，不打断批处理。

### Claude's Discretion

- 加权分布的精确数值（除数学叠加类 ~30% 外，其余 6 类的具体权重百分比）
- 安全约束参数的具体值（透明度 ≤0.15, 平移 ≤3px, 抽帧间隔 ≥15 已由 REQUIREMENTS.md SEED-04 锁定）
- ffprobe 元数据解析的具体字段和实现方式
- 7 种操作类型对应的 FFmpeg 滤镜链构建方式
- Rust 模块组织结构（commands/ 文件拆分粒度）
- IPC 命令命名和签名设计

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Docs

- `.planning/ROADMAP.md` — Phase 2 目标、成功标准、依赖关系（Phase 1）
- `.planning/REQUIREMENTS.md` — SEED-01~06, IMPORT-01~02, QUEUE-01~02, BATCH-01/03/04, OUTPUT-01~02 需求定义
- `.planning/PROJECT.md` — 技术栈、约束、关键决策
- `CLAUDE.md` — 完整技术栈版本表、IPC 模式（Pattern 1-3）、FFmpeg 分发策略

### Prior Phase Context

- `.planning/phases/01-foundation/01-CONTEXT.md` — Phase 1 所有锁定决策（D-01~D-39），特别是 D-16（FFmpeg+FFprobe 已可用）、D-24（tauri-plugin-store 持久化模式）、IPC 模式

### Domain Reference

- `.claude/skills/video-fingerprinting/SKILL.md` — 六类 FFmpeg 操作的指纹修改方法（领域背景参考，若存在）

## Existing Code Insights

### Reusable Assets

- `src-tauri/src/commands/ffmpeg.rs` — FFmpeg 检测和校验逻辑。`detect_ffmpeg_internal()` 和 `verify_ffmpeg()` 可在此阶段直接调用。
- `src-tauri/src/commands/download.rs` — 取消机制（`AtomicBool` + `OnceLock<Mutex<GlobalDownloadState>>`）可作为批处理取消的参考模式。
- `src/types/ffmpeg.ts` — 前端类型定义模板。`#[serde(rename_all = "camelCase")]` 模式确保 Rust 结构体与 TypeScript 接口字段名匹配。

### Established Patterns

- **IPC 命令模式：** `#[tauri::command] async fn` → `tauri::generate_handler![]` → `invoke_handler()`。Phase 2 新增命令遵循相同模式注册到 `lib.rs`。
- **事件流：** 使用 `app_handle.emit("event-name", payload)` 向前端推送进度。Phase 2 批处理进度事件沿用此模式。
- **持久化：** `app_handle.store("sandwich-config.json")` + `.get()/.set()`。Phase 2 种子和队列存储使用同一 store 实例或独立 store 文件。
- **共享状态：** `tokio::sync::Mutex` + `std::sync::atomic::AtomicBool`。Phase 2 批处理状态管理沿用此模式。
- **模块结构：** `src-tauri/src/commands/mod.rs` re-export 模式，每个功能域一个文件。

### Integration Points

- **Phase 1 → Phase 2:** Phase 1 的 FFmpeg 检测/下载/校验命令已注册并可用。Phase 2 假设 FFmpeg 已就绪（用户已通过 Phase 1 UI 完成检测或下载）。
- **Phase 2 → Phase 3:** Phase 2 的 IPC 命令直接在 Phase 3 前端通过 `invoke()` 调用。Rust 结构体的 `#[serde(rename_all = "camelCase")]` 确保前端 TypeScript 接口可直接使用。
- **ffmpeg-sidecar:** `FfmpegCommand::new().arg()...spawn()` API 用于构建和执行 FFmpeg 命令。Phase 1 已安装 v2.5.1。

## Specific Ideas

- 种子操作链对用户透明但需展示摘要——平衡自动化与可理解性
- 批处理并发度从保守默认开始（1），让用户自行调高——渐进式暴露复杂度
- 输出文件自动后缀避免冲突——优先安静完成而非中断用户
- 导入时立即 ffprobe 校验——尽早发现无效文件，避免批处理中途失败

## Deferred Ideas

无——讨论全程在 Phase 2 范围内。

---

*Phase: 2-Rust Backend*
*Context gathered: 2026-05-13*

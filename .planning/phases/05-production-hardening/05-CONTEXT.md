# Phase 5: Production Hardening - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning

<domain>
## Phase Boundary

在 v1.0 功能完整的基础上进行生产加固：跨平台打包（Windows/Linux 安装包 + CI 矩阵构建）、GPU 硬件加速编码 + 并行 pipeline 优化、多种子批处理（一个视频 × N 个种子 = N 个输出）、MD5 完整性校验链（处理前后对比 + 摘要展示）。

**不在范围内：** macOS 打包（已有 CI）、代码签名/商店上架、GPU 编码器手动配置界面、种子导出/导入（v2）、拖拽排序（v2）、缩略图预览（v2）、处理日志历史（v2）。
</domain>

<decisions>
## Implementation Decisions

### 跨平台打包
- **D-01:** Windows 目标格式：`.msi`（Tauri 默认）和 `.exe`（NSIS 安装器）。Linux 目标格式：`.AppImage` 和 `.deb`。
- **D-02:** CI 通过 GitHub Actions 矩阵构建（`os: [macos-latest, ubuntu-latest, windows-latest]`），产物自动上传到 Release。FFmpeg 通过 ffmpeg-sidecar `auto_download()` 按平台获取，不随安装包分发。
- **D-03:** Tauri 现有 macOS 构建经验（本地 `cargo tauri build` 产出 `.dmg`）作为 Windows/Linux 配置的对照模板。

### GPU 加速
- **D-04:** 启动时自动检测可用 GPU 编码器：macOS → VideoToolbox (`hevc_videotoolbox`/`h264_videotoolbox`)，Windows → NVENC (`h264_nvenc`/`hevc_nvenc`) + AMF (`h264_amf`)，Linux → VAAPI (`h264_vaapi`)。
- **D-05:** 编码器自动选择，用户无感知。GPU 编码启动失败时静默回退 CPU (`libx264`)，不中断批处理。
- **D-06:** 不提供手动编码器选择 UI——当前阶段保持简洁，后续可按需添加。

### 并行 Pipeline 优化
- **D-07:** 调度器优化：当前并发模型已验证可用，重点减少 worker 线程空等时间（batch dispatch 预取下一文件、移除不必要的同步屏障）。
- **D-08:** 流式 I/O：FFmpeg 输入/输出已在 executor 中流式传递（`child.iter()` 逐帧消费 stderr），进一步确保 tokio 线程不被大缓冲区阻塞。

### 多种子批处理
- **D-09:** 种子选择器从单选（NSelect `:value`）改为多选（NSelect `multiple` 或 NCheckbox 组）。`seedStore.selectedSeedId` → `seedStore.selectedSeedIds: string[]`。
- **D-10:** 一个视频 × N 个种子 = N 个输出文件。3 个种子 × 10 个视频 = 30 个输出文件。
- **D-11:** 输出文件平铺在同一目录（不分种子子目录）。命名沿用现有规则：`{原文件名}_{种子别名}.{扩展名}`，冲突时追加 `-1`/`-2`。
- **D-12:** Rust 命令 `start_batch` 的 `seed_id: String` 参数改为 `seed_ids: Vec<String>`。处理循环改为双层：迭代文件 → 迭代种子。

### MD5 完整性校验
- **D-13:** 处理前：对每个队列文件计算 MD5 并记录文件大小，存入 `HashMap<String, (String, u64)>`（key=filepath）。
- **D-14:** 处理后：对每个成功输出文件计算 MD5，与处理前对比。MD5 不同 = 文件已修改（pass）；MD5 相同 = 处理无效（warning）；处理失败 = 无法对比（N/A）。
- **D-15:** 结果直接展示在 BatchSummary 的每个文件行中：MD5 前后值 + 状态图标（✓ 已修改 / ⚠ 未变化 / ✗ 失败）。不额外输出日志文件。

### Claude's Discretion
- Tauri `tauri.conf.json` bundle 配置具体字段（Windows NSIS vs msi targets、Linux deb/AppImage targets）
- GitHub Actions workflow YAML 结构和触发条件
- GPU 编码器检测的具体 CLI 探测命令和 ffmpeg 参数
- MD5 计算实现（Rust `md5` crate 或 `std::process::Command` 调用 `md5sum`/`ffmpeg -f hash`）
- 多种子模式下 `PerFileProgress` 事件需要包含 `seed_alias` 字段以区分同一文件的不同种子输出
- `batch-progress` 事件中 `total` 字段计算：`total = 文件数 × 种子数`
- NSelect 多选后 `seedStore` 接口变更的影响范围（BatchControls、composables）
- BatchSummary 行布局调整以容纳 MD5 信息列
- i18n 新增 key（GPU 状态、MD5 对比、多选提示等）
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Foundation
- `.planning/PROJECT.md` — 项目核心价值、技术栈约束（Tauri 2.x + Vue 3 + Rust）
- `.planning/REQUIREMENTS.md` — v1.1 需求定义（CROSS-01~03, PERF-01~02, MULTI-01~02, MD5-01~02）
- `.planning/ROADMAP.md` — Phase 5 成功标准和 6 个 plan 划分

### Tauri Build & Config
- `src-tauri/tauri.conf.json` — 当前只配置了 macOS bundle，需扩展 Windows/Linux targets
- `src-tauri/Cargo.toml` — Rust 依赖声明，需评估 md5/ring crate 等新依赖

### FFmpeg Execution (Phase 2)
- `src-tauri/src/ffmpeg/executor.rs` — 核心执行器，GPU 编码器替换和 MD5 计算将在此模块改动
- `src-tauri/src/ffmpeg/filters.rs` — 滤镜链构建，GPU 编码时需确认 `-c:v` 参数不与滤镜冲突
- `src-tauri/src/commands/batch.rs` — 批处理命令入口，多种子和 MD5 记录将在此层编排

### Frontend State (Phase 3)
- `src/stores/seed.ts` — 种子 store，`selectedSeedId` 需改为 `selectedSeedIds: string[]`
- `src/stores/batch.ts` — 批处理 store，`BatchResult` 类型需扩展 MD5 信息

### Frontend UI (Phase 3-4)
- `src/components/batch/BatchControls.vue` — NSelect 单选 → 多选，启动参数变更
- `src/components/batch/BatchSummary.vue` — 每文件行增加 MD5 前后对比列
- `src/components/batch/BatchBanner.vue` — 批处理横幅，total 计数需反映多种子总量

### Type Definitions
- `src/types/batch.ts` — BatchResult、PerFileProgress 类型需扩展 MD5 字段和 seed_alias 字段

### CI/CD
- `.github/workflows/` — 现有 CI 为 Phase 1 的 lint/test，需新增构建矩阵 workflow
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **FFmpeg executor (`executor.rs:36-191`):** 单文件处理 + 进度事件 + 取消支持已完整。GPU 加速将以 `-c:v` 编码器参数形式注入；多种子为该函数外层循环调用。
- **BatchSummary (`BatchSummary.vue`):** 已有成功/失败分区展示 + NScrollbar + 图标体系。MD5 信息作为行内扩展列追加，不改变布局骨架。
- **BatchControls (`BatchControls.vue`):** NSelect 已使用 `filterable` `clearable`，开启 `multiple` 模式即可支持多选。
- **tauri-plugin-store:** 已用于持久化 concurrency/output_dir 偏好。GPU 偏好无需持久化（自动检测），保持简单。

### Established Patterns
- **IPC 模式:** Rust `#[tauri::command]` → 前端 `invoke()`，progress 通过 `batch-file-progress` / `batch-progress` 事件流式推送。MD5 结果在 `batch-complete` 事件的 `BatchResult` 中返回。
- **Pinia Composition API stores:** `ref()` + `computed()` + 纯函数 setter，无 class/装饰器。多选种子沿用此模式。
- **Naive UI 组件选择:** 已建立 NSelect / NButton / NIcon / NText / NScrollbar / useDialog / useMessage 组合。多选和 MD5 展示不引入新组件库。
- **i18n 双语:** `en.json` + `zh-CN.json`，key 按模块分组（`batch.*`, `notification.*`）。新增 key 归入现有模块。

### Integration Points
- **seedStore → BatchControls:** `selectedSeedId` 改为 `selectedSeedIds` 后，需更新所有引用点（BatchControls, useBatch composable, start_batch 命令参数）。
- **BatchResult 类型:** `src/types/batch.ts` 中 `BatchResult.succeeded` 当前为 `string[]`（output paths）。MD5 集成需扩展为 `{ path: string, md5_before: string, md5_after: string, modified: boolean }[]`。
- **executor → GPU:** `FfmpegCommand::new_with_path()` 接受的 args 已支持任意 CLI 参数，GPU 编码只需在 `all_args` 中替换 `-c:v` 和 `-preset` 即可。
</code_context>

<specifics>
## Specific Ideas

- 用户强调 MD5 展示为"摘要中展示对比"（非独立日志文件），每个文件行显示处理前后 MD5 和状态
- 用户确认多种子输出平铺在同一目录，不建子目录
- 用户选择 GPU 全自动（不提供手动切换界面），失败静默回退 CPU
- 跨平台目标为基础级别（可安装运行），非商店发布级别

</specifics>

<deferred>
## Deferred Ideas

- 代码签名和商店上架 → 后续独立阶段（不在 Phase 5）
- GPU 编码器手动选择 UI → 如用户反馈需要再添加
- 按种子分子目录输出 → 当前选择平铺，如有需要后续可加开关
- 独立 MD5 日志文件 → 当前选择摘要内展示，后续可扩展
- macOS 打包进一步完善（已有 `.dmg`）→ 本次重点补全 Win/Linux
- 视频队列拖拽排序（PROD-01）→ v2
- 缩略图预览（PROD-02）→ v2

</deferred>

---

*Phase: 05-Production-Hardening*
*Context gathered: 2026-05-14*

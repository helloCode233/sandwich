# Phase 1: Foundation - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning

## Phase Boundary

FFmpeg 在用户机器上可靠可用（零配置检测 + 一键下载），同时 Tauri 2.x + Vue 3 + Vite 项目脚手架能构建并运行。

## Implementation Decisions

### 脚手架方式
- **D-01:** `create-tauri-app` 一键生成项目骨架，生成后手动调整依赖版本匹配 CLAUDE.md
- **D-02:** 全部依赖精确版本锁定（pinned），不使用 caret 范围
- **D-03:** TypeScript strict: true，启用全部严格检查
- **D-04:** Tauri identifier: `com.sandwich.app`
- **D-05:** 包管理器: bun
- **D-06:** 原子化 CSS: UnoCSS（Vite 原生集成、零运行时）
- **D-07:** Git hooks: husky + lint-staged 在 Phase 1 配置
- **D-08:** Rust edition: 2024
- **D-09:** 开发命令: `tauri dev`（同时启动 Vite + Tauri）
- **D-10:** CI: GitHub Actions（vue-tsc + cargo check + ESLint + clippy + Vitest + cargo test）
- **D-11:** 前端目录: 分层结构（src/components/、src/stores/、src/composables/、src/types/、src/utils/）
- **D-12:** 窗口: 标题 'Sandwich'，1200x800 默认，可调整大小，最小 900x600
- **D-13:** 国际化: vue-i18n v11，中英双语
- **D-14:** Vue Router 暂不需要（Phase 1 仅单页）

### FFmpeg 下载体验
- **D-15:** 启动时检测 + 用户点击触发下载（非自动下载）
- **D-16:** 下载范围: FFmpeg + FFprobe（满足 Phase 2 元数据提取需求）
- **D-17:** 全屏下载页：显示百分比 + 已下载/总大小 + 下载速度
- **D-18:** 允许用户选择 FFmpeg 存储目录（非强制默认路径）
- **D-19:** 检测策略: PATH 优先 → 否则提示下载（尊重已有安装）
- **D-20:** 下载失败: 显示具体错误原因 + 重试按钮（3 次后提示手动下载指引）
- **D-21:** 下载源: GitHub Releases 默认 + 镜像兜底（国内用户友好）
- **D-22:** 最低版本 FFmpeg ≥ 4.0，低于则提示下载
- **D-23:** 下载完成后自动 `ffmpeg -version` 验证 → 成功自动进入主界面
- **D-24:** 路径持久化: tauri-plugin-store 存储 ffmpeg_path、version、下载时间
- **D-25:** 每次启动检查 GitHub 最新 release，有新版本提示可选更新（不阻塞启动）
- **D-26:** 下载中断后下次启动自动续传
- **D-27:** 下载过程中可取消（清理临时文件，返回初始状态）
- **D-28:** macOS: 下载后自动移除隔离标记（`xattr -dr com.apple.quarantine`）
- **D-29:** 进度展示: 百分比 + 已下载/总大小 + 速度

### Phase 1 UI 范围
- **D-30:** 最小化 FFmpeg 页面——仅服务于 FFmpeg 检测和下载，不做 Phase 3 完整 UI 地基
- **D-31:** 页面元素: 状态指示 + 操作按钮（居中卡片布局）
- **D-32:** Naive UI 暗色主题 Phase 1 就启用（NConfigProvider + darkTheme）
- **D-33:** 前端基础设施: Naive UI + UnoCSS + vue-i18n + Pinia 全部 Phase 1 安装
- **D-34:** 页面状态流转: 检测中 → 检查结果（已找到/未找到）→ 下载中（进度+取消+速度）→ 完成（自动跳转）
- **D-35:** FFmpeg 就绪后显示占位主页（logo + 版本号 + "等待后续功能"）

### 开发工具配置
- **D-36:** ESLint 9 (flat config) + @typescript-eslint + eslint-plugin-vue + Prettier，Phase 1 配置
- **D-37:** Vitest + @vue/test-utils + cargo test，Phase 1 配置测试基础设施
- **D-38:** rustfmt + clippy，Phase 1 启用（CI 中 `cargo fmt --check` + `cargo clippy -- -D warnings`）
- **D-39:** vue-tsc 类型检查，Phase 1 配置

### Claude's Discretion
无——所有决策均由用户明确选择。

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Docs
- `.planning/ROADMAP.md` — Phase 1 目标、成功标准、依赖关系
- `.planning/REQUIREMENTS.md` — FFMPEG-01, FFMPEG-02, FFMPEG-03 需求定义
- `.planning/PROJECT.md` — 技术栈、约束、关键决策
- `CLAUDE.md` — 完整技术栈版本表、IPC 模式、FFmpeg 分发策略

### Domain Reference
- `.claude/skills/video-fingerprinting/SKILL.md` — 六类 FFmpeg 操作的指纹修改方法（作为领域背景参考）

## Existing Code Insights

### Reusable Assets
无——全新项目，Phase 1 是第一个阶段。

### Established Patterns
无——等待 Phase 1 建立。

### Integration Points
无——Phase 1 建立基础，后续阶段在其上构建。

## Specific Ideas

- 用户选择了 bun 作为包管理器（非 Tauri 默认推荐的 pnpm）——注意 create-tauri-app 与 bun 的兼容性
- 用户强调中英双语从第一天就要支持
- FFmpeg 下载体验关注国内网络环境（镜像兜底）
- Phase 1 UI 是最小化的，不要过度建设前端架构

## Deferred Ideas

无——讨论全程在阶段范围内。

---

*Phase: 1-Foundation*
*Context gathered: 2026-05-13*

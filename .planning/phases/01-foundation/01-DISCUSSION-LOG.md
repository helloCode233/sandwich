# Phase 1: Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-13
**Phase:** 1-Foundation
**Areas discussed:** 脚手架方式, FFmpeg 下载体验, Phase 1 UI 范围, 开发工具配置时机

---

## 脚手架方式

| Option | Description | Selected |
|--------|-------------|----------|
| create-tauri-app 一键生成（推荐） | npm create tauri-app@latest 同时生成 Vue + Vite 前端和 src-tauri Rust 后端 | ✓ |
| 分别创建：npm create vue + cargo init | 先创建 Vue 3 前端，再手动 cargo init src-tauri | |
| Tauri 先，Vue 后 | 先创建 Tauri 项目骨架，再手动搭建前端 | |

**User's choice:** create-tauri-app 一键生成
**Notes:** 用户提到「分为两个 create，UI 和核心」，最终选择了一键生成方案，生成后再调整。

---

| Option | Description | Selected |
|--------|-------------|----------|
| 精确版本 - pinned（推荐） | 所有依赖锁定到 CLAUDE.md 指定版本 | ✓ |
| Caret 范围 | 允许小版本自动升级 | |

**User's choice:** 精确版本锁定

---

| Option | Description | Selected |
|--------|-------------|----------|
| strict: true（推荐） | 启用全部 TypeScript 严格检查 | ✓ |
| 基础严格度 | 仅 strict: true 不含额外检查 | |
| 宽松模式 | strict: false，Phase 3 再收紧 | |

**User's choice:** strict: true

---

| Option | Description | Selected |
|--------|-------------|----------|
| com.sandwich.app（推荐） | 简洁，与项目名匹配 | ✓ |
| 自定义 | 用户自定义标识符 | |

**User's choice:** com.sandwich.app

---

| Option | Description | Selected |
|--------|-------------|----------|
| pnpm（推荐） | Tauri 默认推荐 | |
| npm | Node.js 自带 | |
| yarn | 类似 pnpm | |
| bun | 用户 via "Other" | ✓ |

**User's choice:** bun
**Notes:** 用户手动输入 bun 作为包管理器。需验证 create-tauri-app 与 bun 的兼容性。

---

| Option | Description | Selected |
|--------|-------------|----------|
| UnoCSS（推荐） | Vite 原生集成、零运行时 | ✓ |
| Tailwind CSS v4 | 生态更成熟 | |

**User's choice:** UnoCSS

---

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 1 配置（推荐） | husky + lint-staged 现在就配 | ✓ |
| 推迟 | 后续再加 | |

**User's choice:** Phase 1 配置 husky + lint-staged

---

| Option | Description | Selected |
|--------|-------------|----------|
| Rust 2024（推荐） | 最新 edition | ✓ |
| Rust 2021 | 更保守 | |

**User's choice:** Rust edition 2024

---

| Option | Description | Selected |
|--------|-------------|----------|
| tauri dev（推荐） | 一条命令同时启动 | ✓ |
| 分开启动 | vite dev + cargo tauri dev | |

**User's choice:** tauri dev

---

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 1 配置（推荐） | GitHub Actions 现在配 | ✓ |
| 推迟 | 后续再加 | |

**User's choice:** Phase 1 配置 CI

---

| Option | Description | Selected |
|--------|-------------|----------|
| 分层结构（推荐） | components/stores/composables/types/utils | ✓ |
| 扁平 src/ | 不预先建子目录 | |

**User's choice:** 分层目录结构

---

| Option | Description | Selected |
|--------|-------------|----------|
| 标准桌面窗口（推荐） | 1200x800 可调整 最小 900x600 | ✓ |
| 固定大小 | 不允许调整 | |
| 最大化启动 | 启动时最大化 | |

**User's choice:** 标准桌面窗口

---

| Option | Description | Selected |
|--------|-------------|----------|
| 纯中文界面（推荐） | 目标中文用户 | |
| 中英双语 | vue-i18n 从一开始 | ✓ |

**User's choice:** 中英双语

---

| Option | Description | Selected |
|--------|-------------|----------|
| vue-i18n（推荐） | Vue 生态最成熟 | ✓ |
| @intlify/unplugin-vue-i18n | Vite 插件版 | |
| 自建轻量方案 | 零依赖 | |

**User's choice:** vue-i18n v11

---

## FFmpeg 下载体验

| Option | Description | Selected |
|--------|-------------|----------|
| 检测 + 提示，用户点击后下载（推荐） | 符合 success criteria "一键下载" | ✓ |
| 自动静默下载 | 后台自动完成 | |
| 启动时弹窗询问 | 弹窗选下载/跳过 | |

**User's choice:** 检测 + 用户点击下载

---

| Option | Description | Selected |
|--------|-------------|----------|
| FFmpeg + FFprobe（推荐） | Phase 2 元数据需要 | ✓ |
| 仅 FFmpeg | KEEP_ONLY_FFMPEG=1 | |

**User's choice:** FFmpeg + FFprobe 都下载

---

| Option | Description | Selected |
|--------|-------------|----------|
| 全屏下载页 + 进度条（推荐） | 百分比+大小+速度 | ✓ |
| 后台下载 + 状态栏 | 允许其他操作 | |

**User's choice:** 全屏下载页 + 进度条

---

| Option | Description | Selected |
|--------|-------------|----------|
| 默认 app data 目录（推荐） | ffmpeg-sidecar 默认 | |
| 允许用户选择 | 自定义存储路径 | ✓ |

**User's choice:** 允许用户选择存储目录

---

| Option | Description | Selected |
|--------|-------------|----------|
| PATH 优先 + 自备兜底（推荐） | 尊重已有安装 | ✓ |
| 始终使用自下载 FFmpeg | 统一管理 | |

**User's choice:** PATH 优先 + 自备兜底

---

| Option | Description | Selected |
|--------|-------------|----------|
| 提示 + 重试（推荐） | 具体错误 + 重试按钮 | ✓ |
| 简单提示 + 重试 | 不区分错误类型 | |
| 自动重试 | 失败自动重试 | |

**User's choice:** 提示具体错误 + 重试按钮

---

| Option | Description | Selected |
|--------|-------------|----------|
| GitHub Releases + 镜像兜底（推荐） | 国内友好 | ✓ |
| 仅 GitHub Releases | 不走镜像 | |

**User's choice:** GitHub Releases + 镜像兜底

---

| Option | Description | Selected |
|--------|-------------|----------|
| ≥ 4.0，低于则提示下载（推荐） | 稳定 filter API | ✓ |
| 不做版本检查 | 不限制 | |

**User's choice:** FFmpeg ≥ 4.0

---

| Option | Description | Selected |
|--------|-------------|----------|
| 自动验证 + 自动跳转（推荐） | 匹配 success criteria | ✓ |
| 验证后显示结果页 | 用户手动确认 | |

**User's choice:** 自动验证 + 自动跳转

---

| Option | Description | Selected |
|--------|-------------|----------|
| 简单 JSON 配置文件（推荐） | 手写简单 | |
| tauri-plugin-store | Tauri 官方方案 | ✓ |

**User's choice:** tauri-plugin-store

---

| Option | Description | Selected |
|--------|-------------|----------|
| 不自动检查（推荐） | 不检查更新 | |
| 每次启动检查 | GitHub release 检查 | ✓ |

**User's choice:** 每次启动检查更新

---

| Option | Description | Selected |
|--------|-------------|----------|
| 提示「有新版本」+ 可选更新（推荐） | 不阻塞启动 | ✓ |
| 强制更新 | 必须更新 | |
| 仅记录日志 | 开发者控制台 | |

**User's choice:** 可选更新提示

---

| Option | Description | Selected |
|--------|-------------|----------|
| 下次启动续传（推荐） | 检查临时文件 | ✓ |
| 重新下载 | 丢弃重来 | |

**User's choice:** 断点续传

---

| Option | Description | Selected |
|--------|-------------|----------|
| 可以取消（推荐） | 清理临时文件 | ✓ |
| 不可取消 | 必须完成 | |

**User's choice:** 可取消下载

---

| Option | Description | Selected |
|--------|-------------|----------|
| 自动移除隔离标记（推荐） | xattr -dr | ✓ |
| 提示用户手动执行 | 透明但体验差 | |

**User's choice:** macOS 自动移除隔离标记

---

| Option | Description | Selected |
|--------|-------------|----------|
| 百分比 + 已下载/总大小 + 速度（推荐） | 信息丰富 | ✓ |
| 仅百分比 + 进度条 | 极简 | |
| 百分比 + 速度 + ETA | 最完整 | |

**User's choice:** 百分比 + 已下载/总大小 + 速度

---

## Phase 1 UI 范围

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 3 的 UI 地基（推荐） | 完整前端架构 | |
| 最小化 FFmpeg 页面 | 仅 FFmpeg 状态页 | ✓ |

**User's choice:** 最小化 FFmpeg 页面——Phase 3 再做完整 UI

---

| Option | Description | Selected |
|--------|-------------|----------|
| 状态指示 + 操作按钮（推荐） | 居中卡片 | ✓ |
| 仅状态文字 + 下载按钮 | 极简 | |
| 完整下载页 | 功能齐全 | |

**User's choice:** 状态指示 + 操作按钮

---

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 1 就上（推荐） | NConfigProvider + darkTheme | ✓ |
| Phase 3 再加 | 系统默认 | |

**User's choice:** Phase 1 启用暗色主题

---

| Option | Description | Selected |
|--------|-------------|----------|
| Naive UI + 暗色主题（推荐） | 组件 + 主题 | ✓ |
| UnoCSS | 页面样式 | ✓ |
| vue-i18n | 中英双语 | ✓ |
| Pinia | FFmpeg 状态管理 | ✓ |

**User's choice:** 四项全选

---

| Option | Description | Selected |
|--------|-------------|----------|
| 检测中→检查结果→下载中→完成（推荐） | 四态流转 | ✓ |
| 仅成功/失败两种状态 | 简化 | |

**User's choice:** 四态流转

---

| Option | Description | Selected |
|--------|-------------|----------|
| 占位主页（推荐） | logo + 版本号 | ✓ |
| 空白页面 | 仅标题栏 | |
| 始终显示 FFmpeg 状态 | 状态页 | |

**User's choice:** 占位主页

---

## 开发工具配置时机

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 1 就配（推荐） | ESLint 9 + Prettier | ✓ |
| Phase 3 再配 | 推迟 | |

**User's choice:** Phase 1 配置 ESLint + Prettier

---

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 1 配置 Vitest + cargo test（推荐） | 一次配好 | ✓ |
| 分别到各自 Phase 配置 | 分阶段 | |

**User's choice:** Phase 1 配 Vitest + cargo test

---

| Option | Description | Selected |
|--------|-------------|----------|
| rustfmt + clippy（推荐） | 全部启用 | ✓ |
| 只 rustfmt | 先格式 | |
| Phase 2 再说 | 推迟 | |

**User's choice:** rustfmt + clippy 都启用

---

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 1 配（推荐） | vue-tsc -b | ✓ |
| Phase 3 再配 | 推迟 | |

**User's choice:** vue-tsc 在 Phase 1 配置

---

## Claude's Discretion

无——所有选择均由用户明确做出。

## Deferred Ideas

无——讨论始终在阶段范围内。

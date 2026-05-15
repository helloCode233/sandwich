# 视频指纹批量修改工具

## What This Is

一款基于 Tauri 的桌面端视频指纹批量修改工具。用户管理"种子"（自动生成的多操作链处理配方），拖入视频队列，选择种子后批量处理。处理通过 FFmpeg 执行，包括数学叠加、像素变换、时间轴修改、编码参数调整等操作，使同一素材生成多个指纹不同的视频。

## Core Value

**一键批量去重** — 自动生成随机化种子配方，将同一视频源产出多个平台无法识别为重复的变体。

## Requirements

### Validated (Phase 1-5)

- [x] **SEED-01~06**: 自动生成种子 + 别名 + CRUD 管理 — Validated in Phase 2, 3
- [x] **OP-01~02**: 操作链结构 + 7 种操作类型 — Validated in Phase 2
- [x] **VIDEO-01~03**: 视频导入 + 队列管理 + 预览 — Validated in Phase 3
- [x] **BATCH-01~05**: 批处理 + 进度 + 导出 + 取消 + 摘要 — Validated in Phase 3, 4
- [x] **FFMPEG-01~03**: FFmpeg 检测 + 自动下载 + 版本匹配 — Validated in Phase 2
- [x] **IMPORT-01~02**: 拖入/批量导入 — Validated in Phase 3
- [x] **QUEUE-01~02**: 队列管理 + 排序 — Validated in Phase 3
- [x] **OUTPUT-01~02**: 输出目录 + 命名 — Validated in Phase 3
- [x] **UI-01~02**: 双面板布局 + 暗色主题 — Validated in Phase 3
- [x] **CROSS-01~03**: Win/Linux 打包 + CI 矩阵 — Validated in Phase 5
- [x] **PERF-01~02**: GPU 加速 + Pipeline 优化 — Validated in Phase 5
- [x] **MULTI-01~02**: 多种子批处理 — Validated in Phase 5
- [x] **MD5-01~02**: MD5 完整性校验 — Validated in Phase 5

### Active

(None — all v1.1 requirements shipped)

### Out of Scope / Deferred

- 种子手动编辑 → v2
- 视频队列拖拽排序 (PROD-01) → v2
- 缩略图预览 (PROD-02) → v2
- 代码签名/商店上架 → 后续独立阶段
- GPU 编码器手动选择 UI → 按需添加

## Context

- 技术栈：Tauri 2.x + Rust 后端 + Vue 3 前端
- FFmpeg 作为外部进程调用，需自动检测和下载机制
- 操作类型参考 video-fingerprinting skill 中的六类方法（无损结构修改、半透明数学叠加、像素级几何变换、时间轴修改、编码参数调整、多层组合）
- 种子结构：`[{操作类型, 起始帧, 持续帧数, params: {}}]`，由随机种子值驱动参数随机化
- 安全约束：透明度上限 0.15，像素平移 ≤ 3px，抽帧间隔 ≥ 15 帧

## Constraints

- **Tech stack**: Tauri 2.x + Vue 3 + Rust — 必须
- **Bundle size**: FFmpeg 二进制可能较大（~80MB），需考虑下载策略
- **Performance**: 视频处理为 CPU 密集型，必须异步执行避免阻塞 UI

## Key Decisions

| Decision                   | Rationale                  | Outcome             |
| -------------------------- | -------------------------- | ------------------- |
| Tauri 替代 Ratatui TUI     | 用户偏好切换               | — Shipped (Phase 1-5) |
| Vue 3 前端                 | 用户指定                   | — Shipped (Phase 1-5) |
| 自动生成种子（非手动编排） | 用户指定                   | — Shipped (Phase 2) |
| 多种子批处理               | Phase 5 升级               | — Shipped (Phase 5)  |
| GPU 全自动（非手动选择）   | Phase 5 设计决策           | — Shipped (Phase 5)  |
| ffmpeg-sidecar 自动下载    | 避免 ~80MB 安装包体积       | — Shipped (Phase 2)  |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):

1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):

1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---

_Last updated: 2026-05-15 — Phase 5 (Production Hardening) complete. v1.1 shipped._

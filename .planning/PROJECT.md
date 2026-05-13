# 视频指纹批量修改工具

## What This Is

一款基于 Tauri 的桌面端视频指纹批量修改工具。用户管理"种子"（自动生成的多操作链处理配方），拖入视频队列，选择种子后批量处理。处理通过 FFmpeg 执行，包括数学叠加、像素变换、时间轴修改、编码参数调整等操作，使同一素材生成多个指纹不同的视频。

## Core Value

**一键批量去重** — 自动生成随机化种子配方，将同一视频源产出多个平台无法识别为重复的变体。

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] **SEED-01**: 自动生成种子，包含多组随机操作链
- [ ] **SEED-02**: 种子支持设置别名
- [ ] **SEED-03**: 种子列表管理（查看、删除、复制）
- [ ] **OP-01**: 每条操作链包含：操作类型 + 起始帧 + 持续帧数 + 参数
- [ ] **OP-02**: 支持的操作类型：数学叠加（波纹/条纹/同心圆）、像素平移、抽帧、GOP 修改、元数据擦除、音频微调、重封装
- [ ] **VIDEO-01**: 支持拖入/批量导入视频文件
- [ ] **VIDEO-02**: 视频队列管理（添加、移除、排序）
- [ ] **VIDEO-03**: 视频预览播放
- [ ] **BATCH-01**: 选择一个种子→应用到队列中所有视频
- [ ] **BATCH-02**: 显示处理进度
- [ ] **BATCH-03**: 处理后的视频导出到指定目录
- [ ] **FFMPEG-01**: 启动时检测 FFmpeg 是否存在
- [ ] **FFMPEG-02**: FFmpeg 缺失时自动下载
- [ ] **UI-01**: 左侧面板 — 种子列表
- [ ] **UI-02**: 右侧面板 — 视频队列和预览

### Out of Scope

- 不同视频使用不同种子（当前：一个种子处理所有视频）
- 种子手动编辑（当前：仅自动生成）
- macOS / Windows 打包（当前：仅开发验证）

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

| Decision                   | Rationale                  | Outcome   |
| -------------------------- | -------------------------- | --------- |
| Tauri 替代 Ratatui TUI     | 用户偏好切换               | — Pending |
| Vue 3 前端                 | 用户指定                   | — Pending |
| 自动生成种子（非手动编排） | 用户指定                   | — Pending |
| 一个种子处理所有视频       | 暂定，后续支持复杂对应关系 | — Pending |

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

_Last updated: 2026-05-12 after initialization_

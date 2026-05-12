# Research Summary: Sandwich — 视频指纹批量修改工具

**Synthesized:** 2026-05-12
**Sources:** STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md

---

## Executive Summary

Sandwich 是一款基于 Tauri 2.x + Vue 3 桌面端的视频指纹批量修改工具。核心工作流：自动生成"种子"（多操作链处理配方）→ 拖入视频队列 → 选择一个种子 → 一键批量处理。FFmpeg 自动检测/下载，零命令行知识门槛。

**Key risks:** FFmpeg 进程孤儿化、进度解析跨平台脆弱性、Vue 大列表响应式性能、种子生成的随机性质量控制。

---

## Stack Decisions

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Backend | Tauri 2.11 + Rust | Tauri 2 插件生态成熟，Rust 性能和安全性适合 FFmpeg 进程管理 |
| Frontend | Vue 3.5 + TypeScript 6 + Vite 8 | 用户指定，Composition API 简洁 |
| FFmpeg | ffmpeg-sidecar 2.5 | 三合一：二进制自动下载 + 进度结构化解析 + 跨平台路径处理 |
| UI | Naive UI 2.44 | Tree-shaking、内置暗色主题、TypeScript-first、桌面端紧凑主题 |
| State | Pinia 3.0 (前端) + Tauri Managed State (后端) | Rust 持有权威状态，Pinia 镜像渲染，变更仅通过 Tauri commands |
| Testing | Vitest 4.1 + @vue/test-utils (前端), cargo test + rstest (后端) | |

**Anti-recommendations:** ffmpeg-next (分发噩梦，需系统库依赖)、Element Plus (过于 web 向，无原生暗色主题)、Vue Router (MVP 单页两面板布局不需要路由)

---

## Feature Summary

**14 项 MVP (P1)：**
- FFmpeg 自动检测 + 一键下载
- 视频拖拽导入 + 文件选择器
- 视频队列（增删改序）+ 缩略图预览
- 种子列表管理（查看、重命名、删除、复制）
- 自动种子生成（7种操作类型，3-7步随机操作链，安全约束）
- 一个种子 → 整队批量处理
- 逐文件和总体进度跟踪
- 处理取消（优雅关闭）
- 输出目录配置 + 文件命名规则
- 单文件失败隔离（不中断整个批次）

**7 项差异体验 (P2 deferred):**
- 确定性随机种子生成（同 seed 产生相同配方）
- 安全约束引擎、操作链目录
- 处理日志和历史、种子导出/导入
- 一键工作流（零 FFmpeg 知识）、批次摘要

---

## Architecture Highlights

1. **三层架构：** Vue/Pinia (展示层) → Tauri IPC (Commands + Events) → Rust Services (业务逻辑)
2. **Command-Event 分离：** CRUD 用 Commands (`invoke`)，进度推送用 Events (`emit`/`listen`)
3. **Thin Commands, Fat Services：** Tauri command handlers 仅做参数提取和状态注入，实际逻辑在 `src-tauri/src/services/`
4. **Rust Owns Truth / Pinia Mirrors：** Rust `app.manage()` 持有权威状态，Pinia stores 镜像用于 UI 渲染
5. **FFmpeg 通过 `tauri_plugin_shell::ShellExt` 作为子进程调用**（非 sidecar，因为 auto-download 不在编译时）

---

## Critical Pitfalls

| # | Pitfall | Prevention | Phase |
|---|---------|------------|-------|
| 1 | FFmpeg 进程孤儿化 — 关闭窗口后进程继续运行，输出文件损坏 | SIGTERM → 等5秒 → SIGKILL；注册 `onCloseRequested` 守卫；`Mutex<Vec<Child>>` 管理所有活跃子进程 | Phase 3 |
| 2 | Stderr 进度解析跨平台脆弱 — 格式因 OS/FFmpeg 版本/编码器而异 | 使用 `-progress pipe:1` (稳定 key=value 格式)；统一行尾处理 `\r\n` → `\n`；包含 `progress=end` 哨兵检测 | Phase 3 |
| 3 | Vue 大队列响应式陷阱 — `reactive()` 深代理所有元素导致性能崩溃 | `shallowRef()` 存视频队列数组；`markRaw()` 标记队列项防止深度观察 | Phase 4 |
| 4 | UI 主线程阻塞 — 同步文件 I/O 或进程等待冻结界面 | Tauri async commands；`tokio::spawn_blocking()` 隔离 CPU 密集工作 | Phase 3 |
| 5 | FFmpeg GPL 许可证风险 — 默认编解码器含 GPL 组件 | 分发时注意 LGPL 配置；提供许可证声明和源码访问 | Phase 1 |

---

## Roadmap Recommendations

基于依赖链推导 5 个阶段（对应 coarse 粒度）：

1. **Foundation** — FFmpeg 检测/下载 + Tauri 插件注册 + 项目脚手架
2. **Seed Engine** — 种子生成算法 + 数据模型 + FFmpeg 命令构建器
3. **Batch Processing** — 视频导入/队列管理 + FFmpeg 执行 + 进度解析 + 取消/错误处理
4. **Frontend** — Pinia stores + Vue 组件 + 双面板布局 + 暗色主题 + 进度 UI
5. **Integration & Polish** — E2E 流程贯通 + 视频预览/缩略图 + 日志 + 测试 + 打包

---

## Confidence

| Area | Level |
|------|-------|
| Stack | HIGH — 所有库版本通过 Context7 官方文档验证 |
| Features | MEDIUM — 功能认知基于领域知识，竞品分析未完全验证 |
| Architecture | HIGH — Tauri 2.x 官方文档验证 IPC 和状态管理模式 |
| Pitfalls | MEDIUM — FFmpeg 跨平台经验基于 Linux/macOS，Windows 需实测 |

---
*Synthesized: 2026-05-12*

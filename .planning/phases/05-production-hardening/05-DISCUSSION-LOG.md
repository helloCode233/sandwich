# Phase 5: Production Hardening - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-14
**Phase:** 05-production-hardening
**Areas discussed:** 跨平台适配范围, 性能优化方向, 多种子工作模型, MD5校验范围, GPU编码器选择策略, 多种子输出目录结构, MD5展示方式

---

## 跨平台适配范围

| Option | Description | Selected |
|--------|-------------|----------|
| 打包 + 基础验证 | Windows (.msi/.exe) 和 Linux (.AppImage/.deb) 安装包，基本功能验证 | ✓ |
| 完整 CI 多平台构建 | GitHub Actions 矩阵构建 + 自动上传 + 平台特定修复 | |
| 发布就绪 | 完整 CI + 自动更新 + 代码签名 + 商店上架 | |

**User's choice:** 打包 + 基础验证

---

## 性能优化方向

| Option | Description | Selected |
|--------|-------------|----------|
| GPU 加速编码 | NVENC/VideoToolbox/VAAPI 自动检测与选择 | |
| 并行 pipeline + 内存优化 | 调度优化 + 流式 I/O | |
| 全部（GPU + pipeline） | 硬件加速 + 调度优化 | ✓ |

**User's choice:** 全部（GPU + pipeline）

---

## 多种子工作模型

| Option | Description | Selected |
|--------|-------------|----------|
| 一个视频 × N 个种子 = N 个输出 | 每个视频用所有选中种子各处理一次 | ✓ |
| 不同视频绑定不同种子 | 每个视频单独指定种子 | |
| 两者都要 | 以上两种模式都支持 | |

**User's choice:** 一个视频 × N 个种子 = N 个输出

---

## MD5 校验范围

| Option | Description | Selected |
|--------|-------------|----------|
| 处理后校验 | 仅对比输入输出 MD5，摘要中展示 | |
| 完整校验链 | 处理前记录 + 处理后对比 + 差异日志 + 无效告警 | ✓ |

**User's choice:** 完整校验链

---

## GPU 编码器选择策略

| Option | Description | Selected |
|--------|-------------|----------|
| 自动检测+自动选择 | 启动时检测，批处理时自动使用，失败静默回退 CPU | ✓ |
| 用户手动选择 | 设置面板中让用户选择，批处理时使用所选 | |

**User's choice:** 自动检测+自动选择（推荐）

---

## 多种子输出目录结构

| Option | Description | Selected |
|--------|-------------|----------|
| 平铺 | 所有输出文件在同一目录，结构简单 | ✓ |
| 按种子分子目录 | output_dir/seed1/video.mp4，更整洁但更深 | |

**User's choice:** 平铺（推荐）

---

## MD5 结果展示

| Option | Description | Selected |
|--------|-------------|----------|
| 摘要中展示对比 | 每文件行显示处理前后 MD5 + 状态图标 | ✓ |
| 摘要 + 日志文件 | 摘要中简化展示，完整 MD5 写入 JSON 日志 | |

**User's choice:** 摘要中展示对比

---

## Claude's Discretion

- Tauri bundler 配置细节（Windows NSIS/msi, Linux deb/AppImage）
- GitHub Actions CI 矩阵 YAML 结构
- GPU 编码器探测命令和 ffmpeg 参数
- MD5 计算实现方案
- 多选后 BatchControls/seedStore 接口变更细节
- BatchSummary 行布局调整以容纳 MD5
- i18n 新 key 设计和命名

## Deferred Ideas

- 代码签名和商店上架 → 后续阶段
- GPU 编码器手动选择 UI → 等用户反馈
- 按种子分子目录选项 → 后续可添加开关
- 独立 MD5 日志文件 → 后续可扩展
- 拖拽排序（PROD-01）、缩略图预览（PROD-02）→ v2

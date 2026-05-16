# Phase 6: 增强指纹修改 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-16
**Phase:** 06-增强指纹修改
**Areas discussed:** 新增操作类型, 种子智能化升级, 种子管理增强, v2 延迟项落地, 新操作权重分配, UI 布局重新设计, 存量种子兼容策略, FFmpeg 滤镜兼容性

---

## 新增操作类型

| Option | Description | Selected |
|--------|-------------|----------|
| 色彩处理类 | 色相旋转、饱和度微调、亮度偏移、色彩平衡 | ✓ |
| 噪声与纹理类 | 颗粒/噪点、模糊/锐化 | ✓ |
| 几何微调类 | 微小旋转、缩放、翻转 | ✓ |
| 混合叠加类 | 半透明纯色/渐变叠加、水印混合 | ✓ |

**User's choice:** 四类全要
**Notes:** 用户明确表示"都要"，不取舍任何类别。

| Option | Description | Selected |
|--------|-------------|----------|
| 每种 1-2 个 | 每类挑最有效的 1-2 个 | |
| 每种 3+ 个 | 每类至少 3 个变体，总计 12+ | ✓ |
| 你决定 | 由规划阶段决定 | |

**User's choice:** 每种 3+ 个
**Notes:** 用户追求操作类型多样性最大化。

| Option | Description | Selected |
|--------|-------------|----------|
| 硬编码约束 | 代码 clamp 参数上限 | |
| 分档可调 | 保守/标准/激进三档预设 | ✓ |
| 完全暴露参数 | 每操作暴露详细参数 | |

**User's choice:** 分档可调
**Notes:** 用户希望有控制权但不想操作太繁琐。

| Option | Description | Selected |
|--------|-------------|----------|
| 统一随机池 | 新旧操作一起随机 | ✓ |
| 分类选择 | 用户选择基础/增强种子 | |

**User's choice:** 统一随机池

| Option | Description | Selected |
|--------|-------------|----------|
| 纯内置滤镜 | 标准 FFmpeg 内置滤镜 | ✓ |
| 定制编译 FFmpeg | CI 编译含第三方滤镜的版本 | |
| 内置优先 + 运行时排除 | 检测后动态排除 | |

**User's choice:** 纯内置滤镜
**Notes:** 用户询问了如何使用第三方滤镜（frei0r 等）。解释后选择保持简单——标准 FFmpeg 内置滤镜已覆盖四类新操作。

---

## 种子智能化升级

| Option | Description | Selected |
|--------|-------------|----------|
| 保持 3-7 步 | 与现在一样 | |
| 扩展到 5-12 步 | 更多步数更多多样性 | ✓ |
| 步数范围也随机化 | 小/中/大随机 | |

**User's choice:** 扩展到 5-12 步

| Option | Description | Selected |
|--------|-------------|----------|
| 三档预设 | 保守/标准/激进 | ✓ |
| 连续强度滑块 | 1-10 连续可调 | |
| 无需全局强度 | 保留现有 clamp | |

**User's choice:** 三档预设

| Option | Description | Selected |
|--------|-------------|----------|
| 保持随机 | 不排序 | ✓ |
| 管道阶段排序 | 滤镜→几何→编码 | |
| 冲突检测+自动修复 | 检测并处理冲突 | |

**User's choice:** 保持随机
**Notes:** 用户确认随机性价值大于管道优化。

| Option | Description | Selected |
|--------|-------------|----------|
| 随机分配 + 覆盖校验 | 分配片段+校验 ≥70% | ✓ |
| 时间轴分段分配 | 切 N 段各覆盖一段 | |
| 全视频默认覆盖 | start_frame=0 全视频 | |

**User's choice:** 随机分配 + 覆盖校验
**Notes:** 用户明确要求"保证操作覆盖视频百分之七十或以上"。

---

## 种子管理增强

| Option | Description | Selected |
|--------|-------------|----------|
| 单文件 JSON | 单个种子一键导出/导入 | ✓ |
| 批量 JSON 文件 | 多选批量操作 | |
| 两者都要 | | |

**User's choice:** 单文件 JSON

| Option | Description | Selected |
|--------|-------------|----------|
| 每视频独立种子选择 | 每行视频旁加种子多选 | |
| 分组自动分配 | 按规则自动分配 | |
| 暂缓到后续阶段 | 不急 | ✓ |

**User's choice:** 暂缓
**Notes:** SEED-COMPLEX-01 推迟到后续阶段。

| Option | Description | Selected |
|--------|-------------|----------|
| 卡片上操作按钮 | hover 显示导出/导入图标 | ✓ |
| 顶栏 + 卡片分离 | 顶部导入按钮+卡片导出 | |
| 菜单栏操作 | File 菜单 | |

**User's choice:** 卡片上操作按钮

| Option | Description | Selected |
|--------|-------------|----------|
| 重新生成 ID | 导入时新 UUID | ✓ |
| 按 ID 覆盖更新 | 匹配则覆盖 | |
| 每次询问 | 弹确认框 | |

**User's choice:** 重新生成 ID

---

## v2 延迟项落地

| Option | Description | Selected |
|--------|-------------|----------|
| PROD-01 拖拽排序 | HTML5 拖拽重排 | ✓ |
| PROD-02 缩略图预览 | 首帧缩略图 | ✓ |
| PROD-03 处理日志历史 | 搜索/过滤历史 | ✓ |

**User's choice:** 全要

| Option | Description | Selected |
|--------|-------------|----------|
| 持久化 + 顺序执行 | 重启保留，按序批处理 | ✓ |
| 会话内临时排序 | 仅当前会话 | |

**User's choice:** 持久化 + 顺序执行

| Option | Description | Selected |
|--------|-------------|----------|
| 导入时提取首帧 | ffmpeg -ss 1 -vframes 1 | ✓ |
| 按需实时提取 | 选中/悬停时提取 | |
| 多帧预览条 | N 帧 hover 预览 | |

**User's choice:** 导入时提取首帧

| Option | Description | Selected |
|--------|-------------|----------|
| UI 内嵌日志面板 | 搜索/过滤/统计 | ✓ |
| 仅文件输出 | 日志文件 | |
| 完整日志系统 | UI + 导出 + 统计 | |

**User's choice:** UI 内嵌日志面板

---

## 新操作权重分配

| Option | Description | Selected |
|--------|-------------|----------|
| 按操作大类分配 | 数学叠加 15%/色彩 20%/噪声 15%/几何 15%/叠加 10%/其余 25% | ✓ |
| 全部均分 | 所有类型均分 | |
| 保持数学叠加优势 | MathOverlay 30% | |

**User's choice:** 按操作大类分配

---

## UI 布局重新设计

| Option | Description | Selected |
|--------|-------------|----------|
| 增量扩展现有布局 | 双面板 + 底栏，组件内扩展 | ✓ |
| 新增第三面板 | 左种子/中队列/右日志 | |
| 日志用独立抽屉 | Drawer 弹出 | |

**User's choice:** 增量扩展现有布局

---

## 存量种子兼容策略

| Option | Description | Selected |
|--------|-------------|----------|
| 自动兼容 + 手动升级 | 旧格式补默认值，用户可选升级 | |
| 启动时自动迁移 | 自动补充新字段 | ✓ |
| 标记但不转换 | 加 v1 标签 | |

**User's choice:** 启动时自动迁移

---

## Claude's Discretion

- 新操作具体 FFmpeg 滤镜实现和参数范围
- 大类内部子操作权重细化
- 强度档位参数映射规则
- 覆盖率校验算法
- 拖拽排序具体实现
- 缩略图分辨率和大小限制
- 日志面板 UI 细节
- i18n key
- 自动迁移脚本

## Deferred Ideas

- SEED-COMPLEX-01 不同视频用不同种子 → 后续阶段

# Requirements: 视频指纹批量修改工具 (Sandwich)

**Defined:** 2026-05-12
**Core Value:** 一键批量去重 — 自动生成随机化种子配方，将同一视频源产出多个指纹不同的变体

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### FFmpeg Lifecycle

- [ ] **FFMPEG-01**: App 启动时自动检测 FFmpeg 是否在 PATH 中
- [ ] **FFMPEG-02**: FFmpeg 缺失时提供一键下载（下载进度显示，平台自适应）
- [ ] **FFMPEG-03**: 下载完成后自动验证 FFmpeg 可执行

### Seed Management

- [ ] **SEED-01**: 一键生成随机种子（基于随机种子值，自动生成 3-7 步操作链）
- [ ] **SEED-02**: 种子生成包含 7 种操作类型：数学叠加（波纹/条纹/同心圆）、像素平移、抽帧、GOP 修改、元数据擦除、音频微调、重封装
- [ ] **SEED-03**: 种子操作链格式：[操作类型] + [起始帧] + [持续帧数] + [参数]
- [ ] **SEED-04**: 自动生成操作参数时强制安全约束（透明度 ≤ 0.15，像素平移 ≤ 3px，抽帧间隔 ≥ 15）
- [ ] **SEED-05**: 种子支持设置别名
- [ ] **SEED-06**: 种子列表管理（查看、重命名、删除、复制）

### Video Import & Queue

- [ ] **IMPORT-01**: 支持拖拽视频文件到应用窗口导入
- [ ] **IMPORT-02**: 支持文件选择器选择视频文件导入
- [ ] **QUEUE-01**: 视频队列显示（文件名、时长、分辨率、大小）
- [ ] **QUEUE-02**: 视频队列管理（移除单个、清空全部）

### Batch Processing

- [ ] **BATCH-01**: 选择一个种子，应用到队列中所有视频
- [ ] **BATCH-02**: 处理时显示逐文件进度（百分比、当前帧、预估剩余时间）
- [ ] **BATCH-03**: 支持取消正在进行的批处理
- [ ] **BATCH-04**: 单文件失败隔离——一个失败不影响其余文件继续处理
- [ ] **BATCH-05**: 批处理完成后展示结果摘要（成功/失败数）

### Output Management

- [ ] **OUTPUT-01**: 支持选择输出目录（默认 ~/Videos/sandwich-output/）
- [ ] **OUTPUT-02**: 输出文件命名：{原文件名}_{种子别名}.{扩展名}

### UI

- [ ] **UI-01**: 双面板布局——左侧种子列表，右侧视频队列
- [ ] **UI-02**: 暗色主题（Naive UI dark theme）

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Productivity

- **PROD-01**: 视频队列拖拽排序
- **PROD-02**: 视频缩略图预览（队列中显示首帧缩略图）
- **PROD-03**: 处理日志和历史记录（搜索、过滤、导出）

### Seed Features

- **SEED-EXPORT-01**: 种子导出为 JSON 文件
- **SEED-EXPORT-02**: 从 JSON 文件导入种子
- **SEED-COMPLEX-01**: 不同视频使用不同种子

## Out of Scope

| Feature | Reason |
|---------|--------|
| 实时处理预览 | FFmpeg 不支持中间帧流式输出，复杂度极高，对 v1 价值近乎为零 |
| 手动滤镜链编辑器 | 与自动种子生成的核心价值相悖，建设可视化编辑器的工程量巨大 |
| 视频剪辑功能（时间轴、裁剪、转场） | 完整视频编辑是另一个产品品类，严重超出范围 |
| 云端编码 | 需认证、计费、上传基础设施，将桌面应用变成 SaaS |
| 纯音频处理模式 | 音频指纹与视频指纹是不同领域，分散核心关注点 |
| 插件系统 | 安全沙箱插件 API 需数月开发，7 种内置操作已覆盖足够指纹修改面 |
| AI 去重评分 | 平台算法是黑箱且持续变化，评分会误导用户，且逆向工程有法律风险 |
| 项目文件/工作区持久化 | 增加文件格式设计、脏状态追踪、向后兼容等大量工作量 |
| macOS / Windows 打包 | 开发验证完成后单独处理 |
| 系统托盘/后台处理 | 增加托盘集成、后台进程管理复杂度 |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| FFMPEG-01 | Phase 1 | Pending |
| FFMPEG-02 | Phase 1 | Pending |
| FFMPEG-03 | Phase 1 | Pending |
| SEED-01 | Phase 2 | Pending |
| SEED-02 | Phase 2 | Pending |
| SEED-03 | Phase 2 | Pending |
| SEED-04 | Phase 2 | Pending |
| SEED-05 | Phase 2 | Pending |
| SEED-06 | Phase 2 | Pending |
| IMPORT-01 | Phase 2 | Pending |
| IMPORT-02 | Phase 2 | Pending |
| QUEUE-01 | Phase 2 | Pending |
| QUEUE-02 | Phase 2 | Pending |
| BATCH-01 | Phase 2 | Pending |
| BATCH-02 | Phase 4 | Pending |
| BATCH-03 | Phase 2 | Pending |
| BATCH-04 | Phase 2 | Pending |
| BATCH-05 | Phase 4 | Pending |
| OUTPUT-01 | Phase 2 | Pending |
| OUTPUT-02 | Phase 2 | Pending |
| UI-01 | Phase 3 | Pending |
| UI-02 | Phase 3 | Pending |

**Coverage:**
- v1 requirements: 22 total
- Mapped to phases: 22
- Unmapped: 0

---
*Requirements defined: 2026-05-12*
*Last updated: 2026-05-12 after roadmap creation*

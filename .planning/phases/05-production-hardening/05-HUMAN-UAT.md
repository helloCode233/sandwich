---
status: partial
phase: 05-production-hardening
source: [05-VERIFICATION.md]
started: 2026-05-15T12:00:00Z
updated: 2026-05-15T12:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. 跨平台安装包生成
expected: 在 macOS、Windows、Linux 上运行 `cargo tauri build`，各平台产生对应安装包 (.dmg, .msi, .exe, .deb, .AppImage)
result: [pending]

### 2. GPU 编码器检测和吞吐量
expected: 在有 NVIDIA/AMD GPU 的机器上启动应用，gpu-encoder-detected 事件触发，FFmpeg 使用硬件编码器，编码速度明显快于 CPU
result: [pending]

### 3. CI 工作流执行
expected: 推送到 `release` 分支后 GitHub Actions 4 个矩阵任务全部通过，创建包含所有平台产物的 Draft Release
result: [pending]

### 4. MD5 完整性展示端到端
expected: 用多个种子处理视频后，BatchSummary 每行显示截断 MD5（前 8 字符），已修改文件绿色 CheckCircle，未变化文件琥珀色 AlertCircle，有未变化警告横幅
result: [pending]

### 5. v1 回归测试
expected: 所有已有功能（种子生成、视频导入、队列管理、单种子批处理、进度流、取消）端到端正常工作
result: [pending]

## Summary

total: 5
passed: 0
issues: 0
pending: 5
skipped: 0
blocked: 0

## Gaps


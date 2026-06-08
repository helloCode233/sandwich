# Sandwich — Video Fingerprint Batch Modification Tool

[English](#english) | [中文](#中文)

---

## English

A Tauri-based desktop tool for batch modifying video fingerprints. Manage "seeds" (auto-generated multi-operation processing recipes), drag videos into a queue, select seeds, and batch process. Processing is executed via FFmpeg with mathematical overlays, pixel transformations, timeline modifications, encoding parameter adjustments, and more — producing multiple fingerprint-distinct variants from the same source material.

**Core Value: One-click batch deduplication** — Automatically generate randomized seed recipes to produce variants from a single video source that platforms cannot recognize as duplicates.

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Framework | Tauri 2.x |
| Backend | Rust (stable 1.85+) |
| Frontend | Vue 3 + TypeScript + Vite |
| UI Library | Naive UI (dark theme) |
| State Management | Pinia (Composition API) |
| Video Processing | FFmpeg (auto-download via ffmpeg-sidecar) |

### Features

- **30 Operation Types** — Audio resample, volume, pitch, EQ, channel operations, smart crop, metadata write/selective erase, video speed, frame drop, trim edges, hue rotate, saturation, brightness/contrast, color balance, film grain, Gaussian blur, sharpen, micro-rotate, tiny scale, flip, math overlay, pixel shift, solid color overlay, gradient overlay, watermark blend, GOP modify, metadata erase, remux
- **3 Strength Tiers** — Conservative, Standard, Aggressive parameter ranges
- **GPU Acceleration** — NVENC encode + NVDec decode + CUDA filters (scale_cuda, bilateral_cuda, transpose_cuda, yadif_cuda, pad_cuda) for zero-copy GPU pipeline
- **Two-pass Processing** — GPU-native filters run first (NVENC compressed), then CPU-only filters on intermediate output
- **Auto GPU Downgrade** — Falls back to CPU (libx264) when GPU encoder unavailable or fails at runtime
- **Seed Management** — Generate, copy, delete, export/import seeds with schema version migration
- **Batch Processing** — Queue videos, select multiple seeds, process with progress streaming and cancellation
- **MD5 Integrity** — Pre/post processing hash verification with per-file log

### Requirements

- Windows 10+ (x64) or Linux (x64/arm64)
- NVIDIA GPU optional (for hardware acceleration)
- FFmpeg auto-downloaded on first launch (BtbN builds)

### Quick Start

```bash
# Install dependencies
npm install

# Run in development
npm run tauri dev

# Build for production
npm run tauri build
```

### Architecture

```
Video Queue → Seeds → Batch Processing
                         ↓
              ┌──────────────────────┐
              │  GPU Pass (Pass 1)   │
    input.mp4 → NVDec → CUDA filters → NVENC (-cq 28 compressed)
              │  scale_cuda, pad_cuda, bilateral_cuda,
              │  transpose_cuda, yadif_cuda
              └──────────────────────┘
                         ↓ temp_gpu.mp4
              ┌──────────────────────┐
              │  CPU Pass (Pass 2)   │
temp_gpu.mp4 → CPU filters → Encode → output.mp4
              │  select, setpts, trim, unsharp,
              │  hue, eq, rotate, geq, atrim
              └──────────────────────┘
```

---

## 中文

一款基于 Tauri 的桌面端视频指纹批量修改工具。用户管理"种子"（自动生成的多操作链处理配方），拖入视频队列，选择种子后批量处理。处理通过 FFmpeg 执行，包括数学叠加、像素变换、时间轴修改、编码参数调整等操作，使同一素材生成多个指纹不同的视频。

**核心价值：一键批量去重** — 自动生成随机化种子配方，将同一视频源产出多个平台无法识别为重复的变体。

### 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.x |
| 后端 | Rust (stable 1.85+) |
| 前端 | Vue 3 + TypeScript + Vite |
| UI 库 | Naive UI (暗色主题) |
| 状态管理 | Pinia (Composition API) |
| 视频处理 | FFmpeg (ffmpeg-sidecar 自动下载) |

### 功能特性

- **30 种操作类型** — 音频重采样、音量、音调、均衡器、声道操作、智能裁切、元数据写入/选择性擦除、视频变速、帧丢弃、时长裁剪、色相旋转、饱和度、亮度对比度、色彩平衡、胶片颗粒、高斯模糊、锐化、微旋转、微缩放、翻转、数学叠加、像素偏移、纯色叠加、渐变叠加、水印融合、GOP 修改、元数据擦除、重封装
- **三档强度** — 保守、标准、激进参数范围
- **GPU 加速** — NVENC 编码 + NVDec 解码 + CUDA 滤镜（scale_cuda、bilateral_cuda、transpose_cuda、yadif_cuda、pad_cuda）零拷贝 GPU 管线
- **两遍处理** — GPU 原生滤镜先执行（NVENC 压缩），CPU 专属滤镜在中间产物上后执行
- **GPU 自动降级** — GPU 编码器不可用或运行时失败时自动回退 CPU（libx264）
- **种子管理** — 生成、复制、删除、导出/导入种子，附带 schema 版本迁移
- **批量处理** — 队列视频，多选种子，带进度流和取消支持的批处理
- **MD5 校验** — 处理前后哈希对比，逐文件日志

### 系统要求

- Windows 10+ (x64) 或 Linux (x64/arm64)
- NVIDIA GPU 可选（用于硬件加速）
- FFmpeg 首次启动自动下载（BtbN 构建）

### 快速开始

```bash
# 安装依赖
npm install

# 开发运行
npm run tauri dev

# 生产构建
npm run tauri build
```

### 架构

```
视频队列 → 种子 → 批量处理
                    ↓
         ┌──────────────────────┐
         │  GPU Pass (第一遍)    │
input.mp4 → NVDec → CUDA 滤镜 → NVENC (-cq 28 压缩)
         │  scale_cuda、pad_cuda、bilateral_cuda、
         │  transpose_cuda、yadif_cuda
         └──────────────────────┘
                    ↓ temp_gpu.mp4
         ┌──────────────────────┐
         │  CPU Pass (第二遍)    │
temp.mp4 → CPU 滤镜 → 编码 → output.mp4
         │  select、setpts、trim、unsharp、
         │  hue、eq、rotate、geq、atrim
         └──────────────────────┘
```

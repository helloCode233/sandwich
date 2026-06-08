# Sandwich — 视频指纹批量修改工具

输入一个视频，输出 N 个平台无法识别为重复的变体。一键去重。

[English](README.md)

## 原理

通过 FFmpeg 对视频施加多层微小操作（裁切、色相、模糊、变速、像素叠加等），改变数字指纹但不改变观感。随机化组合与参数使每个输出视频指纹唯一。

## 技术栈

Tauri 2.x · Rust · Vue 3 · TypeScript · Naive UI · FFmpeg

## 特性

- **30 种操作类型** — 音频、色彩、几何、纹理、元数据、时间轴
- **GPU 两遍管线** — NVDec → CUDA 滤镜 → NVENC（第一遍），CPU 滤镜（第二遍）
- **自动降级** — GPU 不可用时静默回退 CPU
- **3 档强度** — 保守 / 标准 / 激进
- **种子系统** — 一键生成随机配方，导入/导出，schema 自动迁移
- **MD5 校验** — 逐文件处理前后哈希对比

## 快速开始

```bash
npm install
npm run tauri dev    # 开发
npm run tauri build  # 构建
```

FFmpeg 首次启动自动下载（BtbN 构建），无需手动安装。

## GPU 加速

NVIDIA 显卡自动启用 CUDA 滤镜加速。无 NVIDIA 时走 CPU，效果一致。

## 许可证

MIT

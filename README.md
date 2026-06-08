# Sandwich — Video Fingerprint Batch Modification Tool

Feed one video, get N fingerprint-distinct variants. One-click deduplication.

[中文文档](README_zh.md)

## How It Works

Applies layered micro-operations (crop, hue, blur, speed, pixel overlay, etc.) via FFmpeg to alter digital fingerprints without affecting visual quality. Randomized combinations and parameters make each output fingerprint unique.

## Tech Stack

Tauri 2.x · Rust · Vue 3 · TypeScript · Naive UI · FFmpeg

## Features

- **30 operation types** — audio, color, geometry, texture, metadata, timeline
- **GPU two-pass pipeline** — NVDec → CUDA filters → NVENC (pass 1), CPU filters (pass 2)
- **Auto fallback** — silently switches to CPU when GPU unavailable
- **3 strength tiers** — Conservative / Standard / Aggressive
- **Seeds** — one-click random recipe generation, import/export, schema auto-migration
- **MD5 verification** — per-file hash comparison before and after processing

## Quick Start

```bash
npm install
npm run tauri dev    # development
npm run tauri build  # production
```

FFmpeg auto-downloaded on first launch (BtbN builds). No manual installation required.

## GPU Acceleration

NVIDIA GPU auto-enables CUDA filters. Falls back to CPU transparently when unavailable.

## License

MIT

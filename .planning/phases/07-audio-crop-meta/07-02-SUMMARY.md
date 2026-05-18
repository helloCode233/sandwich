---
phase: 07-audio-crop-meta
plan: 02
subsystem: types, i18n
tags: [typescript, i18n, operation-types, phase-7]
requires: []
provides:
  - 30-member OperationType union (10 new Phase 7 types)
  - Seed.schemaVersion for migration tracking
  - Bilingual i18n labels for all new operation types
affects:
  - src/types/seed.ts (TypeScript type definitions)
  - src/locales/en.json (English display labels)
  - src/locales/zh-CN.json (Chinese display labels)
tech-stack:
  added: []
  patterns: []
key-files:
  created: []
  modified:
    - src/types/seed.ts (OperationType union 20→30 members, schemaVersion field)
    - src/locales/en.json (10 new operationType entries)
    - src/locales/zh-CN.json (10 new operationType entries)
decisions: []
metrics:
  duration: PT1M
  completed: "2026-05-18T11:43:30Z"
---

# Phase 7 Plan 2: TypeScript Frontend Type Extension and i18n Labels for 10 New Operations

**One-liner:** Extended the OperationType TypeScript union from 20 to 30 members covering Audio (5), Crop (1), Metadata (2), and Duration (2) operations, added schemaVersion to Seed interface, and added bilingual i18n labels in English and Chinese.

## Tasks Completed

| # | Task | Type | Commit | Files |
|---|------|------|--------|-------|
| 1 | Add 10 new OperationType union members and schemaVersion to Seed interface | auto | `4b32a7b` | src/types/seed.ts |
| 2 | Add 10 new i18n keys to English locale | auto | `7297eea` | src/locales/en.json |
| 3 | Add 10 new i18n keys to Chinese locale | auto | `0429cba` | src/locales/zh-CN.json |

## Summary

This plan extended the frontend TypeScript type definitions and i18n locale files to support Phase 7's 10 new operation types. All work was pure declaration changes -- no logic, no runtime behavior.

### Task 1: OperationType Union Extension

- Added 10 new string literal members to the `OperationType` union:
  - **Audio (5):** `audioResample`, `audioVolume`, `audioPitch`, `audioEQ`, `audioChannel`
  - **Crop (1):** `crop`
  - **Metadata (2):** `metadataWrite`, `metadataSelectiveErase`
  - **Duration (2):** `videoSpeed`, `trimEdges`
- Preserved `audioTweak` for backward deserialization compatibility
- Added `schemaVersion?: number` optional field to the `Seed` interface
- Union now has 30 members total (20 existing + 10 new)

### Task 2: English i18n Labels

- Added 10 English display labels to `operationTypes` in `src/locales/en.json`:
  - `"Audio Resample"`, `"Audio Volume"`, `"Pitch Shift"`, `"Equalizer"`, `"Channel Op"`
  - `"Smart Crop"`
  - `"Metadata Write"`, `"Selective Erase"`
  - `"Video Speed"`, `"Trim Edges"`
- Total operationType keys: 30
- Valid JSON verified

### Task 3: Chinese i18n Labels

- Added 10 Chinese display labels to `operationTypes` in `src/locales/zh-CN.json`:
  - `"音频重采样"`, `"音频音量"`, `"音调偏移"`, `"均衡器"`, `"声道操作"`
  - `"智能裁切"`
  - `"元数据写入"`, `"选择性擦除"`
  - `"视频变速"`, `"时长裁剪"`
- Total operationType keys: 30
- Valid JSON verified

## Verification Results

| Check | Result |
|-------|--------|
| vue-tsc --noEmit | PASS (exit 0) |
| 10 new union members present | PASS (grep confirmed all 10) |
| audioTweak preserved | PASS (grep confirmed) |
| schemaVersion on Seed | PASS (grep confirmed) |
| en.json valid JSON | PASS (JSON.parse confirmed) |
| zh-CN.json valid JSON | PASS (JSON.parse confirmed) |
| en.json operationType count | PASS (30 keys) |
| zh-CN.json operationType count | PASS (30 keys) |

## Deviations from Plan

None -- plan executed exactly as written.

## Known Stubs

None -- all i18n entries have real display strings. No placeholder text, hardcoded empty values, or unwired data sources.

## Threat Flags

None -- this plan modified only TypeScript type definitions and static locale JSON files. No I/O, no network endpoints, no external input, no process boundary crossing.

## Self-Check: PASSED

- [x] src/types/seed.ts exists with 30 union members and schemaVersion
- [x] src/locales/en.json exists with 10 new operationType entries, valid JSON
- [x] src/locales/zh-CN.json exists with 10 new operationType entries, valid JSON
- [x] Commit 4b32a7b exists (Task 1)
- [x] Commit 7297eea exists (Task 2)
- [x] Commit 0429cba exists (Task 3)

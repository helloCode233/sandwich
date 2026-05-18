# Phase 07: Audio, Crop, Metadata & Duration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-18
**Phase:** 07-audio-crop-meta
**Areas discussed:** 音频操作 (Audio), 裁切行为 (Crop), 元数据范围 (Metadata), 时长修改 (Duration), FrameDrop 默认化

---

## 音频操作 (Audio Operations)

| Option | Description | Selected |
|--------|-------------|----------|
| Full range | EQ bands, pitch shift, channel manipulation, reverb/delay, audio codec/bitrate | |
| Targeted additions | Add 2-3 high-impact types (e.g., pitch + EQ + noise profile) | |
| Deepen existing only | Keep existing 3 effects but increase parameter ranges | |
| 使用中文描述 | Free-text description in Chinese | ✓ |

**User's choice:** 重新采样 + 改变音量 + 音调偏移/EQ/声道操作等
**Notes:** ~5 new audio op types, replace/split existing AudioTweak

| Option | Description | Selected |
|--------|-------------|----------|
| 5 个左右 | Resample, Volume, Pitch shift, EQ bands, Channel remap — 5 new ops | ✓ |
| 3 个精选 | Resample, Volume, Pitch — replace AudioTweak's three sub-effects | |
| 全部音频滤镜 | Every FFmpeg audio filter as an operation | |

**Notes:** AudioTweak gets split/replaced

| Option | Description | Selected |
|--------|-------------|----------|
| 保守范围 | Pitch ±2 semitones, speed 0.97-1.03x, volume ±3dB, resample 22050-48000 | ✓ |
| 较大变化范围 | Pitch ±5 semitones, speed 0.9-1.1x, volume ±6dB | |
| 跟随三档强度 | Tier-driven parameter ranges | |

| Option | Description | Selected |
|--------|-------------|----------|
| 固定常见采样率 | 44100, 48000, 22050, 32000 | |
| 随机范围 | Random within 22050-48000 | ✓ |
| 采样率+比特率 | Change both sample rate and bitrate | |

---

## 裁切行为 (Crop Behavior)

| Option | Description | Selected |
|--------|-------------|----------|
| 每个种子都包含 | Guaranteed in every seed + also in random pool | ✓ |
| 随机池中抽取 | Only in random pool, like other ops | |
| 强制但不随机 | Guaranteed but not in random pool | |

| Option | Description | Selected |
|--------|-------------|----------|
| 1-5 px | Per edge 1-5 px (~0.1%-0.5% at 1080p) | |
| 0.5%-3% | Per edge 0.5%-3% — slightly perceptible | ✓ |
| 单边不对称 | Random single edge 1-10px | |

| Option | Description | Selected |
|--------|-------------|----------|
| 四边随机不对称 | Each side independent random 0.5%-3% | ✓ |
| 四边均匀 | All four sides same value | |
| 随机选边 | Randomly select 1-4 edges to crop | |

| Option | Description | Selected |
|--------|-------------|----------|
| 缩放回原尺寸 | Crop then scale back to original resolution | ✓ |
| 保留裁切尺寸 | Keep cropped dimensions | |
| 跟随强度档位 | Tier-driven ranges + scale back | |

| Option | Description | Selected |
|--------|-------------|----------|
| 三档强度控制 | Conservative/Standard/Aggressive tier ranges | |
| 不分强度档位 | Same range regardless of tier | |
| 跟随但不占步数 | Tier-driven, doesn't count toward step count | ✓ |

| Option | Description | Selected |
|--------|-------------|----------|
| 独立操作类型 | Independent OperationType variant | ✓ |
| executor 隐式注入 | Injected at executor layer | |
| 两种机制共存 | Both mechanisms | |

---

## 元数据范围 (Metadata Scope)

| Option | Description | Selected |
|--------|-------------|----------|
| 写入假字段 + 选择性擦除 | Write fake + selective erase + keep full erase | ✓ |
| 只做选择性擦除 | Keep full erase, add selective only | |
| 完整假元数据替换 | Full fake identity replacement | |

| Option | Description | Selected |
|--------|-------------|----------|
| 常用字段 | creation_time, title, author, comment, copyright, encoder | ✓ |
| 所有可用字段 | All possible metadata fields | |
| 高影响字段 | Only rotation/tag + creation_time | |

| Option | Description | Selected |
|--------|-------------|----------|
| 小偏移 + 词库随机 | creation_time ±30 days; text from word lists | ✓ |
| 完全随机 | Random UUID/hex for all fields | |
| 真实元数据采样 | Sample from real metadata database | |

| Option | Description | Selected |
|--------|-------------|----------|
| 3 个操作类型 | MetadataWrite + MetadataSelectiveErase + existing MetadataErase | ✓ |
| 2 个（W+E 合并） | Write + erase combined | |
| 单个新类型 | Single type with parameter differentiation | |

| Option | Description | Selected |
|--------|-------------|----------|
| 随机选字段擦除 | Randomly select 1-3 fields to erase | |
| 按类别擦除 | Time/device/description categories, 1-3 categories | ✓ |
| 单字段精确擦除 | Single field precise removal | |

| Option | Description | Selected |
|--------|-------------|----------|
| 跟随三档强度 | Tier affects write/erase behavior | |
| 不跟随强度 | Same behavior regardless of tier | ✓ |
| 跟随但不写敏感字段 | Tier-driven but skip sensitive fields | |

---

## 时长修改 (Duration Modification)

| Option | Description | Selected |
|--------|-------------|----------|
| 变速 + 微裁剪 | VideoSpeed (setpts+atempo) + TrimEdges | ✓ |
| 只做变速 | VideoSpeed only | |
| 只做时长裁剪 | Trim only | |

| Option | Description | Selected |
|--------|-------------|----------|
| 0.98-1.02x | ±2% speed — barely perceptible | |
| 0.95-1.05x | ±5% speed — slightly perceptible | ✓ |
| 跟随三档强度 | Tier-driven speed ranges | |

| Option | Description | Selected |
|--------|-------------|----------|
| 头/尾/双端随机 | Random head/tail/both, 1-30 frames | ✓ |
| 只裁开头 | Head only, 1-60 frames | |
| 跟随强度档位 | Tier-driven trim frame counts | |

| Option | Description | Selected |
|--------|-------------|----------|
| 2 个操作类型 | VideoSpeed + TrimEdges independent ops | ✓ |
| 合并为 1 个 | DurationModify with sub-effects | |
| 拆为 3 个 | VideoSpeed, TrimStart, TrimEnd | |

---

## FrameDrop 默认化

**User's choice:** FrameDrop as default operation — real frame dropping (framestep), every 30-50 frames drop 1 frame, follows strength tier
**Notes:** Reverts Phase 6 setpts jitter approach back to true frame dropping. Guaranteed in every seed + random pool.

---

## Claude's Discretion

- 新音频 OperationType 变体名称和数量
- 每个音频操作的 FFmpeg 滤镜选择
- 假元数据词库内容
- 元数据类别字段映射
- 裁切 crop+scale filter builder 实现
- VideoSpeed setpts/atempo 同步机制
- FrameDrop framestep 实现细节
- 随机池权重重分配
- Step count 下限调整
- i18n key 新增
- OperationType 枚举双侧扩展
- 存量种子迁移

## Deferred Ideas

None from this discussion. Prior deferred items (SEED-COMPLEX-01, GPU manual selector, code signing) remain deferred.

//! Integration tests for Phase 7 filter builders.
//! Each test generates a short test video, applies a filter, and verifies
//! the output is valid. Tests skip gracefully if FFmpeg is not available.
//!
//! Run with: cargo test --test filter_integration_tests -- --test-threads=1
//! (single-threaded to avoid FFmpeg temp file conflicts)

use ffmpeg_sidecar::ffprobe::ffprobe_path;
use ffmpeg_sidecar::paths::ffmpeg_path;
use sandwich_lib::ffmpeg::filters::{
    build_audio_channel_filter, build_audio_eq_filter, build_audio_pitch_filter,
    build_audio_resample_filter, build_audio_volume_filter, build_crop_filter,
    build_frame_drop_filter, build_metadata_selective_erase_filter,
    build_metadata_write_filter, build_trim_edges_filter, build_video_speed_filter,
    MetadataContext,
};
use sandwich_lib::ffmpeg::probe::probe_global_metadata;
use sandwich_lib::models::seed::{Operation, OperationType};
use std::path::PathBuf;
use std::process::Command;

// =========================================================================
// Helper functions
// =========================================================================

/// Check if FFmpeg is available on this machine.
/// Tries ffmpeg-sidecar managed binary first, then system PATH.
fn ffmpeg_available() -> bool {
    let path = ffmpeg_path();
    if path.exists() {
        return true;
    }
    // Fallback: check system PATH
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Generate a short test video (2 seconds, 30fps, 320x240, color bars with tone).
/// Returns the path to the generated file.
/// The caller must clean up the file after the test.
fn generate_test_video() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("sandwich_integration_tests");
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir: {}", e))?;
    let output = dir.join("test_source.mp4");

    // Skip generation if already exists (cached across test runs)
    if output.exists() {
        return Ok(output);
    }

    let ffmpeg = ffmpeg_path();
    let ffmpeg_str = if ffmpeg.exists() {
        ffmpeg.to_string_lossy().to_string()
    } else {
        "ffmpeg".to_string()
    };

    let status = Command::new(&ffmpeg_str)
        .args([
            "-f", "lavfi",
            "-i", "testsrc=duration=2:size=320x240:rate=30",
            "-f", "lavfi",
            "-i", "sine=frequency=440:duration=2",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-c:a", "aac",
            "-shortest",
            "-y",
        ])
        .arg(output.to_string_lossy().to_string())
        .status()
        .map_err(|e| format!("FFmpeg spawn failed: {}", e))?;

    if status.success() {
        Ok(output)
    } else {
        Err("Failed to generate test video".to_string())
    }
}

/// Get ffprobe path for metadata queries.
fn ffprobe_bin() -> PathBuf {
    let path = ffprobe_path();
    if path.exists() { path } else { PathBuf::from("ffprobe") }
}

/// Run ffprobe to count video frames in a file.
fn count_frames(filepath: &str) -> Result<u32, String> {
    let output = Command::new(ffprobe_bin())
        .args([
            "-v", "quiet",
            "-select_streams", "v:0",
            "-show_entries", "stream=nb_read_frames",
            "-of", "csv=p=0",
            filepath,
        ])
        .output()
        .map_err(|e| format!("ffprobe: {}", e))?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    text.parse::<u32>().map_err(|e| format!("parse frame count '{}': {}", text, e))
}

/// Get video resolution via ffprobe.
fn get_resolution(filepath: &str) -> Result<(u32, u32), String> {
    let output = Command::new(ffprobe_bin())
        .args([
            "-v", "quiet",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height",
            "-of", "csv=p=0",
            filepath,
        ])
        .output()
        .map_err(|e| format!("ffprobe: {}", e))?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let parts: Vec<&str> = text.split(',').collect();
    if parts.len() >= 2 {
        let w: u32 = parts[0].parse().map_err(|e| format!("parse width: {}", e))?;
        let h: u32 = parts[1].parse().map_err(|e| format!("parse height: {}", e))?;
        Ok((w, h))
    } else {
        Err(format!("Unexpected resolution output: {}", text))
    }
}

/// Run FFmpeg with the given filter args on a test video.
/// Returns Ok(true) if FFmpeg exits successfully, Ok(false) otherwise.
fn run_filter_on_test_video(
    input_path: &str,
    output_path: &str,
    vf_args: &[String],
    af_args: &[String],
    other_args: &[String],
) -> Result<bool, String> {
    let ffmpeg = ffmpeg_path();
    let ffmpeg_str = if ffmpeg.exists() {
        ffmpeg.to_string_lossy().to_string()
    } else {
        "ffmpeg".to_string()
    };

    let mut cmd = Command::new(&ffmpeg_str);
    cmd.arg("-i").arg(input_path);

    // Apply video filters
    if !vf_args.is_empty() {
        cmd.arg("-vf").arg(vf_args.join(","));
    }
    // Apply audio filters
    if !af_args.is_empty() {
        cmd.arg("-af").arg(af_args.join(","));
    }
    // Apply other (metadata, GOP, etc.)
    for arg in other_args {
        cmd.arg(arg);
    }

    cmd.arg("-y").arg(output_path);

    let output = cmd.output().map_err(|e| format!("FFmpeg spawn: {}", e))?;
    Ok(output.status.success())
}

/// Build a test Operation with the given type and params.
fn make_test_op(op_type: OperationType, params: serde_json::Value) -> Operation {
    Operation { op_type, start_frame: 0, duration_frames: 0, params }
}

// =========================================================================
// Filter integration tests — 1 per new Phase 7 filter type
// =========================================================================

#[test]
fn test_audio_resample_produces_valid_output() {
    if !ffmpeg_available() { return; }
    let src = generate_test_video().expect("test video");
    let out = src.with_file_name("test_resample_out.mp4");

    let op = make_test_op(OperationType::AudioResample, serde_json::json!({"sampleRate": 32000}));
    let args = build_audio_resample_filter(&op).expect("build filter");
    let af_args = vec![args[1].clone()];

    let ok = run_filter_on_test_video(
        &src.to_string_lossy(), &out.to_string_lossy(),
        &[], &af_args, &[],
    ).expect("FFmpeg run");
    assert!(ok, "AudioResample filter should produce valid output");
    assert!(out.exists() && out.metadata().map(|m| m.len() > 0).unwrap_or(false),
        "Output file should exist and be non-empty");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_audio_volume_produces_valid_output() {
    if !ffmpeg_available() { return; }
    let src = generate_test_video().expect("test video");
    let out = src.with_file_name("test_volume_out.mp4");

    let op = make_test_op(OperationType::AudioVolume, serde_json::json!({"db": 1.5}));
    let args = build_audio_volume_filter(&op).expect("build filter");
    let af_args = vec![args[1].clone()];

    let ok = run_filter_on_test_video(
        &src.to_string_lossy(), &out.to_string_lossy(),
        &[], &af_args, &[],
    ).expect("FFmpeg run");
    assert!(ok, "AudioVolume filter should produce valid output");
    assert!(out.exists());
    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_audio_pitch_produces_valid_output() {
    if !ffmpeg_available() { return; }
    let src = generate_test_video().expect("test video");
    let out = src.with_file_name("test_pitch_out.mp4");

    // Pitch shift +1 semitone, source sample rate 48000
    let op = make_test_op(OperationType::AudioPitch, serde_json::json!({
        "pitchFactor": 1.0595,  // +1 semitone
        "originalRate": 48000,
    }));
    let args = build_audio_pitch_filter(&op).expect("build filter");
    let af_args = vec![args[1].clone()];

    let ok = run_filter_on_test_video(
        &src.to_string_lossy(), &out.to_string_lossy(),
        &[], &af_args, &[],
    ).expect("FFmpeg run");
    assert!(ok, "AudioPitch filter should produce valid output (asetrate+atempo+aresample chain)");
    assert!(out.exists());
    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_audio_eq_produces_valid_output() {
    if !ffmpeg_available() { return; }
    let src = generate_test_video().expect("test video");
    let out = src.with_file_name("test_eq_out.mp4");

    let op = make_test_op(OperationType::AudioEQ, serde_json::json!({
        "frequency": 1000,
        "gain": 3.0,
        "width": 200,
    }));
    let args = build_audio_eq_filter(&op).expect("build filter");
    let af_args = vec![args[1].clone()];

    let ok = run_filter_on_test_video(
        &src.to_string_lossy(), &out.to_string_lossy(),
        &[], &af_args, &[],
    ).expect("FFmpeg run");
    assert!(ok, "AudioEQ filter should produce valid output");
    assert!(out.exists());
    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_audio_channel_produces_valid_output() {
    if !ffmpeg_available() { return; }
    let src = generate_test_video().expect("test video");
    let out = src.with_file_name("test_channel_out.mp4");

    let op = make_test_op(OperationType::AudioChannel, serde_json::json!({"mode": "swap"}));
    let args = build_audio_channel_filter(&op).expect("build filter");
    let af_args = vec![args[1].clone()];

    let ok = run_filter_on_test_video(
        &src.to_string_lossy(), &out.to_string_lossy(),
        &[], &af_args, &[],
    ).expect("FFmpeg run");
    assert!(ok, "AudioChannel filter should produce valid output");
    assert!(out.exists());
    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_crop_scale_restores_dimensions() {
    if !ffmpeg_available() { return; }
    let src = generate_test_video().expect("test video");
    let out = src.with_file_name("test_crop_out.mp4");

    // Source video is 320x240. Crop 2% from each side.
    let op = make_test_op(OperationType::Crop, serde_json::json!({
        "leftPct": 2.0,
        "rightPct": 2.0,
        "topPct": 2.0,
        "bottomPct": 2.0,
        "origW": 320,
        "origH": 240,
    }));
    let args = build_crop_filter(&op).expect("build filter");
    let vf_args = vec![args[1].clone()];

    let ok = run_filter_on_test_video(
        &src.to_string_lossy(), &out.to_string_lossy(),
        &vf_args, &[], &[],
    ).expect("FFmpeg run");
    assert!(ok, "Crop+scale filter should produce valid output");

    // Verify dimensions match original (D-06: scale back)
    let (w, h) = get_resolution(&out.to_string_lossy()).expect("ffprobe resolution");
    assert_eq!(w, 320, "Crop output width must match original 320, got {}", w);
    assert_eq!(h, 240, "Crop output height must match original 240, got {}", h);

    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_metadata_write_injects_fake_fields() {
    if !ffmpeg_available() { return; }
    let src = generate_test_video().expect("test video");
    let out = src.with_file_name("test_meta_write_out.mp4");

    let op = make_test_op(OperationType::MetadataWrite, serde_json::json!({
        "creationTime": "2026-01-15T12:00:00",
        "title": "Integration Test",
        "author": "sandwich-ci",
        "comment": "auto-generated",
        "copyright": "Copyright 2026",
        "encoder": "Sandwich 0.1.0",
    }));
    let args = build_metadata_write_filter(&op).expect("build filter");

    let ok = run_filter_on_test_video(
        &src.to_string_lossy(), &out.to_string_lossy(),
        &[], &[], &args,
    ).expect("FFmpeg run");
    assert!(ok, "MetadataWrite should produce valid output");

    // Verify metadata was injected
    let tags = probe_global_metadata(&out.to_string_lossy(), None).unwrap_or_default();
    assert_eq!(tags.get("title").map(|s| s.as_str()), Some("Integration Test"),
        "title metadata should be injected");
    assert_eq!(tags.get("author").map(|s| s.as_str()), Some("sandwich-ci"),
        "author metadata should be injected");

    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_metadata_selective_erase_keeps_non_targeted_fields() {
    if !ffmpeg_available() { return; }
    let src = generate_test_video().expect("test video");
    let out = src.with_file_name("test_meta_sel_out.mp4");

    // First, write some metadata so we have fields to selectively erase
    let write_op = make_test_op(OperationType::MetadataWrite, serde_json::json!({
        "title": "Keep Me",
        "author": "Erase Me",  // "author" is in description category
    }));
    let write_args = build_metadata_write_filter(&write_op).expect("build filter");
    let tmp = src.with_file_name("test_meta_sel_tmp.mp4");
    run_filter_on_test_video(
        &src.to_string_lossy(), &tmp.to_string_lossy(),
        &[], &[], &write_args,
    ).expect("write metadata");

    // Now probe and selectively erase "description" category
    let tags = probe_global_metadata(&tmp.to_string_lossy(), None).unwrap_or_default();
    let ctx = MetadataContext { fields: tags };

    let erase_op = make_test_op(OperationType::MetadataSelectiveErase, serde_json::json!({
        "categories": ["description"],  // erase title, author, comment, copyright, etc.
    }));
    let erase_args = build_metadata_selective_erase_filter(&erase_op, Some(&ctx))
        .expect("build selective erase filter");

    let ok = run_filter_on_test_video(
        &tmp.to_string_lossy(), &out.to_string_lossy(),
        &[], &[], &erase_args,
    ).expect("FFmpeg run selective erase");
    assert!(ok, "MetadataSelectiveErase should produce valid output");

    // Verify description fields were erased, non-description fields preserved
    let out_tags = probe_global_metadata(&out.to_string_lossy(), None).unwrap_or_default();
    // "author" is in description category → should be erased
    assert!(out_tags.get("author").map(|s| s.as_str()) != Some("Erase Me"),
        "Author should be erased (description category)");
    // "title" is also in description category → should be erased
    assert!(out_tags.get("title").map(|s| s.as_str()) != Some("Keep Me"),
        "Title should be erased (description category)");

    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_video_speed_produces_valid_output() {
    if !ffmpeg_available() { return; }
    let src = generate_test_video().expect("test video");
    let out = src.with_file_name("test_speed_out.mp4");

    let op = make_test_op(OperationType::VideoSpeed, serde_json::json!({"speedFactor": 1.03}));
    let args = build_video_speed_filter(&op).expect("build filter");
    // args = ["-vf", "setpts=...", "-af", "atempo=..."]
    let vf_args = vec![args[1].clone()];
    let af_args = vec![args[3].clone()];

    let ok = run_filter_on_test_video(
        &src.to_string_lossy(), &out.to_string_lossy(),
        &vf_args, &af_args, &[],
    ).expect("FFmpeg run");
    assert!(ok, "VideoSpeed should produce valid output with sync'd setpts+atempo");
    assert!(out.exists());
    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_trim_edges_produces_valid_output() {
    if !ffmpeg_available() { return; }
    let src = generate_test_video().expect("test video");
    let out = src.with_file_name("test_trim_out.mp4");

    // Trim 10 frames from head
    let op = make_test_op(OperationType::TrimEdges, serde_json::json!({
        "mode": "head",
        "trimFrames": 10,
        "totalFrames": 60,  // 2s * 30fps = 60 frames
    }));
    let args = build_trim_edges_filter(&op).expect("build filter");
    // args = ["-vf", "trim=...", "-af", "atrim=..."]
    let vf_args = vec![args[1].clone()];
    let af_args = vec![args[3].clone()];

    let ok = run_filter_on_test_video(
        &src.to_string_lossy(), &out.to_string_lossy(),
        &vf_args, &af_args, &[],
    ).expect("FFmpeg run");
    assert!(ok, "TrimEdges filter should produce valid output");
    assert!(out.exists());
    let _ = std::fs::remove_file(&out);
}

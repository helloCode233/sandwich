---
status: partial
phase: 04-integration-polish
source: [04-VERIFICATION.md]
started: 2026-05-14T14:58:00+08:00
updated: 2026-05-14T14:58:00+08:00
---

## Current Test

[awaiting human testing]

## Tests

### 1. Test the cancel flow with an actual FFmpeg process running
expected: Clicking Cancel shows the dialog. On confirm, the banner shows 'Cancelling...', the cancel button becomes a disabled loading state, FFmpeg terminates, and the app returns to idle with appropriate batch-cancelled summary.
result: [pending]

### 2. Test per-file progress bars during real FFmpeg processing
expected: During batch processing, the currently-processing file in the queue shows an NProgress bar with percentage, current frame / total frames, and estimated remaining minutes. The bar progresses smoothly. The overall banner shows completed/total.
result: [pending]

### 3. Test the E2E workflow: generate seed -> drag videos -> select seed -> process -> review summary
expected: All steps work without error. Progress visible throughout. Summary shows correct succeeded/failed counts with output paths. Clear Results returns to idle.
result: [pending]

### 4. Verify completion summary with mixed succeeded/failed results
expected: After a batch where some files succeed and some fail, the summary shows correct counts, per-file output paths for succeeded files, per-file error messages for failed files. Banner title correctly indicates completion (not cancellation).
result: [pending]

## Summary

total: 4
passed: 0
issues: 0
pending: 4
skipped: 0
blocked: 0

## Gaps

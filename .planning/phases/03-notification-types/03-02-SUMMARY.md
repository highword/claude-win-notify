---
phase: 3
plan: 2
subsystem: toast-display
tags: [toast, hero-image, audio, winrt, xml]
dependency_graph:
  requires: [03-01]
  provides: [show_typed_toast, build_toast_xml, classification-wiring]
  affects: [src/toast.rs, src/hook.rs, src/main.rs]
tech_stack:
  added: []
  patterns: [xml-builder-extraction, graceful-degradation, classification-pipeline]
key_files:
  created: []
  modified: [src/toast.rs, src/hook.rs, src/main.rs]
decisions:
  - "build_toast_xml extracted as separate testable function (no WinRT dependency for tests)"
  - "audio_loop always false per D-12 design (Error uses Looping.Alarm sound but single-play)"
  - "Hero image path converted to forward slashes for file:/// URI compatibility"
metrics:
  duration: "3m"
  completed: "2026-05-20T16:30:53Z"
  tasks_completed: 3
  tasks_total: 3
  tests_added: 7
  files_modified: 3
---

# Phase 3 Plan 2: Toast Display Integration -- Hero Image & Sound Summary

Typed toast display with per-notification hero images and system sounds via build_toast_xml extraction pattern.

## Commits

| Task | Commit  | Message                                                    |
|------|---------|------------------------------------------------------------|
| 1    | c2814c1 | feat(03-02): add show_typed_toast with hero image and audio support |
| 2    | 00a62e2 | feat(03-02): wire classification pipeline into hook handlers |
| 3    | 43a0f37 | test(03-02): add Toast XML generation unit tests           |

## What Was Built

### Task 1: show_typed_toast with Hero Image & Audio

- Added `build_toast_xml()` helper that generates Toast XML with configurable hero image and audio
- Added `show_typed_toast(ntype, body, attribution)` that calls `ensure_hero_image`, gets audio source, and displays the toast
- Hero image element uses `<image placement="hero" src="file:///{forward-slash-path}"/>`
- Audio element includes `loop="false"` attribute
- Graceful degradation: if hero image extraction fails, toast displays without image

### Task 2: Classification Pipeline Wiring

- Replaced hardcoded `show_toast("Claude Code", ...)` calls with classification pipeline
- `handle_stop` now calls `classify_stop` -> `body_text` -> `show_typed_toast`
- `handle_notification` now calls `classify_notification` -> `body_text` -> `show_typed_toast`
- Extracted `extract_project_name()` helper for DRY

### Task 3: Toast XML Unit Tests

7 new tests covering:
- Hero image presence/absence in XML
- Audio source correctness per type
- loop="false" attribute on Error type
- XML escaping of special characters
- Windows path forward-slash normalization
- Attribution text placement

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added mod declarations to main.rs**
- **Found during:** Task 1
- **Issue:** `src/toast.rs` uses `crate::assets` and `crate::notification` but `main.rs` (the binary crate root) did not declare these modules. Plan 03-01 only registered them in `lib.rs`.
- **Fix:** Added `mod assets;` and `mod notification;` to `src/main.rs`
- **Files modified:** src/main.rs
- **Commit:** c2814c1

## Verification

```
cargo check: PASS (0 errors, warnings only for dead_code on deprecated show_toast)
cargo test:  PASS (59 tests total - 31 unit x2 targets + 7 integration + 21 notification_types)
```

## Self-Check: PASSED

- [x] src/toast.rs exists
- [x] src/hook.rs exists
- [x] src/main.rs exists
- [x] 03-02-SUMMARY.md exists
- [x] Commit c2814c1 found in git log
- [x] Commit 00a62e2 found in git log
- [x] Commit 43a0f37 found in git log

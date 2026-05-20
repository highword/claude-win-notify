---
phase: 3
plan: "03-01"
subsystem: notification-classification
tags: [notification-types, classification, assets, hero-images]
dependency_graph:
  requires: [phase-2-hook-toast]
  provides: [NotificationType-enum, classify_stop, classify_notification, hero-images, assets-module]
  affects: [03-02-toast-display]
tech_stack:
  added: []
  patterns: [include_bytes-embedding, localappdata-extraction, enum-dispatch]
key_files:
  created:
    - src/notification.rs
    - src/assets.rs
    - assets/hero-task-complete.png
    - assets/hero-permission.png
    - assets/hero-question.png
    - assets/hero-error.png
  modified:
    - src/lib.rs
decisions:
  - "Refined ERROR_PATTERNS: replaced 'api error' with 'api error:' and 'api error occurred' to avoid false-positive matching on user task descriptions (D-03 compliance)"
metrics:
  duration: "4m 28s"
  completed: "2026-05-21T00:24:42Z"
  tasks_completed: 5
  tasks_total: 5
  tests_added: 14
  test_total: 24
---

# Phase 3 Plan 01: Notification Type Classification & Assets Module Summary

NotificationType enum with 4-variant classification (TaskComplete, PermissionRequest, Question, Error), conservative error pattern matching, question-mark detection, and include_bytes hero image extraction to LOCALAPPDATA.

## Commits

| Task | Title | Commit | Key Files |
|------|-------|--------|-----------|
| 1 | Create NotificationType enum and classification functions | `744baac` | src/notification.rs |
| 2 | Create hero image PNG assets | `5ef338b` | assets/*.png (4 files) |
| 3 | Create assets embedding and extraction module | `8abd291` | src/assets.rs |
| 4 | Register new modules in lib.rs | `6006188` | src/lib.rs |
| 5 | Unit tests for classification logic | `7a21aeb` | src/notification.rs |

## Implementation Details

### NotificationType Enum (src/notification.rs)

- 4 variants: `TaskComplete`, `PermissionRequest`, `Question`, `Error`
- Methods: `title()`, `audio_src()`, `hero_filename()`, `audio_loop()`
- `classify_stop()`: implements D-02 priority (Error > Question > TaskComplete)
- `classify_notification()`: implements D-17/D-18/D-19 with bug #11964 fallback
- `body_text()`: generates type-specific toast body content

### Assets Module (src/assets.rs)

- 4 hero images embedded via `include_bytes!()` (total ~2KB compiled)
- `assets_dir()` returns `%LOCALAPPDATA%\claude-win-notify\assets`
- `ensure_hero_image()` extracts on first run, skips if file exists
- Graceful degradation: logs error via `crate::log::log_error()`, returns `None`

### Hero Images (assets/)

- 364x180 solid-color PNGs, each under 1KB
- Green (#4CAF50), Orange (#FF9800), Blue (#2196F3), Red (#F44336)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Refined ERROR_PATTERNS to prevent false positives**
- **Found during:** Task 5 (test writing revealed the issue)
- **Issue:** Pattern "api error" in ERROR_PATTERNS would match user messages like "Fixed the API error handling", violating D-03's requirement to only match system-level errors
- **Fix:** Replaced "api error" with "api error:" and "api error occurred" — these only match actual system error messages from Claude Code, not user task descriptions
- **Files modified:** src/notification.rs
- **Commit:** `7a21aeb`

## Verification

```
cargo check: PASS (warnings only - dead code in error.rs and unused HookInput fields)
cargo test: PASS (24 unit tests + 7 integration tests, 0 failures)
```

All 14 classification tests pass, covering:
- Stop hook: default, error patterns (3), case-insensitivity, false-positive rejection, question mark, trailing whitespace, priority order, None message
- Notification hook: permission_prompt, missing type with permission message, other type, no type/no permission

## Self-Check: PASSED

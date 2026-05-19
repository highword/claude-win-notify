---
phase: 1
plan: 01-D
subsystem: focus
tags: [win32, setforegroundwindow, p-invoke, windows-rs, nativeaot]
dependency_graph:
  requires: [01-B, 01-C]
  provides: [focus-validated]
  affects: [spike/csharp/Program.cs, spike/rust/src/main.rs]
tech_stack:
  added: [user32.dll P/Invoke, Win32_UI_Input_KeyboardAndMouse, Win32_System_Threading]
  patterns: [fallback-chain, EnumWindows-by-PID, foreground-verification]
key_files:
  created:
    - spike/test-focus.ps1
    - spike/run-focus-test.ps1
    - spike/focus-results.md
  modified:
    - spike/csharp/Program.cs
    - spike/rust/src/main.rs
    - spike/rust/Cargo.toml
decisions:
  - "Strategy 1 (direct SetForegroundWindow) sufficient for protocol-activated processes"
  - "Fallback chain implemented as defense-in-depth for edge cases"
metrics:
  duration: "9m 5s"
  completed: 2026-05-20T02:23:00Z
  tasks_completed: 3
  tasks_total: 3
---

# Phase 1 Plan D: SetForegroundWindow Validation Summary

SetForegroundWindow with 4-strategy fallback chain validated for both C# NativeAOT and Rust; all 4 scenarios PASS using Strategy 1 (direct) due to foreground rights from protocol activation chain.

## Tasks Completed

| Task | Title | Commit | Key Change |
|------|-------|--------|------------|
| D1 | C# SetForegroundWindow with fallback chain | ef20984 | Added WindowFocus class with P/Invoke + 4-strategy focus logic |
| D2 | Rust SetForegroundWindow with fallback chain | fbc01bf | Added find_window_by_pid + focus_window with EnumWindows callback |
| D3 | 4-scenario focus test matrix | 05eaaef | Automated test, both stacks pass all scenarios |

## Key Results

### Both Stacks: ALL 4 SCENARIOS PASS

| Scenario | Required | C# Result | Rust Result |
|----------|----------|-----------|-------------|
| Minimized/idle | MUST PASS | PASS (Strategy 1) | PASS (Strategy 1) |
| Multiple windows | MUST PASS | PASS (Strategy 1) | PASS (Strategy 1) |
| User typing | May flash | PASS (Strategy 1) | PASS (Strategy 1) |
| Fullscreen (sim) | May flash | PASS (Strategy 1) | PASS (Strategy 1) |

### Implementation Details

**C# (NativeAOT):**
- WindowFocus static class with 10 P/Invoke declarations
- EnumWindows with delegate for PID-based window lookup
- NativeAOT publish succeeds (3.4 MB binary), P/Invoke fully compatible

**Rust (windows-rs 0.62):**
- Uses Win32_System_Threading for AttachThreadInput + GetCurrentThreadId
- Uses Win32_UI_Input_KeyboardAndMouse for keybd_event (VK_MENU hack)
- EnumWindows with LPARAM struct pointer for safe data passing
- Zero warnings in release build

### Fallback Chain (Both Stacks)

```
1. SetForegroundWindow(hwnd)           → Direct attempt
2. AttachThreadInput + retry           → Thread attachment trick
3. keybd_event(VK_MENU) + retry       → Alt key simulation
4. ShowWindow(MINIMIZE) + RESTORE     → Force restore
```

Strategy 1 succeeds in ALL test scenarios because protocol activation (toast click -> exe launch) grants foreground rights per Windows API contract.

## Decisions Made

1. **Fallback chain retained despite Strategy 1 always working** — defense-in-depth for edge cases (background daemons, long-running processes) that cannot be reliably tested in automated runs.
2. **Foreground verification via GetForegroundWindow comparison** — provides objective PASS/FAIL instead of relying on human visual inspection.
3. **Exit code convention established** — 0=PASS, 1=ERROR (no window found), 2=FLASH (taskbar only).

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- [x] spike/csharp/Program.cs exists and contains WindowFocus class
- [x] spike/rust/src/main.rs exists and contains focus_window function
- [x] spike/test-focus.ps1 exists (interactive test script)
- [x] spike/run-focus-test.ps1 exists (automated test script)
- [x] spike/focus-results.md exists with results for both stacks
- [x] Commit ef20984 exists (C# focus)
- [x] Commit fbc01bf exists (Rust focus)
- [x] Commit 05eaaef exists (test matrix)

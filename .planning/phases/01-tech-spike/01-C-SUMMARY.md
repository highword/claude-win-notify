---
phase: 1
plan: 01-C
title: "Protocol Activation Validation (Both Stacks)"
subsystem: protocol-activation
tags: [spike, protocol, registry, uri-parsing, windows]
dependency_graph:
  requires: [01-A]
  provides: [protocol-activation-validated]
  affects: [01-E]
tech_stack:
  added: [url-crate-2.5]
  patterns: [custom-protocol-handler, registry-based-activation, manual-query-parsing]
key_files:
  created:
    - spike/register-protocol.ps1
    - spike/protocol-results.md
  modified:
    - spike/csharp/Program.cs
    - spike/rust/src/main.rs
    - spike/rust/Cargo.toml
decisions:
  - "C# uses manual query string parsing instead of System.Web (NativeAOT-safe)"
  - "Rust uses url crate v2 for robust percent-decoding and query pair extraction"
  - "Both stacks share identical protocol registration mechanism (PowerShell script)"
metrics:
  duration: "5 minutes"
  completed: "2026-05-20T18:09:43Z"
  tasks_completed: 4
  tasks_total: 4
  files_created: 2
  files_modified: 3
---

# Phase 1 Plan C: Protocol Activation Validation Summary

Custom protocol URI scheme registration and argument parsing validated for both C# NativeAOT and Rust stacks in under 5 minutes.

## What Was Done

### Task C1: Register claude-notify:// protocol in registry
- Created `spike/register-protocol.ps1` — parameterized script that writes HKCU registry keys
- Verified: `URL Protocol` property exists, `shell\open\command` contains `--focus "%1"` pattern
- Script supports re-registration (switching between C# and Rust exe targets)

### Task C2: Add protocol argument parsing to C# spike
- Extended `Program.cs` with `--focus <uri>` handling alongside existing toast mode
- Manual query string parsing (splits on `&` and `=`, calls `Uri.UnescapeDataString`)
- Avoids `System.Web.HttpUtility` dependency (not NativeAOT-safe)
- NativeAOT publish succeeds with no trim warnings
- Binary size: 3.3 MB (unchanged from Plan B baseline)

### Task C3: Add protocol argument parsing to Rust spike
- Added `url = "2"` dependency to Cargo.toml
- Extended `main.rs` with `handle_focus()` function using `Url::query_pairs()`
- Full percent-decoding including CJK characters (`%E6%B5%8B%E8%AF%95` -> `测试`)
- Binary size: 381 KB (up from ~280 KB baseline, +~100 KB for url/icu crates)

### Task C4: End-to-end protocol activation test
- Registered protocol pointing to each exe, invoked via `Start-Process "claude-notify://..."`
- Windows correctly launches registered exe with full URI as argument
- Both stacks parse session, pid, and CJK-encoded parameters correctly
- Results documented in `spike/protocol-results.md`

## Key Results

| Metric | C# NativeAOT | Rust |
|--------|--------------|------|
| Protocol parsing | PASS | PASS |
| CJK URL decoding | PASS | PASS |
| E2E activation | PASS | PASS |
| Binary size | 3.3 MB | 381 KB |
| Extra dependencies | None (manual parsing) | url 2.5 |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] vswhere.exe not in PATH for NativeAOT build**
- **Found during:** Task C1 (pre-build for registration)
- **Issue:** `dotnet publish` NativeAOT linker couldn't find vswhere.exe
- **Fix:** Added Visual Studio Installer directory to PATH before build
- **Impact:** None — build succeeds with correct PATH

No other deviations. Plan executed as written.

## Commits

| Task | Commit | Message |
|------|--------|---------|
| C1 | 30b4023 | feat(01-C): add protocol registration script |
| C2 | cb16c3f | feat(01-C): add protocol URI parsing to C# spike |
| C3 | d627513 | feat(01-C): add protocol URI parsing to Rust spike |
| C4 | 7f3ba9e | docs(01-C): record protocol activation E2E test results |

## Known Stubs

None. Both implementations are fully functional for the spike's requirements.

## Self-Check: PASSED

All 5 files verified present. All 4 commit hashes found in git log.

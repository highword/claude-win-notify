---
phase: 1
plan: E
subsystem: tech-spike-decision
tags: [comparison, decision, measurement, rust, csharp]
dependency_graph:
  requires: [01-B, 01-C, 01-D]
  provides: [tech-stack-decision, spike-results-report]
  affects: [phase-2-planning, PROJECT.md, STATE.md]
tech_stack:
  validated: [rust/windows-rs, rust/url, csharp/nativeaot, csharp/cswinrt]
  chosen: rust
  patterns: [single-exe, zero-dependency, native-compilation]
key_files:
  created:
    - spike/RESULTS.md
    - spike/measure-startup.ps1
  modified:
    - spike/rust/src/main.rs
decisions:
  - "Production stack: Rust (enterprise tie on D-10, Rust wins D-11 secondary)"
  - "Binary size is decisive secondary factor (8.6x advantage)"
  - "All 3 integration points validated for both stacks"
metrics:
  duration_seconds: 239
  completed: 2026-05-20T02:30:44Z
  tasks_completed: 3
  tasks_total: 3
  files_created: 2
  files_modified: 1
---

# Phase 1 Plan E: Comparison Report & Stack Decision Summary

Consolidated all spike results into spike/RESULTS.md, measured cold startup times, applied decision criteria D-10 (enterprise) and D-11 (secondary), and recommended Rust as production stack.

## Tasks Completed

| # | Task | Commit | Key Changes |
|---|------|--------|-------------|
| E1 | Measure cold startup time | `1f67054` | Added --version to Rust, created measure-startup.ps1, measured both (Rust 8.9ms avg, C# 10.7ms avg) |
| E2 | Compile comparison report | `b57458e` | Created spike/RESULTS.md with full data tables, enterprise analysis, and recommendation |
| E3 | Document final decision | `cb4c416` | Added decision banner to RESULTS.md (STATE.md deferred to orchestrator) |

## Key Results

### Measurements

| Metric | C# NativeAOT | Rust | Winner |
|--------|--------------|------|--------|
| Binary size | 3.27 MB | 0.38 MB | Rust (8.6x) |
| Cold startup (avg) | 10.7 ms | 8.9 ms | Rust (marginal) |
| Toast notification | PASS | PASS | Tie |
| Protocol activation | PASS | PASS | Tie |
| Focus (4 scenarios) | 4/4 PASS | 4/4 PASS | Tie |

### Decision Logic

1. **D-10 (Enterprise control): TIE** — Both produce standard PE executables with identical code signing, Device Guard, WDAC, and SmartScreen behavior.
2. **D-11 (Secondary factors): Rust wins** — 8.6x smaller binary, simpler build chain (no vswhere dependency), compile-time memory safety.
3. **Recommendation: Rust** — Pending user confirmation per D-17.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] vswhere not in PATH for C# NativeAOT build**
- **Found during:** Task E1
- **Issue:** `dotnet publish` failed because `vswhere.exe` was not in PATH
- **Fix:** Added VS Installer directory to PATH before build
- **Impact:** None on final results — C# built successfully after PATH fix

### Task E3 Scope Reduction (Parallel Execution)

Per worktree parallel execution rules, STATE.md and PROJECT.md modifications were skipped (orchestrator-owned). Decision documented in spike/RESULTS.md only.

## Verification

```
spike/RESULTS.md exists: YES (203 lines)
"Final Decision" present: YES
"Chosen stack: Rust" present: YES
D-10 referenced: YES
All 3 integration points documented: YES
```

## Self-Check: PASSED

- [x] spike/RESULTS.md exists and is >100 lines
- [x] spike/measure-startup.ps1 exists
- [x] spike/rust/src/main.rs modified (--version flag)
- [x] Commit 1f67054 exists
- [x] Commit b57458e exists
- [x] Commit cb4c416 exists

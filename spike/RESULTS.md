# Tech Spike Results: C# NativeAOT vs Rust

> **DECISION: Rust** — Recommended as production stack. Enterprise control is a tie (D-10);
> Rust wins on secondary factors (D-11): 8.6x smaller binary, simpler build chain, memory safety.
> Pending user confirmation per D-17 (collaborative execution).

**Date:** 2026-05-20
**Tester:** Yanwei Gu (automated spike)
**Environment:** Windows 11 Pro 10.0.26200, Intel Core i5-12600KF, 32 GB RAM

## Summary

| Metric | C# NativeAOT | Rust | Winner |
|--------|--------------|------|--------|
| Binary size | 3.27 MB | 0.38 MB | **Rust** (8.6x smaller) |
| Cold startup time (avg) | 10.7 ms | 8.9 ms | **Rust** (marginal) |
| Cold startup time (min) | 8 ms | 6 ms | **Rust** (marginal) |
| Toast display | PASS | PASS | Tie |
| Focus: minimized/idle | PASS | PASS | Tie |
| Focus: multiple windows | PASS | PASS | Tie |
| Focus: user typing | PASS | PASS | Tie |
| Focus: fullscreen (simulated) | PASS | PASS | Tie |
| Protocol activation | PASS | PASS | Tie |
| Build complexity | VS Build Tools + .NET 9 SDK | Rust toolchain only | **Rust** (simpler) |
| NativeAOT issues | vswhere PATH needed | N/A | - |

**Binary sizes measured after full spike implementation** (Toast + Protocol + Focus):
- C# NativeAOT: 3,432,448 bytes (3.27 MB)
- Rust: 400,384 bytes (0.38 MB)

> Note: Toast-only spike measured 3.22 MB (C#) vs 0.14 MB (Rust). After adding URL parsing and Win32 focus code, Rust grew to 0.38 MB while C# remained ~3.3 MB.

## Integration Point Results

### 1. Toast Notification

Both stacks successfully display Windows Toast notifications via WinRT API.

| Test | C# NativeAOT | Rust |
|------|--------------|------|
| Build | PASS | PASS |
| NativeAOT/Release publish | PASS | PASS |
| Toast displayed | PASS | PASS |
| Single self-contained exe | PASS | PASS |
| Binary size (toast only) | 3.22 MB | 0.14 MB |

**Notes:**
- Both use PowerShell AUMID for spike (avoids shortcut requirement)
- C# uses CsWinRT bindings (Windows.UI.Notifications)
- Rust uses windows-rs crate (UI_Notifications feature)
- Custom AUMID registration deferred to Phase 2

### 2. Protocol Activation

Both stacks handle `claude-notify://` URI scheme protocol activation end-to-end.

| Test | C# NativeAOT | Rust |
|------|--------------|------|
| Registry registration (HKCU) | PASS | PASS |
| Direct invocation with `--focus` | PASS | PASS |
| Session parameter extraction | PASS | PASS |
| PID parameter extraction | PASS | PASS |
| CJK URL-decoded characters | PASS | PASS |
| Protocol activation via Start-Process | PASS | PASS |
| Binary size (with protocol parsing) | 3.3 MB | 381 KB |

**Notes:**
- C# uses manual query string parsing (NativeAOT-safe, avoids System.Web)
- Rust uses `url` crate for robust URI parsing
- Both handle URL-encoded CJK characters correctly
- Protocol activation launches new process window (console flash) — acceptable for spike

### 3. SetForegroundWindow (Click-to-Focus)

Both stacks implement a 4-strategy fallback chain and pass ALL test scenarios.

| Scenario | Required | C# NativeAOT | Rust |
|----------|----------|--------------|------|
| Minimized/idle | MUST PASS | **PASS** (Strategy 1) | **PASS** (Strategy 1) |
| Multiple windows | MUST PASS | **PASS** (Strategy 1) | **PASS** (Strategy 1) |
| User typing in foreground | May flash | **PASS** (Strategy 1) | **PASS** (Strategy 1) |
| Fullscreen (simulated) | May flash | **PASS** (Strategy 1) | **PASS** (Strategy 1) |

**Fallback strategies implemented (both stacks):**
1. Direct `SetForegroundWindow` (succeeded in all tests)
2. `AttachThreadInput` + `SetForegroundWindow`
3. Alt key hack (`keybd_event` VK_MENU)
4. Minimize/Restore cycle

**Key finding:** Strategy 1 (direct) succeeded in all scenarios because test process inherits foreground rights from the launching shell. This mirrors production: Toast click -> protocol activation -> exe launch -> `SetForegroundWindow` — the entire chain originates from user interaction, granting foreground permission.

## Startup Time Measurement

Measured using `spike/measure-startup.ps1` with `--version` quick-exit flag (10 iterations each).

| Metric | C# NativeAOT | Rust |
|--------|--------------|------|
| Average | 10.7 ms | 8.9 ms |
| Minimum | 8 ms | 6 ms |
| Maximum | 25 ms | 31 ms |
| First run (cold) | 25 ms | 31 ms |
| Steady state | ~9 ms | ~6-7 ms |

**Analysis:** Both are extremely fast. The difference (< 2ms average) is negligible for a notification helper. First-run cold start includes Windows PE loader cache miss — both settle to single-digit ms after first execution.

## Decision Criteria Application

### PRIMARY: Enterprise Control Compatibility (D-10)

| Factor | C# NativeAOT | Rust |
|--------|--------------|------|
| Code signing (signtool.exe) | Works — standard PE exe | Works — standard PE exe |
| Device Guard / WDAC | Compatible — native code, no JIT | Compatible — native code, no JIT |
| SmartScreen reputation | Standard EV/OV cert process | Standard EV/OV cert process |
| Intune deployment | Standard .exe/.msi packaging | Standard .exe/.msi packaging |
| AppLocker hash rules | Works | Works |
| DLL dependencies | None (self-contained) | None (self-contained) |

**Enterprise winner: TIE**

Both stacks produce standard Windows PE executables with:
- No JIT (both are ahead-of-time compiled native code)
- No runtime dependencies (no .NET runtime, no DLLs)
- Standard `signtool.exe` signing workflow
- Compatible with Device Guard / WDAC code integrity policies
- Standard SmartScreen reputation building process
- Standard enterprise deployment (SCCM, Intune, GPO)

Neither has an advantage or disadvantage for enterprise deployment. The resulting executables are indistinguishable from an enterprise security tooling perspective.

### SECONDARY (D-11, applied because enterprise factor is equal)

| Factor | C# NativeAOT | Rust | Winner |
|--------|--------------|------|--------|
| Development speed | Faster prototyping, LINQ, rich stdlib | Slower initial, but strong ecosystem | C# (marginal) |
| Binary size | 3.27 MB | 0.38 MB | **Rust** (8.6x smaller) |
| Contributor accessibility | Lower barrier (.NET familiarity) | Steeper learning curve (ownership, lifetimes) | C# (marginal) |
| Memory safety guarantees | Runtime null checks only | Compile-time ownership guarantees | **Rust** |
| Build dependency chain | VS Build Tools + .NET 9 SDK + vswhere | Rust toolchain only | **Rust** |
| Compile time | ~15s publish | ~11s release | **Rust** (marginal) |
| Zero-dependency promise | Yes (but 3.27 MB) | Yes (0.38 MB) | **Rust** |

**Secondary assessment:**
- Rust wins on binary size (8.6x), build simplicity, and memory safety
- C# wins on contributor accessibility and initial dev speed
- For a "single exe notification helper" that ships as a plugin, binary size is high-impact (users download per-update)

## Final Decision

**Chosen stack: Rust**

**Rationale:**

1. **Enterprise control: TIE** — Both produce standard PE executables. Code signing, Device Guard, WDAC, SmartScreen, and Intune deployment work identically for both. Neither has an enterprise advantage.

2. **Binary size: Rust wins decisively** — 0.38 MB vs 3.27 MB (8.6x smaller). For a "zero-dependency single exe" plugin distributed alongside Claude Code, smaller binary means faster downloads, less disk footprint, and simpler updates. This directly serves the project's core constraint.

3. **All 3 integration points validated** — Toast (windows-rs), Protocol activation (url crate), SetForegroundWindow (Win32 direct) all work correctly in Rust.

4. **Build simplicity** — Rust toolchain is self-contained (`rustup` + `cargo`). C# NativeAOT requires VS Build Tools + .NET 9 SDK + vswhere PATH configuration. The vswhere issue encountered during this spike is a concrete example of C# build friction.

5. **Memory safety** — Rust's compile-time ownership model eliminates null pointer errors, use-after-free, and data races. For a security-adjacent tool (notification helper with window focus manipulation), this is a meaningful advantage.

6. **Cold startup parity** — Both are under 10ms average. No meaningful difference for notification delivery latency.

**Risks with chosen stack:**

1. **Contributor accessibility** — Rust has a steeper learning curve than C#. Mitigated by: the codebase is small (notification helper, not a framework), and the windows-rs API surface is well-documented.

2. **windows-rs API stability** — The `windows` crate is actively maintained by Microsoft but may have breaking API changes between major versions. Mitigated by: pin dependency versions in Cargo.toml.

3. **Async runtime choice** — If future phases need async Toast callbacks or background polling, a runtime (tokio) adds binary size. Mitigated by: the fire-and-forget architecture minimizes async needs.

4. **Windows SDK binding generation** — Some less-common Win32 APIs may require manual binding. Mitigated by: all required APIs (Toast, SetForegroundWindow, EnumWindows, protocol URI) are already validated in the spike.

## Elimination Notes

- **Go:** Eliminated before spike (D-03) — no COM callback support for WinRT Toast notifications
- **C# NativeAOT:** NOT eliminated — all integration points pass. Ranked second due to larger binary size and build complexity when enterprise control is a tie.

## Appendix: Raw Measurements

### Binary Sizes (Final Spike Build)

```
C# NativeAOT: 3,432,448 bytes (3.27 MB)
Rust:           400,384 bytes (0.38 MB)
Ratio: 8.57x
```

### Startup Times (10 iterations, --version quick-exit)

```
C# NativeAOT: 25, 10, 10, 9, 9, 9, 9, 9, 9, 8 ms (avg=10.7, min=8, max=25)
Rust:          31, 7, 6, 7, 6, 6, 6, 6, 7, 7 ms  (avg=8.9, min=6, max=31)
```

### Build Times

```
C# NativeAOT: ~15s (dotnet publish -c Release -r win-x64)
Rust:          ~11s (cargo build --release)
```

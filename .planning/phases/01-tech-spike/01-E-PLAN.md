---
phase: 1
plan_id: 01-E
title: "Comparison Report & Stack Decision"
wave: 4
depends_on: [01-B, 01-C, 01-D]
files_modified:
  - spike/RESULTS.md
requirements_addressed: [TECH-01]
autonomous: false
must_haves:
  goal: "Decision documented: chosen stack with evidence from spike results"
  truths:
    - "spike/RESULTS.md contains binary size comparison"
    - "spike/RESULTS.md contains cold startup time comparison"
    - "spike/RESULTS.md contains Focus success rate per scenario"
    - "spike/RESULTS.md contains Toast display latency comparison"
    - "spike/RESULTS.md contains final decision with rationale"
    - "Decision matches criteria D-10 (enterprise control primary) and D-11 (secondary weights)"
---

# Plan 01-E: Comparison Report & Stack Decision

## Objective

Consolidate all spike results into a structured comparison report (`spike/RESULTS.md`), apply the decision criteria from D-10 and D-11, and document the final stack choice with evidence. This fulfills the phase's primary deliverable.

## Tasks

<task id="E1">
<title>Measure cold startup time for both exes</title>
<read_first>
- spike/csharp/bin/Release/net9.0-windows10.0.19041.0/win-x64/publish/CSharpSpike.exe (verify exists)
- spike/rust/target/release/rust-spike.exe (verify exists)
</read_first>
<action>
Create a measurement script `spike/measure-startup.ps1`:

```powershell
param([string]$ExePath, [int]$Iterations = 5)

Write-Host "Measuring cold startup: $ExePath ($Iterations iterations)"
$times = @()

for ($i = 1; $i -le $Iterations; $i++) {
    # Clear filesystem cache by reading a large file (rough invalidation)
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $ExePath --version 2>$null  # Quick exit mode (add --version to both exes that just prints and exits)
    $sw.Stop()
    $times += $sw.ElapsedMilliseconds
    Write-Host "  Run $i`: $($sw.ElapsedMilliseconds) ms"
}

$avg = ($times | Measure-Object -Average).Average
$min = ($times | Measure-Object -Minimum).Minimum
$max = ($times | Measure-Object -Maximum).Maximum
Write-Host "Results: avg=$([math]::Round($avg,1))ms min=${min}ms max=${max}ms"
```

Add a `--version` flag to both exes that simply prints "spike v0.1" and exits (for startup timing without side effects).

Run for both:
```powershell
.\spike\measure-startup.ps1 -ExePath "spike\csharp\bin\Release\...\CSharpSpike.exe"
.\spike\measure-startup.ps1 -ExePath "spike\rust\target\release\rust-spike.exe"
```
</action>
<acceptance_criteria>
- Both exes support `--version` flag that exits immediately
- Script `spike/measure-startup.ps1` exists and outputs average/min/max ms
- Startup times recorded (expected: C# <50ms, Rust <10ms)
</acceptance_criteria>
</task>

<task id="E2">
<title>Compile comparison report</title>
<read_first>
- spike/toast-results.md (Toast validation results from Plan B)
- spike/protocol-results.md (Protocol activation results from Plan C)
- spike/focus-results.md (SetForegroundWindow results from Plan D)
- .planning/phases/01-tech-spike/01-CONTEXT.md (D-10, D-11: decision criteria; D-13: report structure)
</read_first>
<action>
Create `spike/RESULTS.md` with this structure:

```markdown
# Tech Spike Results: C# NativeAOT vs Rust

**Date:** [date]
**Tester:** [user]
**Environment:** Windows [version], [CPU], [RAM]

## Summary

| Metric | C# NativeAOT | Rust | Winner |
|--------|-------------|------|--------|
| Binary size | X MB | X MB | [smaller] |
| Cold startup time | X ms | X ms | [faster] |
| Toast display latency | X ms | X ms | [faster] |
| Focus: minimized | PASS/FAIL | PASS/FAIL | - |
| Focus: multiple windows | PASS/FAIL | PASS/FAIL | - |
| Focus: user typing | PASS/FLASH/FAIL | PASS/FLASH/FAIL | - |
| Focus: fullscreen | PASS/FLASH/FAIL | PASS/FLASH/FAIL | - |
| Protocol activation | PASS/FAIL | PASS/FAIL | - |
| Build complexity | [notes] | [notes] | [simpler] |
| NativeAOT issues | [list] | N/A | - |

## Integration Point Results

### 1. Toast Notification
[Paste from toast-results.md with any additional notes]

### 2. Protocol Activation
[Paste from protocol-results.md with any additional notes]

### 3. SetForegroundWindow
[Paste from focus-results.md with any additional notes]

## Decision Criteria Application

### PRIMARY: Enterprise Control Compatibility (D-10)

| Factor | C# NativeAOT | Rust |
|--------|-------------|------|
| Code signing (signtool.exe) | [works/issues] | [works/issues] |
| Device Guard / WDAC | [behavior] | [behavior] |
| SmartScreen reputation | [notes] | [notes] |
| Intune deployment | [notes] | [notes] |

**Enterprise winner:** [choice] because [reason]

### SECONDARY (D-11, only if enterprise factor is equal)

| Factor | C# NativeAOT | Rust |
|--------|-------------|------|
| Development speed | [assessment] | [assessment] |
| Binary size | X MB | X MB |
| Contributor accessibility | [assessment] | [assessment] |

## Final Decision

**Chosen stack: [C# NativeAOT / Rust]**

**Rationale:**
[Evidence-based justification referencing the data above]

**Risks with chosen stack:**
[Known limitations to carry into Phase 2]

## Elimination Notes

[If either stack failed an integration point, document here]
- Go: Eliminated before spike (D-03) — no COM callback support
- [C# / Rust]: [If eliminated, why]
```

Fill in all data from the spike results files.
</action>
<acceptance_criteria>
- File `spike/RESULTS.md` exists and is >100 lines
- Contains the comparison table with all metrics filled in
- Contains "Final Decision" section with chosen stack name
- References D-10 (enterprise control) as primary criterion
- All 3 integration points have documented pass/fail status for both stacks
</acceptance_criteria>
</task>

<task id="E3">
<title>Update project STATE.md with decision</title>
<read_first>
- spike/RESULTS.md (final decision from E2)
- .planning/STATE.md (current state — decision pending)
- .planning/PROJECT.md (key decisions table)
</read_first>
<action>
After the decision is made:

1. Update `.planning/STATE.md`:
   - Under "Decisions": Replace "Tech stack undecided: Phase 1 spike will determine" with the actual decision
   - Update "Current Position" → Status to "Phase 1 complete"

2. Update `.planning/PROJECT.md` Key Decisions table (add row):
   - Decision: "Production tech stack: [C# NativeAOT / Rust]"
   - Rationale: "[one-liner from RESULTS.md]"
   - Date: [today]
   - Phase: 1

3. Remove/resolve blockers that the spike answered:
   - "NativeAOT COM interop risk" → resolved (works / eliminated C#)
   - "SetForegroundWindow from protocol-activated process" → resolved (works with strategy X)
</action>
<acceptance_criteria>
- `.planning/STATE.md` "Decisions" section contains the chosen tech stack (not "undecided")
- `.planning/PROJECT.md` Key Decisions table has a row for "Production tech stack"
- Blockers section updated to reflect resolved/remaining risks
</acceptance_criteria>
</task>

## Verification

```powershell
# File existence
Test-Path spike/RESULTS.md          # True
# Content check
Select-String "Final Decision" spike/RESULTS.md    # Found
Select-String "Chosen stack:" spike/RESULTS.md     # Found
# State updated
Select-String "undecided" .planning/STATE.md       # NOT found (resolved)
```

## Notes

- This plan is `autonomous: false` because the final decision requires user confirmation (D-17: collaborative execution)
- User reviews the comparison data and confirms or overrides the recommendation
- If both stacks fail the same integration point, the spike result is "Architecture change needed" — escalate to user

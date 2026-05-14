# Phase 1: Tech Spike - Context

**Gathered:** 2026-05-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Validate critical integration points (Toast + Protocol activation + SetForegroundWindow) by running parallel spikes for C# NativeAOT and Rust. Produce a comparison report with hard data, then choose the production stack.

</domain>

<decisions>
## Implementation Decisions

### Stack Evaluation Strategy
- **D-01:** Run C# NativeAOT and Rust spikes IN PARALLEL (not sequential)
- **D-02:** If either stack fails ANY of the 3 integration points (Toast via WinRT, Protocol activation, SetForegroundWindow), it is eliminated
- **D-03:** Go is already eliminated — architectural disqualifier (no COM callback support)

### Focus Validation Criteria
- **D-04:** 4 scenarios must be tested: user typing in foreground, window minimized/idle, fullscreen app, multiple terminal windows
- **D-05:** "窗口最小化/空闲" and "多终端窗口" MUST pass — these are hard requirements
- **D-06:** "用户在前台打字" and "全屏应用" MAY fallback to taskbar flash — acceptable degradation

### Spike Scope
- **D-07:** Spike code is throwaway (丢弃型 PoC) — Phase 2 starts fresh with proper architecture
- **D-08:** No time box — take as long as needed to validate clearly
- **D-09:** Code lives in `spike/` directory in current repo (retained for reference, deleted later)

### Final Decision Criteria
- **D-10:** If both stacks pass all integration points, choose based on PRIMARY weight: enterprise control compatibility (Device Guard / code signing / Intune). Whichever is easier to sign and deploy in locked-down environments wins.
- **D-11:** Secondary weights (if enterprise factor is equal): development speed, binary size, contributor accessibility

### Spike Deliverables
- **D-12:** Each stack produces runnable PoC code in `spike/csharp/` and `spike/rust/`
- **D-13:** Structured comparison report with: binary size, cold startup time, Focus success rate per scenario, Toast display latency, compatibility issues encountered
- **D-14:** Final decision document with evidence from spike results

### Environment
- **D-15:** Both .NET 9 SDK and Rust toolchain need to be installed (neither exists on machine currently)
- **D-16:** Protocol URI registration (`claude-notify://`) will be set up as part of each spike

### Execution Model
- **D-17:** Collaborative execution — show key results at checkpoints, user participates in evaluation

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Research
- `.planning/research/STACK.md` — Full C# vs Rust vs Go comparison with API details
- `.planning/research/PITFALLS.md` — Critical pitfalls #1 (SetForegroundWindow), #2 (Toast COM), #4 (NativeAOT COM limitation)
- `.planning/research/ARCHITECTURE.md` — Protocol activation pattern, fire-and-forget architecture

### Project
- `.planning/PROJECT.md` — Core value and constraints
- `.planning/REQUIREMENTS.md` — TECH-01 requirement definition

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield project, no existing code

### Established Patterns
- None yet — this spike will establish the foundational patterns

### Integration Points
- Claude Code hooks system (stdin JSON) — will be tested in Phase 2, not spike
- Windows Toast WinRT API — core spike target
- Windows SetForegroundWindow Win32 API — core spike target
- Protocol activation registry — core spike target

</code_context>

<specifics>
## Specific Ideas

- SetForegroundWindow fallback chain: AttachThreadInput → Alt key hack (SendInput) → ShowWindow minimize/restore
- Protocol activation via registry: `HKCU\Software\Classes\claude-notify\shell\open\command`
- Toast via WinRT: `Windows.UI.Notifications.ToastNotificationManager`
- Enterprise validation: check if signed exe passes SmartScreen on first run

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 1-Tech Spike*
*Context gathered: 2026-05-15*

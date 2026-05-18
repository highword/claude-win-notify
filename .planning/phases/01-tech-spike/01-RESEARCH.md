# Phase 1: Tech Spike - Research

**Researched:** 2026-05-15
**Confidence:** HIGH (verified via Context7 for windows-rs and CsWinRT)

## Research Question

What do I need to know to plan the Phase 1 tech spike (C# NativeAOT vs Rust) well?

## Executive Summary

Phase 1 must validate 3 integration points for both C# NativeAOT and Rust:
1. **Toast via WinRT** — Display a Windows native notification
2. **Protocol activation** — `claude-notify://` URI launches exe with arguments
3. **SetForegroundWindow** — Bring terminal window to foreground from protocol-activated process

Both stacks have viable paths. The spike determines which one works in practice under NativeAOT constraints.

---

## Integration Point 1: Toast Notification via WinRT

### C# NativeAOT Approach

**API:** `Windows.UI.Notifications.ToastNotificationManager` via CsWinRT projections

**Key facts:**
- CsWinRT 2.1+ supports NativeAOT via `CsWinRTAotOptimizerEnabled` source generators
- Target framework: `net9.0-windows10.0.19041.0`
- Package: `Microsoft.Windows.SDK.NET` provides projected types
- AUMID required: For spike, use PowerShell's built-in AUMID or register a minimal one
- Toast XML schema v4 supported on Windows 10 1903+

**Spike code pattern:**
```csharp
// Minimal toast from NativeAOT console app
var xml = ToastNotificationManager.GetTemplateContent(ToastTemplateType.ToastText02);
var textNodes = xml.GetElementsByTagName("text");
textNodes[0].InnerText = "Claude Code";
textNodes[1].InnerText = "Task complete!";

var toast = new ToastNotification(xml);
var notifier = ToastNotificationManager.CreateToastNotifier("ClaudeWinNotify.Spike");
notifier.Show(toast);
```

**Risk:** NativeAOT "No built-in COM" limitation. CsWinRT source generators should handle this, but needs empirical validation.

**Mitigation:** If WinRT projections fail under NativeAOT, try:
1. Windows App SDK `AppNotificationManager` (designed for modern .NET AOT)
2. `[GeneratedComInterface]` (.NET 8+) for source-generated COM interop
3. Fall through to Rust if unresolvable

### Rust Approach

**Crate:** `windows` (microsoft/windows-rs) v0.62+

**Key facts:**
- Feature flags: `Windows_UI_Notifications`, `Data_Xml_Dom`
- Full WinRT projection, no runtime needed
- `win-toast-notify` crate exists but only supports Protocol activation type (limited)
- Direct `windows-rs` gives full control over Toast XML and activation types

**Spike code pattern:**
```rust
use windows::UI::Notifications::*;
use windows::Data::Xml::Dom::*;

let xml = ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText02)?;
let text_nodes = xml.GetElementsByTagName(&"text".into())?;
text_nodes.Item(0)?.SetInnerText(&"Claude Code".into())?;
text_nodes.Item(1)?.SetInnerText(&"Task complete!".into())?;

let toast = ToastNotification::CreateToastNotification(&xml)?;
let notifier = ToastNotificationManager::CreateToastNotifierWithId(&"ClaudeWinNotify.Spike".into())?;
notifier.Show(&toast)?;
```

**Risk:** AUMID registration for unpackaged apps requires Start Menu shortcut with properties. Spike can use an existing AUMID as workaround.

---

## Integration Point 2: Protocol Activation

### Registry Structure (Same for Both Stacks)

```
HKCU\Software\Classes\claude-notify
  (Default) = "URL:Claude Win Notify Protocol"
  URL Protocol = ""
  
HKCU\Software\Classes\claude-notify\shell\open\command
  (Default) = "\"C:\path\to\exe\" --focus \"%1\""
```

### Toast XML with Protocol Activation

```xml
<toast activationType="protocol" launch="claude-notify://focus?session=abc&amp;pid=1234">
  <visual>
    <binding template="ToastGeneric">
      <text>Task Complete</text>
      <text>Spike validation test</text>
    </binding>
  </visual>
</toast>
```

### C# Spike

- Parse `Environment.GetCommandLineArgs()` for `--focus` + URI
- Use `Uri` class to parse query parameters
- Straightforward, no NativeAOT issues expected

### Rust Spike

- Parse `std::env::args()` for `--focus` + URI
- Use `url` crate to parse query parameters
- Straightforward, no issues expected

**Validation:** Click toast → exe launches with URI → parse session/pid from URI → print to console (prove data flow works).

---

## Integration Point 3: SetForegroundWindow

### The Problem

Windows restricts `SetForegroundWindow()` to prevent focus stealing. A protocol-activated process has better chances because activation came from user interaction (toast click). But it's not guaranteed in all scenarios.

### 4 Test Scenarios (from D-04)

| Scenario | Expected Difficulty | MUST Pass? |
|----------|-------------------|-----------|
| Window minimized/idle | Low — protocol activation grants foreground rights | YES (D-05) |
| Multiple terminal windows | Medium — must find correct window by PID | YES (D-05) |
| User typing in foreground | High — foreground lock may interfere | NO (D-06, flash acceptable) |
| Fullscreen app | High — fullscreen app holds foreground lock | NO (D-06, flash acceptable) |

### Fallback Chain (Both Stacks)

```
1. SetForegroundWindow(hwnd)           // Direct attempt
2. → If fails: AttachThreadInput()     // Attach to foreground thread, retry
3. → If fails: SendInput(Alt key)      // Simulate keypress, retry
4. → If fails: ShowWindow(MINIMIZE) → ShowWindow(RESTORE)  // Force restore
5. → If all fail: FlashWindowEx()      // Taskbar flash as graceful degradation
```

### C# P/Invoke Declarations

```csharp
[DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr hWnd);
[DllImport("user32.dll")] static extern bool AttachThreadInput(uint idAttach, uint idAttachTo, bool fAttach);
[DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
[DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
[DllImport("user32.dll")] static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
[DllImport("user32.dll")] static extern bool IsIconic(IntPtr hWnd);
[DllImport("user32.dll")] static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
```

### Rust windows-rs Calls

```rust
use windows::Win32::UI::WindowsAndMessaging::*;

unsafe {
    SetForegroundWindow(hwnd);
    AttachThreadInput(our_tid, fg_tid, true);
    // etc.
}
```

### Key Insight: Protocol Activation Grants Foreground Rights

When Windows launches a process via protocol activation from a toast click, the launched process IS granted foreground window permission. This is because:
- The activation originated from user interaction (clicking the toast)
- Windows treats protocol-launched processes similarly to user-started processes

This means `SetForegroundWindow` should succeed in the "minimized/idle" and "multiple windows" scenarios without needing the fallback chain. The fallback is for edge cases.

---

## Environment Setup Requirements

### .NET 9 SDK Installation

```powershell
# Via winget
winget install Microsoft.DotNet.SDK.9

# Verify
dotnet --version  # Should be 9.0.x
```

**Project setup:**
```powershell
dotnet new console -n CSharpSpike --framework net9.0
# Edit .csproj to add:
# <TargetFramework>net9.0-windows10.0.19041.0</TargetFramework>
# <PublishAot>true</PublishAot>
# <CsWinRTAotOptimizerEnabled>true</CsWinRTAotOptimizerEnabled>
```

### Rust Toolchain Installation

```powershell
# Via official installer
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe -y --default-toolchain stable-x86_64-pc-windows-msvc

# Verify
rustc --version
cargo --version
```

**Project setup:**
```powershell
cargo new rust-spike
# Edit Cargo.toml to add:
# [dependencies.windows]
# version = "0.62"
# features = ["Win32_UI_WindowsAndMessaging", "UI_Notifications", "Data_Xml_Dom"]
```

---

## NativeAOT Constraints & Risks

| Constraint | Impact on Spike | Mitigation |
|-----------|----------------|-----------|
| No built-in COM | WinRT calls may fail | CsWinRT source generators bypass this |
| Trimming removes unused code | COM activation code may be trimmed | `[DynamicDependency]` annotations |
| No reflection-based marshalling | P/Invoke works, COM interop risky | Use `[GeneratedComInterface]` |
| Binary size ~8-12 MB | Acceptable (requirement is <15MB) | `IlcOptimizationPreference=Size` |
| Cold start <50ms | Meets 500ms requirement easily | No mitigation needed |

**Critical test:** Build NativeAOT exe → run on clean machine (no .NET runtime installed) → verify Toast appears. If it doesn't, NativeAOT COM interop is broken for this use case.

---

## Spike Deliverables (from D-12, D-13, D-14)

### Directory Structure

```
spike/
├── csharp/
│   ├── CSharpSpike.csproj
│   ├── Program.cs          # All 3 integration points
│   └── publish/            # NativeAOT-compiled exe
├── rust/
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs         # All 3 integration points
│   └── target/release/     # Compiled exe
└── RESULTS.md              # Comparison report
```

### Comparison Report Structure (D-13)

| Metric | C# NativeAOT | Rust | Winner |
|--------|-------------|------|--------|
| Binary size | ? MB | ? MB | ? |
| Cold startup time | ? ms | ? ms | ? |
| Toast display latency | ? ms | ? ms | ? |
| Focus success: minimized | Pass/Fail | Pass/Fail | ? |
| Focus success: multiple windows | Pass/Fail | Pass/Fail | ? |
| Focus success: user typing | Pass/Fail/Flash | Pass/Fail/Flash | ? |
| Focus success: fullscreen | Pass/Fail/Flash | Pass/Fail/Flash | ? |
| Protocol URI received | Pass/Fail | Pass/Fail | ? |
| NativeAOT/Release build | Pass/Fail | Pass/Fail | ? |
| Compilation issues | List | List | ? |

### Decision Criteria (D-10, D-11)

**PRIMARY (if both pass all integration points):**
- Enterprise control: Device Guard / code signing / Intune compatibility
- Whichever is easier to sign and deploy in locked-down environments wins

**SECONDARY (if enterprise factor is equal):**
1. Development speed
2. Binary size
3. Contributor accessibility

---

## Risks & Unknowns

| Risk | Probability | Impact | Phase Response |
|------|-------------|--------|----------------|
| NativeAOT COM interop fails for WinRT | MEDIUM | HIGH — eliminates C# | Spike exists to validate this |
| SetForegroundWindow fails from protocol activation | LOW | MEDIUM — fallback chain exists | Test all 4 scenarios |
| Neither .NET 9 nor Rust toolchain install cleanly | LOW | LOW — well-documented installers | Manual intervention |
| Toast requires AUMID that needs complex setup | MEDIUM | LOW — can use existing AUMID for spike | Proper AUMID in Phase 2 |
| Protocol URI args parsing fails on special characters | LOW | LOW — use URL encoding | Test with CJK paths |

---

## Validation Architecture

### Test Matrix

Each integration point × each stack = 6 validation tests:

1. C# Toast: Shows notification with text
2. C# Protocol: Exe launches with correct URI args on toast click
3. C# Focus: Terminal window comes to foreground (4 scenarios)
4. Rust Toast: Shows notification with text
5. Rust Protocol: Exe launches with correct URI args on toast click
6. Rust Focus: Terminal window comes to foreground (4 scenarios)

### Success Criteria Mapping

| ROADMAP Success Criterion | Validated By |
|--------------------------|-------------|
| NativeAOT exe displays Toast via WinRT | Test 1 |
| Protocol activation receives URI args | Tests 2, 5 |
| SetForegroundWindow succeeds (with fallback) | Tests 3, 6 |
| Decision documented with evidence | RESULTS.md written after all tests |

---

## Sources

- `.planning/research/STACK.md` — Full stack comparison (Context7 verified)
- `.planning/research/PITFALLS.md` — Pitfalls #1 (SetForegroundWindow), #2 (Toast COM), #4 (NativeAOT COM)
- `.planning/research/ARCHITECTURE.md` — Protocol activation pattern
- Microsoft Learn: NativeAOT deployment limitations
- Microsoft Learn: Toast notification for desktop apps
- Context7: microsoft/windows-rs v0.62 (COM `#[implement]` macro)
- Context7: microsoft/cswinrt (AOT optimizer, source generators)

## RESEARCH COMPLETE

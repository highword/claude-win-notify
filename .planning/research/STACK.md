# Technology Stack

**Project:** claude-win-notify
**Researched:** 2026-05-15
**Overall Confidence:** MEDIUM-HIGH (verified via Context7 for windows-rs and CsWinRT; Go ecosystem limited to training data)

## Recommendation: C# .NET 9 NativeAOT

**One-liner:** C# with .NET 9 NativeAOT delivers the best balance of Windows API depth, developer productivity, and single-exe deployment for this project's requirements.

---

## Detailed Comparison

### 1. C# .NET 9 NativeAOT (RECOMMENDED)

| Criterion | Assessment | Confidence |
|-----------|-----------|------------|
| Toast with interactive buttons | Full WinRT `Windows.UI.Notifications` API via CsWinRT, native COM activation callbacks | HIGH |
| COM activation callback | `INotificationActivationCallback` interface directly implementable; AppUserModelId registration via Shell COM | HIGH |
| SetForegroundWindow + UI Automation | Full Win32 P/Invoke + `Windows.UI.UIAutomation` COM interop | HIGH |
| JSONL streaming parse | `System.Text.Json` with `Utf8JsonReader` — zero-allocation streaming | HIGH |
| Single exe size | ~8-12 MB with aggressive trimming (`PublishTrimmed` + `PublishAot`) | MEDIUM |
| Startup time | <50ms cold start with NativeAOT (no JIT) | MEDIUM |
| Code signing | Standard Authenticode signtool.exe; seamless with CI/CD (GitHub Actions) | HIGH |
| Community/ecosystem | Largest Windows development community; official Microsoft tooling | HIGH |
| AOT compatibility | CsWinRT 2.1+ has `CsWinRTAotOptimizerEnabled`; `IsAotCompatible` property supported | HIGH (verified Context7) |

**Why C# wins for this project:**

1. **Native WinRT Toast API access** — CsWinRT provides first-class projected types for `Windows.UI.Notifications.ToastNotificationManager`, `ToastNotification`, and the full XML-based Toast schema. COM activation callbacks (`INotificationActivationCallback`) are implementable without workarounds.

2. **NativeAOT maturity in .NET 9** — Trim-compatible, produces a true single-file native executable with no runtime dependency. CsWinRT 2.1+ explicitly supports NativeAOT with source generators replacing reflection.

3. **P/Invoke is trivial** — `SetForegroundWindow`, `EnumWindows`, `GetWindowThreadProcessId`, `AttachThreadInput` etc. are all one-line declarations. UI Automation COM interfaces also accessible.

4. **Developer velocity** — Async/await for COM callbacks, LINQ for JSONL processing, pattern matching for state machines. The "notification type inference" state machine will be 3x less code than Rust.

5. **Enterprise alignment** — Target users are Windows enterprise developers. C# is the lingua franca. Contributors will be more plentiful.

**Risks:**
- NativeAOT trim warnings for COM interop require careful handling (use `[DynamicallyAccessedMembers]` and source generators)
- Exe size is larger than Rust (~10MB vs ~3MB) but acceptable for the use case
- Must target `net9.0-windows10.0.19041.0` (Windows 10 2004+) for full Toast API

---

### 2. Rust + windows-rs

| Criterion | Assessment | Confidence |
|-----------|-----------|------------|
| Toast with interactive buttons | Possible via raw WinRT `Windows.UI.Notifications` bindings in windows-rs 0.62 | HIGH (verified Context7) |
| COM activation callback | Implementable via `#[implement(INotificationActivationCallback)]` macro | HIGH (verified Context7) |
| SetForegroundWindow + UI Automation | Full coverage via `Win32_UI_WindowsAndMessaging` + `Win32_UI_Accessibility` features | HIGH (verified Context7) |
| JSONL streaming parse | `serde_json::StreamDeserializer` or `simd-json` — excellent performance | HIGH |
| Single exe size | ~2-4 MB (smallest of all options) | HIGH |
| Startup time | <10ms cold start | HIGH |
| Code signing | Standard signtool.exe; Cargo build integrates with CI | HIGH |
| Community/ecosystem | Growing but smaller Windows-specific community; fewer Toast examples | MEDIUM |
| COM activation complexity | Requires manual COM server registration, CLSID management, registry entries | MEDIUM |

**Why Rust is the strong alternative:**

1. **Smallest binary** — 2-4 MB single exe, fastest startup, lowest resource usage
2. **windows-rs 0.62** is mature — `#[implement]` macro for COM, full WinRT projection
3. **Memory safety** — No GC pauses, predictable performance
4. **Cross-compile** — `x86_64-pc-windows-msvc` target works well

**Why NOT Rust (for this project):**

1. **COM activation callback complexity** — Implementing `INotificationActivationCallback` requires manual COM server registration (write registry keys, implement `DllGetClassObject` or use an out-of-process COM server with a message loop). This is significantly more boilerplate than C#.

2. **Development velocity** — The state machine for notification type inference, JSONL parsing with async I/O, and COM interop will take 2-3x longer to write in Rust. Lifetime annotations with COM pointers are error-prone.

3. **`win-toast-notify` crate limitations** — Only supports `ActivationType::Protocol` (opens URLs/files). Does NOT support `Foreground` or `Background` activation types needed for in-process callbacks (approve/deny buttons). You'd need to bypass this crate and use raw windows-rs WinRT APIs directly.

4. **Contributor barrier** — Target audience (Windows power users, Claude Code users) is more likely C#-fluent than Rust-fluent.

---

### 3. Go + go-toast/CGo

| Criterion | Assessment | Confidence |
|-----------|-----------|------------|
| Toast with interactive buttons | `go-toast/toast` uses PowerShell-based shelling out — cannot receive callbacks | MEDIUM |
| COM activation callback | NOT POSSIBLE without CGo + C++ bridge or embedded helper exe | MEDIUM |
| SetForegroundWindow + UI Automation | Possible via `syscall.NewLazyDLL` but extremely verbose | MEDIUM |
| JSONL streaming parse | `encoding/json.Decoder` — adequate performance | HIGH |
| Single exe size | ~8-15 MB (Go runtime + CGo overhead if needed) | MEDIUM |
| Startup time | ~30-50ms | MEDIUM |
| Code signing | Standard signtool.exe; goreleaser handles CI | HIGH |
| Community/ecosystem | go-toast is unmaintained (last commit 2017); Go-Windows ecosystem sparse | LOW |

**Why NOT Go:**

1. **`go-toast/toast` is architecturally broken for this use case** — It shells out to a PowerShell script that calls `[Windows.UI.Notifications.ToastNotificationManager]`. This means:
   - No COM activation callbacks (approve/deny buttons can only use Protocol activation)
   - No way to receive "button clicked" events back into the Go process
   - Extra process spawn latency (~200ms for PowerShell)

2. **No viable path to interactive Toast** — To receive Toast activation callbacks, you need a COM server registered with Windows. Go has no native COM support. You'd need either:
   - CGo + C++ bridge (defeats "simple build" advantage)
   - Embedded helper exe (exactly what ntfy-toast does — but then why use Go?)
   - Named pipe protocol with an external toast helper (adds complexity)

3. **Windows API access is painful** — Every Win32 call requires `syscall.NewLazyDLL` + `NewProc` + unsafe pointer juggling. UI Automation COM interfaces are practically unusable from pure Go.

4. **Competitor already uses Go** — `claude-notifications-go` is the Go-based competitor. Using Go would mean fighting on their turf with the same architectural limitations they have (which is why they can't do Click-to-Focus on Windows).

5. **go-toast is effectively abandoned** — Last meaningful update was 2017. No WinRT improvements, no COM activation support, no maintained fork.

---

## Recommended Stack (C# .NET 9 NativeAOT)

### Core Framework

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| .NET SDK | 9.0 | Runtime & build system | NativeAOT maturity; LTS-adjacent (net10 in Nov 2025 is LTS but 9.0 is stable now) |
| CsWinRT | 2.1+ | WinRT projection for Toast APIs | AOT-compatible source generators; `CsWinRTAotOptimizerEnabled` |
| TFM | net9.0-windows10.0.19041.0 | Target framework | Minimum Windows 10 2004 for full Toast schema v4 |

### Toast Notification

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Windows.UI.Notifications (WinRT) | Built-in | Toast creation & display | Official API; supports all Toast schema features |
| Microsoft.Windows.SDK.NET | Latest | Windows SDK projections | Provides `ToastNotificationManager`, `ToastNotification` etc. |
| COM Activation (INotificationActivationCallback) | N/A | Receive button click callbacks | Required for Approve/Deny in-process handling |

### Windows Integration

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| P/Invoke (Win32) | N/A | SetForegroundWindow, EnumWindows, AttachThreadInput | Direct Win32 calls for window focus |
| UIAutomation COM | Built-in | Tab navigation in Windows Terminal/Warp | Needed for Ctrl+Tab simulation or tree walking |
| System.Runtime.InteropServices | Built-in | COM interop, P/Invoke declarations | First-class .NET support |

### Data Processing

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| System.Text.Json | Built-in | JSONL parsing | Zero-alloc `Utf8JsonReader` for streaming; trim-safe |
| System.IO.Pipelines | Built-in | High-perf stdin/file reading | Optimal for streaming JSONL from transcript files |

### Build & Deploy

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| PublishAot | .NET 9 | NativeAOT compilation | Single native exe, no .NET runtime required |
| PublishTrimmed + PublishSingleFile | .NET 9 | Size optimization | Removes unused framework code |
| ILLink | Built-in | Trim analyzer | Catches trim-unsafe patterns at build time |

### Testing

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| xUnit | 2.9+ | Unit testing | Most popular .NET test framework |
| NSubstitute | 5.x | Mocking COM interfaces | Clean syntax for interface mocking |
| Verify | Latest | Snapshot testing | Great for JSONL parse output verification |

### CI/CD & Distribution

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| GitHub Actions | N/A | CI/CD pipeline | Free for open source; Windows runners available |
| AzureSignTool | Latest | Code signing in CI | Authenticode signing with Azure Key Vault |
| WinGet | N/A | Package distribution | Windows-native package manager |
| Scoop | N/A | Alternative distribution | Developer-friendly, no admin required |

---

## Project Structure

```
claude-win-notify/
├── src/
│   ├── ClaudeWinNotify/              # Main executable project
│   │   ├── Program.cs                # Entry point, CLI argument parsing
│   │   ├── Notifications/
│   │   │   ├── ToastService.cs       # WinRT Toast API wrapper
│   │   │   ├── ComActivator.cs       # INotificationActivationCallback impl
│   │   │   └── NotificationTemplates.cs
│   │   ├── Focus/
│   │   │   ├── WindowFocusService.cs # SetForegroundWindow + AllowSetForegroundWindow
│   │   │   ├── TabNavigator.cs       # UI Automation for tab switching
│   │   │   └── TerminalDetector.cs   # Detect Warp/WT/ConEmu
│   │   ├── Parsing/
│   │   │   ├── JsonlStreamReader.cs  # Streaming JSONL parser
│   │   │   ├── TranscriptWatcher.cs  # File watcher for transcript changes
│   │   │   └── NotificationClassifier.cs # State machine for type inference
│   │   ├── Hooks/
│   │   │   ├── HookInstaller.cs      # hooks.json injection
│   │   │   └── StdinReader.cs        # Claude Code stdin JSON reader
│   │   └── ClaudeWinNotify.csproj
│   └── ClaudeWinNotify.Tests/
│       └── ...
├── installer/
│   └── Install-ClaudeWinNotify.ps1   # PowerShell one-liner installer
├── .github/
│   └── workflows/
│       ├── build.yml
│       └── release.yml
└── claude-win-notify.sln
```

---

## .csproj Configuration (Key Settings)

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net9.0-windows10.0.19041.0</TargetFramework>
    <RuntimeIdentifier>win-x64</RuntimeIdentifier>
    <PublishAot>true</PublishAot>
    <PublishTrimmed>true</PublishTrimmed>
    <PublishSingleFile>true</PublishSingleFile>
    <SelfContained>true</SelfContained>
    <InvariantGlobalization>true</InvariantGlobalization>
    <IlcOptimizationPreference>Size</IlcOptimizationPreference>
    <IsAotCompatible>true</IsAotCompatible>
    <CsWinRTAotOptimizerEnabled>true</CsWinRTAotOptimizerEnabled>
    <WindowsSdkPackageVersion>10.0.19041.41</WindowsSdkPackageVersion>
  </PropertyGroup>
</Project>
```

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not Alternative |
|----------|-------------|-------------|---------------------|
| Language | C# .NET 9 NativeAOT | Rust + windows-rs 0.62 | 2-3x development time; COM activation boilerplate; smaller contributor pool |
| Language | C# .NET 9 NativeAOT | Go + go-toast | No COM callback support; go-toast abandoned; painful Win32 interop |
| Toast Library | Raw WinRT via CsWinRT | Microsoft.Toolkit.Uwp.Notifications | UWP toolkit has trim/AOT compatibility issues; adds dependency |
| JSON Parser | System.Text.Json | Newtonsoft.Json | Not trim-safe; larger binary; slower for streaming |
| UI Automation | COM interop (IUIAutomation) | Windows.UI.Automation (.NET) | COM is lower-level but more reliable for cross-process automation |
| CLI Framework | System.CommandLine (preview) | Cocona / Spectre.Console.Cli | System.CommandLine is Microsoft-supported, trim-safe |
| Installer | PowerShell script | MSI/MSIX | PowerShell is zero-dependency, fits "30-second setup" narrative |

---

## Critical Technical Decisions

### Toast COM Activation Strategy

**Decision:** Use a registered COM server (CLSID in registry) for `INotificationActivationCallback`.

**Rationale:** Windows Toast notifications with `activationType="foreground"` buttons require the app to be registered as a COM server. When a button is clicked, Windows uses CoCreateInstance to instantiate the registered CLSID and calls `INotificationActivationCallback.Activate()`. This is the ONLY way to receive in-process button click events.

**Implementation:**
1. Register AppUserModelId (AUMID) in Start Menu shortcut (required for Toast)
2. Register CLSID in `HKCU\Software\Classes\CLSID\{guid}\LocalServer32` pointing to exe
3. Implement `INotificationActivationCallback` COM interface
4. On activation, exe is launched (or existing instance receives call via named pipe)

**Alternative considered:** Protocol activation (custom URI scheme like `claude-notify://approve?id=xxx`) — simpler but launches new process each time, losing window context.

### Named Pipe for Single Instance Communication

**Decision:** Use a named pipe (`\\.\pipe\claude-win-notify`) for inter-process communication.

**Rationale:** When Toast COM activation launches a new exe instance (or the exe is already running), the callback data needs to reach the main monitoring loop. A named pipe provides reliable IPC without external dependencies.

### JSONL Parsing Strategy

**Decision:** Use `System.IO.Pipelines` + `Utf8JsonReader` for zero-allocation streaming.

**Rationale:** Claude Code transcript files can grow large during long sessions. Streaming with `ReadOnlySequence<byte>` and `Utf8JsonReader` avoids loading entire files into memory and provides ~3x throughput vs `JsonSerializer.Deserialize<T>()` per line.

---

## Version Pinning & Compatibility Matrix

| Component | Minimum Version | Tested On | Notes |
|-----------|----------------|-----------|-------|
| .NET SDK | 9.0.100 | 9.0.300 | NativeAOT requires 8.0+ but 9.0 has WinRT AOT fixes |
| Windows | 10.0.18362 (1903) | 10.0.22621 (22H2) | Toast XML Schema v4 needs 1903+ |
| CsWinRT | 2.1.0 | 2.1.3 | AOT source generator support |
| Windows Terminal | 1.18+ | 1.21 | UI Automation tree structure |
| Warp | Any | Latest | UI Automation compatibility TBD |

---

## What NOT to Use

| Technology | Why Not |
|------------|---------|
| **WPF/WinForms** | Adds massive framework dependency; we're a CLI tool, not a GUI app |
| **UWP** | Dead platform; cannot do NativeAOT; cannot be a console app |
| **MAUI** | Overkill; not trim-friendly; adds ~50MB to binary |
| **Microsoft.Toolkit.Uwp.Notifications** | NuGet package has trim warnings; wraps same WinRT API we can call directly |
| **Newtonsoft.Json** | Not trim-compatible; reflection-heavy; System.Text.Json is built-in and faster |
| **go-toast** | Abandoned (2017); PowerShell-based; no COM callbacks; architectural dead end |
| **win-toast-notify (Rust crate)** | Only supports Protocol activation; no Foreground/Background COM callbacks |
| **electron/tauri** | Wrong paradigm entirely; we need a headless CLI tool |
| **PowerShell module** | Cannot do COM activation callbacks; startup latency; unsigned script policy issues |

---

## Build Commands

```bash
# Development build
dotnet build

# Release build (NativeAOT single exe)
dotnet publish -c Release -r win-x64

# The above produces: bin/Release/net9.0-windows10.0.19041.0/win-x64/publish/ClaudeWinNotify.exe
# Expected size: ~8-12 MB
# Expected cold start: <50ms
```

---

## Sources

- Context7: `/microsoft/windows-rs` (version 0.62 confirmed, `#[implement]` for COM, WinRT projection)
- Context7: `/microsoft/cswinrt` (AOT support via `CsWinRTAotOptimizerEnabled`, source generators)
- Context7: `/ikineticate/win-toast-notify` (only Protocol activation type — no foreground callbacks)
- Context7: `/go-toast/toast` (PowerShell-based, no COM activation support)
- Context7: `/aetherinox/ntfy-toast` (C++ reference implementation using named pipes for callbacks)
- .NET NativeAOT documentation (training data, MEDIUM confidence on exact exe sizes)
- Windows Toast Notification schema documentation (training data, HIGH confidence on API surface)

---
phase: 1
plan_id: 01-C
title: "Protocol Activation Validation (Both Stacks)"
wave: 2
depends_on: [01-A]
files_modified:
  - spike/csharp/Program.cs
  - spike/rust/src/main.rs
requirements_addressed: [TECH-01]
autonomous: true
must_haves:
  goal: "Protocol activation (claude-notify://) launches exe and passes URI arguments correctly"
  truths:
    - "Registry key HKCU\\Software\\Classes\\claude-notify exists with URL Protocol"
    - "Running Start-Process 'claude-notify://focus?session=test123&pid=999' launches the spike exe"
    - "The launched exe correctly parses session and pid from the URI query string"
    - "Both C# and Rust exes receive and parse the URI identically"
---

# Plan 01-C: Protocol Activation Validation (Both Stacks)

## Objective

Prove that a custom protocol URI scheme (`claude-notify://`) can be registered and that clicking it (or invoking it programmatically) launches our exe with the full URI as an argument. This is integration point #2.

## Tasks

<task id="C1">
<title>Register claude-notify:// protocol in registry</title>
<read_first>
- .planning/phases/01-tech-spike/01-RESEARCH.md (Protocol Activation section — registry structure)
- .planning/research/ARCHITECTURE.md (Pattern 2: Protocol Activation)
- .planning/phases/01-tech-spike/01-CONTEXT.md (D-16: Protocol URI registration as part of spike)
</read_first>
<action>
Create a PowerShell script `spike/register-protocol.ps1` that registers the protocol handler:

```powershell
param(
    [Parameter(Mandatory=$true)]
    [string]$ExePath
)

$exeFullPath = (Resolve-Path $ExePath).Path

# Register protocol handler
$regPath = "HKCU:\Software\Classes\claude-notify"
New-Item -Path $regPath -Force | Out-Null
Set-ItemProperty -Path $regPath -Name "(Default)" -Value "URL:Claude Win Notify Protocol"
Set-ItemProperty -Path $regPath -Name "URL Protocol" -Value ""

New-Item -Path "$regPath\shell\open\command" -Force | Out-Null
Set-ItemProperty -Path "$regPath\shell\open\command" -Name "(Default)" -Value "`"$exeFullPath`" --focus `"%1`""

Write-Host "Registered claude-notify:// protocol -> $exeFullPath"
Write-Host "Registry key: $regPath"
```

Run it pointing to a temporary test exe first (can be either C# or Rust):
```powershell
.\spike\register-protocol.ps1 -ExePath "spike\csharp\bin\Release\net9.0-windows10.0.19041.0\win-x64\publish\CSharpSpike.exe"
```

Verify registry:
```powershell
Get-ItemProperty "HKCU:\Software\Classes\claude-notify\shell\open\command"
```
</action>
<acceptance_criteria>
- File `spike/register-protocol.ps1` exists
- `Get-ItemProperty "HKCU:\Software\Classes\claude-notify"` shows "URL Protocol" property
- `Get-ItemProperty "HKCU:\Software\Classes\claude-notify\shell\open\command"` shows exe path with `--focus "%1"`
</acceptance_criteria>
</task>

<task id="C2">
<title>Add protocol argument parsing to C# spike</title>
<read_first>
- spike/csharp/Program.cs (current code with toast logic)
- .planning/phases/01-tech-spike/01-RESEARCH.md (C# protocol parsing approach)
</read_first>
<action>
Extend `Program.cs` to handle `--focus` argument with URI parsing. The exe should support two modes:
1. No args → show toast (existing behavior from Plan B)
2. `--focus <uri>` → parse URI and print extracted parameters

Add this code path:
```csharp
if (args.Length >= 2 && args[0] == "--focus")
{
    var uri = new Uri(args[1]);
    var query = System.Web.HttpUtility.ParseQueryString(uri.Query);
    // Note: System.Web may not be AOT-safe. Alternative: manual parsing
    // Manual parse if System.Web is not available:
    var queryString = uri.Query.TrimStart('?');
    var pairs = queryString.Split('&')
        .Select(p => p.Split('=', 2))
        .ToDictionary(p => p[0], p => Uri.UnescapeDataString(p.Length > 1 ? p[1] : ""));
    
    var session = pairs.GetValueOrDefault("session", "unknown");
    var pid = pairs.GetValueOrDefault("pid", "0");
    
    Console.WriteLine($"PROTOCOL ACTIVATED:");
    Console.WriteLine($"  Full URI: {args[1]}");
    Console.WriteLine($"  Session: {session}");
    Console.WriteLine($"  PID: {pid}");
    Console.WriteLine("SUCCESS: Protocol activation parsed correctly (C#)");
    return;
}
```

Rebuild NativeAOT and test:
```powershell
dotnet publish -c Release -r win-x64
.\bin\Release\...\CSharpSpike.exe --focus "claude-notify://focus?session=test123&pid=999"
```
</action>
<acceptance_criteria>
- Running `CSharpSpike.exe --focus "claude-notify://focus?session=test123&pid=999"` outputs "Session: test123" and "PID: 999"
- Console output contains "SUCCESS: Protocol activation parsed correctly (C#)"
- NativeAOT publish still succeeds (no trim warnings from URI parsing)
</acceptance_criteria>
</task>

<task id="C3">
<title>Add protocol argument parsing to Rust spike</title>
<read_first>
- spike/rust/src/main.rs (current code with toast logic)
- spike/rust/Cargo.toml (current dependencies)
- .planning/phases/01-tech-spike/01-RESEARCH.md (Rust protocol parsing approach)
</read_first>
<action>
Add `url` crate to `Cargo.toml`:
```toml
[dependencies]
url = "2"
```

Extend `src/main.rs` to handle `--focus` argument:
```rust
use std::env;
use url::Url;

fn main() -> windows::core::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() >= 3 && args[1] == "--focus" {
        return handle_focus(&args[2]);
    }
    
    // Existing toast code...
    show_toast()
}

fn handle_focus(uri_str: &str) -> windows::core::Result<()> {
    let url = Url::parse(uri_str).expect("Failed to parse URI");
    
    let session = url.query_pairs()
        .find(|(k, _)| k == "session")
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    let pid = url.query_pairs()
        .find(|(k, _)| k == "pid")
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| "0".to_string());
    
    println!("PROTOCOL ACTIVATED:");
    println!("  Full URI: {}", uri_str);
    println!("  Session: {}", session);
    println!("  PID: {}", pid);
    println!("SUCCESS: Protocol activation parsed correctly (Rust)");
    Ok(())
}
```

Build and test:
```powershell
cargo build --release
.\target\release\rust-spike.exe --focus "claude-notify://focus?session=test123&pid=999"
```
</action>
<acceptance_criteria>
- Running `rust-spike.exe --focus "claude-notify://focus?session=test123&pid=999"` outputs "Session: test123" and "PID: 999"
- Console output contains "SUCCESS: Protocol activation parsed correctly (Rust)"
- `cargo build --release` succeeds without errors
</acceptance_criteria>
</task>

<task id="C4">
<title>End-to-end protocol activation test</title>
<read_first>
- spike/register-protocol.ps1 (protocol registration script)
- spike/csharp/Program.cs (to verify --focus handling exists)
</read_first>
<action>
Test the full flow: invoke protocol URI → Windows launches exe → exe parses args.

1. Register protocol pointing to C# spike exe:
```powershell
.\spike\register-protocol.ps1 -ExePath "spike\csharp\bin\Release\net9.0-windows10.0.19041.0\win-x64\publish\CSharpSpike.exe"
```

2. Invoke protocol (this should launch the exe):
```powershell
Start-Process "claude-notify://focus?session=e2etest&pid=12345"
```

3. Expected: A new console window briefly appears showing the parsed URI parameters.
   (In production, the exe would focus a window instead of printing to console.)

4. Repeat with Rust exe:
```powershell
.\spike\register-protocol.ps1 -ExePath "spike\rust\target\release\rust-spike.exe"
Start-Process "claude-notify://focus?session=e2etest&pid=12345"
```

5. Test with special characters (CJK path simulation):
```powershell
Start-Process "claude-notify://focus?session=test&pid=123&cwd=C%3A%5CUsers%5C%E6%B5%8B%E8%AF%95%5Cproject"
```

Record results in `spike/protocol-results.md`.
</action>
<acceptance_criteria>
- `Start-Process "claude-notify://focus?session=e2etest&pid=12345"` launches the registered exe
- The exe correctly prints the session and pid values from the URI
- URL-encoded CJK characters in the URI are decoded correctly
- File `spike/protocol-results.md` exists with pass/fail for both stacks
</acceptance_criteria>
</task>

## Verification

```powershell
# Registry check
(Get-ItemProperty "HKCU:\Software\Classes\claude-notify")."URL Protocol" -eq ""  # True

# Direct invocation (no Windows launch, just arg parsing)
& spike\csharp\bin\Release\*\win-x64\publish\CSharpSpike.exe --focus "claude-notify://focus?session=verify&pid=1"
& spike\rust\target\release\rust-spike.exe --focus "claude-notify://focus?session=verify&pid=1"
```

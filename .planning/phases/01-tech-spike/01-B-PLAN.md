---
phase: 1
plan_id: 01-B
title: "Toast Notification Validation (Both Stacks)"
wave: 2
depends_on: [01-A]
files_modified:
  - spike/csharp/Program.cs
  - spike/csharp/CSharpSpike.csproj
  - spike/rust/src/main.rs
  - spike/rust/Cargo.toml
requirements_addressed: [TECH-01]
autonomous: true
must_haves:
  goal: "Both C# NativeAOT and Rust produce a native Toast notification via WinRT API"
  truths:
    - "C# NativeAOT-compiled exe shows a visible Windows Toast notification"
    - "Rust release-compiled exe shows a visible Windows Toast notification"
    - "Both toasts display custom title and body text"
    - "Both exes run without .NET runtime or any external dependency"
---

# Plan 01-B: Toast Notification Validation (Both Stacks)

## Objective

Prove that both C# NativeAOT and Rust can display a Windows native Toast notification via the WinRT `Windows.UI.Notifications` API. This is integration point #1 from the spike scope.

## Tasks

<task id="B1">
<title>Implement C# Toast via WinRT (CsWinRT)</title>
<read_first>
- spike/csharp/CSharpSpike.csproj (current project configuration)
- spike/csharp/Program.cs (current code)
- .planning/phases/01-tech-spike/01-RESEARCH.md (C# NativeAOT Toast approach)
- .planning/research/PITFALLS.md (Pitfall #4: NativeAOT COM limitation)
</read_first>
<action>
1. Ensure `.csproj` has the Windows SDK package reference for WinRT projections:
```xml
<ItemGroup>
  <PackageReference Include="Microsoft.Windows.SDK.NET.Ref" Version="10.0.19041.34" />
</ItemGroup>
```
If that doesn't work with NativeAOT, try the CsWinRT package directly:
```xml
<ItemGroup>
  <PackageReference Include="Microsoft.Windows.CsWinRT" Version="2.1.3" />
</ItemGroup>
```

2. Replace `Program.cs` with Toast display code:
```csharp
using Windows.UI.Notifications;
using Windows.Data.Xml.Dom;

// Use PowerShell's AUMID for spike (avoids Start Menu shortcut requirement)
const string AUMID = "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe";

var toastXml = new XmlDocument();
toastXml.LoadXml(@"
<toast>
  <visual>
    <binding template='ToastGeneric'>
      <text>Claude Code [C# Spike]</text>
      <text>Toast notification via NativeAOT — integration point #1 validated!</text>
    </binding>
  </visual>
  <audio src='ms-winsoundevent:Notification.Default'/>
</toast>");

var toast = new ToastNotification(toastXml);
var notifier = ToastNotificationManager.CreateToastNotifier(AUMID);
notifier.Show(toast);

Console.WriteLine("SUCCESS: Toast notification displayed (C# NativeAOT)");
```

3. Build and publish NativeAOT:
```powershell
dotnet publish -c Release -r win-x64
```

4. Run the published exe (NOT `dotnet run` — must validate the NativeAOT binary):
```powershell
.\bin\Release\net9.0-windows10.0.19041.0\win-x64\publish\CSharpSpike.exe
```

5. If NativeAOT build fails with trim/COM warnings, document the exact errors. Try mitigations:
   - Add `<TrimmerSingleWarn>false</TrimmerSingleWarn>` to see all warnings
   - Try `[DynamicallyAccessedMembers]` annotations
   - Try Windows App SDK `AppNotificationManager` as alternative
   - If unresolvable, mark C# as FAILED for this integration point
</action>
<acceptance_criteria>
- Running the NativeAOT-published exe (not `dotnet run`) produces a visible Windows Toast notification
- Console output contains "SUCCESS: Toast notification displayed (C# NativeAOT)"
- The exe runs on a machine without .NET runtime installed (or: exe file has no .NET DLLs alongside it)
- If FAILED: a file `spike/csharp/TOAST-FAIL.md` documents the exact error and mitigations attempted
</acceptance_criteria>
</task>

<task id="B2">
<title>Implement Rust Toast via windows-rs</title>
<read_first>
- spike/rust/Cargo.toml (current dependencies)
- spike/rust/src/main.rs (current code)
- .planning/phases/01-tech-spike/01-RESEARCH.md (Rust Toast approach)
- .planning/research/STACK.md (Rust windows-rs section)
</read_first>
<action>
1. Ensure `Cargo.toml` has the required feature flags:
```toml
[dependencies.windows]
version = "0.62"
features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "UI_Notifications",
    "Data_Xml_Dom",
]
```

2. Replace `src/main.rs` with Toast display code:
```rust
use windows::core::*;
use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

fn main() -> Result<()> {
    // Use PowerShell's AUMID for spike
    let aumid = "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe";
    
    let toast_xml = XmlDocument::new()?;
    toast_xml.LoadXml(&HSTRING::from(
        r#"<toast>
          <visual>
            <binding template="ToastGeneric">
              <text>Claude Code [Rust Spike]</text>
              <text>Toast notification via windows-rs — integration point #1 validated!</text>
            </binding>
          </visual>
          <audio src="ms-winsoundevent:Notification.Default"/>
        </toast>"#,
    ))?;

    let toast = ToastNotification::CreateToastNotification(&toast_xml)?;
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(aumid))?;
    notifier.Show(&toast)?;

    println!("SUCCESS: Toast notification displayed (Rust)");
    Ok(())
}
```

3. Build release:
```powershell
cargo build --release
```

4. Run the release binary:
```powershell
.\target\release\rust-spike.exe
```

5. If build fails (missing features, API changes in 0.62), document errors. Try:
   - Check if feature names changed (e.g., `Windows_UI_Notifications` vs `UI_Notifications`)
   - Check windows-rs 0.62 docs for correct HSTRING usage
   - If unresolvable, mark Rust as FAILED for this integration point
</action>
<acceptance_criteria>
- Running `spike/rust/target/release/rust-spike.exe` produces a visible Windows Toast notification
- Console output contains "SUCCESS: Toast notification displayed (Rust)"
- The exe is a single file with no external DLL dependencies
- If FAILED: a file `spike/rust/TOAST-FAIL.md` documents the exact error and mitigations attempted
</acceptance_criteria>
</task>

<task id="B3">
<title>Record Toast validation results</title>
<read_first>
- spike/csharp/Program.cs (to check if toast code exists)
- spike/rust/src/main.rs (to check if toast code exists)
</read_first>
<action>
After running both spikes, record results in a temporary notes file `spike/toast-results.md`:

```markdown
# Toast Validation Results

## C# NativeAOT
- Build: PASS/FAIL
- NativeAOT publish: PASS/FAIL  
- Toast displayed: PASS/FAIL
- Binary size: X MB
- Notes: [any warnings, workarounds needed]

## Rust
- Build: PASS/FAIL
- Release build: PASS/FAIL
- Toast displayed: PASS/FAIL
- Binary size: X MB
- Notes: [any warnings, workarounds needed]

## AUMID Used
- PowerShell AUMID for both (avoids shortcut requirement in spike)
- Custom AUMID registration deferred to Phase 2
```

Measure binary sizes:
```powershell
(Get-Item spike\csharp\bin\Release\net9.0-windows10.0.19041.0\win-x64\publish\CSharpSpike.exe).Length / 1MB
(Get-Item spike\rust\target\release\rust-spike.exe).Length / 1MB
```
</action>
<acceptance_criteria>
- File `spike/toast-results.md` exists
- File contains binary sizes for both exes
- File contains PASS or FAIL status for each stack's toast validation
</acceptance_criteria>
</task>

## Verification

```powershell
# Visual verification required: user must confirm toast appeared on screen
# Automated checks:
Test-Path spike/csharp/bin/Release/*/win-x64/publish/CSharpSpike.exe  # True
Test-Path spike/rust/target/release/rust-spike.exe                     # True
Test-Path spike/toast-results.md                                       # True
```

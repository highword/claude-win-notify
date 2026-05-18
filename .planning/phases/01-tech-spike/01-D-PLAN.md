---
phase: 1
plan_id: 01-D
title: "SetForegroundWindow Validation (Both Stacks)"
wave: 3
depends_on: [01-B, 01-C]
files_modified:
  - spike/csharp/Program.cs
  - spike/rust/src/main.rs
requirements_addressed: [TECH-01]
autonomous: false
must_haves:
  goal: "SetForegroundWindow (with fallback chain) brings a terminal window to foreground from protocol-activated process"
  truths:
    - "C# exe brings a target window to foreground when window is minimized"
    - "Rust exe brings a target window to foreground when window is minimized"
    - "Focus works when multiple terminal windows are open (correct window targeted by PID)"
    - "Fallback chain engages when primary SetForegroundWindow fails"
    - "Results for all 4 scenarios documented with pass/fail/flash status"
---

# Plan 01-D: SetForegroundWindow Validation (Both Stacks)

## Objective

Prove that both C# and Rust can bring a target terminal window to the foreground using `SetForegroundWindow` with a fallback chain. Test across 4 scenarios defined in D-04. This is integration point #3 and the core differentiator.

## Tasks

<task id="D1">
<title>Implement SetForegroundWindow with fallback chain in C#</title>
<read_first>
- spike/csharp/Program.cs (current code with toast + protocol handling)
- .planning/phases/01-tech-spike/01-RESEARCH.md (SetForegroundWindow section — fallback chain, P/Invoke declarations)
- .planning/research/PITFALLS.md (Pitfall #1: SetForegroundWindow silently fails)
</read_first>
<action>
Add a `--focus-window` mode to the C# spike that:
1. Takes a PID as argument
2. Finds the main window of that process via EnumWindows
3. Attempts to bring it to foreground using the fallback chain

Add P/Invoke declarations:
```csharp
using System.Runtime.InteropServices;

[DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr hWnd);
[DllImport("user32.dll")] static extern bool AttachThreadInput(uint idAttach, uint idAttachTo, bool fAttach);
[DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
[DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
[DllImport("user32.dll")] static extern uint GetCurrentThreadId();
[DllImport("user32.dll")] static extern bool IsIconic(IntPtr hWnd);
[DllImport("user32.dll")] static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
[DllImport("user32.dll")] static extern bool FlashWindowEx(ref FLASHWINFO pwfi);
[DllImport("user32.dll")] static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);
[DllImport("user32.dll")] static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
[DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hWnd);

delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

const int SW_RESTORE = 9;
const int SW_MINIMIZE = 6;
const int SW_SHOW = 5;
const byte VK_MENU = 0x12; // Alt key
const uint KEYEVENTF_EXTENDEDKEY = 0x0001;
const uint KEYEVENTF_KEYUP = 0x0002;
```

Implement the focus logic:
```csharp
static IntPtr FindMainWindowByPid(uint targetPid)
{
    IntPtr found = IntPtr.Zero;
    EnumWindows((hWnd, _) => {
        GetWindowThreadProcessId(hWnd, out uint pid);
        if (pid == targetPid && IsWindowVisible(hWnd))
        {
            found = hWnd;
            return false; // stop enumeration
        }
        return true;
    }, IntPtr.Zero);
    return found;
}

static bool FocusWindow(IntPtr hwnd)
{
    // Restore if minimized
    if (IsIconic(hwnd))
        ShowWindow(hwnd, SW_RESTORE);
    
    // Strategy 1: Direct SetForegroundWindow
    if (SetForegroundWindow(hwnd))
    {
        Console.WriteLine("  Strategy 1 (direct): SUCCESS");
        return true;
    }
    
    // Strategy 2: AttachThreadInput
    var fgHwnd = GetForegroundWindow();
    var fgTid = GetWindowThreadProcessId(fgHwnd, out _);
    var ourTid = GetCurrentThreadId();
    AttachThreadInput(ourTid, fgTid, true);
    bool result = SetForegroundWindow(hwnd);
    AttachThreadInput(ourTid, fgTid, false);
    if (result)
    {
        Console.WriteLine("  Strategy 2 (AttachThreadInput): SUCCESS");
        return true;
    }
    
    // Strategy 3: Alt key hack
    keybd_event(VK_MENU, 0, KEYEVENTF_EXTENDEDKEY, UIntPtr.Zero);
    keybd_event(VK_MENU, 0, KEYEVENTF_KEYUP, UIntPtr.Zero);
    result = SetForegroundWindow(hwnd);
    if (result)
    {
        Console.WriteLine("  Strategy 3 (Alt key hack): SUCCESS");
        return true;
    }
    
    // Strategy 4: Minimize then restore
    ShowWindow(hwnd, SW_MINIMIZE);
    ShowWindow(hwnd, SW_RESTORE);
    Console.WriteLine("  Strategy 4 (minimize/restore): ATTEMPTED");
    return GetForegroundWindow() == hwnd;
}
```

Wire up to `--focus` handler: after parsing the URI, call `FindMainWindowByPid` with the PID and then `FocusWindow`.

Rebuild NativeAOT and test against an open Notepad window:
```powershell
# Open notepad, get its PID, then test focus
$np = Start-Process notepad -PassThru
Start-Sleep 1
.\CSharpSpike.exe --focus "claude-notify://focus?session=test&pid=$($np.Id)"
```
</action>
<acceptance_criteria>
- Running `CSharpSpike.exe --focus "claude-notify://focus?session=x&pid=<notepad_pid>"` brings the Notepad window to foreground
- Console output shows which strategy succeeded (1, 2, 3, or 4)
- NativeAOT publish succeeds with no errors from P/Invoke declarations
- If the window was minimized before the test, it is restored and brought to front
</acceptance_criteria>
</task>

<task id="D2">
<title>Implement SetForegroundWindow with fallback chain in Rust</title>
<read_first>
- spike/rust/src/main.rs (current code with toast + protocol handling)
- spike/rust/Cargo.toml (current dependencies)
- .planning/phases/01-tech-spike/01-RESEARCH.md (Rust windows-rs SetForegroundWindow calls)
- .planning/research/PITFALLS.md (Pitfall #1: SetForegroundWindow silently fails)
</read_first>
<action>
Ensure `Cargo.toml` has the required Win32 features:
```toml
[dependencies.windows]
version = "0.62"
features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_System_Threading",
    "UI_Notifications",
    "Data_Xml_Dom",
]
```

Add focus implementation to `src/main.rs`:
```rust
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::Foundation::*;

unsafe fn find_window_by_pid(target_pid: u32) -> Option<HWND> {
    let mut result: Option<HWND> = None;
    let result_ptr: *mut Option<HWND> = &mut result;
    
    EnumWindows(Some(enum_callback), LPARAM(result_ptr as isize));
    // Need to pass target_pid somehow — use a struct or thread-local
    result
}

unsafe fn focus_window(hwnd: HWND) -> bool {
    // Restore if minimized
    if IsIconic(hwnd).as_bool() {
        ShowWindow(hwnd, SW_RESTORE);
    }
    
    // Strategy 1: Direct
    if SetForegroundWindow(hwnd).as_bool() {
        println!("  Strategy 1 (direct): SUCCESS");
        return true;
    }
    
    // Strategy 2: AttachThreadInput
    let fg_hwnd = GetForegroundWindow();
    let fg_tid = GetWindowThreadProcessId(fg_hwnd, None);
    let our_tid = GetCurrentThreadId();
    AttachThreadInput(our_tid, fg_tid, true);
    let result = SetForegroundWindow(hwnd).as_bool();
    AttachThreadInput(our_tid, fg_tid, false);
    if result {
        println!("  Strategy 2 (AttachThreadInput): SUCCESS");
        return true;
    }
    
    // Strategy 3: Alt key hack
    keybd_event(VK_MENU.0 as u8, 0, KEYEVENTF_EXTENDEDKEY, 0);
    keybd_event(VK_MENU.0 as u8, 0, KEYEVENTF_KEYUP, 0);
    if SetForegroundWindow(hwnd).as_bool() {
        println!("  Strategy 3 (Alt key hack): SUCCESS");
        return true;
    }
    
    // Strategy 4: Minimize/restore
    ShowWindow(hwnd, SW_MINIMIZE);
    ShowWindow(hwnd, SW_RESTORE);
    println!("  Strategy 4 (minimize/restore): ATTEMPTED");
    GetForegroundWindow() == hwnd
}
```

Note: The actual EnumWindows callback with PID matching requires careful unsafe code. Use a static/thread-local to pass the target PID, or use a closure-based wrapper.

Build and test:
```powershell
cargo build --release
$np = Start-Process notepad -PassThru
Start-Sleep 1
.\target\release\rust-spike.exe --focus "claude-notify://focus?session=test&pid=$($np.Id)"
```
</action>
<acceptance_criteria>
- Running `rust-spike.exe --focus "claude-notify://focus?session=x&pid=<notepad_pid>"` brings the Notepad window to foreground
- Console output shows which strategy succeeded (1, 2, 3, or 4)
- `cargo build --release` succeeds without errors
- If the window was minimized before the test, it is restored and brought to front
</acceptance_criteria>
</task>

<task id="D3">
<title>Run 4-scenario focus test matrix</title>
<read_first>
- .planning/phases/01-tech-spike/01-CONTEXT.md (D-04: 4 scenarios, D-05: MUST pass, D-06: MAY flash)
- spike/csharp/Program.cs (to verify focus code exists)
- spike/rust/src/main.rs (to verify focus code exists)
</read_first>
<action>
Create a test script `spike/test-focus.ps1` that automates the 4 scenarios:

```powershell
param([string]$ExePath)

Write-Host "=== SetForegroundWindow Test Matrix ==="
Write-Host "Exe: $ExePath"
Write-Host ""

# Scenario 1: Window minimized/idle (MUST PASS per D-05)
Write-Host "--- Scenario 1: Window minimized/idle ---"
$notepad = Start-Process notepad -PassThru
Start-Sleep 1
# Minimize it
Add-Type -TypeDefinition 'using System;using System.Runtime.InteropServices;public class Win32{[DllImport("user32.dll")]public static extern bool ShowWindow(IntPtr hWnd,int nCmdShow);}'
[Win32]::ShowWindow($notepad.MainWindowHandle, 6) # SW_MINIMIZE
Start-Sleep 1
Write-Host "Notepad minimized. Running focus..."
& $ExePath --focus "claude-notify://focus?session=s1&pid=$($notepad.Id)"
Start-Sleep 1
Write-Host "Is Notepad foreground? (Check visually)"
Read-Host "Press Enter to continue"
Stop-Process $notepad

# Scenario 2: Multiple windows (MUST PASS per D-05)
Write-Host ""
Write-Host "--- Scenario 2: Multiple terminal windows ---"
$np1 = Start-Process notepad -PassThru
$np2 = Start-Process notepad -PassThru
Start-Sleep 1
Write-Host "Two notepads open. Focusing PID $($np1.Id) (first one)..."
& $ExePath --focus "claude-notify://focus?session=s2&pid=$($np1.Id)"
Start-Sleep 1
Write-Host "Did the FIRST notepad come to front? (Check visually)"
Read-Host "Press Enter to continue"
Stop-Process $np1
Stop-Process $np2

# Scenario 3: User typing in foreground (MAY flash per D-06)
Write-Host ""
Write-Host "--- Scenario 3: User typing in foreground ---"
$np = Start-Process notepad -PassThru
Start-Sleep 1
Write-Host "Start typing in THIS terminal window, then press Enter..."
Read-Host ""
& $ExePath --focus "claude-notify://focus?session=s3&pid=$($np.Id)"
Start-Sleep 1
Write-Host "Did Notepad come to front, or just taskbar flash? (Note result)"
Read-Host "Press Enter to continue"
Stop-Process $np

# Scenario 4: Fullscreen app (MAY flash per D-06)
Write-Host ""
Write-Host "--- Scenario 4: Fullscreen app ---"
Write-Host "Open a fullscreen app (e.g., press F11 in a browser)"
Write-Host "Then press Enter to run focus test..."
$np = Start-Process notepad -PassThru
Start-Sleep 1
Read-Host ""
& $ExePath --focus "claude-notify://focus?session=s4&pid=$($np.Id)"
Start-Sleep 1
Write-Host "Did Notepad come to front, or just taskbar flash? (Note result)"
Read-Host "Press Enter to continue"
Stop-Process $np

Write-Host ""
Write-Host "=== Test complete. Record results in spike/focus-results.md ==="
```

Run for both stacks and record results in `spike/focus-results.md`:
```markdown
# SetForegroundWindow Test Results

## C# NativeAOT

| Scenario | Required | Result | Strategy Used |
|----------|----------|--------|---------------|
| Minimized/idle | MUST PASS | ? | ? |
| Multiple windows | MUST PASS | ? | ? |
| User typing | May flash | ? | ? |
| Fullscreen | May flash | ? | ? |

## Rust

| Scenario | Required | Result | Strategy Used |
|----------|----------|--------|---------------|
| Minimized/idle | MUST PASS | ? | ? |
| Multiple windows | MUST PASS | ? | ? |
| User typing | May flash | ? | ? |
| Fullscreen | May flash | ? | ? |
```
</action>
<acceptance_criteria>
- File `spike/test-focus.ps1` exists and runs the 4 scenarios
- File `spike/focus-results.md` exists with results for both stacks
- Both stacks PASS scenarios 1 and 2 (minimized, multiple windows) — hard requirement from D-05
- Scenarios 3 and 4 documented as PASS or FLASH (both acceptable per D-06)
</acceptance_criteria>
</task>

## Verification

```powershell
# Core requirement: scenarios 1 & 2 must pass for at least one stack
Test-Path spike/focus-results.md  # True
# Manual verification: review focus-results.md for PASS on mandatory scenarios
```

## Notes

- This plan is `autonomous: false` because scenarios 3 and 4 require user interaction (typing, fullscreen app)
- D-17 (Collaborative execution): Show results at checkpoint, user participates

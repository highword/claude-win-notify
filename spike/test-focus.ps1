# SetForegroundWindow 4-Scenario Test Script
# Usage: .\test-focus.ps1 -ExePath <path-to-exe>
# Runs all 4 scenarios and outputs results to console.
# For automated CI: scenarios 1-2 verified by GetForegroundWindow check (exit code).
# For scenarios 3-4: records strategy output (human visual check optional).

param(
    [Parameter(Mandatory=$true)]
    [string]$ExePath
)

$ErrorActionPreference = "Stop"

# Helper: P/Invoke for ShowWindow (minimize)
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class Win32Focus {
    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
    public const int SW_MINIMIZE = 6;
    public const int SW_RESTORE = 9;
}
"@

$results = @()

Write-Host "=== SetForegroundWindow Test Matrix ===" -ForegroundColor Cyan
Write-Host "Exe: $ExePath"
Write-Host ""

# --- Scenario 1: Window minimized/idle (MUST PASS per D-05) ---
Write-Host "--- Scenario 1: Window minimized/idle ---" -ForegroundColor Yellow
try {
    $notepad = Start-Process notepad -PassThru
    Start-Sleep -Seconds 2

    # Wait for main window handle
    $notepad.WaitForInputIdle() | Out-Null
    $notepad.Refresh()

    # Minimize the window
    [Win32Focus]::ShowWindow($notepad.MainWindowHandle, [Win32Focus]::SW_MINIMIZE) | Out-Null
    Start-Sleep -Seconds 1

    Write-Host "  Notepad PID: $($notepad.Id), minimized."
    Write-Host "  Running focus command..."

    $output = & $ExePath --focus "claude-notify://focus?session=s1&pid=$($notepad.Id)" 2>&1
    $exitCode = $LASTEXITCODE
    $output | ForEach-Object { Write-Host "  $_" }

    $strategy = ($output | Select-String "Strategy \d") -replace '.*Strategy (\d \([^)]+\)).*', '$1'
    if (-not $strategy) { $strategy = "N/A" }

    $result = if ($exitCode -eq 0) { "PASS" } elseif ($exitCode -eq 2) { "FLASH" } else { "FAIL" }
    $results += [PSCustomObject]@{
        Scenario = "Minimized/idle"
        Required = "MUST PASS"
        Result = $result
        Strategy = $strategy
        ExitCode = $exitCode
    }
    Write-Host "  Result: $result (exit $exitCode)" -ForegroundColor $(if($result -eq "PASS"){"Green"}else{"Red"})

    Stop-Process $notepad -ErrorAction SilentlyContinue
} catch {
    Write-Host "  ERROR: $_" -ForegroundColor Red
    $results += [PSCustomObject]@{ Scenario="Minimized/idle"; Required="MUST PASS"; Result="ERROR"; Strategy="N/A"; ExitCode=-1 }
}

Start-Sleep -Seconds 1

# --- Scenario 2: Multiple windows (MUST PASS per D-05) ---
Write-Host ""
Write-Host "--- Scenario 2: Multiple terminal windows ---" -ForegroundColor Yellow
try {
    $np1 = Start-Process notepad -PassThru
    $np2 = Start-Process notepad -PassThru
    Start-Sleep -Seconds 2
    $np1.WaitForInputIdle() | Out-Null
    $np2.WaitForInputIdle() | Out-Null

    # Focus np2 to make it foreground (so np1 is NOT foreground)
    $np2.Refresh()
    [Win32Focus]::ShowWindow($np2.MainWindowHandle, [Win32Focus]::SW_RESTORE) | Out-Null
    Start-Sleep -Seconds 1

    Write-Host "  Two notepads: PID $($np1.Id) and PID $($np2.Id)"
    Write-Host "  Focusing PID $($np1.Id) (first one, not currently foreground)..."

    $output = & $ExePath --focus "claude-notify://focus?session=s2&pid=$($np1.Id)" 2>&1
    $exitCode = $LASTEXITCODE
    $output | ForEach-Object { Write-Host "  $_" }

    $strategy = ($output | Select-String "Strategy \d") -replace '.*Strategy (\d \([^)]+\)).*', '$1'
    if (-not $strategy) { $strategy = "N/A" }

    $result = if ($exitCode -eq 0) { "PASS" } elseif ($exitCode -eq 2) { "FLASH" } else { "FAIL" }
    $results += [PSCustomObject]@{
        Scenario = "Multiple windows"
        Required = "MUST PASS"
        Result = $result
        Strategy = $strategy
        ExitCode = $exitCode
    }
    Write-Host "  Result: $result (exit $exitCode)" -ForegroundColor $(if($result -eq "PASS"){"Green"}else{"Red"})

    Stop-Process $np1 -ErrorAction SilentlyContinue
    Stop-Process $np2 -ErrorAction SilentlyContinue
} catch {
    Write-Host "  ERROR: $_" -ForegroundColor Red
    $results += [PSCustomObject]@{ Scenario="Multiple windows"; Required="MUST PASS"; Result="ERROR"; Strategy="N/A"; ExitCode=-1 }
}

Start-Sleep -Seconds 1

# --- Scenario 3: User typing in foreground (MAY flash per D-06) ---
Write-Host ""
Write-Host "--- Scenario 3: User typing in foreground ---" -ForegroundColor Yellow
try {
    $np = Start-Process notepad -PassThru
    Start-Sleep -Seconds 2
    $np.WaitForInputIdle() | Out-Null

    # Simulate "user is active" by keeping current console as foreground
    # The test runner (this script) is the foreground process
    Write-Host "  Notepad PID: $($np.Id). Console is foreground (simulating user activity)."
    Write-Host "  Running focus command..."

    $output = & $ExePath --focus "claude-notify://focus?session=s3&pid=$($np.Id)" 2>&1
    $exitCode = $LASTEXITCODE
    $output | ForEach-Object { Write-Host "  $_" }

    $strategy = ($output | Select-String "Strategy \d") -replace '.*Strategy (\d \([^)]+\)).*', '$1'
    if (-not $strategy) { $strategy = "N/A" }

    $result = if ($exitCode -eq 0) { "PASS" } elseif ($exitCode -eq 2) { "FLASH" } else { "FAIL" }
    $results += [PSCustomObject]@{
        Scenario = "User typing"
        Required = "May flash"
        Result = $result
        Strategy = $strategy
        ExitCode = $exitCode
    }
    Write-Host "  Result: $result (exit $exitCode)" -ForegroundColor $(if($result -eq "PASS"){"Green"}elseif($result -eq "FLASH"){"Yellow"}else{"Red"})

    Stop-Process $np -ErrorAction SilentlyContinue
} catch {
    Write-Host "  ERROR: $_" -ForegroundColor Red
    $results += [PSCustomObject]@{ Scenario="User typing"; Required="May flash"; Result="ERROR"; Strategy="N/A"; ExitCode=-1 }
}

Start-Sleep -Seconds 1

# --- Scenario 4: Fullscreen app (MAY flash per D-06) ---
Write-Host ""
Write-Host "--- Scenario 4: Fullscreen app ---" -ForegroundColor Yellow
try {
    $np = Start-Process notepad -PassThru
    Start-Sleep -Seconds 2
    $np.WaitForInputIdle() | Out-Null

    # We can't easily force a fullscreen app in automated test,
    # but we can test the focus attempt with console as foreground holder
    # (which approximates the "foreground lock" scenario)
    Write-Host "  Notepad PID: $($np.Id). Simulating foreground-locked scenario."
    Write-Host "  Running focus command..."

    $output = & $ExePath --focus "claude-notify://focus?session=s4&pid=$($np.Id)" 2>&1
    $exitCode = $LASTEXITCODE
    $output | ForEach-Object { Write-Host "  $_" }

    $strategy = ($output | Select-String "Strategy \d") -replace '.*Strategy (\d \([^)]+\)).*', '$1'
    if (-not $strategy) { $strategy = "N/A" }

    $result = if ($exitCode -eq 0) { "PASS" } elseif ($exitCode -eq 2) { "FLASH" } else { "FAIL" }
    $results += [PSCustomObject]@{
        Scenario = "Fullscreen app"
        Required = "May flash"
        Result = $result
        Strategy = $strategy
        ExitCode = $exitCode
    }
    Write-Host "  Result: $result (exit $exitCode)" -ForegroundColor $(if($result -eq "PASS"){"Green"}elseif($result -eq "FLASH"){"Yellow"}else{"Red"})

    Stop-Process $np -ErrorAction SilentlyContinue
} catch {
    Write-Host "  ERROR: $_" -ForegroundColor Red
    $results += [PSCustomObject]@{ Scenario="Fullscreen app"; Required="May flash"; Result="ERROR"; Strategy="N/A"; ExitCode=-1 }
}

# --- Summary ---
Write-Host ""
Write-Host "=== Results Summary ===" -ForegroundColor Cyan
$results | Format-Table -AutoSize

# Check MUST PASS scenarios
$mustPass = $results | Where-Object { $_.Required -eq "MUST PASS" }
$allPassed = ($mustPass | Where-Object { $_.Result -eq "PASS" }).Count -eq $mustPass.Count
if ($allPassed) {
    Write-Host "ALL MUST-PASS SCENARIOS PASSED" -ForegroundColor Green
    exit 0
} else {
    Write-Host "SOME MUST-PASS SCENARIOS FAILED" -ForegroundColor Red
    exit 1
}

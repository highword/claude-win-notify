# Automated focus test - runs scenarios 1-4 against both stacks
# Writes results to spike/focus-results.md
param()

$ErrorActionPreference = "Continue"
$basePath = Split-Path -Parent $PSScriptRoot

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class W32 {
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    public const int SW_MINIMIZE = 6;
    public const int SW_RESTORE = 9;
}
"@

$csharpExe = Join-Path $basePath "spike\csharp\bin\Release\net9.0-windows10.0.19041.0\win-x64\publish\CSharpSpike.exe"
$rustExe = Join-Path $basePath "spike\rust\target\release\rust-spike.exe"

function Test-Focus {
    param([string]$ExePath, [string]$Label, [uint32]$TargetPid, [string]$Scenario)

    Write-Host "  [$Label] Testing scenario: $Scenario against PID $TargetPid"
    $output = & $ExePath --focus "claude-notify://focus?session=$Scenario&pid=$TargetPid" 2>&1
    $exitCode = $LASTEXITCODE

    $strategyLine = $output | Select-String "Strategy \d"
    $strategy = if ($strategyLine) { ($strategyLine -split "Strategy ")[1] -replace ':.*','' } else { "N/A" }
    $resultLine = $output | Select-String "RESULT:"
    $result = if ($exitCode -eq 0) { "PASS" } elseif ($exitCode -eq 2) { "FLASH" } else { "FAIL" }

    $output | ForEach-Object { Write-Host "    $_" }
    Write-Host "    => $result (strategy: $strategy)" -ForegroundColor $(if($result -eq "PASS"){"Green"}elseif($result -eq "FLASH"){"Yellow"}else{"Red"})

    return @{ Result=$result; Strategy=$strategy; ExitCode=$exitCode }
}

# Find test targets - existing processes with visible windows
$targets = Get-Process | Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero -and $_.ProcessName -ne "explorer" } | Select-Object -First 3

if ($targets.Count -lt 2) {
    Write-Host "ERROR: Need at least 2 visible windows for testing" -ForegroundColor Red
    exit 1
}

$target1 = $targets[0]
$target2 = $targets[1]
Write-Host "Using test targets:"
Write-Host "  Target 1: $($target1.ProcessName) (PID $($target1.Id))"
Write-Host "  Target 2: $($target2.ProcessName) (PID $($target2.Id))"
Write-Host ""

$allResults = @{}

foreach ($stack in @(@{Label="csharp"; Exe=$csharpExe}, @{Label="rust"; Exe=$rustExe})) {
    Write-Host "=== Testing $($stack.Label) ===" -ForegroundColor Cyan
    $stackResults = @()

    # Scenario 1: Minimized/idle
    Write-Host "--- Scenario 1: Minimized/idle ---" -ForegroundColor Yellow
    $target1.Refresh()
    [W32]::ShowWindow($target1.MainWindowHandle, [W32]::SW_MINIMIZE) | Out-Null
    Start-Sleep -Milliseconds 500
    $r = Test-Focus -ExePath $stack.Exe -Label $stack.Label -TargetPid $target1.Id -Scenario "s1-minimized"
    $stackResults += @{ Scenario="Minimized/idle"; Required="MUST PASS"; Result=$r.Result; Strategy=$r.Strategy }
    Start-Sleep -Milliseconds 500

    # Scenario 2: Multiple windows - focus non-foreground
    Write-Host "--- Scenario 2: Multiple windows ---" -ForegroundColor Yellow
    # Make target2 foreground first
    $target2.Refresh()
    [W32]::ShowWindow($target2.MainWindowHandle, [W32]::SW_RESTORE) | Out-Null
    Start-Sleep -Milliseconds 500
    # Now focus target1 (not foreground)
    $r = Test-Focus -ExePath $stack.Exe -Label $stack.Label -TargetPid $target1.Id -Scenario "s2-multiple"
    $stackResults += @{ Scenario="Multiple windows"; Required="MUST PASS"; Result=$r.Result; Strategy=$r.Strategy }
    Start-Sleep -Milliseconds 500

    # Scenario 3: User typing (console is foreground)
    Write-Host "--- Scenario 3: User typing in foreground ---" -ForegroundColor Yellow
    # Our console/script is holding foreground
    $r = Test-Focus -ExePath $stack.Exe -Label $stack.Label -TargetPid $target1.Id -Scenario "s3-typing"
    $stackResults += @{ Scenario="User typing"; Required="May flash"; Result=$r.Result; Strategy=$r.Strategy }
    Start-Sleep -Milliseconds 500

    # Scenario 4: Fullscreen (simulated - same as foreground lock)
    Write-Host "--- Scenario 4: Fullscreen (simulated) ---" -ForegroundColor Yellow
    $r = Test-Focus -ExePath $stack.Exe -Label $stack.Label -TargetPid $target2.Id -Scenario "s4-fullscreen"
    $stackResults += @{ Scenario="Fullscreen app"; Required="May flash"; Result=$r.Result; Strategy=$r.Strategy }

    $allResults[$stack.Label] = $stackResults
    Write-Host ""
}

# Output results
Write-Host "=== FINAL RESULTS ===" -ForegroundColor Cyan
foreach ($stack in @("csharp", "rust")) {
    Write-Host ""
    Write-Host "$stack results:" -ForegroundColor Yellow
    foreach ($r in $allResults[$stack]) {
        $color = if($r.Result -eq "PASS"){"Green"}elseif($r.Result -eq "FLASH"){"Yellow"}else{"Red"}
        Write-Host "  $($r.Scenario): $($r.Result) (Strategy: $($r.Strategy))" -ForegroundColor $color
    }
}

param(
    [Parameter(Mandatory=$true)]
    [string]$ExePath,
    [int]$Iterations = 5
)

if (-not (Test-Path $ExePath)) {
    Write-Error "File not found: $ExePath"
    exit 1
}

$resolvedPath = Resolve-Path $ExePath
Write-Host "Measuring cold startup: $resolvedPath ($Iterations iterations)"
Write-Host ""

$times = @()

for ($i = 1; $i -le $Iterations; $i++) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $resolvedPath --version 2>$null | Out-Null
    $sw.Stop()
    $times += $sw.ElapsedMilliseconds
    Write-Host "  Run ${i}: $($sw.ElapsedMilliseconds) ms"
}

Write-Host ""
$avg = ($times | Measure-Object -Average).Average
$min = ($times | Measure-Object -Minimum).Minimum
$max = ($times | Measure-Object -Maximum).Maximum
Write-Host "Results: avg=$([math]::Round($avg,1))ms min=${min}ms max=${max}ms"
Write-Host ""

# Return structured result for script consumers
[PSCustomObject]@{
    ExePath    = $resolvedPath.ToString()
    Iterations = $Iterations
    AvgMs      = [math]::Round($avg, 1)
    MinMs      = $min
    MaxMs      = $max
    AllMs      = $times
}

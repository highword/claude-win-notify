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

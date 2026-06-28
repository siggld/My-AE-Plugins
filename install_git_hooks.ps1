Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = $PSScriptRoot
Push-Location $repoRoot
try {
    git rev-parse --show-toplevel | Out-Null
    git config core.hooksPath .githooks
    powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $repoRoot 'cleanup_git_desktop_ini.ps1') -Quiet
    Write-Host 'Installed core.hooksPath=.githooks'
    Write-Host 'Initial desktop.ini cleanup completed.'
    Write-Host 'If GitHub Desktop was open, restart it once.'
} finally {
    Pop-Location
}

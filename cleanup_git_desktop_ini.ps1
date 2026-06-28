param(
    [switch]$Quiet
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$gitDirRaw = & git rev-parse --git-dir 2>$null
if (-not $gitDirRaw) {
    throw 'Not inside a Git repository.'
}

$gitDir = [System.IO.Path]::GetFullPath((Join-Path (Get-Location) $gitDirRaw.Trim()))

if (-not (Test-Path -LiteralPath $gitDir -PathType Container)) {
    throw "Git dir not found: $gitDir"
}

$targets = Get-ChildItem -LiteralPath $gitDir -Recurse -Force -File -Filter 'desktop.ini'
$removed = 0

foreach ($file in $targets) {
    $full = [System.IO.Path]::GetFullPath($file.FullName)
    if (-not $full.StartsWith($gitDir, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to delete outside .git: $full"
    }
    Remove-Item -LiteralPath $full -Force
    $removed++
}

if (-not $Quiet) {
    if ($removed -gt 0) {
        Write-Host "Removed $removed desktop.ini file(s) from $gitDir"
    } else {
        Write-Host "No desktop.ini found under $gitDir"
    }
}

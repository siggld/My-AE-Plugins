# AE Cache Cleanup Script

$versions = Get-ChildItem -Path 'HKCU:\Software\Adobe\After Effects' | Where-Object { $_.PSChildName -match '^\d+\.\d+$' }

foreach ($v in $versions) {
    $cachePath = "HKCU:\Software\Adobe\After Effects\$($v.PSChildName)\PluginCache"
    if (Test-Path $cachePath) {
        Remove-Item -Path $cachePath -Recurse -Force
        Write-Host "Deleted Registry Cache: $cachePath"
    }
}

$mediaCorePath = "C:\Program Files\Adobe\Common\Plug-ins\7.0\MediaCore"
if (Test-Path $mediaCorePath) {
    # Specifically look for any temporary or corrupt files maybe? 
    # For now, let's just list the .aex files modified today as a check
    Write-Host "Recently modified .aex in MediaCore:"
    Get-ChildItem -Path $mediaCorePath -Recurse -Filter "*.aex" | Where-Object { $_.LastWriteTime -ge [DateTime]::Today } | Select-Object FullName, LastWriteTime
}

Write-Host "Cleanup completed."

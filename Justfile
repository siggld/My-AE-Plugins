set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Optional: comma-separated plugin selectors under plugins/
# (e.g. vector-curve-blur, vector_curve_blur, VectorCurveBlur, red-noise).
# If unset or empty, all plugins with a Justfile are built. Set in CI via vars.RELEASE_PLUGINS.

[windows]
build:
	$ErrorActionPreference = "Stop"; \
	$root = "{{justfile_directory()}}"; \
	$justfiles = Get-ChildItem -Path (Join-Path $root "plugins") -Filter Justfile -Recurse; \
	$envFilter = $env:RELEASE_PLUGINS; \
	function Normalize-PluginName([string]$name) { \
	  if (-not $name) { return "" }; \
	  return (($name.ToLowerInvariant()) -replace '[-_\s]', ''); \
	}; \
	if ($envFilter -and $envFilter.Trim()) { \
	  $names = $envFilter.Split(',') | ForEach-Object { Normalize-PluginName ($_.Trim()) } | Where-Object { $_ }; \
	  $justfiles = $justfiles | Where-Object { \
	    $dirName = (Get-Item $_.FullName).Directory.Name; \
	    $names -contains (Normalize-PluginName $dirName) \
	  }; \
	}; \
	if (-not $justfiles) { Write-Error "No plugin Justfiles matched RELEASE_PLUGINS=$envFilter" }; \
	$justfiles | ForEach-Object { just -f $_.FullName build }

[windows]
release:
	$ErrorActionPreference = "Stop"; \
	$root = "{{justfile_directory()}}"; \
	$justfiles = Get-ChildItem -Path (Join-Path $root "plugins") -Filter Justfile -Recurse; \
	$envFilter = $env:RELEASE_PLUGINS; \
	$runId = "pre-fix-1"; \
	$logPath = Join-Path $root "debug-1e940a.log"; \
	#region agent log \
	Add-Content -Path $logPath -Value ((@{sessionId="1e940a";runId=$runId;hypothesisId="H3";location="Justfile:release:windows:init";message="windows release init";data=@{root=$root;pluginsPath=(Join-Path $root "plugins");justfileCount=@($justfiles).Count};timestamp=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()} | ConvertTo-Json -Compress)); \
	#endregion \
	#region agent log \
	Add-Content -Path $logPath -Value ((@{sessionId="1e940a";runId=$runId;hypothesisId="H4";location="Justfile:release:windows:env";message="raw RELEASE_PLUGINS";data=@{envFilterRaw=($envFilter -replace "`r","\\r" -replace "`n","\\n")};timestamp=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()} | ConvertTo-Json -Compress)); \
	#endregion \
	function Normalize-PluginName([string]$name) { \
	  if (-not $name) { return "" }; \
	  return (($name.ToLowerInvariant()) -replace '[-_\s]', ''); \
	}; \
	$allDirNames = $justfiles | ForEach-Object { (Get-Item $_.FullName).Directory.Name }; \
	#region agent log \
	Add-Content -Path $logPath -Value ((@{sessionId="1e940a";runId=$runId;hypothesisId="H1";location="Justfile:release:windows:scan";message="detected plugin justfiles";data=@{dirNames=$allDirNames;normalized=($allDirNames | ForEach-Object { Normalize-PluginName $_ })};timestamp=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()} | ConvertTo-Json -Compress)); \
	#endregion \
	if ($envFilter -and $envFilter.Trim()) { \
	  $names = $envFilter.Split(',') | ForEach-Object { Normalize-PluginName ($_.Trim()) } | Where-Object { $_ }; \
	  #region agent log \
	  Add-Content -Path $logPath -Value ((@{sessionId="1e940a";runId=$runId;hypothesisId="H2";location="Justfile:release:windows:filter";message="normalized filter names";data=@{names=$names};timestamp=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()} | ConvertTo-Json -Compress)); \
	  #endregion \
	  $justfiles = $justfiles | Where-Object { \
	    $dirName = (Get-Item $_.FullName).Directory.Name; \
	    $names -contains (Normalize-PluginName $dirName) \
	  }; \
	}; \
	#region agent log \
	Add-Content -Path $logPath -Value ((@{sessionId="1e940a";runId=$runId;hypothesisId="H1";location="Justfile:release:windows:result";message="post-filter justfiles";data=@{matchedCount=@($justfiles).Count;matchedDirNames=($justfiles | ForEach-Object { (Get-Item $_.FullName).Directory.Name })};timestamp=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()} | ConvertTo-Json -Compress)); \
	#endregion \
	if (-not $justfiles) { Write-Error "No plugin Justfiles matched RELEASE_PLUGINS=$envFilter" }; \
	$justfiles | ForEach-Object { just -f $_.FullName release }

[macos]
build:
	#!/bin/bash
	set -euo pipefail
	root="{{justfile_directory()}}"
	filter_raw="${RELEASE_PLUGINS:-}"
	normalize_plugin_name() {
	  echo "$1" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]_-'
	}
	matched_any=0
	while IFS= read -r -d '' f; do
	  if [[ -n "$(echo "$filter_raw" | tr -d '[:space:]')" ]]; then
	    name="$(basename "$(dirname "$f")")"
	    norm_name="$(normalize_plugin_name "$name")"
	    matched=0
	    IFS=',' read -ra PARTS <<< "$filter_raw" || true
	    for p in "${PARTS[@]}"; do
	      q="$(echo "$p" | xargs)"
	      [[ -z "$q" ]] && continue
	      if [[ "$norm_name" == "$(normalize_plugin_name "$q")" ]]; then matched=1; break; fi
	    done
	    [[ "$matched" -eq 0 ]] && continue
	  fi
	  matched_any=1
	  just -f "$f" build
	done < <(find "$root/plugins" -name Justfile -type f -print0)
	if [[ "$matched_any" -eq 0 ]]; then
	  echo "No plugin Justfiles matched RELEASE_PLUGINS=$filter_raw" >&2
	  exit 1
	fi

[macos]
release:
	#!/bin/bash
	set -euo pipefail
	root="{{justfile_directory()}}"
	filter_raw="${RELEASE_PLUGINS:-}"
	run_id="pre-fix-1"
	log_path="$root/debug-1e940a.log"
	#region agent log
	printf '{"sessionId":"1e940a","runId":"%s","hypothesisId":"H3","location":"Justfile:release:mac:init","message":"mac release init","data":{"root":"%s","pluginsPath":"%s"},"timestamp":%s}\n' "$run_id" "$root" "$root/plugins" "$(date +%s000)" >> "$log_path"
	#endregion
	#region agent log
	printf '{"sessionId":"1e940a","runId":"%s","hypothesisId":"H4","location":"Justfile:release:mac:env","message":"raw RELEASE_PLUGINS","data":{"envFilterRaw":"%s"},"timestamp":%s}\n' "$run_id" "$filter_raw" "$(date +%s000)" >> "$log_path"
	#endregion
	normalize_plugin_name() {
	  echo "$1" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]_-'
	}
	all_dirs="$(find "$root/plugins" -name Justfile -type f -print0 | xargs -0 -I{} dirname "{}" | xargs -I{} basename "{}" 2>/dev/null || true)"
	#region agent log
	printf '{"sessionId":"1e940a","runId":"%s","hypothesisId":"H1","location":"Justfile:release:mac:scan","message":"detected plugin justfiles","data":{"dirNames":"%s"},"timestamp":%s}\n' "$run_id" "$all_dirs" "$(date +%s000)" >> "$log_path"
	#endregion
	matched_any=0
	while IFS= read -r -d '' f; do
	  if [[ -n "$(echo "$filter_raw" | tr -d '[:space:]')" ]]; then
	    name="$(basename "$(dirname "$f")")"
	    norm_name="$(normalize_plugin_name "$name")"
	    matched=0
	    IFS=',' read -ra PARTS <<< "$filter_raw" || true
	    for p in "${PARTS[@]}"; do
	      q="$(echo "$p" | xargs)"
	      [[ -z "$q" ]] && continue
	      if [[ "$norm_name" == "$(normalize_plugin_name "$q")" ]]; then matched=1; break; fi
	    done
	    [[ "$matched" -eq 0 ]] && continue
	  fi
	  matched_any=1
	  just -f "$f" release
	done < <(find "$root/plugins" -name Justfile -type f -print0)
	#region agent log
	printf '{"sessionId":"1e940a","runId":"%s","hypothesisId":"H1","location":"Justfile:release:mac:result","message":"post-filter result","data":{"matchedAny":%s},"timestamp":%s}\n' "$run_id" "$matched_any" "$(date +%s000)" >> "$log_path"
	#endregion
	if [[ "$matched_any" -eq 0 ]]; then
	  echo "No plugin Justfiles matched RELEASE_PLUGINS=$filter_raw" >&2
	  exit 1
	fi

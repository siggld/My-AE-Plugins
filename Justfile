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
	  just -f "$f" release
	done < <(find "$root/plugins" -name Justfile -type f -print0)
	if [[ "$matched_any" -eq 0 ]]; then
	  echo "No plugin Justfiles matched RELEASE_PLUGINS=$filter_raw" >&2
	  exit 1
	fi

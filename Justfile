set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Optional: comma-separated plugin folder names under plugins/ (e.g. vector-curve-blur,red-noise).
# If unset or empty, all plugins with a Justfile are built. Set in CI via vars.RELEASE_PLUGINS.

[windows]
build:
	$ErrorActionPreference = "Stop"; \
	$root = "{{justfile_directory()}}"; \
	$justfiles = Get-ChildItem -Path (Join-Path $root "plugins") -Filter Justfile -Recurse; \
	$envFilter = $env:RELEASE_PLUGINS; \
	if ($envFilter -and $envFilter.Trim()) { \
	  $names = $envFilter.Split(',') | ForEach-Object { $_.Trim() } | Where-Object { $_ }; \
	  $justfiles = $justfiles | Where-Object { $names -contains (Get-Item $_.FullName).Directory.Name }; \
	}; \
	if (-not $justfiles) { Write-Error "No plugin Justfiles matched RELEASE_PLUGINS=$envFilter" }; \
	$justfiles | ForEach-Object { just -f $_.FullName build }

[windows]
release:
	$ErrorActionPreference = "Stop"; \
	$root = "{{justfile_directory()}}"; \
	$justfiles = Get-ChildItem -Path (Join-Path $root "plugins") -Filter Justfile -Recurse; \
	$envFilter = $env:RELEASE_PLUGINS; \
	if ($envFilter -and $envFilter.Trim()) { \
	  $names = $envFilter.Split(',') | ForEach-Object { $_.Trim() } | Where-Object { $_ }; \
	  $justfiles = $justfiles | Where-Object { $names -contains (Get-Item $_.FullName).Directory.Name }; \
	}; \
	if (-not $justfiles) { Write-Error "No plugin Justfiles matched RELEASE_PLUGINS=$envFilter" }; \
	$justfiles | ForEach-Object { just -f $_.FullName release }

[macos]
build:
	#!/bin/bash
	set -euo pipefail
	root="{{justfile_directory()}}"
	filter_raw="${RELEASE_PLUGINS:-}"
	while IFS= read -r -d '' f; do
	  if [[ -n "$(echo "$filter_raw" | tr -d '[:space:]')" ]]; then
	    name="$(basename "$(dirname "$f")")"
	    matched=0
	    IFS=',' read -ra PARTS <<< "$filter_raw" || true
	    for p in "${PARTS[@]}"; do
	      q="$(echo "$p" | xargs)"
	      [[ -z "$q" ]] && continue
	      if [[ "$name" == "$q" ]]; then matched=1; break; fi
	    done
	    [[ "$matched" -eq 0 ]] && continue
	  fi
	  just -f "$f" build
	done < <(find "$root/plugins" -name Justfile -type f -print0)

[macos]
release:
	#!/bin/bash
	set -euo pipefail
	root="{{justfile_directory()}}"
	filter_raw="${RELEASE_PLUGINS:-}"
	while IFS= read -r -d '' f; do
	  if [[ -n "$(echo "$filter_raw" | tr -d '[:space:]')" ]]; then
	    name="$(basename "$(dirname "$f")")"
	    matched=0
	    IFS=',' read -ra PARTS <<< "$filter_raw" || true
	    for p in "${PARTS[@]}"; do
	      q="$(echo "$p" | xargs)"
	      [[ -z "$q" ]] && continue
	      if [[ "$name" == "$q" ]]; then matched=1; break; fi
	    done
	    [[ "$matched" -eq 0 ]] && continue
	  fi
	  just -f "$f" release
	done < <(find "$root/plugins" -name Justfile -type f -print0)

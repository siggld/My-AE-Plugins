set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

WorkspaceRoot := justfile_directory() + "/../.."
TargetDir := env_var_or_default("CARGO_TARGET_DIR", WorkspaceRoot + "/target")

[windows]
build:
    cargo build -p {{CrateName}}
    if (-not $env:NO_INSTALL) { \
        Start-Process PowerShell -Verb runAs -ArgumentList "-command Set-Location '{{source_directory()}}'; Copy-Item -Force '{{TargetDir}}\debug\{{BinaryName}}.dll' 'C:\Program Files\Adobe\Common\Plug-ins\7.0\MediaCore\{{PluginName}}.aex'" \
    }

[windows]
release:
    #region agent log
    Add-Content -Path '{{justfile_directory()}}\debug-1e940a.log' -Value ((@{sessionId="1e940a";runId="pre-fix-2";hypothesisId="H6";location="AdobePlugin.just:windows:release:start";message="plugin release start";data=@{crateName="{{CrateName}}";binaryName="{{BinaryName}}";pluginName="{{PluginName}}";targetDir="{{TargetDir}}"};timestamp=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()} | ConvertTo-Json -Compress))
    #endregion
    cargo build -p {{CrateName}} --release
    #region agent log
    Add-Content -Path '{{justfile_directory()}}\debug-1e940a.log' -Value ((@{sessionId="1e940a";runId="pre-fix-2";hypothesisId="H7";location="AdobePlugin.just:windows:release:post-build";message="post cargo build";data=@{dllExists=(Test-Path '{{TargetDir}}\release\{{BinaryName}}.dll');dllPath='{{TargetDir}}\release\{{BinaryName}}.dll'};timestamp=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()} | ConvertTo-Json -Compress))
    #endregion
    Copy-Item -Force '{{TargetDir}}\release\{{BinaryName}}.dll' '{{TargetDir}}\release\{{PluginName}}.aex'
    #region agent log
    Add-Content -Path '{{justfile_directory()}}\debug-1e940a.log' -Value ((@{sessionId="1e940a";runId="pre-fix-2";hypothesisId="H8";location="AdobePlugin.just:windows:release:post-copy";message="post dll->aex copy";data=@{aexExists=(Test-Path '{{TargetDir}}\release\{{PluginName}}.aex');aexPath='{{TargetDir}}\release\{{PluginName}}.aex'};timestamp=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()} | ConvertTo-Json -Compress))
    #endregion
    if (-not $env:NO_INSTALL) { \
        Start-Process PowerShell -Verb runAs -ArgumentList "-command Set-Location '{{source_directory()}}'; Copy-Item -Force '{{TargetDir}}\release\{{BinaryName}}.dll' 'C:\Program Files\Adobe\Common\Plug-ins\7.0\MediaCore\{{PluginName}}.aex'" \
    }

[macos]
build:
    cargo build -p {{CrateName}}
    just -f {{justfile()}} create_bundle debug {{TargetDir}}

[macos]
release:
    cargo build -p {{CrateName}} --release
    just -f {{justfile()}} create_bundle release {{TargetDir}}

[macos]
create_bundle profile TargetDir:
    #!/bin/bash
    set -e
    echo "Creating plugin bundle"
    lib_name="{{BinaryName}}"
    lib_name="${lib_name//-/_}"
    rm -Rf {{TargetDir}}/{{profile}}/{{PluginName}}.plugin
    mkdir -p {{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/Resources
    mkdir -p {{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/MacOS

    if [ "{{profile}}" == "release" ]; then
        # Build universal binary
        rustup target add aarch64-apple-darwin
        rustup target add x86_64-apple-darwin

        cargo build -p {{CrateName}} --release --target x86_64-apple-darwin
        cargo build -p {{CrateName}} --release --target aarch64-apple-darwin

        cp {{TargetDir}}/x86_64-apple-darwin/release/{{BinaryName}}.rsrc {{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/Resources/{{PluginName}}.rsrc
        cp {{TargetDir}}/x86_64-apple-darwin/release/{{BinaryName}}_PkgInfo {{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/PkgInfo
        cp {{TargetDir}}/x86_64-apple-darwin/release/{{BinaryName}}_Info.plist {{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/Info.plist
        lipo {{TargetDir}}/{x86_64,aarch64}-apple-darwin/release/lib${lib_name}.dylib -create -output {{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/MacOS/{{PluginName}}.dylib
        mv {{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/MacOS/{{PluginName}}.dylib {{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/MacOS/{{PluginName}}
    else
        cp {{TargetDir}}/{{profile}}/{{BinaryName}}.rsrc {{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/Resources/{{PluginName}}.rsrc
        cp {{TargetDir}}/{{profile}}/{{BinaryName}}_PkgInfo {{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/PkgInfo
        cp {{TargetDir}}/{{profile}}/{{BinaryName}}_Info.plist {{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/Info.plist
        cp {{TargetDir}}/{{profile}}/lib${lib_name}.dylib {{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/MacOS/{{PluginName}}
    fi
    /usr/libexec/PlistBuddy -c 'Set :CFBundleIdentifier "{{BundleIdentifier}}"' {{TargetDir}}/{{profile}}/{{PluginName}}.plugin/Contents/Info.plist

    DEV_CERT=$(security find-identity -v -p codesigning | grep -m 1 "Apple Development" | awk -F ' ' '{print $2}')

    # Perform signing - as of AE and PR 25.2 all macos plugins require a signature or else
    # they will fail to load with error code "2685337601" - see your "Plugin Loading.log" for details
    SIGN_TARGET="{{TargetDir}}/{{profile}}/{{PluginName}}.plugin"
    SIGN_OK=0
    if [ -n "$DEV_CERT" ]; then
       # Developer cert: timestamped hardened signature
       if codesign --force --deep --options runtime --timestamp --strict --sign "$DEV_CERT" "$SIGN_TARGET"; then
           SIGN_OK=1
       else
           echo "Warning: Developer certificate signing failed; attempting ad-hoc signing."
       fi
    fi
    if [ "$SIGN_OK" -ne 1 ]; then
       # Ad-hoc signature: keep command minimal for CI compatibility
       if codesign --force --deep --sign - "$SIGN_TARGET"; then
           SIGN_OK=1
           echo "Note: Using ad-hoc signature. For distribution, a valid Apple Developer certificate is recommended."
       fi
    fi
    if [ "$SIGN_OK" -ne 1 ]; then
       if [ -n "${CI:-}" ] || [ -n "${GITHUB_ACTIONS:-}" ] || [ -n "${NO_INSTALL:-}" ]; then
           echo "Warning: codesign failed in CI/non-install mode; continuing packaging."
       else
           echo "Error: codesign failed."
           exit 1
       fi
    fi

    # Install
    if [ -z "$NO_INSTALL" ]; then
        echo "Installing to /Library/Application Support/Adobe/Common/Plug-ins/7.0/MediaCore/"
        sudo cp -rf "{{TargetDir}}/{{profile}}/{{PluginName}}.plugin" "/Library/Application Support/Adobe/Common/Plug-ins/7.0/MediaCore/"
    fi

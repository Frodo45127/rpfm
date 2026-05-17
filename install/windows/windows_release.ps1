# Remember to execute this from the root of RPFM's git folder.
Set-Variable -Name "RPFM_PATH" -Value ((Get-Location).path)
Set-Variable -Name "RPFM_VERSION" -Value (Select-String -Path Cargo.toml -Pattern '^version = \"(.*)\"$').Matches.Groups[1].value

# Load Sentry secrets from the repo-root .env (gitignored). Loaded before
# `cargo build` so `option_env!` in rpfm_ui/rpfm_server picks up the DSNs and
# bakes them into the binaries; reused below for the symbol upload step.
if (Test-Path "$RPFM_PATH\.env") {
    Get-Content "$RPFM_PATH\.env" | ForEach-Object {
        if ($_ -match '^\s*([^#=]+)\s*=\s*(.+)\s*$') {
            [Environment]::SetEnvironmentVariable($matches[1].Trim(), $matches[2].Trim(), "Process")
        }
    }
} else {
    Write-Host "Warning: .env not found at repo root; binaries will be built without Sentry DSNs."
}

# Clean qt_rpfm_extensions so stale artifacts are not reused.
if (Test-Path "$RPFM_PATH\3rdparty\src\qt_rpfm_extensions\Makefile") {
    Push-Location "$RPFM_PATH\3rdparty\src\qt_rpfm_extensions"
    nmake clean 2>$null
    Pop-Location
}

# Build the tools.
cargo build --release --bin rpfm_server --bin rpfm_ui

# Upload debug symbols to Sentry so stack traces in crash reports get
# resolved to function names and source lines. UI and server live in
# different Sentry projects, so each binary uploads under its own slug.
if ($env:SENTRY_AUTH_TOKEN) {
    if ($env:SENTRY_ORG -and $env:RPFM_UI_SENTRY_PROJECT) {
        Write-Host "Uploading rpfm_ui debug symbols to Sentry..."
        .\sentry-cli.exe debug-files upload `
            --org $env:SENTRY_ORG `
            --project $env:RPFM_UI_SENTRY_PROJECT `
            --include-sources `
            $RPFM_PATH\target\release\rpfm_ui.exe `
            $RPFM_PATH\target\release\rpfm_ui.pdb
    } else {
        Write-Host "Warning: SENTRY_ORG / RPFM_UI_SENTRY_PROJECT not set, skipping rpfm_ui symbol upload."
    }

    if ($env:SENTRY_ORG -and $env:RPFM_SERVER_SENTRY_PROJECT) {
        Write-Host "Uploading rpfm_server debug symbols to Sentry..."
        .\sentry-cli.exe debug-files upload `
            --org $env:SENTRY_ORG `
            --project $env:RPFM_SERVER_SENTRY_PROJECT `
            --include-sources `
            $RPFM_PATH\target\release\rpfm_server.exe `
            $RPFM_PATH\target\release\rpfm_server.pdb
    } else {
        Write-Host "Warning: SENTRY_ORG / RPFM_SERVER_SENTRY_PROJECT not set, skipping rpfm_server symbol upload."
    }
} else {
    Write-Host "Warning: SENTRY_AUTH_TOKEN not set, skipping symbol upload."
}

# Prepare the paths for the deployment.
Set-Location I:\
Remove-Item -r -fo I:\deploy
mkdir deploy
Set-Location deploy
mkdir rpfm-release-assets
Set-Location rpfm-release-assets

# Here we copy all the dlls required by RPFM. Otherwise we'll have to manually update them on every freaking release, and for 2 months that's been a royal PITA.
mkdir designer
Copy-Item C:\CraftRoot\plugins\designer\*.dll I:\deploy\rpfm-release-assets\designer\

mkdir iconengines
Copy-Item C:\CraftRoot\plugins\kiconthemes6\iconengines\KIconEnginePlugin.dll I:\deploy\rpfm-release-assets\iconengines\
Copy-Item C:\CraftRoot\plugins\iconengines\qsvgicon.dll I:\deploy\rpfm-release-assets\iconengines\

mkdir imageformats
Copy-Item C:\CraftRoot\plugins\imageformats\*.dll I:\deploy\rpfm-release-assets\imageformats\
Copy-Item $RPFM_PATH\3rdparty\builds\qdds.dll I:\deploy\rpfm-release-assets\imageformats\

# TODO: Check if we have to copy the kf6 folder.

mkdir platforms
Copy-Item C:\CraftRoot\plugins\platforms\qwindows.dll I:\deploy\rpfm-release-assets\platforms\

mkdir styles
Copy-Item C:\CraftRoot\plugins\styles\qmodernwindowsstyle.dll I:\deploy\rpfm-release-assets\styles\

Copy-Item C:\CraftRoot\bin\dbus-1-3.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\editorconfig.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\freetype.dll I:\deploy\rpfm-release-assets\

Copy-Item C:\CraftRoot\bin\harfbuzz.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\iconv.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\icudt??.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\icuin??.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\icuuc??.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\intl-8.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\jpeg62.dll I:\deploy\rpfm-release-assets\

Copy-Item C:\CraftRoot\bin\KF6Archive.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6BreezeIcons.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6Codecs.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6Completion.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6ConfigCore.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6ConfigGui.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6ConfigWidgets.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6CoreAddons.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6Crash.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6DBusAddons.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6GuiAddons.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6I18n.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6IconWidgets.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6IconThemes.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6ItemViews.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6JobWidgets.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6KIOCore.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6KIOGui.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6KIOWidgets.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6Notifications.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6Parts.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6Service.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6Solid.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6SonnetCore.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6SonnetUi.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6SyntaxHighlighting.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6TextEditor.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6TextWidgets.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6WidgetsAddons.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6WindowSystem.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6XmlGui.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF6ColorScheme.dll I:\deploy\rpfm-release-assets\

Copy-Item C:\CraftRoot\bin\libcrypto*.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\liblzma.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\libpng16.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\libssl*.dll I:\deploy\rpfm-release-assets\

Copy-Item C:\CraftRoot\bin\msvcp140.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\msvcp140_1.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\msvcp140_2.dll I:\deploy\rpfm-release-assets\

Copy-Item C:\CraftRoot\bin\pcre2-8.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\pcre2-16.dll I:\deploy\rpfm-release-assets\

Copy-Item C:\CraftRoot\bin\Qt6Core.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt6DBus.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt6Gui.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt6Network.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt6PrintSupport.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt6Qml.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt6Svg.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt6TextToSpeech.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt6Widgets.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt6Xml.dll I:\deploy\rpfm-release-assets\

Copy-Item C:\CraftRoot\bin\vcruntime140.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\vcruntime140_1.dll I:\deploy\rpfm-release-assets\

Copy-Item C:\CraftRoot\bin\b2-1.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\bz2.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\brotlicommon.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\brotlidec.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\tiff.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\zlib1.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\zstd.dll I:\deploy\rpfm-release-assets\

Copy-Item $RPFM_PATH/target/release/rpfm_server.exe I:\deploy\rpfm-release-assets
Copy-Item $RPFM_PATH/target/release/rpfm_server.pdb I:\deploy\rpfm-release-assets
Copy-Item $RPFM_PATH/target/release/rpfm_ui.exe I:\deploy\rpfm-release-assets
Copy-Item $RPFM_PATH/target/release/rpfm_ui.pdb I:\deploy\rpfm-release-assets
mkdir icons
mkdir locale
mkdir ui
Copy-Item $RPFM_PATH/LICENSE I:\deploy\rpfm-release-assets
Copy-Item $RPFM_PATH/CHANGELOG.md I:\deploy\rpfm-release-assets
Copy-Item $RPFM_PATH/CHANGELOG.md I:\deploy\rpfm-release-assets\Changelog.txt
Copy-Item $RPFM_PATH/icons/* I:\deploy\rpfm-release-assets\icons\
Copy-Item $RPFM_PATH/locale/* I:\deploy\rpfm-release-assets\locale\
Copy-Item $RPFM_PATH/rpfm_ui/ui_templates/* I:\deploy\rpfm-release-assets\ui\

# These assets are for the model renderer.
# mkdir assets
# Copy-Item -R $RPFM_PATH/assets/* I:\deploy\rpfm-release-assets\assets\

# Execute windeployqt6 to add missing translations and the vcredist if needed.
windeployqt6 rpfm_ui.exe

# Remove extra files that are not really needed for execution.
Remove-Item -fo I:\deploy\rpfm-release-assets\vc_redist.x64.exe

Set-Location I:\deploy
7z a rpfm-v$RPFM_VERSION-x86_64-pc-windows-msvc.zip .\rpfm-release-assets\**

# Move back to the original folder.
Set-Location $RPFM_PATH

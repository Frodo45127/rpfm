# Remember to execute this from the root of RPFM's git folder.
Set-Variable -Name "RPFM_PATH" -Value ((Get-Location).path)
Set-Variable -Name "RPFM_VERSION" -Value (Select-String -Path Cargo.toml -Pattern '^version = \"(.*)\"$').Matches.Groups[1].value

# Build the tools.
cargo build --release --bin rpfm_cli
cargo build --release --features "enable_tools" --bin rpfm_ui

# Prepare the paths for the deployment.
Set-Location I:\
Remove-Item -r -fo I:\deploy
mkdir deploy
Set-Location deploy
mkdir rpfm-release-assets
Set-Location rpfm-release-assets

# Copy Breeze icons into the release.
mkdir -p data/icons
Copy-Item "C:\CraftRoot\bin\data\icons\breeze" "I:\deploy\rpfm-release-assets\data\icons\" -recurse
Copy-Item "C:\CraftRoot\bin\data\icons\breeze-dark" "I:\deploy\rpfm-release-assets\data\icons\" -recurse

# Here we copy all the dlls required by RPFM. Otherwise we'll have to manually update them on every freaking release, and for 2 months that's been a royal PITA.
mkdir designer
Copy-Item C:\CraftRoot\plugins\designer\*.dll I:\deploy\rpfm-release-assets\designer\

mkdir iconengines
Copy-Item C:\CraftRoot\plugins\iconengines\KIconEnginePlugin.dll I:\deploy\rpfm-release-assets\iconengines\
Copy-Item C:\CraftRoot\plugins\iconengines\qsvgicon.dll I:\deploy\rpfm-release-assets\iconengines\

mkdir imageformats
Copy-Item C:\CraftRoot\plugins\imageformats\*.dll I:\deploy\rpfm-release-assets\imageformats\
Copy-Item $RPFM_PATH\3rdparty\builds\qdds.dll I:\deploy\rpfm-release-assets\imageformats\

# TODO: Check if we have to copy the kf5 folder.

mkdir platforms
Copy-Item C:\CraftRoot\plugins\platforms\qwindows.dll I:\deploy\rpfm-release-assets\platforms\

mkdir styles
Copy-Item C:\CraftRoot\plugins\styles\qwindowsvistastyle.dll I:\deploy\rpfm-release-assets\styles\

Copy-Item C:\CraftRoot\bin\d3dcompiler_47.dll I:\deploy\rpfm-release-assets\
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

Copy-Item C:\CraftRoot\bin\KF5Archive.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5Codecs.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5Completion.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5ConfigCore.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5ConfigGui.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5ConfigWidgets.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5CoreAddons.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5Crash.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5DBusAddons.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5GuiAddons.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5I18n.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5IconThemes.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5ItemViews.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5JobWidgets.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5KIOCore.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5KIOGui.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5KIOWidgets.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5Parts.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5Service.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5Solid.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5SonnetCore.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5SonnetUi.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5SyntaxHighlighting.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5TextEditor.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5TextWidgets.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5WidgetsAddons.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5WindowSystem.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\KF5XmlGui.dll I:\deploy\rpfm-release-assets\

Copy-Item C:\CraftRoot\bin\libbzip2.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\libcrypto*.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\libEGL.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\libGLESV2.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\liblzma.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\libpng16.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\libssl*.dll I:\deploy\rpfm-release-assets\

Copy-Item C:\CraftRoot\bin\msvcp140.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\msvcp140_1.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\msvcp140_2.dll I:\deploy\rpfm-release-assets\

Copy-Item C:\CraftRoot\bin\pcre2-8.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\pcre2-16.dll I:\deploy\rpfm-release-assets\

Copy-Item C:\CraftRoot\bin\Qt5Core.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt5DBus.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt5Gui.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt5Network.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt5PrintSupport.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt5Qml.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt5Svg.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt5TextToSpeech.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt5Widgets.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\Qt5Xml.dll I:\deploy\rpfm-release-assets\

Copy-Item C:\CraftRoot\bin\vcruntime140.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\vcruntime140_1.dll I:\deploy\rpfm-release-assets\

Copy-Item C:\CraftRoot\bin\tiff.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\zlib1.dll I:\deploy\rpfm-release-assets\
Copy-Item C:\CraftRoot\bin\zstd.dll I:\deploy\rpfm-release-assets\

Copy-Item $RPFM_PATH/target/release/rpfm_ui.exe I:\deploy\rpfm-release-assets
Copy-Item $RPFM_PATH/target/release/rpfm_ui.pdb I:\deploy\rpfm-release-assets
Copy-Item $RPFM_PATH/target/release/rpfm_cli.exe I:\deploy\rpfm-release-assets
Copy-Item $RPFM_PATH/target/release/rpfm_cli.pdb I:\deploy\rpfm-release-assets

mkdir icons
mkdir locale
mkdir ui
Copy-Item $RPFM_PATH/LICENSE I:\deploy\rpfm-release-assets
Copy-Item $RPFM_PATH/Changelog.md I:\deploy\rpfm-release-assets
Copy-Item $RPFM_PATH/Changelog.md I:\deploy\rpfm-release-assets\Changelog.txt
Copy-Item $RPFM_PATH/dark-theme.qss I:\deploy\rpfm-release-assets
Copy-Item $RPFM_PATH/icons/* I:\deploy\rpfm-release-assets\icons\
Copy-Item $RPFM_PATH/locale/* I:\deploy\rpfm-release-assets\locale\
Copy-Item $RPFM_PATH/rpfm_ui/ui_templates/* I:\deploy\rpfm-release-assets\ui\

# These assets are for the model renderer.
# mkdir assets
# Copy-Item -R $RPFM_PATH/assets/* I:\deploy\rpfm-release-assets\assets\

# Execute windeployqt to add missing translations and the vcredist if needed.
windeployqt rpfm_ui.exe

# Remove extra files that are not really needed for execution.
Remove-Item -fo I:\deploy\rpfm-release-assets\vc_redist.x64.exe
Remove-Item -fo I:\deploy\rpfm-release-assets\icons\breeze-icons.rcc
Remove-Item -fo I:\deploy\rpfm-release-assets\icons\breeze-icons-dark.rcc

Set-Location I:\deploy
7z a rpfm-v$RPFM_VERSION-x86_64-pc-windows-msvc.zip .\rpfm-release-assets\**

# Move back to the original folder.
Set-Location $RPFM_PATH

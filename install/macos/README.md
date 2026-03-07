# macOS Build Notes

There is no automated macOS build or packaging script yet. This document describes what would be needed to build RPFM on macOS.

## Prerequisites

- Rust toolchain (stable, >= 1.81)
- Qt 5 and KDE Frameworks 5 (via Homebrew: `brew install qt@5 kf5-completion kf5-iconthemes kf5-ktexteditor kf5-kxmlgui kf5-kwidgetsaddons`)
- CMake (`brew install cmake`)
- GNU Make (`brew install make`, provides `gmake`)

## Building

From the repository root:

```bash
cargo build --release --bin rpfm_server
cargo build --release --features "enable_tools" --bin rpfm_ui
```

The custom Qt extensions library (`3rdparty/src/qt_rpfm_extensions`) is compiled automatically by `rpfm_ui/build.rs` using `gmake` on macOS.

## Packaging Considerations

- macOS applications are typically distributed as `.app` bundles inside `.dmg` disk images
- An `.app` bundle requires a specific directory structure (`Contents/MacOS/`, `Contents/Resources/`, `Contents/Frameworks/`, `Info.plist`)
- Qt dependencies would need to be bundled using `macdeployqt`
- Code signing and notarization are required for distribution outside the App Store
- No CI runner is currently configured for macOS builds

# Build & Release Packaging

This directory contains all platform-specific build scripts, packaging manifests, and release tooling for RPFM.

## Directory Structure

```
install/
├── linux/                     Linux packaging
│   ├── linux_release.sh           Build script: compile, collect assets, create tar.zst
│   ├── linux_flatpak_release.sh   Build script: regenerate sources, build & install Flatpak
│   ├── arch/                  Arch Linux AUR packages
│   │   ├── rpfm-bin/          Precompiled binary package (downloads from GitHub releases)
│   │   ├── rpfm-git/          Source build package (clones and compiles from git)
│   │   └── rpfm.desktop       Desktop entry for application menu integration
│   └── flatpak/               Flatpak containerized build
│       ├── com.github.frodo45127.rpfm.yaml          Flatpak manifest
│       ├── com.github.frodo45127.rpfm.desktop       Desktop entry for Flatpak
│       ├── com.github.frodo45127.rpfm.metainfo.xml  AppStream metadata (license, version)
│       ├── cargo-sources.json                       Vendored dependency manifest
│       └── README.md                                Flatpak-specific build instructions
├── macos/                     macOS (placeholder — see macos/README.md)
│   └── README.md              Build notes and packaging considerations
└── windows/                   Windows packaging
    ├── windows_release.ps1    Build script: compile, collect DLLs/assets, create zip
    └── post_release.ps1       Publish crates to crates.io after release
```

## Prerequisites

### All Platforms

- Rust toolchain (stable, >= 1.81)
- CMake

### Linux

- Qt 5, KDE Frameworks 5 (KCompletion, KIconThemes, KTextEditor, KXmlGui, KWidgetsAddons)
- Breeze icons
- libgit2, xz, p7zip
- zstd (for tar.zst archive creation)
- make

### Windows

- MSVC toolchain (Visual Studio Build Tools)
- Qt 5 + KDE Frameworks 5 via [Craft](https://community.kde.org/Craft)
- 7-Zip (for archive creation)

### macOS

See [macos/README.md](macos/README.md) for details. No automated build script exists yet.

## Feature Flags

Official release builds use these cargo feature flags:

| Binary | Features |
|---|---|
| `rpfm_server` | (default) |
| `rpfm_ui` | `enable_tools` |

## Building a Release

### Linux

```bash
./install/linux/linux_release.sh
```

This compiles all binaries, collects assets into an FHS-compliant structure, and creates a `rpfm-v<VERSION>-x86_64-unknown-linux-gnu.tar.zst` archive in the repository root.

### Windows

```powershell
.\install\windows\windows_release.ps1
```

This compiles all binaries, collects Qt/KDE DLLs from the Craft installation, and creates a zip archive. Must be run from the repository root with Craft environment available.

### Arch Linux (AUR)

```bash
# Precompiled binary package (from GitHub release):
cd install/linux/arch/rpfm-bin && makepkg -si

# Source build (from git):
cd install/linux/arch/rpfm-git && makepkg -si
```

### Flatpak

```bash
./install/linux/linux_flatpak_release.sh
```

This regenerates `cargo-sources.json` from `Cargo.lock`, then builds and installs the Flatpak locally. Use `--skip-cargo-sources` to skip regeneration if `Cargo.lock` hasn't changed. See [linux/flatpak/README.md](linux/flatpak/README.md) for prerequisites and troubleshooting.

## CI/CD

- **`.github/workflows/test.yml`** — Runs on push/PR to master/develop: builds libs and runs tests
- **`.github/workflows/post_release.yaml`** — Triggered when a release is published: publishes the `rpfm-bin` AUR package for stable releases

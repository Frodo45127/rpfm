# install

Per-platform build scripts, packaging manifests and release tooling for ***Rusted PackFile Manager***.

> For user-facing project info (installation, building instructions, FAQ, contributing), see the [workspace README](../README.md) and the [manual][manual].
>
> This README targets maintainers cutting releases or working on the packaging itself.

[manual]: https://frodo45127.github.io/rpfm/manual/

## Layout

```
install/
├── linux/                            Linux packaging
│   ├── linux_release.sh                  Build: compile, collect assets, create tar.zst
│   ├── linux_flatpak_release.sh          Build: regenerate sources, build & install Flatpak
│   ├── arch/                             Arch Linux AUR packages
│   │   ├── rpfm-bin/                     Precompiled binary package (downloads from GitHub releases)
│   │   ├── rpfm-git/                     Source build package (clones and compiles from git)
│   │   └── rpfm.desktop                  Desktop entry for application menu integration
│   └── flatpak/                          Flatpak containerized build
│       ├── com.github.frodo45127.rpfm.yaml          Flatpak manifest
│       ├── com.github.frodo45127.rpfm.desktop       Desktop entry for Flatpak
│       ├── com.github.frodo45127.rpfm.metainfo.xml  AppStream metadata (license, version)
│       ├── cargo-sources.json                       Vendored dependency manifest
│       └── README.md                                Flatpak-specific build notes
├── macos/                            macOS placeholder
│   └── README.md                         Build notes and packaging considerations
└── windows/                          Windows packaging
    ├── windows_release.ps1               Build: compile, collect DLLs/assets, create zip
    └── post_release.ps1                  Publish crates to crates.io after release
```

## Prerequisites

### All platforms

- Rust toolchain (stable, ≥ 1.81).
- CMake.

### Linux

- Qt6 + KDE Frameworks 6 (KCompletion, KIconThemes, KTextEditor, KXmlGui, KWidgetsAddons).
- Breeze icons.
- libgit2, xz, p7zip.
- zstd (used for the `tar.zst` archive).
- make.

### Windows

- MSVC toolchain (Visual Studio Build Tools).
- Qt6 + KDE Frameworks 6 via [Craft](https://community.kde.org/Craft).
- 7-Zip (used for the zip archive).

### macOS

See [macos/README.md](macos/README.md). No automated build script exists yet.

## Release feature flags

Official release builds use these cargo feature flags:

| Binary        | Features        |
|---------------|-----------------|
| `rpfm_server` | (defaults)      |
| `rpfm_ui`     | `enable_tools`  |

The release scripts build both binaries in a single invocation (`cargo build --release --bin rpfm_server --bin rpfm_ui`) so the workspace resolves once and shared crates are compiled with a unified feature set. `enable_tools` is already on rpfm_ui's default feature list, so no explicit `--features` flag is needed; keep the scripts in sync if the feature set changes.

## Building a release

### Linux (tar.zst)

```bash
./install/linux/linux_release.sh
```

Compiles every binary, collects assets into an FHS-compliant layout, and produces `rpfm-v<VERSION>-x86_64-unknown-linux-gnu.tar.zst` in the repository root.

### Windows (zip)

```powershell
.\install\windows\windows_release.ps1
```

Compiles every binary, pulls Qt and KDE DLLs from the Craft prefix, and produces a zip archive. Run from the repository root with the Craft environment loaded.

### Arch Linux (AUR)

```bash
# Precompiled binary package (from a GitHub release):
cd install/linux/arch/rpfm-bin && makepkg -si

# Source build (from git):
cd install/linux/arch/rpfm-git && makepkg -si
```

### Flatpak

```bash
./install/linux/linux_flatpak_release.sh
```

Regenerates `cargo-sources.json` from `Cargo.lock`, then builds and installs the Flatpak locally. Pass `--skip-cargo-sources` to skip regeneration when `Cargo.lock` is unchanged. See [linux/flatpak/README.md](linux/flatpak/README.md) for prerequisites and troubleshooting.

## CI/CD

| Workflow                                | Trigger                            | Purpose                                                                  |
|-----------------------------------------|------------------------------------|--------------------------------------------------------------------------|
| `.github/workflows/test.yml`            | push / PR to `master` or `develop` | Builds the libraries and runs the test suite.                            |
| `.github/workflows/deploy_docs.yaml`    | push to `develop` / manual         | Builds the mdBook manual and deploys it to GitHub Pages.                 |
| `.github/workflows/post_release.yaml`   | release published                  | For stable releases, publishes the `rpfm-bin` AUR package.               |

## License

This project is licensed under the MIT License — see the [LICENSE](../LICENSE) file for details.

## Support

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

[Patreon]: https://www.patreon.com/RPFM

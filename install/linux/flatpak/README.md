# How to build the RPFM Flatpak

## Prerequisites

1.  **Install Flatpak and Flatpak-builder**:
    Follow the instructions for your distribution on the [Flatpak website](https://flatpak.org/setup/).

2.  **Install the KDE SDK, Platform, and Rust extension**:
    ```bash
    flatpak install flathub org.kde.Platform//5.15-25.08
    flatpak install flathub org.kde.Sdk//5.15-25.08
    flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//25.08
    ```

## Quick Build

From the repository root, run the build script:

```bash
./install/linux/linux_flatpak_release.sh
```

Use `--skip-cargo-sources` to skip regenerating `cargo-sources.json` if `Cargo.lock` hasn't changed.

## Manual Building

1.  **Generate `cargo-sources.json`** (if Cargo.lock has changed):

    You will need `flatpak-cargo-generator.py` from the [flatpak-builder-tools repository](https://github.com/flatpak/flatpak-builder-tools/tree/master/cargo).

    Install its dependencies:
    ```bash
    pip install tomlkit aiohttp
    ```

    Run the generator from the RPFM repository root:
    ```bash
    python3 flatpak-cargo-generator.py Cargo.lock -o install/linux/flatpak/cargo-sources.json
    ```

2.  **Build and install the Flatpak**:
    ```bash
    cd /path/to/rpfm
    flatpak-builder --force-clean --install --user build-dir install/linux/flatpak/com.github.frodo45127.rpfm.yaml
    ```

3.  **Run RPFM**:
    ```bash
    flatpak run com.github.frodo45127.rpfm
    ```

## Troubleshooting

- If the build fails with network errors, ensure `cargo-sources.json` is up to date with the current `Cargo.lock`.
- For debugging, you can enter the build environment with:
  ```bash
  flatpak-builder --run build-dir install/linux/flatpak/com.github.frodo45127.rpfm.yaml bash
  ```

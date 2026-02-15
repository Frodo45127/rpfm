#!/usr/bin/env bash
#
# Flatpak build script for RPFM.
#
# Regenerates cargo-sources.json from Cargo.lock, then builds and installs
# the RPFM Flatpak package locally.
#
# Prerequisites:
#   - flatpak and flatpak-builder installed
#   - KDE Platform/SDK and Rust extension installed:
#       flatpak install flathub org.kde.Platform//5.15-25.08
#       flatpak install flathub org.kde.Sdk//5.15-25.08
#       flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//25.08
#   - flatpak-cargo-generator.py available on PATH (or pip install tomlkit aiohttp)
#     https://github.com/flatpak/flatpak-builder-tools/tree/master/cargo
#
# Usage: Run from the repository root.
#   ./install/linux/linux_flatpak_release.sh
#
# Options:
#   --skip-cargo-sources   Skip regenerating cargo-sources.json

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
FLATPAK_DIR="$SCRIPT_DIR/flatpak"
MANIFEST="$FLATPAK_DIR/com.github.frodo45127.rpfm.yaml"
CARGO_SOURCES="$FLATPAK_DIR/cargo-sources.json"
BUILD_DIR="$REPO_ROOT/flatpak-build-dir"

SKIP_CARGO_SOURCES=false
for arg in "$@"; do
    case "$arg" in
        --skip-cargo-sources) SKIP_CARGO_SOURCES=true ;;
        *) echo "Unknown option: $arg" >&2; exit 1 ;;
    esac
done

cd "$REPO_ROOT"

# Regenerate cargo-sources.json if needed.
if [ "$SKIP_CARGO_SOURCES" = false ]; then
    echo "Regenerating cargo-sources.json from Cargo.lock..."

    # The command may be called with or without the .py extension depending on
    # how it was installed (pip vs pacman/system package).
    CARGO_GENERATOR=""
    if command -v flatpak-cargo-generator &> /dev/null; then
        CARGO_GENERATOR="flatpak-cargo-generator"
    elif command -v flatpak-cargo-generator.py &> /dev/null; then
        CARGO_GENERATOR="flatpak-cargo-generator.py"
    else
        echo "Error: flatpak-cargo-generator not found on PATH." >&2
        echo "Get it from: https://github.com/flatpak/flatpak-builder-tools/tree/master/cargo" >&2
        echo "Or run with --skip-cargo-sources to use the existing file." >&2
        exit 1
    fi

    "$CARGO_GENERATOR" Cargo.lock -o "$CARGO_SOURCES"
    echo "cargo-sources.json updated."
else
    echo "Skipping cargo-sources.json regeneration (using existing file)."
fi

# Build and install the Flatpak.
echo "Building Flatpak..."
echo "Build directory: ${BUILD_DIR}"

flatpak-builder --force-clean --install --user "$BUILD_DIR" "$MANIFEST"

echo "Done. Run with: flatpak run com.github.frodo45127.rpfm"

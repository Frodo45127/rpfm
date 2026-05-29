#!/usr/bin/env bash
#
# Flatpak build script for RPFM.
#
# Regenerates cargo-sources.json from Cargo.lock, then builds a
# redistributable Flatpak bundle (.flatpak) for the RPFM package.
#
# Prerequisites:
#   - flatpak and flatpak-builder installed
#   - flatpak-cargo-generator.py available on PATH (or pip install tomlkit aiohttp)
#     https://github.com/flatpak/flatpak-builder-tools/tree/master/cargo
#
# The required runtimes (KDE Platform/SDK, Rust extension) are automatically
# installed if missing, based on the versions declared in the manifest.
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
REPO_DIR="$REPO_ROOT/flatpak-repo"
BUNDLE="$REPO_ROOT/rpfm.flatpak"
METAINFO="$FLATPAK_DIR/com.github.frodo45127.rpfm.metainfo.xml"

SKIP_CARGO_SOURCES=false
for arg in "$@"; do
    case "$arg" in
        --skip-cargo-sources) SKIP_CARGO_SOURCES=true ;;
        *) echo "Unknown option: $arg" >&2; exit 1 ;;
    esac
done

cd "$REPO_ROOT"

if [ ! -f "$REPO_ROOT/.env" ]; then
    echo "Warning: .env not found at repo root; the Flatpak will be built without Sentry DSNs / PostHog keys (telemetry and crash reporting disabled)."
fi

# Read the runtime version from the manifest so we have a single source of truth.
RUNTIME_VERSION=$(grep 'runtime-version:' "$MANIFEST" | head -1 | sed "s/.*runtime-version: *['\"]\\{0,1\\}\\([^'\"]*\\)['\"]\\{0,1\\}/\\1/")
echo "Runtime version from manifest: ${RUNTIME_VERSION}"

# Ensure the Flathub remote is available.
if ! flatpak remote-list | grep -q flathub; then
    echo "Adding Flathub remote..."
    flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
fi

# Install required runtimes if not already present.
install_if_missing() {
    local ref="$1"
    if ! flatpak info "$ref" &>/dev/null; then
        echo "Installing ${ref}..."
        flatpak install -y --noninteractive flathub "$ref"
    else
        echo "Found ${ref}"
    fi
}

# The Rust SDK extension is versioned against the Freedesktop SDK, not the KDE
# runtime. Extract the base Freedesktop version from the installed KDE SDK metadata.
get_freedesktop_version() {
    # First ensure the KDE SDK is present so we can inspect it.
    install_if_missing "org.kde.Sdk//${RUNTIME_VERSION}" >&2

    local fd_ver
    fd_ver=$(flatpak info -m "org.kde.Sdk//${RUNTIME_VERSION}" 2>/dev/null \
        | grep -A5 '\[Extension org.freedesktop.Platform.GL\]' \
        | grep '^versions=' | head -1 \
        | sed 's/versions=//;s/;.*//')

    if [ -z "$fd_ver" ]; then
        echo "Warning: could not detect Freedesktop SDK version, falling back to 25.08" >&2
        fd_ver="25.08"
    fi
    echo "$fd_ver"
}

echo "Checking required runtimes..."
install_if_missing "org.kde.Platform//${RUNTIME_VERSION}"
install_if_missing "org.kde.Sdk//${RUNTIME_VERSION}"

FREEDESKTOP_VERSION=$(get_freedesktop_version)
echo "Freedesktop SDK version: ${FREEDESKTOP_VERSION}"
install_if_missing "org.freedesktop.Sdk.Extension.rust-stable//${FREEDESKTOP_VERSION}"

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

# Update metainfo version and date from Cargo.toml.
VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')
DATE=$(date +%Y-%m-%d)
echo "Updating metainfo: version=${VERSION}, date=${DATE}"
sed -i "s/<release version=\"[^\"]*\" date=\"[^\"]*\"/<release version=\"${VERSION}\" date=\"${DATE}\"/" "$METAINFO"

# Build the Flatpak into a local repo.
echo "Building Flatpak..."
echo "Build directory: ${BUILD_DIR}"
echo "Repository: ${REPO_DIR}"

flatpak-builder --force-clean --repo="$REPO_DIR" "$BUILD_DIR" "$MANIFEST"

# Export as a redistributable bundle.
echo "Creating redistributable bundle: ${BUNDLE}"
flatpak build-bundle "$REPO_DIR" "$BUNDLE" com.github.frodo45127.rpfm

echo "Done. Bundle created at: ${BUNDLE}"
echo "Install with: flatpak install --user rpfm.flatpak"

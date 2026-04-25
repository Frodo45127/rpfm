#!/usr/bin/env bash
#
# Linux release build script for RPFM.
#
# Compiles release binaries, collects assets into an FHS-compliant directory
# structure, and produces a versioned tar.zst archive.
#
# Usage: Run from the repository root.
#   ./install/linux/linux_release.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TARGET="x86_64-unknown-linux-gnu"

cd "$REPO_ROOT"

# Read version from Cargo.toml.
VERSION="$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)"
if [ -z "$VERSION" ]; then
    echo "Error: Could not read version from Cargo.toml" >&2
    exit 1
fi

ARCHIVE_NAME="rpfm-v${VERSION}-${TARGET}.tar.zst"
BUILD_DIR="$(mktemp -d)"

echo "Building RPFM v${VERSION} for ${TARGET}..."
echo "Build directory: ${BUILD_DIR}"

# Load Sentry secrets from the repo-root .env (gitignored). Sourced before
# `cargo build` so `option_env!` in rpfm_ui/rpfm_server picks up the DSNs and
# bakes them into the binaries; reused below for the symbol upload step.
if [ -f "$REPO_ROOT/.env" ]; then
    set -a; . "$REPO_ROOT/.env"; set +a
else
    echo "Warning: .env not found at repo root; binaries will be built without Sentry DSNs."
fi

# Clean and rebuild qt_rpfm_extensions so stale artifacts are not reused.
if [ -f "3rdparty/src/qt_rpfm_extensions/Makefile" ]; then
    make -C 3rdparty/src/qt_rpfm_extensions clean || true
fi

# Build release binaries.
cargo clean
cargo build --release --bin rpfm_server
cargo build --release --features "enable_tools" --bin rpfm_ui

# Upload debug symbols to Sentry so stack traces in crash reports get
# resolved to function names and source lines. UI and server live in
# different Sentry projects, so each binary uploads under its own slug.
if [ -n "${SENTRY_AUTH_TOKEN:-}" ]; then
    if [ -n "${SENTRY_ORG:-}" ] && [ -n "${RPFM_UI_SENTRY_PROJECT:-}" ]; then
        echo "Uploading rpfm_ui debug symbols to Sentry..."
        sentry-cli debug-files upload \
            --org "$SENTRY_ORG" \
            --project "$RPFM_UI_SENTRY_PROJECT" \
            --include-sources \
            target/release/rpfm_ui
    else
        echo "Warning: SENTRY_ORG / RPFM_UI_SENTRY_PROJECT not set, skipping rpfm_ui symbol upload."
    fi

    if [ -n "${SENTRY_ORG:-}" ] && [ -n "${RPFM_SERVER_SENTRY_PROJECT:-}" ]; then
        echo "Uploading rpfm_server debug symbols to Sentry..."
        sentry-cli debug-files upload \
            --org "$SENTRY_ORG" \
            --project "$RPFM_SERVER_SENTRY_PROJECT" \
            --include-sources \
            target/release/rpfm_server
    else
        echo "Warning: SENTRY_ORG / RPFM_SERVER_SENTRY_PROJECT not set, skipping rpfm_server symbol upload."
    fi
else
    echo "Warning: SENTRY_AUTH_TOKEN not set, skipping symbol upload."
fi

echo "Collecting assets..."

# Create FHS directory structure.
mkdir -p "$BUILD_DIR/usr/bin"
mkdir -p "$BUILD_DIR/usr/share/rpfm/icons"
mkdir -p "$BUILD_DIR/usr/share/rpfm/locale"
mkdir -p "$BUILD_DIR/usr/share/rpfm/ui"
mkdir -p "$BUILD_DIR/usr/share/applications"
mkdir -p "$BUILD_DIR/usr/share/licenses/rpfm"

# Binaries.
cp target/release/rpfm_server "$BUILD_DIR/usr/bin/rpfm_server"
cp target/release/rpfm_ui "$BUILD_DIR/usr/bin/rpfm_ui"

# Icons.
cp -R icons/* "$BUILD_DIR/usr/share/rpfm/icons/"

# Locale.
cp -R locale/* "$BUILD_DIR/usr/share/rpfm/locale/"

# UI templates.
cp -R rpfm_ui/ui_templates/* "$BUILD_DIR/usr/share/rpfm/ui/"

# Desktop shortcut.
cp install/linux/arch/rpfm.desktop "$BUILD_DIR/usr/share/applications/rpfm.desktop"

# License.
cp LICENSE "$BUILD_DIR/usr/share/licenses/rpfm/LICENSE"

# Create the archive.
echo "Creating archive: ${ARCHIVE_NAME}"
tar --zstd -cf "$ARCHIVE_NAME" -C "$BUILD_DIR" usr

# Cleanup.
rm -rf "$BUILD_DIR"

echo "Done. Archive created at: ${REPO_ROOT}/${ARCHIVE_NAME}"

#!/usr/bin/env bash
# pack.sh — Unified packaging script for SoupRune releases.
#
# Usage:
#   ./scripts/pack.sh linux               # x86_64-unknown-linux-gnu → .tar.gz
#   ./scripts/pack.sh windows             # x86_64-pc-windows-gnu   → .zip
#   ./scripts/pack.sh linux-arm           # aarch64-unknown-linux-gnu → .tar.gz
#   ./scripts/pack.sh <custom-target>     # any Rust target triple
#
# The script:
#   1. Builds a release binary for the given target
#   2. Copies only mods listed in mods.toml (whitelist)
#   3. Copies only git-tracked files per mod (excludes .gitignore'd files)
#   4. Creates a distributable archive in dist/

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MODS_TOML="$REPO_ROOT/mods.toml"
PROJECT="souprune"

# --- Target resolution ---
resolve_target() {
    case "${1:-}" in
        linux)      echo "x86_64-unknown-linux-gnu" ;;
        windows)    echo "x86_64-pc-windows-gnu" ;;
        linux-arm)  echo "aarch64-unknown-linux-gnu" ;;
        "")
            echo "Usage: $0 <platform>" >&2
            echo "  Aliases: linux, windows, linux-arm" >&2
            echo "  Or any Rust target triple (e.g. x86_64-apple-darwin)" >&2
            exit 1
            ;;
        *)          echo "$1" ;;
    esac
}

target_os() {
    case "$1" in
        *windows*) echo "windows" ;;
        *linux*)   echo "linux" ;;
        *darwin*)  echo "macos" ;;
        *android*) echo "android" ;;
        *)         echo "unknown" ;;
    esac
}

target_arch() {
    case "$1" in
        x86_64*)   echo "x86_64" ;;
        aarch64*)  echo "aarch64" ;;
        i686*)     echo "i686" ;;
        arm*)      echo "arm" ;;
        *)         echo "unknown" ;;
    esac
}

binary_name() {
    case "$1" in
        *windows*) echo "${PROJECT}.exe" ;;
        *)         echo "${PROJECT}" ;;
    esac
}

archive_ext() {
    case "$1" in
        *windows*) echo "zip" ;;
        *)         echo "tar.gz" ;;
    esac
}

# --- Main ---
TARGET=$(resolve_target "${1:-}")
OS=$(target_os "$TARGET")
ARCH=$(target_arch "$TARGET")
BINARY=$(binary_name "$TARGET")
EXT=$(archive_ext "$TARGET")

cd "$REPO_ROOT"
VERSION=$(grep '^version' crates/souprune/Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
DIST="souprune-${VERSION}-${OS}-${ARCH}"

echo "🔨 Building release for ${TARGET}..."
cargo build -p "${PROJECT}" --release --target "${TARGET}"

echo "📁 Staging ${DIST}..."
rm -rf "dist/${DIST}"
mkdir -p "dist/${DIST}/projects"

# Copy binary
cp "target/${TARGET}/release/${BINARY}" "dist/${DIST}/"

# Copy projects config
cp projects/config.toml "dist/${DIST}/projects/"

# Copy whitelisted mods (only git-tracked files)
if [ ! -f "$MODS_TOML" ]; then
    echo "⚠️  mods.toml not found, skipping mod packaging"
else
    for mod_name in $(grep -oP '^\[mods\.\K[^]]+' "$MODS_TOML"); do
        mod_dir="projects/${mod_name}"
        if [ ! -d "${mod_dir}" ]; then
            echo "⚠️  Mod not installed: ${mod_name}"
            continue
        fi
        echo "📦 Including mod: ${mod_name}"
        mkdir -p "dist/${DIST}/${mod_dir}"
        git -C "${mod_dir}" ls-files -z | while IFS= read -r -d '' file; do
            dir_part=$(dirname "${file}")
            mkdir -p "dist/${DIST}/${mod_dir}/${dir_part}"
            cp "${mod_dir}/${file}" "dist/${DIST}/${mod_dir}/${file}"
        done
    done
fi

# Create archive
echo "�� Creating archive..."
cd dist
case "$EXT" in
    tar.gz) tar czf "${DIST}.tar.gz" "${DIST}" ;;
    zip)    zip -qr "${DIST}.zip" "${DIST}" ;;
esac

echo "✅ Packaged: dist/${DIST}.${EXT}"

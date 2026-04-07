#!/usr/bin/env bash
# pack.sh — Unified packaging script for SoupRune releases.
#
# Usage:
#   ./scripts/pack.sh linux                              # Build + package
#   ./scripts/pack.sh windows                            # Build + package
#   ./scripts/pack.sh linux-arm                          # Build + package
#   ./scripts/pack.sh <rust-target-triple>               # Build + package
#   ./scripts/pack.sh linux --skip-build --binary-path path/to/binary  # CI mode
#
# Options:
#   --skip-build       Skip cargo build (use with --binary-path)
#   --binary-path PATH Use a pre-built binary instead of the default target/ path
#   --dist-name NAME   Override the distribution folder name
#
# The script:
#   1. Builds a release binary (unless --skip-build)
#   2. Copies only mods listed in mods.toml (whitelist)
#   3. Copies only git-tracked files per mod + root-level .wasm files
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
        macos)      echo "x86_64-apple-darwin" ;;
        "")
            echo "Usage: $0 <platform> [--skip-build] [--binary-path PATH] [--dist-name NAME]" >&2
            echo "  Aliases: linux, windows, linux-arm, macos" >&2
            echo "  Or any Rust target triple" >&2
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

# --- Parse arguments ---
PLATFORM=""
SKIP_BUILD=false
BINARY_PATH=""
DIST_NAME=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-build)   SKIP_BUILD=true; shift ;;
        --binary-path)  BINARY_PATH="$2"; shift 2 ;;
        --dist-name)    DIST_NAME="$2"; shift 2 ;;
        -*)             echo "Unknown option: $1" >&2; exit 1 ;;
        *)              PLATFORM="$1"; shift ;;
    esac
done

TARGET=$(resolve_target "${PLATFORM}")
OS=$(target_os "$TARGET")
ARCH=$(target_arch "$TARGET")
BINARY=$(binary_name "$TARGET")
EXT=$(archive_ext "$TARGET")

cd "$REPO_ROOT"
VERSION=$(grep '^version' crates/souprune/Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
DIST="${DIST_NAME:-souprune-${VERSION}-${OS}-${ARCH}}"

# --- Build ---
if [ "$SKIP_BUILD" = false ]; then
    echo "🔨 Building release for ${TARGET}..."
    cargo build -p "${PROJECT}" --release --target "${TARGET}"
fi

# --- Resolve binary path ---
if [ -n "$BINARY_PATH" ]; then
    SRC_BINARY="$BINARY_PATH"
else
    SRC_BINARY="target/${TARGET}/release/${BINARY}"
fi

if [ ! -f "$SRC_BINARY" ]; then
    echo "❌ Binary not found: $SRC_BINARY" >&2
    exit 1
fi

# --- Stage ---
echo "📁 Staging ${DIST}..."
rm -rf "dist/${DIST}" "dist/${DIST}.tar.gz" "dist/${DIST}.zip"
mkdir -p "dist/${DIST}/projects"

cp "$SRC_BINARY" "dist/${DIST}/"

# Copy builtin WASM
BUILTIN_WASM="crates/souprune_builtins/target/wasm32-wasip2/release/souprune_builtins.wasm"
if [ -f "$BUILTIN_WASM" ]; then
    echo "📦 Including builtin WASM"
    mkdir -p "dist/${DIST}/builtins"
    cp "$BUILTIN_WASM" "dist/${DIST}/builtins/"
else
    echo "⚠️  Builtin WASM not found: $BUILTIN_WASM (danmaku patterns will not work)"
fi

# Copy projects config
cp projects/config.toml "dist/${DIST}/projects/"

# Copy whitelisted mods (only git-tracked files + root .wasm)
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
        # Copy git-tracked files
        git -C "${mod_dir}" ls-files -z | while IFS= read -r -d '' file; do
            dir_part=$(dirname "${file}")
            mkdir -p "dist/${DIST}/${mod_dir}/${dir_part}"
            cp "${mod_dir}/${file}" "dist/${DIST}/${mod_dir}/${file}"
        done
        # Also copy .wasm files (gitignored but needed at runtime)
        find "${mod_dir}" -maxdepth 1 -name '*.wasm' -exec cp {} "dist/${DIST}/${mod_dir}/" \;
    done
fi

# --- Archive ---
echo "📦 Creating archive..."
cd dist
case "$EXT" in
    tar.gz) tar czf "${DIST}.tar.gz" "${DIST}" ;;
    zip)    zip -qr "${DIST}.zip" "${DIST}" ;;
esac

echo "✅ Packaged: dist/${DIST}.${EXT}"

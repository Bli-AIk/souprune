#!/usr/bin/env bash
# tokei_check.sh — Lint for code quality: max line count + no mod.rs files
# Usage: ./dev/tokei_check.sh [max_lines] [search_dir]
#   max_lines  — maximum allowed code lines per file (default: 800)
#   search_dir — directory to scan (default: crates/)

set -euo pipefail

# Colors (only if stdout is a terminal)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    CYAN='\033[0;36m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    CYAN=''
    BOLD=''
    RESET=''
fi

MAX_LINES="${1:-800}"
SEARCH_DIR="${2:-crates/}"

errors=0

# --- Check 1: No mod.rs files (Rust 2018+ module style) ---
# Exclude examples/ directories: Cargo treats .rs files in examples/ as binaries,
# so mod.rs is the only viable pattern for shared helper modules there.
mod_files=$(find "$SEARCH_DIR" -name 'mod.rs' -type f -not -path '*/examples/*' 2>/dev/null || true)
if [ -n "$mod_files" ]; then
    echo -e "${RED}${BOLD}Error:${RESET} Found mod.rs files. Use Rust 2018+ module naming instead:"
    echo "$mod_files" | while read -r f; do echo -e "  ${YELLOW}$f${RESET}"; done
    errors=1
fi

# --- Check 2: No Rust file exceeds max code lines (via tokei) ---
over_limit=$(tokei "$SEARCH_DIR" --output json --files \
    | jq -r --argjson max "$MAX_LINES" \
        '.Rust.reports[]? | select(.stats.code > $max) | "\(.name)|\(.stats.code)"')
if [ -n "$over_limit" ]; then
    while IFS='|' read -r file lines; do
        echo -e "${RED}${BOLD}Error:${RESET} ${YELLOW}$file${RESET} has ${CYAN}$lines${RESET} lines of code (max ${CYAN}$MAX_LINES${RESET})"
    done <<< "$over_limit"
    errors=1
fi

if [ "$errors" -ne 0 ]; then
    exit 1
else
    echo -e "${GREEN}${BOLD}Tokei OK:${RESET} All Rust files under ${CYAN}$MAX_LINES${RESET} lines of code, no mod.rs found."
fi

# --- Check 3: No allow(clippy::...) anywhere — use clippy.toml for global config ---
# Both #[allow(clippy::...)] and #![allow(clippy::...)] are banned.
# Global lint thresholds belong in clippy.toml.
# Individual exceptions should use #[expect(clippy::...)] with a reason.
# Exclude submodule directories (they are independent repos)
SUBMODULE_EXCLUDES=""
for sub in $(git config --file .gitmodules --get-regexp path | awk '{print $2}' 2>/dev/null); do
    SUBMODULE_EXCLUDES="$SUBMODULE_EXCLUDES --exclude-dir=$(basename "$sub")"
done

allow_hits=$(grep -rn 'allow(clippy::' "$SEARCH_DIR" --include="*.rs" $SUBMODULE_EXCLUDES 2>/dev/null || true)
if [ -n "$allow_hits" ]; then
    echo -e "${RED}${BOLD}Error:${RESET} Found allow(clippy::...). Use clippy.toml for global config or #[expect] for individual cases:"
    echo "$allow_hits" | while read -r line; do echo -e "  ${YELLOW}$line${RESET}"; done
    errors=1
fi

# --- Check 4: #[expect(clippy::...)] must have a // reason: comment ---
expect_no_reason=$(grep -rn '#\[expect(clippy::' "$SEARCH_DIR" --include="*.rs" $SUBMODULE_EXCLUDES 2>/dev/null \
    | grep -v '// reason:' || true)
if [ -n "$expect_no_reason" ]; then
    echo -e "${RED}${BOLD}Error:${RESET} Found #[expect(clippy::...)] without // reason: comment:"
    echo "$expect_no_reason" | while read -r line; do echo -e "  ${YELLOW}$line${RESET}"; done
    errors=1
fi

if [ "$errors" -ne 0 ]; then
    exit 1
else
    echo -e "${GREEN}${BOLD}Lint OK:${RESET} No #[allow(clippy::...)] found, all #[expect] have reasons."
fi

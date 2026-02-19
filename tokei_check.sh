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
mod_files=$(find "$SEARCH_DIR" -name 'mod.rs' -type f 2>/dev/null || true)
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

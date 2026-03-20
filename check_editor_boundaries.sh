#!/usr/bin/env bash
# check_editor_boundaries.sh — Architecture guardrail for editor -> engine internals.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR"

cd "$ROOT_DIR"

hits="$(rg -n 'souprune::(core|app_state|extra)::' crates/souprune_editor/src -g '*.rs' || true)"

if [ -n "$hits" ]; then
    echo "Error: souprune_editor is reaching into engine internals."
    echo "Route the dependency through souprune::editor_api or a top-level public API instead."
    echo "$hits"
    exit 1
fi

echo "Editor boundary OK: no deep engine internal paths are used."

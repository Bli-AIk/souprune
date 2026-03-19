#!/usr/bin/env bash
# check_core_boundaries.sh — Architecture guardrail for core -> app_state dependencies.
#
# This check is baseline-based on purpose:
# - existing violations are frozen in a baseline file
# - newly introduced violations fail the check
# - removed violations are reported as cleanup progress

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR"
BASELINE_FILE="$ROOT_DIR/core_to_app_state_baseline.txt"
TMP_CURRENT="$(mktemp)"
trap 'rm -f "$TMP_CURRENT"' EXIT

if [ ! -f "$BASELINE_FILE" ]; then
    echo "Missing baseline file: $BASELINE_FILE" >&2
    exit 1
fi

(
    cd "$ROOT_DIR"
    rg -n "crate::app_state(::|\\b)" crates/souprune/src/core | sort > "$TMP_CURRENT" || true
)

new_hits="$(comm -13 "$BASELINE_FILE" "$TMP_CURRENT" || true)"
removed_hits="$(comm -23 "$BASELINE_FILE" "$TMP_CURRENT" || true)"

if [ -n "$new_hits" ]; then
    echo "Error: new core -> app_state dependencies were introduced."
    echo "Move the behavior out of core/, or update the architecture intentionally before changing the baseline."
    echo "$new_hits"
    exit 1
fi

if [ -n "$removed_hits" ]; then
    echo "Progress: some historical core -> app_state dependencies disappeared."
    echo "Shrink core_to_app_state_baseline.txt after the refactor is reviewed:"
    echo "$removed_hits"
fi

echo "Core boundary OK: no new core -> app_state dependencies."

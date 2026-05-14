#!/usr/bin/env bash
# check_core_boundaries.sh — Architecture guardrail for core -> app_state dependencies.
#
# This check is baseline-based on purpose:
# - existing violations are frozen in a baseline file
# - newly introduced violations fail the check
# - removed violations are reported as cleanup progress

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BASELINE_FILE="$ROOT_DIR/core_to_app_state_baseline.txt"
TMP_CURRENT="$(mktemp)"
trap 'rm -f "$TMP_CURRENT"' EXIT

if [ ! -f "$BASELINE_FILE" ]; then
    echo "Missing baseline file: $BASELINE_FILE" >&2
    exit 1
fi

PYTHON_BIN="${PYTHON:-}"
if [ -z "$PYTHON_BIN" ]; then
    if command -v python3 >/dev/null 2>&1; then
        PYTHON_BIN=python3
    elif command -v python >/dev/null 2>&1; then
        PYTHON_BIN=python
    else
        echo "Missing Python interpreter: expected python3 or python." >&2
        exit 1
    fi
fi

(
    cd "$ROOT_DIR"
    "$PYTHON_BIN" - <<'PY' > "$TMP_CURRENT"
import re
import subprocess

result = subprocess.run(
    ["rg", "-n", r"crate::app_state(::|\b)", "crates/souprune/src/core"],
    check=False,
    capture_output=True,
    text=True,
)

hits = set()
for line in result.stdout.splitlines():
    file_path, _, text = line.split(":", 2)
    match = re.search(r"crate::app_state(?:::[A-Za-z0-9_]+)*", text)
    if not match:
        continue

    path = match.group(0)

    # Normalize imports to the module path, so changing imported symbols
    # does not look like a brand-new dependency.
    if text.lstrip().startswith("use "):
        parts = path.split("::")
        if len(parts) > 2:
            path = "::".join(parts[:-1])

    hits.add(f"{file_path}:{path}")

for hit in sorted(hits):
    print(hit)
PY
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

preset_hits="$(
    cd "$ROOT_DIR"
    rg -n "crate::preset(::|\b)|super::preset(::|\b)" crates/souprune/src/core -g '*.rs' || true
)"

if [ -n "$preset_hits" ]; then
    echo "Error: core -> preset dependencies are not allowed."
    echo "Core is the generic framework layer; move shared code into core/ or invert the dependency."
    echo "$preset_hits"
    exit 1
fi

echo "Core boundary OK: no core -> preset dependencies."

battle_semantic_hits="$(
    cd "$ROOT_DIR"
    rg -n "BattleBox|BattlePlayer|BoundToBattleBox" crates/souprune/src/core -g '*.rs' || true
)"

if [ -n "$battle_semantic_hits" ]; then
    echo "Error: battle gameplay abstractions are not allowed in core/."
    echo "Core may provide generic primitives, but BattleBox/BattlePlayer semantics must live outside core."
    echo "$battle_semantic_hits"
    exit 1
fi

echo "Core boundary OK: no BattleBox/BattlePlayer gameplay abstractions in core."

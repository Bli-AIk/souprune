#!/usr/bin/env bash
# Create a dated backup of this repository under /tmp.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BACKUP_DIR="/tmp/souprune-backup-$(date +%Y%m%d-%H%M%S)"

mkdir -p "${BACKUP_DIR}"

rsync -a \
    --exclude='/target/' \
    --exclude='**/target/' \
    --exclude='/node_modules/' \
    --exclude='**/node_modules/' \
    --exclude='/dist/' \
    --exclude='**/dist/' \
    --exclude='/.direnv/' \
    --exclude='**/.direnv/' \
    --exclude='/.cache/' \
    --exclude='**/.cache/' \
    "${REPO_ROOT}/" \
    "${BACKUP_DIR}/"

printf 'Backup written to %s\n' "${BACKUP_DIR}"

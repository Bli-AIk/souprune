#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODS_TOML="$SCRIPT_DIR/mods.toml"
MOD_REPO="$SCRIPT_DIR/.mod-repo"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

show_help() {
    cat << EOF
Usage: ./setup_mods.sh [OPTIONS] [MOD_NAME]

Setup SoupRune example mods using git worktree.

Options:
    -h, --help      Show this help message
    -c, --clean     Remove all mod worktrees
    -l, --list      List available mods
    -u, --update    Update mod repository and worktrees

Arguments:
    MOD_NAME        Specific mod to install (optional, installs all by default)

Examples:
    ./setup_mods.sh                    # Install all mods
    ./setup_mods.sh example_mod        # Install only example_mod
    ./setup_mods.sh --clean            # Remove all worktrees
    ./setup_mods.sh --update           # Update from remote
EOF
}

parse_toml_mods() {
    grep -E '^\[mods\.' "$MODS_TOML" | sed 's/\[mods\.//' | sed 's/\]//'
}

get_mod_property() {
    local mod_name="$1"
    local prop="$2"
    grep -A 10 "^\[mods\.$mod_name\]" "$MODS_TOML" | grep "^$prop" | sed 's/.*= *"\(.*\)"/\1/'
}

get_repo_url() {
    grep '^url' "$MODS_TOML" | sed 's/.*= *"\(.*\)"/\1/'
}

init_mod_repo() {
    if [ -d "$MOD_REPO" ]; then
        log_info "Mod repository already exists at $MOD_REPO"
        return 0
    fi

    local repo_url
    repo_url=$(get_repo_url)

    log_info "Cloning mod repository to $MOD_REPO..."
    git clone --bare "$repo_url" "$MOD_REPO"
    log_info "Mod repository initialized"
}

setup_worktree() {
    local mod_name="$1"
    local branch
    local path

    branch=$(get_mod_property "$mod_name" "branch")
    path=$(get_mod_property "$mod_name" "path")

    local full_path="$SCRIPT_DIR/$path"

    if [ -d "$full_path" ]; then
        log_warn "Worktree already exists at $path"
        return 0
    fi

    log_info "Creating worktree for $mod_name (branch: $branch)..."
    cd "$MOD_REPO"
    git worktree add "../$path" "$branch"
    log_info "Created worktree at $path"
}

remove_worktree() {
    local mod_name="$1"
    local path

    path=$(get_mod_property "$mod_name" "path")
    local full_path="$SCRIPT_DIR/$path"

    if [ ! -d "$full_path" ]; then
        log_warn "Worktree does not exist at $path"
        return 0
    fi

    log_info "Removing worktree at $path..."
    cd "$MOD_REPO"
    git worktree remove "$full_path" --force 2>/dev/null || rm -rf "$full_path"
    log_info "Removed worktree at $path"
}

update_mod_repo() {
    if [ ! -d "$MOD_REPO" ]; then
        log_error "Mod repository not initialized. Run setup first."
        exit 1
    fi

    log_info "Updating mod repository..."
    cd "$MOD_REPO"
    git fetch origin
    log_info "Mod repository updated"
}

list_mods() {
    echo "Available mods:"
    echo ""
    for mod in $(parse_toml_mods); do
        local desc
        desc=$(get_mod_property "$mod" "description")
        printf "  %-25s %s\n" "$mod" "$desc"
    done
}

clean_all() {
    if [ ! -d "$MOD_REPO" ]; then
        log_warn "No mod repository found"
        return 0
    fi

    log_info "Removing all worktrees..."
    cd "$MOD_REPO"

    for mod in $(parse_toml_mods); do
        remove_worktree "$mod"
    done

    log_info "Removing mod repository..."
    rm -rf "$MOD_REPO"
    log_info "Cleanup complete"
}

main() {
    local action="setup"
    local target_mod=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -c|--clean)
                action="clean"
                shift
                ;;
            -l|--list)
                action="list"
                shift
                ;;
            -u|--update)
                action="update"
                shift
                ;;
            *)
                target_mod="$1"
                shift
                ;;
        esac
    done

    if [ ! -f "$MODS_TOML" ]; then
        log_error "mods.toml not found at $MODS_TOML"
        exit 1
    fi

    case $action in
        list)
            list_mods
            ;;
        clean)
            clean_all
            ;;
        update)
            update_mod_repo
            ;;
        setup)
            init_mod_repo

            if [ -n "$target_mod" ]; then
                if grep -q "^\[mods\.$target_mod\]" "$MODS_TOML"; then
                    setup_worktree "$target_mod"
                else
                    log_error "Unknown mod: $target_mod"
                    list_mods
                    exit 1
                fi
            else
                for mod in $(parse_toml_mods); do
                    setup_worktree "$mod"
                done
            fi

            log_info "Setup complete!"
            log_info "Configure active mod in projects/config.toml"
            ;;
    esac
}

main "$@"

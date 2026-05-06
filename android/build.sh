#!/bin/bash
# ============================================================
#   SoupRune Android 构建脚本
#   交互式构建、安装、同步 mod
# ============================================================

set -euo pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/.."
ANDROID_DIR="$SCRIPT_DIR"
ANDROID_MOD_BASE="/sdcard/SoupRune/projects"
ANDROID_BUILTINS_DIR="/sdcard/SoupRune/builtins"
APK_PATH="$ANDROID_DIR/app/build/outputs/apk/debug/app-debug.apk"
PACKAGE_NAME="com.bliaik.souprune"
BUILD_DEBUG=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

banner() {
    echo
    echo -e "${CYAN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║         🤖 SoupRune Android 构建工具                     ║${NC}"
    echo -e "${CYAN}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo
}

# ── 环境检查 ──────────────────────────────────────────────

check_rust_target() {
    local ok=true
    if ! rustup target list --installed | grep -q "aarch64-linux-android"; then
        echo -e "${YELLOW}⚠ 未安装 aarch64-linux-android target${NC}"
        echo -e "  运行: ${BOLD}rustup target add aarch64-linux-android${NC}"
        ok=false
    fi
    if ! rustup target list --installed | grep -q "wasm32-wasip2"; then
        echo -e "${YELLOW}⚠ 未安装 wasm32-wasip2 target${NC}"
        echo -e "  运行: ${BOLD}rustup target add wasm32-wasip2${NC}"
        ok=false
    fi
    [ "$ok" = true ]
}

read_active_mod_name() {
    sed -n 's/^mod_name[[:space:]]*=[[:space:]]*"\(.*\)".*/\1/p' "$PROJECT_ROOT/projects/config.toml"
}

resolve_mod_order() {
    local mod_name="$1"
    cd "$PROJECT_ROOT"
    CARGO_TARGET_DIR="$PROJECT_ROOT/target/cauld-ron-deps" \
        cargo run -p souprune_cauld_ron --features deps-cli --bin cauld-ron-deps -- "$mod_name"
}

find_ndk() {
    if [ -n "${ANDROID_NDK_HOME:-}" ] && [ -d "$ANDROID_NDK_HOME" ]; then
        echo "$ANDROID_NDK_HOME"
        return 0
    fi
    local sdk_root="${ANDROID_HOME:-$HOME/Android/Sdk}"
    if [ -d "$sdk_root/ndk" ]; then
        local ndk_dir
        ndk_dir=$(ls -d "$sdk_root/ndk"/*/ 2>/dev/null | sort -V | tail -1)
        if [ -n "$ndk_dir" ]; then
            echo "${ndk_dir%/}"
            return 0
        fi
    fi
    return 1
}

find_java_home() {
    if [ -n "${JAVA_HOME:-}" ] && [ -d "$JAVA_HOME" ]; then
        echo "$JAVA_HOME"
        return 0
    fi
    # Try common locations
    for jdir in /usr/lib/jvm/java-21-openjdk /usr/lib/jvm/java-17-openjdk \
                /usr/lib/jvm/java-21-openjdk-amd64 /usr/lib/jvm/java-17-openjdk-amd64; do
        if [ -d "$jdir" ]; then
            echo "$jdir"
            return 0
        fi
    done
    return 1
}

check_env() {
    local ok=true
    echo -e "${BOLD}检查构建环境...${NC}"
    echo

    # Rust + cargo
    if command -v cargo >/dev/null 2>&1; then
        echo -e "  ✅ cargo $(cargo --version | awk '{print $2}')"
    else
        echo -e "  ${RED}❌ 未找到 cargo${NC}"
        ok=false
    fi

    # Rust targets for Android native and WASM assets
    if check_rust_target; then
        echo -e "  ✅ aarch64-linux-android 与 wasm32-wasip2 targets 已安装"
    else
        ok=false
    fi

    # Android SDK
    local sdk_root="${ANDROID_HOME:-$HOME/Android/Sdk}"
    if [ -d "$sdk_root" ]; then
        echo -e "  ✅ Android SDK: $sdk_root"
        export ANDROID_HOME="$sdk_root"
    else
        echo -e "  ${RED}❌ 未找到 Android SDK (设置 ANDROID_HOME)${NC}"
        ok=false
    fi

    # NDK
    if NDK_HOME=$(find_ndk); then
        echo -e "  ✅ NDK: $NDK_HOME"
        export ANDROID_NDK_HOME="$NDK_HOME"
    else
        echo -e "  ${RED}❌ 未找到 Android NDK${NC}"
        ok=false
    fi

    # Java
    if JAVA=$(find_java_home); then
        echo -e "  ✅ Java: $JAVA"
        export JAVA_HOME="$JAVA"
    else
        echo -e "  ${RED}❌ 未找到 JDK (需要 Java 17+)${NC}"
        ok=false
    fi

    # Gradle wrapper
    if [ -x "$ANDROID_DIR/gradlew" ]; then
        echo -e "  ✅ Gradle wrapper 就绪"
    else
        echo -e "  ${RED}❌ 缺少 gradlew (android/gradlew)${NC}"
        ok=false
    fi

    echo
    if [ "$ok" = false ]; then
        echo -e "${RED}环境检查失败，请先解决以上问题。${NC}"
        exit 1
    fi
    echo -e "${GREEN}环境检查通过！${NC}"
    echo
}

# ── 构建 ──────────────────────────────────────────────────

prepare_assets() {
    local mod_name
    mod_name=$(read_active_mod_name)
    if [ -z "$mod_name" ]; then
        echo -e "${RED}❌ 无法从 projects/config.toml 读取 mod_name${NC}"
        return 1
    fi

    if ! resolve_mod_order "$mod_name" >/dev/null; then
        echo -e "${RED}❌ 无法解析 mod 依赖顺序: $mod_name${NC}"
        return 1
    fi

    echo -e "${GREEN}▶ [assets] 构建 builtin WASM 与 mod 内容: $mod_name...${NC}"
    cd "$PROJECT_ROOT"
    just mod="$mod_name" prepare-assets-release
    echo -e "${GREEN}✅ 资源与 mod 构建完成${NC}"
}

build_native() {
    local features="android"
    if [ "$BUILD_DEBUG" = true ]; then
        features="android,bevy_debug"
        echo -e "${YELLOW}▶ [1/3] 构建 aarch64 native library (release + bevy debug names)...${NC}"
    else
        echo -e "${GREEN}▶ [1/3] 构建 aarch64 native library (release)...${NC}"
    fi

    local TOOLCHAIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64"
    export CC_aarch64_linux_android="$TOOLCHAIN/bin/aarch64-linux-android21-clang"
    export CXX_aarch64_linux_android="$TOOLCHAIN/bin/aarch64-linux-android21-clang++"
    export AR_aarch64_linux_android="$TOOLCHAIN/bin/llvm-ar"
    export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$TOOLCHAIN/bin/aarch64-linux-android21-clang"

    cd "$PROJECT_ROOT"
    cargo build -p souprune --target aarch64-linux-android --features "$features" --release

    echo -e "${GREEN}✅ Native library 构建完成${NC}"
}

copy_so() {
    echo -e "${GREEN}▶ [2/3] 复制 .so 到 jniLibs...${NC}"

    local JNILIB_DIR="$ANDROID_DIR/app/src/main/jniLibs/arm64-v8a"
    mkdir -p "$JNILIB_DIR"

    local SO_FILE="$PROJECT_ROOT/target/aarch64-linux-android/release/libsouprune.so"
    if [ ! -f "$SO_FILE" ]; then
        echo -e "${RED}❌ 找不到: $SO_FILE${NC}"
        return 1
    fi
    cp "$SO_FILE" "$JNILIB_DIR/libsouprune.so"

    # Copy libc++_shared.so from NDK if available
    local LIBCXX="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so"
    if [ -f "$LIBCXX" ]; then
        cp "$LIBCXX" "$JNILIB_DIR/libc++_shared.so"
    fi

    echo -e "${GREEN}✅ .so 文件已复制${NC}"
    ls -lh "$JNILIB_DIR/"
}

build_apk() {
    echo -e "${GREEN}▶ [3/3] 构建 APK...${NC}"

    cd "$ANDROID_DIR"
    JAVA_HOME="$JAVA_HOME" ANDROID_HOME="$ANDROID_HOME" \
        ./gradlew assembleDebug --no-daemon -q

    if [ -f "$APK_PATH" ]; then
        echo
        echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║              🎉 APK 构建成功!                             ║${NC}"
        echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
        echo
        echo -e "  📁 位置: $APK_PATH"
        echo -e "  📏 大小: $(du -h "$APK_PATH" | cut -f1)"
    else
        echo -e "${RED}❌ APK 构建失败${NC}"
        return 1
    fi
}

do_build() {
    prepare_assets && build_native && copy_so && build_apk
}

# ── 设备操作 ──────────────────────────────────────────────

check_device() {
    if ! command -v adb >/dev/null 2>&1; then
        echo -e "${RED}❌ 未找到 adb 命令${NC}"
        return 1
    fi
    local devices
    devices=$(adb devices 2>/dev/null | grep -w "device$" | wc -l)
    if [ "$devices" -eq 0 ]; then
        echo -e "${YELLOW}⚠ 未检测到已连接的安卓设备${NC}"
        echo -e "  请通过 USB 连接设备并启用 USB 调试"
        return 1
    fi
    return 0
}

do_install() {
    if ! check_device; then return 1; fi

    if [ ! -f "$APK_PATH" ]; then
        echo -e "${RED}❌ APK 不存在，请先构建${NC}"
        return 1
    fi

    echo -e "${GREEN}▶ 正在安装 APK...${NC}"
    adb install -r "$APK_PATH"

    # Grant permissions
    echo -e "${GREEN}▶ 设置权限...${NC}"
    adb shell "appops set $PACKAGE_NAME MANAGE_EXTERNAL_STORAGE allow" 2>/dev/null || true
    adb shell "pm grant $PACKAGE_NAME android.permission.READ_EXTERNAL_STORAGE" 2>/dev/null || true

    echo -e "${GREEN}✅ 安装完成${NC}"
}

do_sync_mods() {
    if ! check_device; then return 1; fi

    echo -e "${GREEN}▶ 同步 mod 文件夹到设备...${NC}"

    # Read current mod name from config.toml
    local config_file="$PROJECT_ROOT/projects/config.toml"
    if [ ! -f "$config_file" ]; then
        echo -e "${RED}❌ 找不到 projects/config.toml${NC}"
        return 1
    fi

    local mod_name
    mod_name=$(read_active_mod_name)
    if [ -z "$mod_name" ]; then
        echo -e "${RED}❌ 无法从 config.toml 读取 mod_name${NC}"
        return 1
    fi

    local mod_order_output
    if ! mod_order_output=$(resolve_mod_order "$mod_name"); then
        echo -e "${RED}❌ 无法解析 mod 依赖顺序: $mod_name${NC}"
        return 1
    fi
    mapfile -t mod_order < <(printf '%s\n' "$mod_order_output" | sed '/^[[:space:]]*$/d')
    if [ "${#mod_order[@]}" -eq 0 ]; then
        echo -e "${RED}❌ mod 依赖顺序为空: $mod_name${NC}"
        return 1
    fi

    echo -e "  📦 Mod: ${BOLD}$mod_name${NC}"
    echo -e "  📦 同步顺序: ${mod_order[*]}"
    echo -e "  📱 设备: $ANDROID_MOD_BASE"

    # Create base dir on device
    adb shell "mkdir -p $ANDROID_MOD_BASE" 2>/dev/null || true

    for sync_mod_name in "${mod_order[@]}"; do
        local local_mod_dir="$PROJECT_ROOT/projects/$sync_mod_name"
        if [ ! -d "$local_mod_dir" ]; then
            echo -e "${RED}❌ 本地 mod 目录不存在: $local_mod_dir${NC}"
            return 1
        fi

        echo -e "  🗑️  删除设备上的旧 mod 文件夹: $sync_mod_name"
        adb shell "rm -rf $ANDROID_MOD_BASE/$sync_mod_name" 2>/dev/null || true

        echo -e "  📤 推送 mod 文件到设备: $sync_mod_name"
        adb push "$local_mod_dir" "$ANDROID_MOD_BASE/" 2>&1

        for wasm_file in "$local_mod_dir/.build/runtime.wasm" "$local_mod_dir/.build/content.wasm"; do
            if [ -f "$wasm_file" ]; then
                echo -e "    📦 ${wasm_file#$PROJECT_ROOT/}"
            else
                echo -e "  ${YELLOW}⚠ 未找到 ${wasm_file#$PROJECT_ROOT/}，请确认 prepare-assets-release 已成功${NC}"
            fi
        done
    done

    # Also push config.toml
    echo -e "  📤 推送 config.toml..."
    adb push "$config_file" "$ANDROID_MOD_BASE/config.toml" 2>&1

    # Sync builtin WASM to device
    echo -e "  📤 同步 builtin WASM 到设备..."
    adb shell "mkdir -p $ANDROID_BUILTINS_DIR" 2>/dev/null || true

    local builtin_wasm="$PROJECT_ROOT/assets/builtins/souprune_builtins.wasm"
    if [ -f "$builtin_wasm" ]; then
        adb push "$builtin_wasm" "$ANDROID_BUILTINS_DIR/souprune_builtins.wasm" 2>&1
        echo -e "  ${GREEN}✅ souprune_builtins.wasm 已同步${NC}"
    else
        echo -e "  ${YELLOW}⚠ 未找到 assets/builtins/souprune_builtins.wasm，请先运行构建流程${NC}"
    fi

    echo -e "${GREEN}✅ Mod 同步完成${NC}"
}

# ── 主菜单 ────────────────────────────────────────────────

show_menu() {
    echo
    echo -e "${BOLD}── 选择操作 ──${NC}"
    echo -e "  ${CYAN}1${NC}. 🔨 构建 + 安装 + 同步 mod"
    echo -e "  ${CYAN}2${NC}. 🐛 构建 (Bevy debug names) + 安装 + 同步 mod"
    echo -e "  ${CYAN}3${NC}. 📱 安装 APK 到设备"
    echo -e "  ${CYAN}4${NC}. 📂 同步 mod 文件夹到设备"
    echo -e "  ${CYAN}5${NC}. 🚪 退出"
    echo
}

menu_loop() {
    while true; do
        show_menu
        read -rp "请选择 [1-5]: " choice
        echo
        case "$choice" in
            1)
                BUILD_DEBUG=false
                do_build && do_install && do_sync_mods || true
                ;;
            2)
                BUILD_DEBUG=true
                do_build && do_install && do_sync_mods || true
                ;;
            3)
                do_install || true
                ;;
            4)
                do_sync_mods || true
                ;;
            5)
                echo -e "${GREEN}👋 再见！${NC}"
                exit 0
                ;;
            *)
                echo -e "${YELLOW}⚠ 无效选择，请输入 1-5${NC}"
                ;;
        esac
    done
}

# ── 入口 ──────────────────────────────────────────────────

# Support --option N to skip interactive menu
OPTION=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --option)
            OPTION="$2"
            shift 2
            ;;
        *)
            echo -e "${RED}未知参数: $1${NC}"
            exit 1
            ;;
    esac
done

banner
check_env

if [ -n "$OPTION" ]; then
    case "$OPTION" in
        1)
            BUILD_DEBUG=false
            do_build && do_install && do_sync_mods
            ;;
        2)
            BUILD_DEBUG=true
            do_build && do_install && do_sync_mods
            ;;
        3)
            do_install
            ;;
        4)
            do_sync_mods
            ;;
        5)
            echo -e "${GREEN}👋 再见！${NC}"
            ;;
        *)
            echo -e "${YELLOW}⚠ 无效选项: $OPTION${NC}"
            exit 1
            ;;
    esac
else
    menu_loop
fi

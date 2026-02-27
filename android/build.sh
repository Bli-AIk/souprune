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
ANDROID_INTERNAL_MODS="/data/data/com.bliaik.souprune/mods"
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
    if ! rustup target list --installed | grep -q "aarch64-linux-android"; then
        echo -e "${YELLOW}⚠ 未安装 aarch64-linux-android target${NC}"
        echo -e "  运行: ${BOLD}rustup target add aarch64-linux-android${NC}"
        return 1
    fi
    return 0
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

    # aarch64 target
    if check_rust_target; then
        echo -e "  ✅ aarch64-linux-android target 已安装"
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

build_native() {
    local features="android"
    if [ "$BUILD_DEBUG" = true ]; then
        features="android,bevy_debug"
        echo -e "${YELLOW}▶ [1/3] 构建 aarch64 native library (release + bevy/debug)...${NC}"
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
    build_native && copy_so && build_apk
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
    mod_name=$(grep 'mod_name' "$config_file" | sed 's/.*= *"\(.*\)"/\1/')
    if [ -z "$mod_name" ]; then
        echo -e "${RED}❌ 无法从 config.toml 读取 mod_name${NC}"
        return 1
    fi

    local local_mod_dir="$PROJECT_ROOT/projects/$mod_name"
    if [ ! -d "$local_mod_dir" ]; then
        echo -e "${RED}❌ 本地 mod 目录不存在: $local_mod_dir${NC}"
        return 1
    fi

    echo -e "  📦 Mod: ${BOLD}$mod_name${NC}"
    echo -e "  📁 本地: $local_mod_dir"
    echo -e "  📱 设备: $ANDROID_MOD_BASE/$mod_name"

    # Create base dir on device
    adb shell "mkdir -p $ANDROID_MOD_BASE" 2>/dev/null || true

    # Delete existing mod folder on device
    echo -e "  🗑️  删除设备上的旧 mod 文件夹..."
    adb shell "rm -rf $ANDROID_MOD_BASE/$mod_name" 2>/dev/null || true

    # Push entire mod folder (excluding code/ directory which is Rust source)
    echo -e "  📤 推送 mod 文件到设备..."
    adb push "$local_mod_dir" "$ANDROID_MOD_BASE/" 2>&1

    # Also push config.toml
    echo -e "  📤 推送 config.toml..."
    adb push "$config_file" "$ANDROID_MOD_BASE/../config.toml" 2>&1 || \
    adb shell "mkdir -p /sdcard/SoupRune/projects" && \
    adb push "$config_file" "/sdcard/SoupRune/projects/config.toml" 2>&1

    # Sync .so files to app internal storage
    echo -e "  📤 同步 mod .so 文件到应用内部存储..."
    adb shell "run-as $PACKAGE_NAME mkdir -p $ANDROID_INTERNAL_MODS" 2>/dev/null || true

    # Find .so files for this mod
    local so_found=false
    for so_file in "$local_mod_dir"/*_android.so "$local_mod_dir"/*.so; do
        if [ -f "$so_file" ]; then
            local so_name
            so_name=$(basename "$so_file")
            echo -e "    📦 $so_name"
            # Push to sdcard first, then copy to internal via run-as
            adb push "$so_file" "/sdcard/SoupRune/$so_name" 2>/dev/null
            adb shell "cat /sdcard/SoupRune/$so_name | run-as $PACKAGE_NAME sh -c 'cat > $ANDROID_INTERNAL_MODS/$so_name'" 2>/dev/null
            adb shell "rm /sdcard/SoupRune/$so_name" 2>/dev/null || true
            so_found=true
        fi
    done

    if [ "$so_found" = false ]; then
        echo -e "  ${YELLOW}⚠ 未找到 mod .so 文件 (可能需要先构建 mod)${NC}"
    fi

    echo -e "${GREEN}✅ Mod 同步完成${NC}"
}

# ── 主菜单 ────────────────────────────────────────────────

show_menu() {
    echo
    echo -e "${BOLD}── 选择操作 ──${NC}"
    echo -e "  ${CYAN}1${NC}. 🔨 构建 + 安装 + 同步 mod"
    echo -e "  ${CYAN}2${NC}. 🐛 构建 (debug features) + 安装 + 同步 mod"
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

banner
check_env
menu_loop

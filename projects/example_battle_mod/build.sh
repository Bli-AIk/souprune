#!/bin/bash
set -e

# 注意：此脚本设计为在 Linux 环境下运行。
# 它支持本地构建 (Linux .so) 以及通过交叉编译构建 (Windows .dll)。
# 在进行 Windows 构建前，请确保已安装相应依赖：
# 
# Debian/Ubuntu:
#   1. sudo apt-get install mingw-w64
#   2. rustup target add x86_64-pc-windows-gnu x86_64-pc-windows-msvc
#
# Arch Linux:
#   1. sudo pacman -S mingw-w64-gcc
#   2. rustup target add x86_64-pc-windows-gnu x86_64-pc-windows-msvc
# 
# 对于 MSVC 构建 (cargo xwin):
#   cargo install cargo-xwin (或 Arch AUR: yay -S cargo-xwin)

# 获取脚本所在的绝对路径
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
MOD_SOURCE_DIR="$SCRIPT_DIR/code/mod_example"
DESTINATION_DIR="$SCRIPT_DIR"

# 默认配置
BUILD_MODE="debug"
CARGO_FLAGS=""
BUILD_WIN=false

# 解析参数
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --release) BUILD_MODE="release"; CARGO_FLAGS="--release" ;;
        --win) BUILD_WIN=true ;;
        *) echo "未知参数: $1"; exit 1 ;;
    esac
    shift
done

# 构建函数
build_target() {
    local target=$1
    local cargo_output_name=$2
    local final_output_name=$3
    local custom_build_cmd=${4:-"cargo build"}
    local cargo_target_flag=""
    local target_dir="target/$BUILD_MODE"

    if [ -n "$target" ]; then
        cargo_target_flag="--target $target"
        target_dir="target/$target/$BUILD_MODE"
        echo "--- 开始构建 mod_example ($BUILD_MODE) Target: $target ---"
    else
        echo "--- 开始构建 mod_example ($BUILD_MODE) Native ---"
    fi

    pushd "$MOD_SOURCE_DIR" > /dev/null
    $custom_build_cmd $CARGO_FLAGS $cargo_target_flag
    popd > /dev/null

    local source_file="$MOD_SOURCE_DIR/$target_dir/$cargo_output_name"
    # 兼容某些情况下产物在 deps 目录的问题
    if [ ! -f "$source_file" ]; then
        source_file="$MOD_SOURCE_DIR/$target_dir/deps/$cargo_output_name"
    fi

    if [ -f "$source_file" ]; then
        echo "正在同步 $cargo_output_name 到 $DESTINATION_DIR/$final_output_name ..."
        cp -f "$source_file" "$DESTINATION_DIR/$final_output_name"
    else
        echo "错误：找不到构建产物 $source_file"
        exit 1
    fi
}

# 检查依赖函数
check_rust_target() {
    local target=$1
    if ! rustup target list --installed | grep -q "^$target$"; then
        echo "错误: 未安装 Rust 目标: $target"
        echo "请运行: rustup target add $target"
        return 1
    fi
    return 0
}

check_command() {
    local cmd=$1
    local package_hint=$2
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "错误: 未找到命令: $cmd"
        echo "请安装: $package_hint"
        return 1
    fi
    return 0
}

# 检查源代码目录
if [ ! -d "$MOD_SOURCE_DIR" ]; then
    echo "错误: 源代码目录不存在: $MOD_SOURCE_DIR"
    exit 1
fi

# 执行构建
if [ "$BUILD_WIN" = true ]; then
    echo "=== Windows 构建模式 ==="
    
    # 1. 构建 MSVC 版本 (使用 cargo-xwin)
    if command -v cargo-xwin >/dev/null 2>&1; then
        echo "检测到 cargo-xwin，准备构建 MSVC 版本..."
        if check_rust_target "x86_64-pc-windows-msvc"; then
             # MSVC 不需要手动设置 static-crt，通常默认兼容性较好，或者由 xwin 处理
             # 清除 RUSTFLAGS 以避免冲突 (如 gnu 的 static-crt 标志)
             export RUSTFLAGS="" 
             build_target "x86_64-pc-windows-msvc" "mod_example.dll" "example_mod_msvc.dll" "cargo xwin build"
        else
             echo "警告: 跳过 MSVC 构建 (缺少 Rust Target)。"
        fi
    else
        echo "警告: 未找到 cargo-xwin，跳过 MSVC 构建。"
        echo "提示: 在 Linux 上构建 MSVC 版本需要 cargo-xwin (cargo install cargo-xwin)"
    fi

    # 2. 构建 GNU 版本 (使用 MinGW)
    echo "---"
    echo "准备构建 GNU 版本..."
    
    MISSING_GNU_DEPS=false
    
    # 检查 Rust Target
    if ! check_rust_target "x86_64-pc-windows-gnu"; then
        MISSING_GNU_DEPS=true
    fi
    
    # 检查 MinGW Linker
    if ! check_command "x86_64-w64-mingw32-gcc" "mingw-w64 (sudo apt install mingw-w64)"; then
        MISSING_GNU_DEPS=true
    fi
    
    if [ "$MISSING_GNU_DEPS" = false ]; then
        # 使用 static-crt 防止依赖 libgcc_s_seh-1.dll
        export RUSTFLAGS="-C target-feature=+crt-static"
        build_target "x86_64-pc-windows-gnu" "mod_example.dll" "example_mod_gnu.dll"
    else
        echo "错误: 缺少构建 GNU 版本的依赖，构建失败。"
        exit 1
    fi
else
    # 默认构建本地 Linux 版本
    build_target "" "libmod_example.so" "example_mod.so"
fi

echo "构建与同步完成！"
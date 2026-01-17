#!/bin/bash
set -e

# 注意：此脚本设计为在 Linux 环境下运行。
# 它支持本地构建 (Linux .so) 以及通过交叉编译构建 (Windows .dll)。
# 在进行 Windows 构建前，请确保已安装：
# 1. sudo apt-get install mingw-w64
# 2. rustup target add x86_64-pc-windows-gnu

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
    local output_name=$2
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
    cargo build $CARGO_FLAGS $cargo_target_flag
    popd > /dev/null

    local source_file="$MOD_SOURCE_DIR/$target_dir/$output_name"
    # 兼容某些情况下产物在 deps 目录的问题
    if [ ! -f "$source_file" ]; then
        source_file="$MOD_SOURCE_DIR/$target_dir/deps/$output_name"
    fi

    if [ -f "$source_file" ]; then
        echo "正在同步 $output_name 到 $DESTINATION_DIR ..."
        cp -f "$source_file" "$DESTINATION_DIR/$output_name"
    else
        echo "错误：找不到构建产物 $source_file"
        exit 1
    fi
}

# 检查源代码目录
if [ ! -d "$MOD_SOURCE_DIR" ]; then
    echo "错误: 源代码目录不存在: $MOD_SOURCE_DIR"
    exit 1
fi

# 执行构建
if [ "$BUILD_WIN" = true ]; then
    # 构建 Windows 版本
    build_target "x86_64-pc-windows-gnu" "mod_example.dll"
else
    # 默认构建本地 Linux 版本
    build_target "" "libmod_example.so"
fi

echo "构建与同步完成！"
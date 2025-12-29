#!/bin/bash
set -e

# 获取脚本所在的绝对路径
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
MOD_SOURCE_DIR="$SCRIPT_DIR/code/mod_example"
TARGET_FILE_NAME="libmod_example.so"
DESTINATION_DIR="$SCRIPT_DIR"

# 默认构建模式
BUILD_MODE="debug"
CARGO_FLAGS=""

# 解析参数
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --release) BUILD_MODE="release"; CARGO_FLAGS="--release" ;;
        *) echo "未知参数: $1"; exit 1 ;;
    esac
    shift
done

echo "--- 开始构建 mod_example ($BUILD_MODE) ---"

# 检查源代码目录是否存在
if [ ! -d "$MOD_SOURCE_DIR" ]; then
    echo "错误: 源代码目录不存在: $MOD_SOURCE_DIR"
    exit 1
fi

# 进入 mod 源代码目录进行构建
pushd "$MOD_SOURCE_DIR" > /dev/null

echo "正在执行: cargo build $CARGO_FLAGS"
cargo build $CARGO_FLAGS

popd > /dev/null

# 确定源文件路径
SOURCE_FILE="$MOD_SOURCE_DIR/target/$BUILD_MODE/$TARGET_FILE_NAME"
DESTINATION_FILE="$DESTINATION_DIR/$TARGET_FILE_NAME"

# 鲁棒性检查：源文件是否存在
if [ ! -f "$SOURCE_FILE" ]; then
    # 尝试在 deps 目录下查找 (有时 cargo 的行为取决于配置)
    ALT_SOURCE_FILE="$MOD_SOURCE_DIR/target/$BUILD_MODE/deps/$TARGET_FILE_NAME"
    if [ -f "$ALT_SOURCE_FILE" ]; then
        SOURCE_FILE="$ALT_SOURCE_FILE"
    else
        echo "错误：找不到构建产物 $SOURCE_FILE"
        exit 1
    fi
fi

echo "正在强制复制 $TARGET_FILE_NAME 到 $DESTINATION_DIR ..."

# 使用 -f 强制覆盖
cp -f "$SOURCE_FILE" "$DESTINATION_FILE"

echo "构建与同步完成！"
echo "产物路径: $DESTINATION_FILE"

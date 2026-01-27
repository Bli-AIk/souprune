#!/bin/bash
set -e

# 获取脚本所在的绝对路径
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
MOD_SOURCE_DIR="$SCRIPT_DIR/code/mod_example_csharp"
TARGET_FILE_NAME="libmod_example_csharp.so"
DESTINATION_DIR="$SCRIPT_DIR"

# 默认构建模式
BUILD_MODE="Release"

# 解析参数
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --debug) BUILD_MODE="Debug" ;;
        *) echo "未知参数: $1"; exit 1 ;;
    esac
    shift
done

echo "--- 开始构建 mod_example_csharp ($BUILD_MODE) ---"

# 检查是否安装了 .NET SDK
if ! command -v dotnet &> /dev/null; then
    echo "错误: 未找到 dotnet 命令。请安装 .NET 9.0 SDK。"
    echo "访问: https://dotnet.microsoft.com/download/dotnet/9.0"
    exit 1
fi

# 检查 .NET 版本
DOTNET_VERSION=$(dotnet --version 2>/dev/null || echo "0")
echo "检测到 .NET SDK 版本: $DOTNET_VERSION"

# 检查源代码目录是否存在
if [ ! -d "$MOD_SOURCE_DIR" ]; then
    echo "错误: 源代码目录不存在: $MOD_SOURCE_DIR"
    exit 1
fi

# 进入 mod 源代码目录进行构建
pushd "$MOD_SOURCE_DIR" > /dev/null

echo "正在执行: dotnet publish -c $BUILD_MODE -r linux-x64"

# 使用 Native AOT 发布
dotnet publish -c "$BUILD_MODE" -r linux-x64 --self-contained true

popd > /dev/null

# 确定源文件路径
SOURCE_FILE="$MOD_SOURCE_DIR/bin/$BUILD_MODE/net9.0/linux-x64/publish/$TARGET_FILE_NAME"

# 鲁棒性检查：源文件是否存在
if [ ! -f "$SOURCE_FILE" ]; then
    # 尝试其他可能的路径
    ALT_SOURCE_FILE="$MOD_SOURCE_DIR/bin/$BUILD_MODE/net9.0/linux-x64/native/$TARGET_FILE_NAME"
    if [ -f "$ALT_SOURCE_FILE" ]; then
        SOURCE_FILE="$ALT_SOURCE_FILE"
    else
        echo "错误：找不到构建产物"
        echo "尝试查找的路径："
        echo "  - $SOURCE_FILE"
        echo "  - $ALT_SOURCE_FILE"
        echo ""
        echo "实际构建输出目录内容："
        find "$MOD_SOURCE_DIR/bin" -name "*.so" -o -name "*.dll" 2>/dev/null || echo "(无文件)"
        exit 1
    fi
fi

DESTINATION_FILE="$DESTINATION_DIR/$TARGET_FILE_NAME"

echo "正在复制 $TARGET_FILE_NAME 到 $DESTINATION_DIR ..."

# 使用 -f 强制覆盖
cp -f "$SOURCE_FILE" "$DESTINATION_FILE"

echo "构建与同步完成！"
echo "产物路径: $DESTINATION_FILE"

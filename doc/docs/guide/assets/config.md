# Mod 配置 (mod.toml)

每个 Mod 目录下都必须包含一个 `mod.toml` 文件，用于告诉引擎如何加载你的 Mod。

## 示例配置

```toml
name = "example_mod"
version = "0.1.0"
authors = ["Your Name"]
description = "An example mod for SoupRune."

[dependencies]
# 在这里列出你的 Mod 依赖的其他 Mod

[soul_modes]
# 定义灵魂模式对应的动态链接库
"soul_red" = "libmod_example.so"
"soul_blue" = "libmod_example.so"
```

## 字段说明

*   **name**: Mod 的唯一标识符。
*   **version**: 版本号，遵循语义化版本规范。
*   **soul_modes**: 这是一个键值对映射。
    *   Key: 灵魂模式的 ID（如 "soul_red"）。
    *   Value: 包含该模式逻辑的编译后的库文件名称（如 "libmod_example.so"）。这允许你用 Rust 编写自定义的灵魂移动和交互逻辑。

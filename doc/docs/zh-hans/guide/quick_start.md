# 快速开始

本节将指导你如何运行 SoupRune 并启动示例 Mod。

## 环境准备

在开始之前，请确保你已经安装了以下工具：

1.  **Rust 工具链**: 访问 [rustup.rs](https://rustup.rs/) 安装。
2.  **系统依赖 (Linux)**:
    ```bash
    sudo apt-get install g++ pkg-config libx11-dev libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
    ```

## 运行项目

SoupRune 是一个 Rust 工作空间。要运行主程序并加载示例 Mod，请在项目根目录下执行：

```bash
cargo run --package souprune
```

首次编译可能需要一些时间，请耐心等待。

## 运行示例

如果你想单独测试某个组件（例如对话系统 UI），可以运行特定的 Example：

```bash
cargo run -p bevy_mortar_bond --example dialogue_ui
```

## 下一步

成功运行后，你应该能看到游戏窗口。接下来，你可以查看 [项目结构](project_structure.md) 来了解如何修改和创建自己的 Mod。

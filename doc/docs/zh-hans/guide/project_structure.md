# 项目结构

SoupRune 的项目结构设计清晰，将核心引擎代码与用户内容分离。作为 Mod 创作者，你主要关注 `projects/` 目录。

## 目录概览

```
souprune/
├── crates/                 # 引擎核心源码 (Rust)
├── projects/               # 用户项目目录
│   ├── config.toml         # 全局配置
│   └── example_mod/        # 示例 Mod
│       ├── mod.toml        # Mod 元数据
│       ├── battle/         # 战斗相关资源
│       ├── overworld/      # 地图相关资源
│       ├── code/           # 脚本与逻辑代码
│       └── shared/         # 共享资源 (图片、文本等)
└── Cargo.toml              # 工作空间配置
```

## Mod 结构详解

一个标准的 Mod 文件夹（如 `example_mod`）包含以下部分：

### 1. mod.toml
Mod 的核心配置文件，定义了 Mod 的名称、版本、作者以及灵魂模式（Soul Modes）的绑定。

### 2. battle/
包含战斗系统的所有数据：
*   **chapters/**: 定义战斗章节流程（`.ron` 格式）。
*   **players/**: 定义战斗角色属性。
*   **ui/**: 战斗界面的布局配置。

### 3. overworld/
包含大地图（Overworld）的数据：
*   **levels/**: Tiled 地图项目文件（`.tiled-project`, `.world`）。
*   **characters/**: 地图角色定义。

### 4. code/
存放 Mod 的逻辑代码。
*   **mod_example/**: 通常是一个 Rust Crate，可以编译为动态链接库 (`.so` / `.dll`)，用于扩展更高级的游戏逻辑。

### 5. shared/
存放通用的游戏素材：
*   **textures/**: 纹理图片。
*   **items/**: 物品定义。
*   **locales/**: 本地化文本文件。

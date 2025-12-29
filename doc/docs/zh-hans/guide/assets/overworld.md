# 地图系统 (Overworld)

SoupRune 使用 **Tiled Map Editor** 作为主要的关卡设计工具。

## 文件格式

在 `projects/<mod>/overworld/levels/` 目录下，你会看到以下文件：

*   **`.tiled-project`**: Tiled 项目文件，管理图块集 (Tilesets) 和对象模板。
*   **`.world`**: Tiled 世界文件，用于将多个地图 (`.tmx` 或内部格式) 组合在一起，构建无缝的大地图。

## 制作流程

1.  **下载 Tiled**: 请访问 [mapeditor.org](https://www.mapeditor.org/) 下载最新版。
2.  **创建项目**: 在 `overworld/levels` 下创建一个新项目。
3.  **绘制地图**:
    *   **图块层 (Tile Layers)**: 用于绘制地形、墙壁等静态视觉元素。
    *   **对象层 (Object Layers)**: 用于放置碰撞体、NPC、传送门等交互对象。
4.  **定义属性**:
    *   你可以为对象添加自定义属性（如 `script` 指向 Mortar 脚本，或 `target_map` 指向传送目标），引擎会在加载时读取这些属性并绑定相应的逻辑。

## 碰撞检测

通常，你需要在 Tiled 中创建一个专门的“碰撞层”或使用对象层中的形状（矩形、多边形）来定义可行走区域和障碍物。

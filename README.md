# SoupRune

SoupRune is a framework built on Bevy for data-driven RPG-style projects.

SoupRune 是构建在 Bevy 之上的通用游戏框架，用于数据驱动的 RPG 风格项目。

## Architecture

The distributed binary contains generic framework runtime only. Concrete game
semantics live in project content and project WASM runtime modules under
`projects/`.

分发的二进制只包含通用框架运行时。具体游戏语义位于 `projects/` 下的项目
content 与项目 WASM runtime 中。

Core may provide high-frequency primitives such as View/SDF rendering,
tilemaps, top-down movement, collision regions, fixed-camera scene wiring,
danmaku, input transactions, and FRE bridges. Project `mod.toml` owns runtime
mode names and primitive composition through `game.modes.<id>`.

core 可以提供 View/SDF、tilemap、俯视角移动、碰撞区域、固定相机场景接线、
弹幕、输入事务、FRE bridge 等高频基础能力。项目 `mod.toml` 通过
`game.modes.<id>` 拥有运行时 mode 名称与 primitive 组合权。

## Project Surface

Mod authors work in `projects/`:

- content crates generate RON assets through cauld-ron;
- Mortar files define dialogue text and scripts;
- FRE rule files express data-driven rules;
- WASM runtime crates own custom behavior and custom actions.

mod 作者只在 `projects/` 中工作：

- content crate 通过 cauld-ron 生成 RON 资源；
- Mortar 文件定义对话文本与脚本；
- FRE rule 文件表达数据驱动规则；
- WASM runtime crate 拥有自定义行为与自定义 action。

For the detailed architecture, see [doc/architecture.md](doc/architecture.md)
and [doc/architecture_zh-hans.md](doc/architecture_zh-hans.md).

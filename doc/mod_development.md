# WASM Mod 开发指南

本文档介绍如何为 SoupRune 开发 WASM 模组（mod）。

## 概述

SoupRune 的 mod 系统基于 **WASM Component Model**。每个 mod 编译为 `.wasm` 组件，
运行时通过 [Wasmtime](https://wasmtime.dev/) 加载。

接口契约使用 [WIT](https://component-model.bytecodealliance.org/design/wit.html)
（WebAssembly Interface Types）定义，位于 `crates/souprune_api/wit/souprune-mod.wit`。

## 快速开始

### 1. 创建 Mod 项目

```bash
cargo init --lib my_mod
```

编辑 `Cargo.toml`：

```toml
[package]
name = "my_mod"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
souprune_sdk = { path = "<path-to-souprune>/crates/souprune_sdk" }
```

### 2. 实现 Mod

在 `src/lib.rs` 中：

```rust
use souprune_sdk::prelude::*;

struct MyBehavior;

impl Behavior for MyBehavior {
    fn on_enter(&mut self, ctx: &mut Context) {
        ctx.log("Hello from my mod!");
    }

    fn on_update(&mut self, ctx: &mut Context, delta_time: f32) {
        if ctx.input().pressed(Action::Confirm) {
            ctx.log("Confirm pressed!");
        }
    }

    fn on_exit(&mut self, ctx: &mut Context) {
        ctx.log("Goodbye!");
    }
}

export_mod! {
    behaviors: [
        ("my_behavior", MyBehavior, || MyBehavior),
    ],
    danmaku: [],
}
```

### 3. 编译为 WASM

```bash
# 安装 WASM 编译目标（仅需一次）
rustup target add wasm32-wasip2

# 编译
cargo build --target wasm32-wasip2 --release
```

产物位于 `target/wasm32-wasip2/release/my_mod.wasm`。

### 4. 测试

使用 `souprune_mock_host` 独立测试你的 mod：

```bash
cargo run -p souprune_mock_host -- path/to/my_mod.wasm
```

或使用 justfile 快捷命令：

```bash
just wasm-build   # 编译 souprune_mod_test
just wasm-test    # 编译并通过 mock_host 运行
```

## API 参考

### Context

`Context` 是 mod 与框架交互的唯一入口：

| 方法 | 描述 |
|------|------|
| `ctx.log(msg)` | 输出日志到宿主控制台 |
| `ctx.input().pressed(action)` | 检查语义输入是否按下 |
| `ctx.kinematics().set_velocity(x, y)` | 设置实体速度 |

### Action 枚举

```rust
enum Action {
    Up, Down, Left, Right,
    Confirm, Cancel, Menu,
}
```

### Behavior trait

```rust
trait Behavior {
    fn on_enter(&mut self, ctx: &mut Context);
    fn on_update(&mut self, ctx: &mut Context, delta_time: f32);
    fn on_exit(&mut self, ctx: &mut Context);
}
```

### DanmakuBehavior trait

```rust
trait DanmakuBehavior {
    fn on_enter(&mut self, ctx: &BulletContext);
    fn on_update(&mut self, ctx: &BulletContext) -> BulletOutput;
    fn on_exit(&mut self);
}
```

### BulletContext

弹幕回调每帧接收的上下文：

| 字段 | 类型 | 描述 |
|------|------|------|
| `elapsed` | `f32` | 自弹幕生成以来的经过时间 |
| `delta_time` | `f32` | 帧间隔时间 |
| `spawn_pos` | `Vec2` | 弹幕生成位置 |
| `offset` | `Vec2` | 当前偏移 |
| `initial_angle` | `f32` | 初始角度 |
| `initial_radius` | `f32` | 初始半径 |
| `player_pos` | `Vec2` | 玩家位置 |
| `props` | `Vec<Prop>` | RON 配置中的自定义浮点属性 |

### export_mod! 宏

将你的实现注册为 WASM 导出：

```rust
export_mod! {
    behaviors: [
        ("behavior_id", BehaviorType, || BehaviorType::new()),
    ],
    danmaku: [
        ("algorithm_id", DanmakuType, || DanmakuType::default()),
    ],
}
```

- 第一个元素是字符串 ID（框架通过此 ID 查找）
- 第二个是实现类型
- 第三个是构造闭包

## 架构说明

```
┌─────────────────────┐       WIT        ┌──────────────────┐
│   SoupRune Engine   │◄════════════════►│   WASM Mod       │
│   (Wasmtime Host)   │                  │   (Guest)        │
│                     │   imports:       │                  │
│  host-api impl ─────┼─────────────────►│  ctx.log()       │
│                     │                  │  ctx.input()     │
│                     │   exports:       │                  │
│  mod_system ◄───────┼─────────────────┤  Behavior        │
│  danmaku    ◄───────┼─────────────────┤  DanmakuBehavior │
└─────────────────────┘                  └──────────────────┘
```

## 注意事项

- Mod 运行在 WASM 沙箱中，无法直接访问文件系统或网络
- 所有与框架的交互必须通过 `Context` API
- WIT 文件（`souprune-mod.wit`）是接口的单一事实来源
- `souprune_mod_test` 是完整的参考实现，可作为模板

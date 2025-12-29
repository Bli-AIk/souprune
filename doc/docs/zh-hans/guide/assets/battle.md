# 战斗系统配置

SoupRune 的战斗系统是高度数据驱动的。战斗流程主要通过 `.chapter.ron` 文件定义，它位于 `projects/<mod>/battle/chapters/` 目录下。

## 章节文件 (RON)

RON (Rusty Object Notation) 是一种类似于 JSON 但支持 Rust 类型的格式。一个战斗章节通常是一个动作列表。

### 示例结构

```rust
[
    // 1. 初始化相机
    SetCamera(SetZoom(0.4)),
    
    // 2. 加载 UI 布局
    UIInteraction(ui_layout: "battle/ui/undertale.ui_layout.ron"),
    
    // 3. 生成玩家 (灵魂)
    SetPlayer(Spawn(
        config_path: "battle/players/player.battle_player.ron", 
        position: (0.0, -80.0)
    )),
    
    // 4. 等待 5 秒
    Wait(5.0),
    
    // 5. 执行弹幕模式
    BulletPattern(
        pattern_id: ["flowey_pellets_circle"]
    ),
    
    // 6. 嵌套序列
    Nested([
        Wait(0.5),
        SetPlayer(Despawn),
    ]),
]
```

## 常用指令

*   **SetCamera**: 控制战斗时的相机缩放和位置。
*   **UIInteraction**: 加载或操作战斗 UI。
*   **SetPlayer**: 管理玩家灵魂的生成 (`Spawn`) 和销毁 (`Despawn`)。
*   **Wait**: 等待指定的时间（秒）。
*   **BulletPattern**: 触发预定义的弹幕模式。弹幕逻辑通常在 Rust 代码中实现，并通过 ID 引用。
*   **Nested**: 执行一组嵌套的动作序列。

## 角色配置

战斗角色定义在 `battle/players/` 目录下，通常包含角色的属性（HP, ATK）和外观配置。

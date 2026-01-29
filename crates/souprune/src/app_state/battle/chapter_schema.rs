//! # chapter_schema.rs
//!
//! # chapter_schema.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines the schema for Battle Chapters in RON files.
//! This module contains pure data structures that map directly to the `battle.ron` format.
//! It serves as the data contract between the configuration files and the game logic.
//!
//! 定义 Battle Chapter 在 RON 文件中的 Schema。
//! 本模块包含直接映射到 `battle.ron` 格式的纯数据结构。
//! 它作为配置文件与游戏逻辑之间的数据契约。
//!
//! Chapter is the minimal unit of the linear sequence in the battle system.
//! It is an enum type representing different events in the battle.
//! For example, player choices, bullet pattern generation, dialogues, and nested Chapters.
//! Chapter itself does not contain definitions or implementations of bullet patterns or UI.
//!
//! Chapter 是 战斗系统中线性序列的最小单位。
//! 它是一个枚举类型，表示战斗中的不同事件。
//! 例如，玩家选择、弹幕生成、对话、以及 Chapter 的嵌套等。
//! Chapter 本身不包含 弹幕 或 UI 的定义与具体实现。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 3D vector tuple type for coordinates like translation and scale.
///
/// 三维向量元组类型，用于表示位置和缩放等坐标。
pub type Vec3Tuple = (Val<f32>, Val<f32>, Val<f32>);

/// 2D vector tuple type for coordinates.
///
/// 二维向量元组类型，用于表示坐标。
pub type Vec2Tuple = (Val<f32>, Val<f32>);

/// Color tuple type with RGBA components.
///
/// RGBA 颜色元组类型。
pub type ColorTuple = (Val<f32>, Val<f32>, Val<f32>, Val<f32>);

/// Generic value that can be either static or computed from an expression.
///
/// 泛型值，可以是静态值或从表达式计算得出。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Val<T> {
    /// Static value.
    ///
    /// 静态值。
    Static(T),
    /// Dynamic expression string.
    ///
    /// 动态表达式字符串。
    Expr(String),
}

impl<T> Val<T> {
    /// Returns true if this is a dynamic expression.
    ///
    /// 如果是动态表达式则返回 true。
    pub fn is_expr(&self) -> bool {
        matches!(self, Val::Expr(_))
    }

    /// Alias for `is_expr()` for backward compatibility.
    ///
    /// 为保持向后兼容的 `is_expr()` 别名。
    pub fn is_dynamic(&self) -> bool {
        self.is_expr()
    }

    /// Get the static value if available.
    ///
    /// 获取静态值（如果可用）。
    pub fn as_static(&self) -> Option<&T> {
        match self {
            Val::Static(v) => Some(v),
            Val::Expr(_) => None,
        }
    }

    /// Get the expression string if this is an expression.
    ///
    /// 获取表达式字符串（如果是表达式）。
    pub fn as_expr(&self) -> Option<&str> {
        match self {
            Val::Expr(s) => Some(s),
            Val::Static(_) => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Chapter {
    /// View Interaction Chapter.
    ///
    /// This chapter allows players to interact with the view.
    /// It loads a view layout file for View interaction scenarios, such as player choices or dialogues.
    ///
    /// 视图交互章节。
    ///
    /// 此章节允许玩家与视图交互。
    /// 它加载视图布局文件用于 View 交互场景，如玩家选择、对话等。
    ViewInteraction { view_layout: String },

    /// Await View Interaction Chapter (Phase 6).
    ///
    /// Blocks the battle sequencer until player confirms a selection in the specified
    /// interactive layer. This is used for player choice menus like FIGHT/ACT/ITEM/MERCY.
    ///
    /// 等待视图交互章节（Phase 6）。
    ///
    /// 阻塞战斗 sequencer 直到玩家在指定的交互层中确认选择。
    /// 用于玩家选择菜单，如 FIGHT/ACT/ITEM/MERCY。
    ///
    /// # Example / 示例
    /// ```ron
    /// AwaitViewInteraction(
    ///     layer_id: "BattleMainMenu",
    ///     initial_selection: 0,
    /// ),
    /// ```
    AwaitViewInteraction {
        /// The ID of the interactive layer to activate.
        ///
        /// 要激活的交互层的 ID。
        layer_id: String,

        /// Initial selection index (default: 0).
        ///
        /// 初始选择索引（默认：0）。
        #[serde(default)]
        initial_selection: usize,
    },

    /// Danmaku Performance Chapter.
    ///
    /// The Chapter is responsible for playing a complete danmaku performance (timeline-based).
    ///
    /// 弹幕演出章节。
    ///
    /// 此章节负责播放完整的弹幕演出（基于时间轴）。
    DanmakuPerformance {
        /// Path to the performance file (e.g., "battle/performances/boss_attack.performance.ron")
        performance: String,
        /// Optional spawn translation override (defaults to center of battle box)
        #[serde(default)]
        translation: Option<(f32, f32)>,
    },

    /// Alight Motion Animation Performance Chapter.
    ///
    /// Plays an Alight Motion project (.amproj) as a battle animation.
    /// Layers with names starting with "#B" are treated as bullets (with collision).
    /// Layers with names starting with "#C" are treated as battle box boundaries.
    ///
    /// Alight Motion 动画演出章节。
    ///
    /// 播放 Alight Motion 项目 (.amproj) 作为战斗动画。
    /// 名称以 "#B" 开头的图层被视为弹幕（带碰撞体）。
    /// 名称以 "#C" 开头的图层被视为战斗框边界。
    AmPerformance {
        /// Path to the .amproj file (e.g., "demo_turn.amproj")
        amproj_path: String,
        /// Wait for animation to complete before continuing (default: true)
        #[serde(default = "default_true")]
        wait_for_completion: bool,
    },

    /// Tween View Element Chapter.
    ///
    /// Applies a tween animation to a view element over time.
    /// Supports various targets like position, scale, color, box size, etc.
    ///
    /// 补间视图元素章节。
    ///
    /// 在一段时间内对视图元素应用补间动画。
    /// 支持各种目标，如位置、缩放、颜色、框大小等。
    TweenViewElement {
        /// Element selector (how to find the target element).
        ///
        /// 元素选择器（如何查找目标元素）。
        selector: ElementSelector,
        /// What property to animate and its target value.
        ///
        /// 要动画的属性及其目标值。
        target: TweenTarget,
        /// Duration of the animation in seconds.
        ///
        /// 动画持续时间（秒）。
        duration: f32,
        /// Easing function for the animation.
        ///
        /// 动画的缓动函数。
        #[serde(default)]
        easing: EasingFunction,
        /// Whether to wait for the tween to complete before continuing.
        ///
        /// 是否在继续之前等待补间完成。
        #[serde(default = "default_true")]
        wait_for_completion: bool,
    },

    /// Simple Wait Chapter.
    ///
    /// 简单的等待章节。
    Wait(f32),

    /// Sequential Chapter Group.
    /// Chapters are executed one after another.
    ///
    /// 顺序执行的章节组。
    /// 章节会一个接一个地执行。
    Sequence(Vec<Chapter>),

    /// Parallel Chapter Group.
    /// All chapters start execution simultaneously.
    /// The group finishes when all child chapters are finished.
    ///
    /// 并行执行的章节组。
    /// 所有章节同时开始执行。
    /// 当所有子章节都完成时，该组才算完成。
    Parallel(Vec<Chapter>),

    /// Set Player State Chapter.
    /// Please note that in Battle, a player entity is not generated by default.
    /// If a player entity is needed to participate in the battle chapter,
    /// it must be generated through this chapter.
    ///
    /// 设置玩家状态的章节。
    /// 请注意，Battle 中，默认不会生成一个玩家实体。
    /// 如果需要玩家实体参与战斗章节，必须通过此章节进行生成。
    SetPlayer(PlayerAction),

    /// Set UI State Chapter.
    ///
    /// 设置 UI 状态的章节。
    SetUI(UIAction),

    /// Modify View Element Chapter.
    ///
    /// Modify properties of view elements at runtime by selector.
    ///
    /// 修改视图元素章节。
    ///
    /// 通过选择器在运行时修改视图元素的属性。
    ModifyViewElement {
        selector: ElementSelector,
        modification: ElementModification,
    },

    /// Set Camera State Chapter.
    ///
    /// 设置 摄像机 状态的章节。
    SetCamera(CameraAction),
}

fn default_true() -> bool {
    true
}

/// Camera Action Enum.
///
/// 摄像机操作枚举。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CameraAction {
    /// Set Camera Position.
    ///
    /// 设置摄像机位置。
    SetPosition(Vec2),

    /// Set Camera Zoom Level.
    ///
    /// 设置摄像机缩放级别。
    SetZoom(f32),

    /// Start Camera Shake Effect.
    ///
    /// 开始摄像机震动效果。
    Shake { duration: f32, intensity: f32 },

    /// Set Camera to Follow Player.
    ///
    /// 设置摄像机跟随玩家。
    FollowPlayer(bool),
}

/// UI Action Enum.
///
/// UI 操作枚举。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum UIAction {
    LoadLayout(String),
    Show(String),
    Hide(String),
    SetText { id: String, content: String },
    SetVariable { name: String, value: String },
    PlayAnimation { id: String, clip: String },
}

/// Player Action Enum.
///
/// 操作玩家的一系列枚举项。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PlayerAction {
    /// Set Mode for the player.
    /// Mode refers to different behavioral states defined in the Character Asset,
    /// such as "movable", "jumpable", "shootable", etc.
    /// The String references the mode names defined in the Character Asset.
    ///
    /// 设置玩家的模式。
    /// 模式 即 角色资产 中定义的 不同行为状态。
    /// 如“可移动”、“可跳跃”、“可射击”等。
    /// String 引用的是 角色资产 中定义的 模式 名称。
    SetMode(Vec<String>),

    /// Spawn a new player entity based on a config file.
    ///
    /// 根据配置文件生成一个新的玩家实体。
    Spawn { config_path: String, position: Vec2 },

    /// Teleport the player to a specified position.
    ///
    /// 将玩家传送到指定位置。
    Teleport(Vec2),

    /// Set the active state of the player.
    ///
    /// 设置玩家的激活状态。
    SetActive(bool),

    /// Despawn the player entity.
    ///
    /// 销毁玩家实体。
    Despawn,
}

/// Element Selector - specifies which elements to target.
///
/// 元素选择器 - 指定目标元素。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ElementSelector {
    /// Select by fully qualified name (namespace::name).
    ///
    /// 通过完全限定名（namespace::name）选择。
    FullName(String),

    /// Select by local name within the current layout's namespace.
    ///
    /// 在当前布局的命名空间内通过局部名称选择。
    LocalName(String),

    /// Select all elements with a specific tag.
    ///
    /// 选择所有具有特定标签的元素。
    Tag(String),
}

/// Element Modification - property changes to apply.
///
/// 元素修改 - 要应用的属性更改。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ElementModification {
    /// Set sprite texture path.
    ///
    /// 设置精灵贴图路径。
    SetTexture(String),

    /// Set position (x, y, z).
    ///
    /// Each coordinate can be either a static float or a dynamic expression string.
    /// Expressions support sin/cos/snap and random() functions.
    /// Use "@current" to keep the existing coordinate value.
    ///
    /// 设置位置 (x, y, z)。
    ///
    /// 每个坐标可以是静态浮点数或动态表达式字符串。
    /// 表达式支持 sin/cos/snap 和 random() 函数。
    /// 使用 "@current" 保持现有坐标值。
    SetPosition(Val<f32>, Val<f32>, Val<f32>),

    /// Set scale (x, y, z).
    ///
    /// Each coordinate can be either a static float or a dynamic expression string.
    ///
    /// 设置缩放 (x, y, z)。
    ///
    /// 每个坐标可以是静态浮点数或动态表达式字符串。
    SetScale(Val<f32>, Val<f32>, Val<f32>),

    /// Set color (r, g, b, a) - values 0.0 to 1.0.
    ///
    /// Each channel can be either a static float or a dynamic expression string.
    ///
    /// 设置颜色 (r, g, b, a) - 值范围 0.0 至 1.0。
    ///
    /// 每个通道可以是静态浮点数或动态表达式字符串。
    SetColor(Val<f32>, Val<f32>, Val<f32>, Val<f32>),

    /// Set visibility (true = visible, false = hidden).
    ///
    /// Can be either a static bool or a dynamic expression string.
    ///
    /// 设置可见性（true = 可见，false = 隐藏）。
    ///
    /// 可以是静态布尔值或动态表达式字符串。
    SetVisibility(Val<bool>),

    /// Set UIBox dimensions (width, height).
    ///
    /// 设置 UIBox 尺寸（宽度，高度）。
    SetBoxSize(Val<f32>, Val<f32>),

    /// Undo last modification for this element.
    ///
    /// 撤销此元素的最后一次修改。
    Undo,

    /// Redo last undone modification for this element.
    ///
    /// 重做此元素最后撤销的修改。
    Redo,

    /// Reset element to its original spawn state.
    ///
    /// 将元素重置为其原始生成状态。
    Reset,
}

/// Easing function for tween animations.
///
/// 缓动函数用于补间动画。
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum EasingFunction {
    /// Linear interpolation (no easing).
    ///
    /// 线性插值（无缓动）。
    #[default]
    Linear,
    /// Quadratic ease in.
    ///
    /// 二次缓入。
    QuadIn,
    /// Quadratic ease out.
    ///
    /// 二次缓出。
    QuadOut,
    /// Quadratic ease in-out.
    ///
    /// 二次缓入缓出。
    QuadInOut,
    /// Cubic ease in.
    ///
    /// 三次缓入。
    CubicIn,
    /// Cubic ease out.
    ///
    /// 三次缓出。
    CubicOut,
    /// Cubic ease in-out.
    ///
    /// 三次缓入缓出。
    CubicInOut,
    /// Sine ease in.
    ///
    /// 正弦缓入。
    SineIn,
    /// Sine ease out.
    ///
    /// 正弦缓出。
    SineOut,
    /// Sine ease in-out.
    ///
    /// 正弦缓入缓出。
    SineInOut,
    /// Exponential ease in.
    ///
    /// 指数缓入。
    ExpoIn,
    /// Exponential ease out.
    ///
    /// 指数缓出。
    ExpoOut,
    /// Exponential ease in-out.
    ///
    /// 指数缓入缓出。
    ExpoInOut,
    /// Elastic ease in.
    ///
    /// 弹性缓入。
    ElasticIn,
    /// Elastic ease out.
    ///
    /// 弹性缓出。
    ElasticOut,
    /// Elastic ease in-out.
    ///
    /// 弹性缓入缓出。
    ElasticInOut,
    /// Bounce ease in.
    ///
    /// 弹跳缓入。
    BounceIn,
    /// Bounce ease out.
    ///
    /// 弹跳缓出。
    BounceOut,
    /// Bounce ease in-out.
    ///
    /// 弹跳缓入缓出。
    BounceInOut,
    /// Back ease in.
    ///
    /// 回退缓入。
    BackIn,
    /// Back ease out.
    ///
    /// 回退缓出。
    BackOut,
    /// Back ease in-out.
    ///
    /// 回退缓入缓出。
    BackInOut,
}

/// Tween target property to animate.
///
/// Supports three syntaxes for BoxSize:
/// - `BoxSize(to: (w, h))` - animate to target value from current
/// - `BoxSize((w, h))` - shorthand for the above (note: needs double parentheses)
/// - `BoxSize(from: (w1, h1), to: (w2, h2))` - animate from explicit start to end
///
/// 要动画的补间目标属性。
///
/// BoxSize 支持三种语法：
/// - `BoxSize(to: (w, h))` - 从当前值动画到目标值
/// - `BoxSize((w, h))` - 上述语法的简写（注意：需要双括号）
/// - `BoxSize(from: (w1, h1), to: (w2, h2))` - 从显式起点动画到终点
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TweenTarget {
    /// Animate position (x, y, z).
    ///
    /// 动画位置 (x, y, z)。
    Position {
        #[serde(default)]
        from: Option<Vec3Tuple>,
        to: Vec3Tuple,
    },
    /// Animate scale (x, y, z).
    ///
    /// 动画缩放 (x, y, z)。
    Scale {
        #[serde(default)]
        from: Option<Vec3Tuple>,
        to: Vec3Tuple,
    },
    /// Animate color (r, g, b, a).
    ///
    /// 动画颜色 (r, g, b, a)。
    Color {
        #[serde(default)]
        from: Option<ColorTuple>,
        to: ColorTuple,
    },
    /// Animate UIBox size (width, height).
    /// Syntax: `BoxSize(to: (w, h))` or `BoxSize(from: (w1, h1), to: (w2, h2))`
    ///
    /// 动画 UIBox 尺寸 (宽度, 高度)。
    BoxSize {
        #[serde(default)]
        from: Option<Vec2Tuple>,
        to: Vec2Tuple,
    },
    /// Animate rotation (radians around Z axis).
    ///
    /// 动画旋转（绕 Z 轴的弧度）。
    Rotation {
        #[serde(default)]
        from: Option<Val<f32>>,
        to: Val<f32>,
    },
    /// Animate alpha/opacity only.
    ///
    /// 仅动画透明度。
    Alpha {
        #[serde(default)]
        from: Option<Val<f32>>,
        to: Val<f32>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_val_f32_parse() {
        // Test that Val<f32> can parse both static and expr
        let ron_static = "130.0";
        let ron_expr = r#""@current""#;

        let result_static: Result<Val<f32>, _> = ron::from_str(ron_static);
        let result_expr: Result<Val<f32>, _> = ron::from_str(ron_expr);

        println!("Static parse result: {:?}", result_static);
        println!("Expr parse result: {:?}", result_expr);

        assert!(
            result_static.is_ok(),
            "Failed to parse static: {:?}",
            result_static.err()
        );
        assert!(
            result_expr.is_ok(),
            "Failed to parse expr: {:?}",
            result_expr.err()
        );
    }

    #[test]
    fn test_vec2_tuple_parse() {
        let ron = r#"("@current", 130.0)"#;
        let result: Result<Vec2Tuple, _> = ron::from_str(ron);
        println!("Vec2Tuple parse result: {:?}", result);
        assert!(
            result.is_ok(),
            "Failed to parse Vec2Tuple: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_tween_view_element_chapter() {
        // Use the struct syntax with `to:`
        let ron = r#"TweenViewElement(
            selector: LocalName("BattleBox"),
            target: BoxSize(to: ("@current", 130.0)),
            duration: 0.5,
            easing: QuadInOut,
            wait_for_completion: true,
        )"#;
        let result: Result<Chapter, _> = ron::from_str(ron);
        match &result {
            Ok(v) => println!("TweenViewElement OK: {:?}", v),
            Err(e) => println!("TweenViewElement ERR: {}", e),
        }
        assert!(
            result.is_ok(),
            "Failed to parse TweenViewElement: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_tween_target_box_size() {
        // Use the struct syntax with `to:`
        let ron = r#"BoxSize(to: ("@current", 130.0))"#;
        let result: Result<TweenTarget, _> = ron::from_str(ron);
        match &result {
            Ok(v) => println!("TweenTarget BoxSize OK: {:?}", v),
            Err(e) => println!("TweenTarget BoxSize ERR: {}", e),
        }
        assert!(
            result.is_ok(),
            "Failed to parse TweenTarget::BoxSize: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_tween_target_box_size_with_from() {
        // Note: RON requires Some() wrapper for Option values (unless using #![enable(implicit_some)])
        let ron = r#"BoxSize(from: Some((100.0, 100.0)), to: (566.0, "@current"))"#;
        let result: Result<TweenTarget, _> = ron::from_str(ron);
        match &result {
            Ok(v) => println!("TweenTarget BoxSize with from OK: {:?}", v),
            Err(e) => println!("TweenTarget BoxSize with from ERR: {}", e),
        }
        assert!(
            result.is_ok(),
            "Failed to parse TweenTarget::BoxSize with from: {:?}",
            result.err()
        );
    }
}

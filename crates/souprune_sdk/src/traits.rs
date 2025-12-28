//! Defines the `Behavior` trait that mod developers implement to create game logic.
//! It abstracts away the low-level C ABI callbacks into standard Rust lifecycle methods like `on_update`.
//!
//! 定义模组开发者需要实现的 `Behavior` trait 以创建游戏逻辑。
//! 它将底层的 C ABI 回调抽象为标准的 Rust 生命周期方法，如 `on_update`。

use crate::context::{BulletContext, BulletOutput, Context};

/// All Behaviors must implement this trait.
/// Corresponds to the lifecycle in api_design.md.
///
/// 所有的 Behavior 都要实现这个特征
/// 对应 api_design.md 里的生命周期
pub trait Behavior {
    /// Initialization
    /// 初始化
    fn on_enter(&mut self, _context: &mut Context) {}

    /// Per-frame update
    /// 每帧更新
    fn on_update(&mut self, context: &mut Context, dt: f32);

    /// Exit cleanup
    /// 退出清理
    fn on_exit(&mut self, _context: &mut Context) {}
}

/// Trait for danmaku (bullet pattern) behaviors.
/// Similar to Behavior but operates on bullet context instead of player context.
///
/// 弹幕行为的 Trait。
/// 类似于 Behavior，但操作弹幕上下文而非玩家上下文。
pub trait DanmakuBehavior {
    /// Called once when the bullet is spawned.
    /// Use this to capture initial state (e.g., player position for homing).
    ///
    /// 弹幕生成时调用一次。
    /// 用于捕获初始状态（如：自机狙的玩家位置）。
    fn on_enter(&mut self, _context: &BulletContext) {}

    /// Called every frame to compute bullet movement.
    /// Returns the position offset and rotation for this frame.
    ///
    /// 每帧调用以计算弹幕移动。
    /// 返回本帧的位置偏移和旋转。
    fn on_update(&mut self, context: &BulletContext) -> BulletOutput;

    /// Called when the bullet is despawned.
    ///
    /// 弹幕销毁时调用。
    fn on_exit(&mut self) {}
}

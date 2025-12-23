//! Defines the `SoulMode` trait that mod developers implement to create game logic.
//! It abstracts away the low-level C ABI callbacks into standard Rust lifecycle methods like `on_update`.
//!
//! 定义模组开发者需要实现的 `SoulMode` trait 以创建游戏逻辑。
//! 它将底层的 C ABI 回调抽象为标准的 Rust 生命周期方法，如 `on_update`。

use crate::context::Context;

/// All Soul Modes must implement this trait.
/// Corresponds to the lifecycle in api_design.md.
///
/// 所有的 Soul Mode 都要实现这个特征
/// 对应 api_design.md 里的生命周期
pub trait SoulMode {
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

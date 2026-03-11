//! Defines the `Behavior` and `DanmakuBehavior` traits that mod developers implement.
//!
//! 定义模组开发者需要实现的 `Behavior` 和 `DanmakuBehavior` trait。

use crate::context::Context;
use crate::{BulletContext, BulletOutput};

/// Player/entity behavior trait.
///
/// Behavior controls entity actions using host-api (input, velocity).
/// Unlike DanmakuBehavior, it accesses engine state through the global
/// host-api imports (via Context), not through per-instance parameters.
///
/// 玩家/实体行为 trait。
/// 通过 host-api 全局导入（经由 Context）访问引擎状态，
/// 与 DanmakuBehavior 不同，不依赖每实例参数。
pub trait Behavior {
    fn on_enter(&mut self, _context: &mut Context) {}
    fn on_update(&mut self, context: &mut Context, dt: f32);
    fn on_exit(&mut self, _context: &mut Context) {}
}

/// Danmaku (bullet pattern) behavior trait.
///
/// Each callback receives a BulletContext with bullet-specific state
/// (position, elapsed time, props). This is different from Behavior,
/// which uses the global host-api for engine interaction.
///
/// 弹幕行为 trait。
/// 每次回调接收 BulletContext（含子弹位置、时间、属性），
/// 与 Behavior 不同，依赖每实例上下文而非全局 host-api。
pub trait DanmakuBehavior {
    fn on_enter(&mut self, _ctx: &BulletContext) {}
    fn on_update(&mut self, context: &BulletContext) -> BulletOutput;
    fn on_exit(&mut self) {}
}

/// No-op behavior used when an unknown ID is requested.
#[doc(hidden)]
pub struct NoopBehavior;

impl Behavior for NoopBehavior {
    fn on_update(&mut self, _context: &mut Context, _dt: f32) {}
}

/// No-op danmaku used when an unknown ID is requested.
#[doc(hidden)]
pub struct NoopDanmaku;

impl DanmakuBehavior for NoopDanmaku {
    fn on_update(&mut self, _context: &BulletContext) -> BulletOutput {
        BulletOutput::ZERO
    }
}

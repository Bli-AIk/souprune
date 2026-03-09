//! Defines the `Behavior` and `DanmakuBehavior` traits that mod developers implement.
//!
//! 定义模组开发者需要实现的 `Behavior` 和 `DanmakuBehavior` trait。

use crate::context::Context;
use crate::{BulletContext, BulletOutput};

/// Player/entity behavior trait.
///
/// 玩家/实体行为 trait。
pub trait Behavior {
    fn on_enter(&mut self, _context: &mut Context) {}
    fn on_update(&mut self, context: &mut Context, dt: f32);
    fn on_exit(&mut self, _context: &mut Context) {}
}

/// Danmaku (bullet pattern) behavior trait.
///
/// 弹幕行为 trait。
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

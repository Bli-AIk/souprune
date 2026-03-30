//! Defines the `Behavior`, `DanmakuBehavior`, and `SpawnPatternBehavior` traits
//! that mod developers implement.
//!
//! 定义模组开发者需要实现的 `Behavior`、`DanmakuBehavior`
//! 和 `SpawnPatternBehavior` trait。

use crate::context::Context;
use crate::{BulletContext, BulletOutput, SpawnContext, SpawnOutput, SpawnParam};

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

/// Spawn pattern trait — generates a list of spawn points.
///
/// Unlike DanmakuBehavior (per-frame), spawn patterns are invoked once
/// per TimelineEvent to compute where bullets should appear.
///
/// 生成模式 trait — 生成一组生成点。
/// 与弹幕行为（逐帧）不同，生成模式在 TimelineEvent 触发时
/// 一次性调用以计算子弹生成位置。
pub trait SpawnPatternBehavior {
    fn generate(&self, ctx: &SpawnContext, params: &[SpawnParam]) -> Vec<SpawnOutput>;
}

/// A key-value parameter carried by a custom FRE action.
///
/// 自定义 FRE action 携带的键值参数。
#[derive(Debug, Clone)]
pub struct ActionParam {
    pub name: String,
    pub value: String,
}

/// Custom FRE action handler — optional mod capability.
///
/// Implement this trait to process FRE Custom actions from WASM.
/// Use `Context` to read/write facts and emit further events.
///
/// 自定义 FRE action 处理器 — 可选模组能力。
/// 实现此 trait 以处理来自 WASM 的 FRE Custom action。
/// 通过 `Context` 读写 Fact 和发出更多事件。
pub trait CustomActionHandler {
    /// Return all FRE custom action types this handler responds to.
    fn handled_actions() -> Vec<String>
    where
        Self: Sized;

    /// Handle a custom FRE action. Return `true` if the action was consumed.
    fn handle_action(&self, ctx: &Context, action_type: &str, params: &[ActionParam]) -> bool;
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

/// No-op pattern used when an unknown ID is requested.
#[doc(hidden)]
pub struct NoopPattern;

impl SpawnPatternBehavior for NoopPattern {
    fn generate(&self, _ctx: &SpawnContext, _params: &[SpawnParam]) -> Vec<SpawnOutput> {
        vec![]
    }
}

/// No-op custom action handler — used when `export_mod!` omits `custom_actions`.
#[doc(hidden)]
#[derive(Default)]
pub struct NoopCustomActionHandler;

impl CustomActionHandler for NoopCustomActionHandler {
    fn handled_actions() -> Vec<String> {
        vec![]
    }

    fn handle_action(&self, _ctx: &Context, _action_type: &str, _params: &[ActionParam]) -> bool {
        false
    }
}

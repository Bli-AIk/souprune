//! Defines the `Behavior`, `DanmakuBehavior`, and `SpawnPatternBehavior` traits
//! that mod developers implement.
//!
//! 定义模组开发者需要实现的 `Behavior`、`DanmakuBehavior`
//! 和 `SpawnPatternBehavior` trait。

use crate::context::Context;
use crate::{
    BulletContext, BulletOutput, InputCommand, InputContextId, SpawnContext, SpawnOutput,
    SpawnParam,
};

/// Player/entity behavior trait.
///
/// Implement this to define behaviors driven by host-api (input, facts, events).
/// Velocity is optional — pure-logic behaviors (e.g. fight bar, UI controllers)
/// can skip `kinematics()` entirely; the host gracefully handles missing velocity.
///
/// 玩家/实体行为 trait。
/// 实现此 trait 定义通过 host-api（输入、Fact、事件）驱动的行为。
/// 速度是可选的 — 纯逻辑行为（如攻击条、UI 控制器）可省略 `kinematics()` 调用。
pub trait Behavior {
    fn on_enter(&mut self, _context: &mut Context) {}
    fn on_input(
        &mut self,
        _context: &mut Context,
        _input_context: InputContextId,
        _command: InputCommand,
    ) {
    }
    fn on_update(&mut self, context: &mut Context, dt: f32);
    fn on_exit(&mut self, _context: &mut Context) {}
}

/// Danmaku (bullet pattern) behavior trait.
///
/// Each callback receives a BulletContext with bullet-specific state
/// (position, elapsed time, props). This is different from Behavior,
/// which uses the global host-api for framework interaction.
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

// === E4: Mode Lifecycle ===

/// Mode lifecycle trait — respond to mode transitions.
///
/// Implement this to have your mod react to project-declared mode changes.
///
/// 模式生命周期 trait — 响应模式切换。
/// 实现此 trait 让模组对模式变化做出响应（如进入战斗模式）。
pub trait ModeLifecycle {
    /// Called when the host enters a new mode.
    fn on_mode_enter(&self, _mode: &str) {}

    /// Called when the host exits the current mode.
    fn on_mode_exit(&self, _mode: &str) {}

    /// Called when the sub-state changes within the current mode.
    fn on_sub_state_change(&self, _mode: &str, _old_state: &str, _new_state: &str) {}
}

/// No-op mode lifecycle — used when `export_mod!` omits `mode_lifecycle`.
#[doc(hidden)]
#[derive(Default)]
pub struct NoopModeLifecycle;

impl ModeLifecycle for NoopModeLifecycle {}

// === E5: Rule Provider ===

/// A FRE rule modification definition for rules provided by WASM mods.
#[derive(Debug, Clone)]
pub struct RuleModification {
    pub key: String,
    pub op: String,
    pub value: String,
}

/// A FRE rule action definition for rules provided by WASM mods.
#[derive(Debug, Clone)]
pub struct RuleAction {
    pub action_type: String,
    pub params: Vec<(String, String)>,
}

/// A complete FRE rule definition provided by a WASM mod.
#[derive(Debug, Clone)]
pub struct RuleDef {
    pub id: String,
    pub priority: i32,
    pub trigger_event: String,
    pub conditions: Vec<String>,
    pub actions: Vec<RuleAction>,
    pub modifications: Vec<RuleModification>,
    pub outputs: Vec<String>,
}

/// Rule provider trait — supply FRE rules programmatically.
///
/// Implement this to have your mod provide FRE rules that the framework
/// registers into the layered rule registry at mod load time.
///
/// 规则提供者 trait — 程序化提供 FRE 规则。
/// 实现此 trait 让模组在加载时向框架注册 FRE 规则。
pub trait RuleProvider {
    /// Return all FRE rule definitions this mod provides.
    fn list_rules() -> Vec<RuleDef>
    where
        Self: Sized;
}

/// No-op rule provider — used when `export_mod!` omits `rule_provider`.
#[doc(hidden)]
#[derive(Default)]
pub struct NoopRuleProvider;

impl RuleProvider for NoopRuleProvider {
    fn list_rules() -> Vec<RuleDef> {
        vec![]
    }
}

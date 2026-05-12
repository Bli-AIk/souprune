//! WASM Modding SDK for SoupRune.
//!
//! Provides the guest-side bindings for the WASM Component Model interface.
//! Mod developers implement the `Behavior`, `DanmakuBehavior`, and
//! `SpawnPatternBehavior` traits, then use `export_mod!` to register them.
//!
//! SoupRune 的 WASM 模组开发 SDK。
//! 模组开发者实现 `Behavior`、`DanmakuBehavior` 和 `SpawnPatternBehavior` trait，
//! 然后使用 `export_mod!` 注册实现。
//!
//! ## Key Concepts / 核心概念
//!
//! - **Typed Facts**: Facts are typed values (`FactValue`: Bool, Int, Float, Text).
//!   Use `ctx.get_fact_bool()`, `ctx.set_fact_float()`, etc. for type-safe access.
//!   类型化 Fact：通过 `get_fact_bool/int/float/string` 和对应 set 方法进行类型安全访问。
//!
//! - **Optional Velocity**: Behaviors can be pure logic (e.g. FightBar) without
//!   any movement component. Calling `ctx.kinematics().set_velocity()` is a no-op
//!   when the entity has no `BehaviorVelocity`.
//!   可选速度：纯逻辑行为无需运动组件，调用 set_velocity 在无组件时安全忽略。
//!
//! - **Custom Actions**: Implement `CustomActionHandler` to handle FRE custom
//!   actions from WASM, enabling data-driven game logic.
//!   自定义 Action：实现 `CustomActionHandler` 处理 FRE 自定义动作。
//!
//! # Example
//!
//! ```ignore
//! use souprune_sdk::prelude::*;
//!
//! struct MySoul { counter: u32 }
//!
//! impl Behavior for MySoul {
//!     fn on_update(&mut self, ctx: &mut Context, dt: f32) {
//!         ctx.log("hello from wasm!");
//!         let speed = ctx.get_fact_float("move:speed").unwrap_or(100.0);
//!         if ctx.input().pressed(Action::Right) {
//!             ctx.kinematics().set_velocity(speed as f32, 0.0);
//!         }
//!     }
//! }
//!
//! export_mod! {
//!     behaviors: [("my_soul", MySoul, || MySoul { counter: 0 })],
//! }
//! ```

pub mod context;
pub mod traits;
pub mod types;

pub use context::{Context, FactValue, Vec2};
pub use traits::{
    ActionParam, Behavior, CustomActionHandler, DanmakuBehavior, ModeLifecycle, RuleAction,
    RuleDef, RuleModification, RuleProvider, SpawnPatternBehavior,
};
pub use types::{
    BulletContext, BulletOutput, Direction, InputCommand, InputContextId, InputEffect,
    InputEnvelope, InputResult, InputTarget, SpawnContext, SpawnOutput, SpawnParam,
};

// Generate guest bindings from the WIT interface
wit_bindgen::generate!({
    path: "wit",
    world: "souprune-mod",
    pub_export_macro: true,
});

// Re-export generated types under a `wit` module for advanced use
pub mod wit {
    pub use super::exports::souprune::plugin::behavior::GuestBehaviorInstance;
    pub use super::exports::souprune::plugin::danmaku::GuestDanmakuInstance;
    pub use super::exports::souprune::plugin::mode_lifecycle::Guest as GuestModeLifecycle;
    pub use super::exports::souprune::plugin::rule_provider::Guest as GuestRuleProvider;
    pub use super::exports::souprune::plugin::spawn_pattern::GuestPatternInstance;
    pub use super::souprune::plugin::host_api;
}

/// Semantic input action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
    Menu,
}

impl Action {
    fn to_wit(self) -> souprune::plugin::host_api::Action {
        use souprune::plugin::host_api::Action as WitAction;
        match self {
            Self::Up => WitAction::Up,
            Self::Down => WitAction::Down,
            Self::Left => WitAction::Left,
            Self::Right => WitAction::Right,
            Self::Confirm => WitAction::Confirm,
            Self::Cancel => WitAction::Cancel,
            Self::Menu => WitAction::Menu,
        }
    }
}

/// Convenience prelude for mod developers.
pub mod prelude {
    pub use crate::context::{Context, FactValue, Vec2};
    pub use crate::traits::{
        ActionParam, Behavior, CustomActionHandler, DanmakuBehavior, ModeLifecycle, RuleAction,
        RuleDef, RuleModification, RuleProvider, SpawnPatternBehavior,
    };
    pub use crate::{
        Action, BulletContext, BulletOutput, Direction, InputCommand, InputContextId, InputEffect,
        InputEnvelope, InputResult, InputTarget, SpawnContext, SpawnOutput, SpawnParam, export_mod,
    };
}

/// Register behavior, danmaku, spawn pattern, and custom action handler implementations
/// and export them as a WASM component.
///
/// 注册行为、弹幕、生成模式和自定义 action 处理器实现，并将它们导出为 WASM 组件。
///
/// # Syntax
///
/// ```ignore
/// // Full form with custom action handler:
/// export_mod! {
///     behaviors: [("id", Type, || Type { field: value })],
///     danmaku: [("id", Type, || Type::new())],
///     patterns: [("id", Type, || Type::new())],
///     custom_actions: MyActionHandler,
/// }
///
/// // Without custom actions (defaults to no-op handler):
/// export_mod! {
///     behaviors: [("id", Type)],
///     danmaku: [("id", Type)],
///     patterns: [("id", Type)],
/// }
/// ```
#[macro_export]
macro_rules! export_mod {
    // =========================================================================
    // PRIMARY FORM — generates all code (requires all explicit types)
    // =========================================================================
    (
        behaviors: [ $( ($b_id:literal, $b_type:ty, $b_ctor:expr) ),* $(,)? ] $(,)?
        danmaku: [ $( ($d_id:literal, $d_type:ty, $d_ctor:expr) ),* $(,)? ] $(,)?
        patterns: [ $( ($p_id:literal, $p_type:ty, $p_ctor:expr) ),* $(,)? ] $(,)?
        custom_actions: $ca_type:ty,
        mode_lifecycle: $ml_type:ty,
        rule_provider: $rp_type:ty $(,)?
    ) => {
        // --- Behavior resource wrapper ---
        struct WasmBehaviorInstance {
            inner: std::cell::RefCell<Box<dyn $crate::Behavior>>,
        }

        impl $crate::wit::GuestBehaviorInstance for WasmBehaviorInstance {
            fn new(id: String) -> Self {
                let inner: Box<dyn $crate::Behavior> = match id.as_str() {
                    $( $b_id => Box::new($b_ctor()), )*
                    other => {
                        eprintln!("[souprune_sdk] WARNING: unknown behavior ID \"{other}\", using NoopBehavior");
                        Box::new($crate::traits::NoopBehavior)
                    }
                };
                Self { inner: std::cell::RefCell::new(inner) }
            }

            fn on_enter(&self) {
                let mut ctx = $crate::context::Context::new();
                self.inner.borrow_mut().on_enter(&mut ctx);
            }

            fn on_input(
                &self,
                input_context: $crate::exports::souprune::plugin::behavior::InputContextId,
                command: $crate::exports::souprune::plugin::behavior::InputCommand,
            ) {
                let mut ctx = $crate::context::Context::new();
                let input_context = $crate::InputContextId::from_wit(&input_context);
                let command = $crate::InputCommand::from_wit(command);
                self.inner.borrow_mut().on_input(&mut ctx, input_context, command);
            }

            fn on_update(&self, delta_time: f32) {
                let mut ctx = $crate::context::Context::new();
                self.inner.borrow_mut().on_update(&mut ctx, delta_time);
            }

            fn on_exit(&self) {
                let mut ctx = $crate::context::Context::new();
                self.inner.borrow_mut().on_exit(&mut ctx);
            }
        }

        // --- Danmaku resource wrapper ---
        struct WasmDanmakuInstance {
            inner: std::cell::RefCell<Box<dyn $crate::DanmakuBehavior>>,
        }

        impl $crate::wit::GuestDanmakuInstance for WasmDanmakuInstance {
            fn new(id: String) -> Self {
                let inner: Box<dyn $crate::DanmakuBehavior> = match id.as_str() {
                    $( $d_id => Box::new($d_ctor()), )*
                    other => {
                        eprintln!("[souprune_sdk] WARNING: unknown danmaku ID \"{other}\", using NoopDanmaku");
                        Box::new($crate::traits::NoopDanmaku)
                    }
                };
                Self { inner: std::cell::RefCell::new(inner) }
            }

            fn on_enter(
                &self,
                ctx: $crate::exports::souprune::plugin::danmaku::BulletContext,
            ) {
                let bc = $crate::BulletContext::from_wit(&ctx);
                self.inner.borrow_mut().on_enter(&bc);
            }

            fn on_update(
                &self,
                ctx: $crate::exports::souprune::plugin::danmaku::BulletContext,
            ) -> $crate::exports::souprune::plugin::danmaku::BulletOutput {
                let bc = $crate::BulletContext::from_wit(&ctx);
                self.inner.borrow_mut().on_update(&bc).to_wit()
            }

            fn on_exit(&self) {
                self.inner.borrow_mut().on_exit();
            }
        }

        // --- Pattern resource wrapper ---
        struct WasmPatternInstance {
            inner: Box<dyn $crate::SpawnPatternBehavior>,
        }

        impl $crate::wit::GuestPatternInstance for WasmPatternInstance {
            fn new(id: String) -> Self {
                let inner: Box<dyn $crate::SpawnPatternBehavior> = match id.as_str() {
                    $( $p_id => Box::new($p_ctor()), )*
                    other => {
                        eprintln!("[souprune_sdk] WARNING: unknown pattern ID \"{other}\", using NoopPattern");
                        Box::new($crate::traits::NoopPattern)
                    }
                };
                Self { inner }
            }

            fn generate(
                &self,
                ctx: $crate::exports::souprune::plugin::spawn_pattern::SpawnContext,
                params: Vec<$crate::exports::souprune::plugin::spawn_pattern::PatternParam>,
            ) -> Vec<$crate::exports::souprune::plugin::spawn_pattern::SpawnPoint> {
                let sc = $crate::SpawnContext::from_wit(&ctx);
                let sp: Vec<$crate::SpawnParam> = params.iter().map($crate::SpawnParam::from_wit).collect();
                self.inner.generate(&sc, &sp).into_iter().map(|o| o.to_wit()).collect()
            }
        }

        // --- Module-level exports ---
        struct ModComponent;

        impl $crate::exports::souprune::plugin::behavior::Guest for ModComponent {
            type BehaviorInstance = WasmBehaviorInstance;

            fn list_behaviors() -> Vec<String> {
                vec![ $( $b_id.to_string(), )* ]
            }
        }

        impl $crate::exports::souprune::plugin::danmaku::Guest for ModComponent {
            type DanmakuInstance = WasmDanmakuInstance;

            fn list_algorithms() -> Vec<String> {
                vec![ $( $d_id.to_string(), )* ]
            }
        }

        impl $crate::exports::souprune::plugin::spawn_pattern::Guest for ModComponent {
            type PatternInstance = WasmPatternInstance;

            fn list_patterns() -> Vec<String> {
                vec![ $( $p_id.to_string(), )* ]
            }
        }

        impl $crate::exports::souprune::plugin::custom_action_handler::Guest for ModComponent {
            fn list_handled_actions() -> Vec<String> {
                <$ca_type as $crate::CustomActionHandler>::handled_actions()
            }

            fn handle_action(
                action_type: String,
                params: Vec<$crate::exports::souprune::plugin::custom_action_handler::ActionParam>,
            ) -> bool {
                let handler = <$ca_type as Default>::default();
                let sdk_params: Vec<$crate::ActionParam> = params
                    .into_iter()
                    .map(|p| $crate::ActionParam { name: p.name, value: p.value })
                    .collect();
                let ctx = $crate::context::Context::new();
                handler.handle_action(&ctx, &action_type, &sdk_params)
            }
        }

        impl $crate::exports::souprune::plugin::mode_lifecycle::Guest for ModComponent {
            fn on_mode_enter(mode: String) {
                let handler = <$ml_type as Default>::default();
                <$ml_type as $crate::ModeLifecycle>::on_mode_enter(&handler, &mode);
            }

            fn on_mode_exit(mode: String) {
                let handler = <$ml_type as Default>::default();
                <$ml_type as $crate::ModeLifecycle>::on_mode_exit(&handler, &mode);
            }

            fn on_sub_state_change(mode: String, old_state: String, new_state: String) {
                let handler = <$ml_type as Default>::default();
                <$ml_type as $crate::ModeLifecycle>::on_sub_state_change(
                    &handler, &mode, &old_state, &new_state,
                );
            }
        }

        impl $crate::exports::souprune::plugin::rule_provider::Guest for ModComponent {
            fn list_rules() -> Vec<$crate::exports::souprune::plugin::rule_provider::RuleDef> {
                let rules = <$rp_type as $crate::RuleProvider>::list_rules();
                rules
                    .into_iter()
                    .map(|r| $crate::exports::souprune::plugin::rule_provider::RuleDef {
                        id: r.id,
                        priority: r.priority,
                        trigger_event: r.trigger_event,
                        conditions: r.conditions,
                        actions: r.actions
                            .into_iter()
                            .map(|a| $crate::exports::souprune::plugin::rule_provider::RuleAction {
                                action_type: a.action_type,
                                params: a.params,
                            })
                            .collect(),
                        modifications: r.modifications
                            .into_iter()
                            .map(|m| $crate::exports::souprune::plugin::rule_provider::RuleModification {
                                key: m.key,
                                op: m.op,
                                value: m.value,
                            })
                            .collect(),
                        outputs: r.outputs,
                    })
                    .collect()
            }
        }

        $crate::export!(ModComponent with_types_in $crate);
    };

    // =========================================================================
    // ALL REMAINING FORMS — delegate to primary with defaults for new fields
    // =========================================================================

    // custom_actions only (no mode_lifecycle/rule_provider) — backward compatible
    (
        behaviors: [ $( ($b_id:literal, $b_type:ty, $b_ctor:expr) ),* $(,)? ] $(,)?
        danmaku: [ $( ($d_id:literal, $d_type:ty, $d_ctor:expr) ),* $(,)? ] $(,)?
        patterns: [ $( ($p_id:literal, $p_type:ty, $p_ctor:expr) ),* $(,)? ] $(,)?
        custom_actions: $ca_type:ty $(,)?
    ) => {
        $crate::export_mod! {
            behaviors: [ $( ($b_id, $b_type, $b_ctor), )* ],
            danmaku: [ $( ($d_id, $d_type, $d_ctor), )* ],
            patterns: [ $( ($p_id, $p_type, $p_ctor), )* ],
            custom_actions: $ca_type,
            mode_lifecycle: $crate::traits::NoopModeLifecycle,
            rule_provider: $crate::traits::NoopRuleProvider,
        }
    };

    // 3-tuple: all 3 interfaces, no custom_actions
    (
        behaviors: [ $( ($b_id:literal, $b_type:ty, $b_ctor:expr) ),* $(,)? ] $(,)?
        danmaku: [ $( ($d_id:literal, $d_type:ty, $d_ctor:expr) ),* $(,)? ] $(,)?
        patterns: [ $( ($p_id:literal, $p_type:ty, $p_ctor:expr) ),* $(,)? ] $(,)?
    ) => {
        $crate::export_mod! {
            behaviors: [ $( ($b_id, $b_type, $b_ctor), )* ],
            danmaku: [ $( ($d_id, $d_type, $d_ctor), )* ],
            patterns: [ $( ($p_id, $p_type, $p_ctor), )* ],
            custom_actions: $crate::traits::NoopCustomActionHandler,
            mode_lifecycle: $crate::traits::NoopModeLifecycle,
            rule_provider: $crate::traits::NoopRuleProvider,
        }
    };

    // 3-tuple: behaviors + danmaku only
    (
        behaviors: [ $( ($b_id:literal, $b_type:ty, $b_ctor:expr) ),* $(,)? ] $(,)?
        danmaku: [ $( ($d_id:literal, $d_type:ty, $d_ctor:expr) ),* $(,)? ] $(,)?
    ) => {
        $crate::export_mod! {
            behaviors: [ $( ($b_id, $b_type, $b_ctor), )* ],
            danmaku: [ $( ($d_id, $d_type, $d_ctor), )* ],
            patterns: [],
            custom_actions: $crate::traits::NoopCustomActionHandler,
            mode_lifecycle: $crate::traits::NoopModeLifecycle,
            rule_provider: $crate::traits::NoopRuleProvider,
        }
    };

    // Shorthand: behaviors only (3-tuple)
    (
        behaviors: [ $( ($b_id:literal, $b_type:ty, $b_ctor:expr) ),* $(,)? ] $(,)?
    ) => {
        $crate::export_mod! {
            behaviors: [ $( ($b_id, $b_type, $b_ctor), )* ],
            danmaku: [],
            patterns: [],
            custom_actions: $crate::traits::NoopCustomActionHandler,
            mode_lifecycle: $crate::traits::NoopModeLifecycle,
            rule_provider: $crate::traits::NoopRuleProvider,
        }
    };

    // Shorthand: danmaku only (3-tuple)
    (
        danmaku: [ $( ($d_id:literal, $d_type:ty, $d_ctor:expr) ),* $(,)? ] $(,)?
    ) => {
        $crate::export_mod! {
            behaviors: [],
            danmaku: [ $( ($d_id, $d_type, $d_ctor), )* ],
            patterns: [],
            custom_actions: $crate::traits::NoopCustomActionHandler,
            mode_lifecycle: $crate::traits::NoopModeLifecycle,
            rule_provider: $crate::traits::NoopRuleProvider,
        }
    };

    // Shorthand: patterns only (3-tuple)
    (
        patterns: [ $( ($p_id:literal, $p_type:ty, $p_ctor:expr) ),* $(,)? ] $(,)?
    ) => {
        $crate::export_mod! {
            behaviors: [],
            danmaku: [],
            patterns: [ $( ($p_id, $p_type, $p_ctor), )* ],
            custom_actions: $crate::traits::NoopCustomActionHandler,
            mode_lifecycle: $crate::traits::NoopModeLifecycle,
            rule_provider: $crate::traits::NoopRuleProvider,
        }
    };

    // =========================================================================
    // 2-TUPLE FORMS (Default::default() constructor)
    // =========================================================================

    // 2-tuple: all 3 interfaces with custom_actions
    (
        behaviors: [ $( ($b_id:literal, $b_type:ty) ),* $(,)? ] $(,)?
        danmaku: [ $( ($d_id:literal, $d_type:ty) ),* $(,)? ] $(,)?
        patterns: [ $( ($p_id:literal, $p_type:ty) ),* $(,)? ] $(,)?
        custom_actions: $ca_type:ty $(,)?
    ) => {
        $crate::export_mod! {
            behaviors: [ $( ($b_id, $b_type, || <$b_type as Default>::default()), )* ],
            danmaku: [ $( ($d_id, $d_type, || <$d_type as Default>::default()), )* ],
            patterns: [ $( ($p_id, $p_type, || <$p_type as Default>::default()), )* ],
            custom_actions: $ca_type,
            mode_lifecycle: $crate::traits::NoopModeLifecycle,
            rule_provider: $crate::traits::NoopRuleProvider,
        }
    };

    // 2-tuple: all 3 interfaces, no custom_actions
    (
        behaviors: [ $( ($b_id:literal, $b_type:ty) ),* $(,)? ] $(,)?
        danmaku: [ $( ($d_id:literal, $d_type:ty) ),* $(,)? ] $(,)?
        patterns: [ $( ($p_id:literal, $p_type:ty) ),* $(,)? ] $(,)?
    ) => {
        $crate::export_mod! {
            behaviors: [ $( ($b_id, $b_type, || <$b_type as Default>::default()), )* ],
            danmaku: [ $( ($d_id, $d_type, || <$d_type as Default>::default()), )* ],
            patterns: [ $( ($p_id, $p_type, || <$p_type as Default>::default()), )* ],
            custom_actions: $crate::traits::NoopCustomActionHandler,
            mode_lifecycle: $crate::traits::NoopModeLifecycle,
            rule_provider: $crate::traits::NoopRuleProvider,
        }
    };

    // 2-tuple: behaviors + danmaku only
    (
        behaviors: [ $( ($b_id:literal, $b_type:ty) ),* $(,)? ] $(,)?
        danmaku: [ $( ($d_id:literal, $d_type:ty) ),* $(,)? ] $(,)?
    ) => {
        $crate::export_mod! {
            behaviors: [ $( ($b_id, $b_type, || <$b_type as Default>::default()), )* ],
            danmaku: [ $( ($d_id, $d_type, || <$d_type as Default>::default()), )* ],
            patterns: [],
            custom_actions: $crate::traits::NoopCustomActionHandler,
            mode_lifecycle: $crate::traits::NoopModeLifecycle,
            rule_provider: $crate::traits::NoopRuleProvider,
        }
    };

    // 2-tuple shorthand: behaviors only
    (
        behaviors: [ $( ($b_id:literal, $b_type:ty) ),* $(,)? ] $(,)?
    ) => {
        $crate::export_mod! {
            behaviors: [ $( ($b_id, $b_type, || <$b_type as Default>::default()), )* ],
            danmaku: [],
            patterns: [],
            custom_actions: $crate::traits::NoopCustomActionHandler,
            mode_lifecycle: $crate::traits::NoopModeLifecycle,
            rule_provider: $crate::traits::NoopRuleProvider,
        }
    };

    // 2-tuple shorthand: danmaku only
    (
        danmaku: [ $( ($d_id:literal, $d_type:ty) ),* $(,)? ] $(,)?
    ) => {
        $crate::export_mod! {
            behaviors: [],
            danmaku: [ $( ($d_id, $d_type, || <$d_type as Default>::default()), )* ],
            patterns: [],
            custom_actions: $crate::traits::NoopCustomActionHandler,
            mode_lifecycle: $crate::traits::NoopModeLifecycle,
            rule_provider: $crate::traits::NoopRuleProvider,
        }
    };

    // 2-tuple shorthand: patterns only
    (
        patterns: [ $( ($p_id:literal, $p_type:ty) ),* $(,)? ] $(,)?
    ) => {
        $crate::export_mod! {
            behaviors: [],
            danmaku: [],
            patterns: [ $( ($p_id, $p_type, || <$p_type as Default>::default()), )* ],
            custom_actions: $crate::traits::NoopCustomActionHandler,
            mode_lifecycle: $crate::traits::NoopModeLifecycle,
            rule_provider: $crate::traits::NoopRuleProvider,
        }
    };
}

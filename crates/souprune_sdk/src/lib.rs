//! WASM Modding SDK for SoupRune.
//!
//! Provides the guest-side bindings for the WASM Component Model interface.
//! Mod developers implement the `Behavior` and `DanmakuBehavior` traits,
//! then use `export_mod!` to register their implementations.
//!
//! SoupRune 的 WASM 模组开发 SDK。
//! 模组开发者实现 `Behavior` 和 `DanmakuBehavior` trait，
//! 然后使用 `export_mod!` 注册实现。
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
//!         if ctx.input().pressed(Action::Right) {
//!             ctx.kinematics().set_velocity(100.0, 0.0);
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

pub use context::{Context, FactValue, Vec2};
pub use traits::{
    ActionParam, Behavior, CustomActionHandler, DanmakuBehavior, SpawnPatternBehavior,
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

/// Bullet context for danmaku callbacks.
#[derive(Debug, Clone)]
pub struct BulletContext {
    pub elapsed: f32,
    pub delta_time: f32,
    pub spawn_pos: Vec2,
    pub offset: Vec2,
    pub initial_angle: f32,
    pub initial_radius: f32,
    pub player_pos: Vec2,
    props: Vec<(String, f32)>,
}

impl BulletContext {
    #[doc(hidden)]
    pub fn from_wit(ctx: &exports::souprune::plugin::danmaku::BulletContext) -> Self {
        Self {
            elapsed: ctx.elapsed,
            delta_time: ctx.delta_time,
            spawn_pos: Vec2::new(ctx.spawn_pos.x, ctx.spawn_pos.y),
            offset: Vec2::new(ctx.offset.x, ctx.offset.y),
            initial_angle: ctx.initial_angle,
            initial_radius: ctx.initial_radius,
            player_pos: Vec2::new(ctx.player_pos.x, ctx.player_pos.y),
            props: ctx
                .props
                .iter()
                .map(|p| (p.name.clone(), p.value))
                .collect(),
        }
    }

    /// Get the actual spawn position of this bullet (center + offset).
    pub fn spawn_position(&self) -> Vec2 {
        self.spawn_pos + self.offset
    }

    /// Get a named property value, or None if not found.
    pub fn get_float(&self, name: &str) -> Option<f32> {
        self.props.iter().find(|(n, _)| n == name).map(|(_, v)| *v)
    }
}

/// Context for spawn pattern generation.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpawnContext {
    pub center: Vec2,
    pub player_pos: Vec2,
    pub time: f32,
}

impl SpawnContext {
    #[doc(hidden)]
    pub fn from_wit(ctx: &exports::souprune::plugin::spawn_pattern::SpawnContext) -> Self {
        Self {
            center: Vec2::new(ctx.center_x, ctx.center_y),
            player_pos: Vec2::new(ctx.player_x, ctx.player_y),
            time: ctx.time,
        }
    }
}

/// A computed spawn point output from a pattern.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpawnOutput {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub radius: f32,
}

impl SpawnOutput {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            angle: 0.0,
            radius: 0.0,
        }
    }

    pub fn with_angle(mut self, angle: f32) -> Self {
        self.angle = angle;
        self
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    #[doc(hidden)]
    pub fn to_wit(self) -> exports::souprune::plugin::spawn_pattern::SpawnPoint {
        exports::souprune::plugin::spawn_pattern::SpawnPoint {
            x: self.x,
            y: self.y,
            angle: self.angle,
            radius: self.radius,
        }
    }
}

/// Named parameter for spawn patterns.
#[derive(Debug, Clone)]
pub struct SpawnParam {
    pub name: String,
    pub value: f64,
}

impl SpawnParam {
    #[doc(hidden)]
    pub fn from_wit(p: &exports::souprune::plugin::spawn_pattern::PatternParam) -> Self {
        Self {
            name: p.name.clone(),
            value: p.value,
        }
    }

    /// Get as f32 for convenience.
    pub fn as_f32(&self) -> f32 {
        self.value as f32
    }

    /// Get as usize (clamped to 0).
    pub fn as_usize(&self) -> usize {
        self.value.max(0.0) as usize
    }
}

/// Output from a danmaku update.
#[derive(Debug, Clone, Copy, Default)]
pub struct BulletOutput {
    pub offset: Vec2,
    pub rotation: f32,
    /// Opacity override. Negative means no change.
    pub opacity: f32,
    /// Scale delta (0.0 = no change from base scale).
    pub scale_x: f32,
    pub scale_y: f32,
}

impl BulletOutput {
    pub const ZERO: Self = Self {
        offset: Vec2::ZERO,
        rotation: 0.0,
        opacity: -1.0,
        scale_x: 0.0,
        scale_y: 0.0,
    };

    pub fn new(x: f32, y: f32) -> Self {
        Self {
            offset: Vec2::new(x, y),
            ..Self::ZERO
        }
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_scale(mut self, sx: f32, sy: f32) -> Self {
        self.scale_x = sx;
        self.scale_y = sy;
        self
    }

    #[doc(hidden)]
    pub fn to_wit(self) -> exports::souprune::plugin::danmaku::BulletOutput {
        exports::souprune::plugin::danmaku::BulletOutput {
            offset: souprune::plugin::host_api::Vec2 {
                x: self.offset.x,
                y: self.offset.y,
            },
            rotation: self.rotation,
            opacity: self.opacity,
            scale_x: self.scale_x,
            scale_y: self.scale_y,
        }
    }
}

/// Convenience prelude for mod developers.
pub mod prelude {
    pub use crate::context::{Context, FactValue, Vec2};
    pub use crate::traits::{
        ActionParam, Behavior, CustomActionHandler, DanmakuBehavior, SpawnPatternBehavior,
    };
    pub use crate::{
        Action, BulletContext, BulletOutput, SpawnContext, SpawnOutput, SpawnParam, export_mod,
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
    // PRIMARY FORM — generates all code (requires explicit custom_actions type)
    // =========================================================================
    (
        behaviors: [ $( ($b_id:literal, $b_type:ty, $b_ctor:expr) ),* $(,)? ] $(,)?
        danmaku: [ $( ($d_id:literal, $d_type:ty, $d_ctor:expr) ),* $(,)? ] $(,)?
        patterns: [ $( ($p_id:literal, $p_type:ty, $p_ctor:expr) ),* $(,)? ] $(,)?
        custom_actions: $ca_type:ty $(,)?
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

        $crate::export!(ModComponent with_types_in $crate);
    };

    // =========================================================================
    // ALL REMAINING FORMS — delegate to primary with NoopCustomActionHandler
    // =========================================================================

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
        }
    };
}

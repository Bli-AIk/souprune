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

pub use context::{Context, Vec2};
pub use traits::{Behavior, DanmakuBehavior};

// Generate guest bindings from the WIT interface
wit_bindgen::generate!({
    path: "../souprune_api/wit",
    world: "souprune-mod",
    pub_export_macro: true,
});

// Re-export generated types under a `wit` module for advanced use
pub mod wit {
    pub use super::exports::souprune::plugin::behavior::GuestBehaviorInstance;
    pub use super::exports::souprune::plugin::danmaku::GuestDanmakuInstance;
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

/// Output from a danmaku update.
#[derive(Debug, Clone, Copy, Default)]
pub struct BulletOutput {
    pub offset: Vec2,
    pub rotation: f32,
}

impl BulletOutput {
    pub const ZERO: Self = Self {
        offset: Vec2::ZERO,
        rotation: 0.0,
    };

    pub fn new(x: f32, y: f32) -> Self {
        Self {
            offset: Vec2::new(x, y),
            rotation: 0.0,
        }
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
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
        }
    }
}

/// Convenience prelude for mod developers.
pub mod prelude {
    pub use crate::context::{Context, Vec2};
    pub use crate::traits::{Behavior, DanmakuBehavior};
    pub use crate::{Action, BulletContext, BulletOutput, export_mod};
}

/// Register behavior and danmaku implementations and export them as a WASM component.
///
/// 注册行为和弹幕实现，并将它们导出为 WASM 组件。
///
/// # Syntax
///
/// ```ignore
/// export_mod! {
///     behaviors: [
///         ("id", Type, || constructor()),
///     ],
///     danmaku: [
///         ("id", Type, || constructor()),
///     ],
/// }
/// ```
#[macro_export]
macro_rules! export_mod {
    (
        behaviors: [ $( ($b_id:literal, $b_type:ty, $b_ctor:expr) ),* $(,)? ] $(,)?
        danmaku: [ $( ($d_id:literal, $d_type:ty, $d_ctor:expr) ),* $(,)? ] $(,)?
    ) => {
        // --- Behavior resource wrapper ---
        struct WasmBehaviorInstance {
            inner: std::cell::RefCell<Box<dyn $crate::Behavior>>,
        }

        impl $crate::wit::GuestBehaviorInstance for WasmBehaviorInstance {
            fn new(id: String) -> Self {
                let inner: Box<dyn $crate::Behavior> = match id.as_str() {
                    $( $b_id => Box::new($b_ctor()), )*
                    _ => Box::new($crate::traits::NoopBehavior),
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
                    _ => Box::new($crate::traits::NoopDanmaku),
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

        $crate::export!(ModComponent with_types_in $crate);
    };

    // Shorthand: behaviors only
    (
        behaviors: [ $( ($b_id:literal, $b_type:ty, $b_ctor:expr) ),* $(,)? ] $(,)?
    ) => {
        $crate::export_mod! {
            behaviors: [ $( ($b_id, $b_type, $b_ctor), )* ],
            danmaku: [],
        }
    };

    // Shorthand: danmaku only
    (
        danmaku: [ $( ($d_id:literal, $d_type:ty, $d_ctor:expr) ),* $(,)? ] $(,)?
    ) => {
        $crate::export_mod! {
            behaviors: [],
            danmaku: [ $( ($d_id, $d_type, $d_ctor), )* ],
        }
    };
}

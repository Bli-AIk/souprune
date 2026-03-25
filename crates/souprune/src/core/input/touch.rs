//! # touch.rs
//!
//! ## Module Overview / 模块概述
//!
//! Virtual touch overlay for mobile/touch platforms.
//! Renders on-screen D-pad and action buttons, translates Bevy UI interactions
//! into leafwing_input_manager actions via `InputManagerSystem::ManualControl`.
//!
//! 移动/触屏平台的虚拟触控覆盖层。
//! 渲染屏幕上的方向键和动作按钮，通过 `InputManagerSystem::ManualControl`
//! 将 Bevy UI 交互转换为 leafwing_input_manager 动作。
//!
//! ## Press/Hold/Release 处理
//!
//! leafwing 的 Update 系统会在物理按键未按下时调用 release()，
//! 导致 ManualControl 中再调用 press() 会每帧触发 JustPressed。
//! 因此我们自行追踪上一帧的触控状态，直接设置 ButtonState，
//! 绕过 press()/release() 的状态机问题。

mod input_state;
mod spawn;
mod visuals;

use super::actions::ActionRegistry;
use bevy::prelude::*;
use leafwing_input_manager::plugin::InputManagerSystem;
use std::collections::HashSet;

pub use input_state::update_controller_directions;
pub use spawn::spawn_touch_overlay;
pub use visuals::{
    tick_touch_button_animations, toggle_touch_overlay, update_controller_overlays,
    update_touch_button_visuals,
};

pub(super) const FALLBACK_OPACITY: f32 = 0.45;
pub(super) const FALLBACK_BTN_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, FALLBACK_OPACITY);
pub(super) const BTN_PRESSED_COLOR: Color = Color::srgba(0.7, 0.9, 1.0, 0.7);

/// Marker component for the touch overlay root entity.
#[derive(Component)]
pub struct TouchOverlayRoot;

/// Component linking a UI button to a game action name.
#[derive(Component)]
pub struct TouchAction(pub String);

/// Resource controlling touch overlay visibility.
#[derive(Resource)]
pub struct TouchOverlayEnabled(pub bool);

impl Default for TouchOverlayEnabled {
    fn default() -> Self {
        Self(true)
    }
}

/// Tracks which touch actions were pressed in the previous frame.
#[derive(Resource, Default)]
pub(super) struct PrevTouchPressed(pub(super) HashSet<String>);

/// Active direction actions from the controller zone, determined by touch position.
#[derive(Resource, Default)]
pub struct ControllerDirections(pub HashSet<String>);

/// Tracks which touch actions are currently pressed via multitouch hit testing.
#[derive(Resource, Default)]
pub struct MultitouchPressed(pub(super) HashSet<String>);

/// Stores the normal-state image handle for pressed/released visual swap.
#[derive(Component)]
pub struct TouchNormalImage(pub Option<Handle<Image>>);

/// Stores the pressed-state image handle for visual swap.
#[derive(Component)]
pub struct TouchPressedImage(pub Option<Handle<Image>>);

/// Marker for invisible controller touch zones (excluded from visual updates).
#[derive(Component)]
pub struct TouchControllerZone;

/// Animation frame handles for a button [idle, pressing, pressed, releasing].
#[derive(Component)]
pub struct TouchAnimFrames(pub Vec<Handle<Image>>);

/// Current animation state for a button with frame-based animation.
#[derive(Component)]
pub struct TouchAnimState {
    pub current_frame: usize,
    pub timer: Timer,
    pub phase: AnimPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimPhase {
    Idle,
    Pressing,
    Held,
    Releasing,
}

/// Marker for controller direction overlay entities. Stores the action name.
#[derive(Component)]
pub struct TouchControllerOverlay(pub String);

pub(crate) struct TouchPlugin;

impl Plugin for TouchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TouchOverlayEnabled>()
            .init_resource::<PrevTouchPressed>()
            .init_resource::<ControllerDirections>()
            .init_resource::<MultitouchPressed>()
            .add_systems(
                PreUpdate,
                (
                    input_state::detect_multitouch_pressed,
                    input_state::inject_touch_actions.after(input_state::detect_multitouch_pressed),
                )
                    .in_set(InputManagerSystem::ManualControl)
                    .run_if(resource_exists::<ActionRegistry>),
            );
    }
}

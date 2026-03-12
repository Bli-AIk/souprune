//! # camera_debug.rs
//!
//! ## Module Overview
//!
//! Debug camera mode that overrides normal camera follow behavior.
//! When active, allows free-roaming camera control via mouse scroll (zoom) and middle-click drag (pan).
//!
//! ## 模块概述
//!
//! 调试摄像机模式，覆盖正常的摄像机跟随行为。
//! 激活后，可通过鼠标滚轮缩放、中键拖拽平移来自由控制摄像机。
//!
//! ## Hotkey / 快捷键
//!
//! | Key | Function | 功能 |
//! |-----|----------|------|
//! | F8  | Toggle Debug Camera | 切换调试摄像机 |

#[cfg(feature = "debug")]
pub mod debug_camera {
    use crate::core::camera::{CameraControlOverride, MainGameCamera};
    use crate::extra::debug::DebugToastEvent;
    use bevy::ecs::message::{MessageReader, MessageWriter};
    use bevy::input::mouse::{MouseMotion, MouseWheel};
    use bevy::prelude::*;

    #[derive(Resource)]
    pub(in crate::extra::debug) struct DebugCameraState {
        pub active: bool,
        current_scale: f32,
        target_scale: f32,
    }

    impl Default for DebugCameraState {
        fn default() -> Self {
            Self {
                active: false,
                current_scale: 1.0,
                target_scale: 1.0,
            }
        }
    }

    pub fn setup_camera_debug(app: &mut App) {
        app.init_resource::<DebugCameraState>().add_systems(
            Update,
            (
                toggle_debug_camera_system,
                debug_camera_zoom_system,
                debug_camera_pan_system,
            ),
        );
    }

    fn toggle_debug_camera_system(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut state: ResMut<DebugCameraState>,
        mut commands: Commands,
        mut camera_q: Query<(Entity, &Projection), With<MainGameCamera>>,
        mut toast_events: MessageWriter<DebugToastEvent>,
    ) {
        if !keyboard.just_pressed(KeyCode::F8) {
            return;
        }

        state.active = !state.active;

        let Ok((camera_entity, projection)) = camera_q.single_mut() else {
            return;
        };

        if state.active {
            // Sync zoom state from current projection
            if let Projection::Orthographic(ortho) = projection {
                state.current_scale = ortho.scale;
                state.target_scale = ortho.scale;
            }
            commands.entity(camera_entity).insert(CameraControlOverride);
            info!("Debug Camera: ON (scroll=zoom, middle-drag=pan)");
            toast_events.write(DebugToastEvent {
                message: "Debug Camera: ON (scroll=zoom, middle-drag=pan)".into(),
            });
        } else {
            commands
                .entity(camera_entity)
                .remove::<CameraControlOverride>();
            info!("Debug Camera: OFF");
            toast_events.write(DebugToastEvent {
                message: "Debug Camera: OFF".into(),
            });
        }
    }

    fn debug_camera_zoom_system(
        state: ResMut<DebugCameraState>,
        mut scroll_events: MessageReader<MouseWheel>,
        time: Res<Time>,
        mut camera_q: Query<&mut Projection, With<MainGameCamera>>,
    ) {
        if !state.active {
            scroll_events.clear();
            return;
        }

        let state = state.into_inner();
        let Ok(mut projection) = camera_q.single_mut() else {
            return;
        };
        let Projection::Orthographic(ref mut ortho) = *projection else {
            return;
        };

        // Accumulate scroll delta
        let scroll_delta: f32 = scroll_events.read().map(|e| e.y).sum();

        if scroll_delta.abs() > 0.001 {
            let zoom_factor = 1.0 - scroll_delta * 0.1;
            state.target_scale = (state.target_scale * zoom_factor).clamp(0.1, 20.0);
        }

        // Smooth zoom interpolation
        let lerp_speed = 12.0;
        let dt = time.delta_secs();
        state.current_scale +=
            (state.target_scale - state.current_scale) * (1.0 - (-lerp_speed * dt).exp());
        ortho.scale = state.current_scale;
    }

    fn debug_camera_pan_system(
        state: Res<DebugCameraState>,
        mouse_button: Res<ButtonInput<MouseButton>>,
        mut motion_events: MessageReader<MouseMotion>,
        mut camera_q: Query<(&mut Transform, &Projection), With<MainGameCamera>>,
    ) {
        if !state.active {
            motion_events.clear();
            return;
        }

        let Ok((mut transform, projection)) = camera_q.single_mut() else {
            return;
        };

        // Pan with middle mouse button drag
        if !mouse_button.pressed(MouseButton::Middle) {
            motion_events.clear();
            return;
        }

        let Projection::Orthographic(ref ortho) = *projection else {
            return;
        };

        let total_delta: Vec2 = motion_events.read().map(|e| e.delta).sum();

        if total_delta.length_squared() > 0.01 {
            // Scale movement by current zoom level for consistent feel
            transform.translation.x -= total_delta.x * ortho.scale;
            transform.translation.y += total_delta.y * ortho.scale;
        }
    }
}

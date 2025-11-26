use super::components::CameraAnchored;
use crate::app_state::overworld::OverworldState;
use bevy::prelude::*;

/// Keep camera-anchored UI in place even if the camera is still interpolating.
///
/// 在摄像机插值移动时保持 UI 的相对位置不漂移。
pub(crate) fn update_camera_anchored_ui_system(
    overworld_state: Res<State<OverworldState>>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut anchored_ui_query: Query<(&CameraAnchored, &mut Transform), Without<Camera2d>>,
) {
    if overworld_state.get() != &OverworldState::Backpack {
        return;
    }

    let Ok(camera_transform) = camera_query.single() else {
        warn!("No Camera2d available for anchoring UI");
        return;
    };

    for (anchor, mut transform) in anchored_ui_query.iter_mut() {
        let new_translation = camera_transform.translation + anchor.offset;
        if transform.translation != new_translation {
            transform.translation = new_translation;
        }
    }
}

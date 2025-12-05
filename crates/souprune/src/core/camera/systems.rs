use super::components::Followable;
use bevy::prelude::*;

pub(crate) fn update_followable_camera_system(
    mut camera: Query<(&Followable, &mut Transform, &Camera)>,
    target: Query<&Transform, Without<Camera>>,
) {
    for (followable, mut transform, _) in camera.iter_mut() {
        if let Some(target_entity) = followable.target
            && let Ok(target_transform) = target.get(target_entity)
        {
            let mut new_x = target_transform.translation.x;
            let mut new_y = target_transform.translation.y;

            // Apply bounds if enabled.
            //
            // 如果启用了边界限制，则应用边界限制。
            if followable.bounds_enabled {
                new_x = new_x.clamp(followable.min_x, followable.max_x);
                new_y = new_y.clamp(followable.min_y, followable.max_y);
            }

            transform.translation.x = new_x;
            transform.translation.y = new_y;
        }
    }
}

use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Followable {
    pub(crate) target: Option<Entity>,
}
pub(crate) fn update_followable_camera_system(
    mut camera: Query<(&Followable, &mut Transform, &Camera)>,
    target: Query<&Transform, Without<Camera>>,
) {
    for (followable, mut transform, _) in camera.iter_mut() {
        if let Some(target_entity) = followable.target
            && let Ok(target_transform) = target.get(target_entity)
        {
            transform.translation.x = target_transform.translation.x;
            transform.translation.y = target_transform.translation.y;
        }
    }
}

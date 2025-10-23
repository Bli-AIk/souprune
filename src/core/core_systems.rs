use crate::core::core_components::*;
use bevy::prelude::*;

pub(crate) fn update_transform_sync_system(
    mut query: Query<(&Position, &Rotation, &mut Transform)>,
) {
    for (pos, rotation, mut transform) in query.iter_mut() {
        transform.translation = pos.value.extend(0.0);
        transform.rotation = Quat::from_rotation_z(rotation.angle);
    }
}

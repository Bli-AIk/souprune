//! Spatial input ray-hit helpers for 3D View planes.
//!
//! 3D View 平面的空间输入射线命中辅助逻辑。

use super::ViewSpatialRoot;
use crate::core::view::layout::{ViewSpatialInputDef, ViewWorld3dPlaneDef};
use crate::extra::debug::DebugCamera;
use bevy::prelude::{
    Camera, Camera3d, Commands, Component, Dir3, Entity, GlobalTransform, InfinitePlane3d, Query,
    Ray3d, Vec2, Vec3, With, Without,
};
use bevy::window::{PrimaryWindow, Window};

/// Ray-hit data on a spatial View plane.
///
/// 空间 View 平面上的射线命中数据。
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ViewSpatialHit {
    /// Hit point in world space.
    ///
    /// 世界空间中的命中点。
    pub world_position: Vec3,
    /// Hit point in root-local plane units.
    ///
    /// 根局部平面单位中的命中点。
    pub plane_position: Vec2,
    /// Hit point in layout pixels.
    ///
    /// 布局像素中的命中点。
    pub layout_position: Vec2,
    /// Distance from the ray origin to the hit point.
    ///
    /// 从射线原点到命中点的距离。
    pub distance: f32,
}

/// Intersect a world-space ray with a spatial View plane.
///
/// 将世界空间射线与空间 View 平面求交。
pub fn intersect_spatial_plane(
    ray: &Ray3d,
    root_transform: &GlobalTransform,
    plane: &ViewWorld3dPlaneDef,
) -> Option<ViewSpatialHit> {
    let plane_origin = root_transform.transform_point(Vec3::ZERO);
    let plane_normal = Dir3::new(root_transform.affine().transform_vector3(Vec3::Z)).ok()?;
    let distance = ray.intersect_plane(plane_origin, InfinitePlane3d::new(plane_normal))?;
    let world_position = ray.get_point(distance);
    let local_position = root_transform
        .affine()
        .inverse()
        .transform_point3(world_position);

    let half_width = plane.plane_size.0 * 0.5;
    let half_height = plane.plane_size.1 * 0.5;
    if local_position.x.abs() > half_width || local_position.y.abs() > half_height {
        return None;
    }

    let plane_position = Vec2::new(local_position.x, local_position.y);
    let pixels_per_unit = valid_pixels_per_unit(plane.pixels_per_unit);
    let layout_position = Vec2::new(
        local_position.x * pixels_per_unit,
        -local_position.y * pixels_per_unit,
    );

    Some(ViewSpatialHit {
        world_position,
        plane_position,
        layout_position,
        distance,
    })
}

/// Update spatial View ray-hit components from the primary pointer position.
///
/// 根据主指针位置更新空间 View 射线命中组件。
pub fn update_spatial_view_hits_system(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<
        (&Camera, &GlobalTransform),
        (
            With<Camera3d>,
            With<crate::core::camera::MainGameCamera>,
            Without<DebugCamera>,
        ),
    >,
    spatial_roots: Query<(Entity, &ViewSpatialRoot, &GlobalTransform)>,
) {
    let cursor_position = windows.single().ok().and_then(Window::cursor_position);
    let camera = cameras.iter().find(|(camera, _)| camera.is_active);
    let ray =
        cursor_position
            .zip(camera)
            .and_then(|(cursor_position, (camera, camera_transform))| {
                camera
                    .viewport_to_world(camera_transform, cursor_position)
                    .ok()
            });

    for (entity, spatial_root, root_transform) in &spatial_roots {
        if !matches!(spatial_root.plane.input, ViewSpatialInputDef::PlaneRay) {
            commands.entity(entity).remove::<ViewSpatialHit>();
            continue;
        }

        let hit =
            ray.and_then(|ray| intersect_spatial_plane(&ray, root_transform, &spatial_root.plane));
        if let Some(hit) = hit {
            commands.entity(entity).insert(hit);
        } else {
            commands.entity(entity).remove::<ViewSpatialHit>();
        }
    }
}

fn valid_pixels_per_unit(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::view::layout::{
        SerializableTransform, ViewCameraTargetDef, ViewWorld3dPlaneDef,
    };

    fn plane() -> ViewWorld3dPlaneDef {
        ViewWorld3dPlaneDef {
            transform: SerializableTransform::default(),
            rotation_degrees: None,
            plane_size: (4.0, 2.0),
            pixels_per_unit: 100.0,
            camera: ViewCameraTargetDef::Main,
            anchor: Default::default(),
            orientation: Default::default(),
            depth: Default::default(),
            input: Default::default(),
        }
    }

    #[test]
    fn intersect_spatial_plane_hits_center() {
        let ray = Ray3d::new(Vec3::new(0.0, 0.0, 5.0), Dir3::NEG_Z);

        let hit = intersect_spatial_plane(&ray, &GlobalTransform::IDENTITY, &plane())
            .expect("ray should hit the centered plane");

        assert_eq!(hit.world_position, Vec3::ZERO);
        assert_eq!(hit.plane_position, Vec2::ZERO);
        assert_eq!(hit.layout_position, Vec2::ZERO);
        assert!((hit.distance - 5.0).abs() < 0.0001);
    }

    #[test]
    fn intersect_spatial_plane_misses_outside_centered_bounds() {
        let ray = Ray3d::new(Vec3::new(3.0, 0.0, 5.0), Dir3::NEG_Z);

        let hit = intersect_spatial_plane(&ray, &GlobalTransform::IDENTITY, &plane());

        assert_eq!(hit, None);
    }
}

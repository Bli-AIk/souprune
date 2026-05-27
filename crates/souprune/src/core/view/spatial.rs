//! Spatial placement helpers for View roots.
//!
//! View 根的空间放置辅助逻辑。

pub mod input;

use crate::core::view::layout::{
    SerializableTransform, ViewLayoutSlot, ViewSpatialAnchorDef, ViewSpatialOrientationDef,
    ViewWorld3dPlaneDef, serializable_vec3_to_static,
};
use crate::extra::debug::DebugCamera;
use bevy::prelude::{
    Camera, Camera3d, Component, Dir3, EulerRot, GlobalTransform, Name, Quat, Query, Transform,
    Vec3, With, Without, debug,
};

pub use input::{ViewSpatialHit, intersect_spatial_plane};

/// Runtime marker for a View root mounted on a 3D plane.
///
/// 挂载到 3D 平面的 View 根运行时标记。
#[derive(Component, Debug, Clone)]
pub struct ViewSpatialRoot {
    /// Plane placement data used by this View root.
    ///
    /// 此 View 根使用的平面放置数据。
    pub plane: ViewWorld3dPlaneDef,
}

/// Convert a solved 2D layout slot into local coordinates on a 3D View plane.
///
/// 将求解后的二维布局槽位转换为 3D View 平面的局部坐标。
pub fn layout_slot_to_plane_translation(
    slot: &ViewLayoutSlot,
    plane: &ViewWorld3dPlaneDef,
) -> Vec3 {
    let pixels_per_unit = valid_pixels_per_unit(plane.pixels_per_unit);
    Vec3::new(slot.x / pixels_per_unit, -slot.y / pixels_per_unit, 0.0)
}

/// Build the world transform for a spatial View root from schema data.
///
/// 从 schema 数据构建空间 View 根的世界变换。
pub fn spatial_root_transform(plane: &ViewWorld3dPlaneDef) -> Transform {
    let mut transform = transform_from_serializable(&plane.transform);
    if let Some(rotation) = &plane.rotation_degrees {
        let rotation = serializable_vec3_to_static(rotation);
        transform.rotation *= Quat::from_euler(
            EulerRot::XYZ,
            rotation.x.to_radians(),
            rotation.y.to_radians(),
            rotation.z.to_radians(),
        );
    }
    transform
}

/// Resolve a spatial View root transform from plane, anchor, and camera data.
///
/// 依据平面、锚点与相机数据求解空间 View 根变换。
pub fn resolve_spatial_root_transform(
    plane: &ViewWorld3dPlaneDef,
    anchor_transform: Option<&GlobalTransform>,
    camera_transform: Option<&GlobalTransform>,
) -> Transform {
    let authored = spatial_root_transform(plane);
    let mut transform = match (&plane.anchor, anchor_transform) {
        (ViewSpatialAnchorDef::World, _) | (ViewSpatialAnchorDef::Named(_), None) => authored,
        (ViewSpatialAnchorDef::Named(_), Some(anchor)) => {
            anchor.mul_transform(authored).compute_transform()
        }
    };
    apply_orientation(&mut transform, plane.orientation, camera_transform);
    transform
}

/// Synchronize runtime spatial View roots to their configured anchors.
///
/// 将运行时空间 View 根同步到其配置的锚点。
pub fn sync_spatial_view_roots_system(
    mut roots: Query<(&mut Transform, &ViewSpatialRoot)>,
    anchors: Query<(&Name, &GlobalTransform), Without<ViewSpatialRoot>>,
    cameras: Query<
        (&Camera, &GlobalTransform),
        (
            With<Camera3d>,
            With<crate::core::camera::MainGameCamera>,
            Without<DebugCamera>,
        ),
    >,
) {
    let camera_transform = active_camera_transform(&cameras);
    for (mut transform, spatial_root) in &mut roots {
        let anchor_transform = match &spatial_root.plane.anchor {
            ViewSpatialAnchorDef::World => None,
            ViewSpatialAnchorDef::Named(name) => match find_anchor_transform(&anchors, name) {
                Some(anchor) => Some(anchor),
                None => {
                    debug!(
                        "[View Spatial] Named anchor '{}' was not found; preserving current root transform",
                        name
                    );
                    continue;
                }
            },
        };
        *transform =
            resolve_spatial_root_transform(&spatial_root.plane, anchor_transform, camera_transform);
    }
}

fn transform_from_serializable(transform: &SerializableTransform) -> Transform {
    let mut result = Transform::default();
    if let Some(translation) = &transform.translation {
        result.translation = serializable_vec3_to_static(translation);
    }
    if let Some(rotation) = &transform.rotation {
        result.rotation = Quat::from_rotation_z(static_float(rotation).to_radians());
    }
    if let Some(scale) = &transform.scale {
        result.scale = serializable_vec3_to_static(scale);
    }
    result
}

fn static_float(value: &crate::core::sequencer::chapter_schema::Value<f32>) -> f32 {
    match value {
        crate::core::sequencer::chapter_schema::Value::Static(value) => *value,
        crate::core::sequencer::chapter_schema::Value::Expr(_) => 0.0,
    }
}

pub(crate) fn valid_pixels_per_unit(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

fn apply_orientation(
    transform: &mut Transform,
    orientation: ViewSpatialOrientationDef,
    camera_transform: Option<&GlobalTransform>,
) {
    let Some(camera_transform) = camera_transform else {
        return;
    };
    let to_camera = camera_transform.translation() - transform.translation;
    let normal = match orientation {
        ViewSpatialOrientationDef::Fixed => return,
        ViewSpatialOrientationDef::FaceCamera => normalized_or_none(to_camera),
        ViewSpatialOrientationDef::FaceCameraYaw => {
            normalized_or_none(Vec3::new(to_camera.x, 0.0, to_camera.z))
        }
    };
    let Some(normal) = normal else {
        return;
    };
    transform.rotation = Quat::from_rotation_arc(Vec3::Z, *normal);
}

fn normalized_or_none(direction: Vec3) -> Option<Dir3> {
    Dir3::new(direction).ok()
}

fn active_camera_transform<'a>(
    cameras: &'a Query<
        (&Camera, &GlobalTransform),
        (
            With<Camera3d>,
            With<crate::core::camera::MainGameCamera>,
            Without<DebugCamera>,
        ),
    >,
) -> Option<&'a GlobalTransform> {
    cameras
        .iter()
        .find(|(camera, _)| camera.is_active)
        .map(|(_, transform)| transform)
}

fn find_anchor_transform<'a>(
    anchors: &'a Query<(&Name, &GlobalTransform), Without<ViewSpatialRoot>>,
    target_name: &str,
) -> Option<&'a GlobalTransform> {
    anchors
        .iter()
        .find(|(name, _)| name.as_str() == target_name)
        .map(|(_, transform)| transform)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sequencer::chapter_schema::Value;
    use crate::core::view::layout::{
        SerializableTransform, ViewCameraTargetDef, ViewLayoutSlot, ViewSpatialAnchorDef,
        ViewSpatialDepthDef, ViewSpatialInputDef, ViewSpatialOrientationDef, ViewWorld3dPlaneDef,
    };
    use bevy::prelude::{EulerRot, Vec3};

    fn plane() -> ViewWorld3dPlaneDef {
        ViewWorld3dPlaneDef {
            transform: SerializableTransform {
                translation: Some((Value::Static(2.0), Value::Static(3.0), Value::Static(4.0))),
                rotation: Some(Value::Static(30.0)),
                scale: Some((Value::Static(1.0), Value::Static(1.0), Value::Static(1.0))),
            },
            rotation_degrees: None,
            plane_size: (6.4, 4.8),
            pixels_per_unit: 100.0,
            camera: ViewCameraTargetDef::Main,
            anchor: ViewSpatialAnchorDef::World,
            orientation: ViewSpatialOrientationDef::Fixed,
            depth: ViewSpatialDepthDef::TreeOrder,
            input: ViewSpatialInputDef::Disabled,
        }
    }

    #[test]
    fn layout_slot_maps_pixels_to_plane_local_translation() {
        let slot = ViewLayoutSlot {
            path: "Root/Child".to_string(),
            name: "Child".to_string(),
            x: 100.0,
            y: 50.0,
            width: 40.0,
            height: 20.0,
        };

        let translation = layout_slot_to_plane_translation(&slot, &plane());

        assert_eq!(translation, Vec3::new(1.0, -0.5, 0.0));
    }

    #[test]
    fn spatial_root_transform_uses_plane_transform() {
        let transform = spatial_root_transform(&plane());

        assert_eq!(transform.translation, Vec3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn spatial_root_transform_applies_plane_rotation_degrees() {
        let mut plane = plane();
        plane.transform.rotation = None;
        plane.rotation_degrees =
            Some((Value::Static(45.0), Value::Static(0.0), Value::Static(0.0)));

        let transform = spatial_root_transform(&plane);
        let (x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);

        assert!((x.to_degrees() - 45.0).abs() < 0.0001);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);
    }

    #[test]
    fn named_anchor_composes_plane_transform_under_anchor() {
        let mut plane = plane();
        plane.anchor = ViewSpatialAnchorDef::Named("Anchor".to_string());
        plane.transform.translation =
            Some((Value::Static(1.0), Value::Static(2.0), Value::Static(3.0)));
        let anchor = GlobalTransform::from(Transform::from_translation(Vec3::new(4.0, 5.0, 6.0)));

        let transform = resolve_spatial_root_transform(&plane, Some(&anchor), None);

        assert_eq!(transform.translation, Vec3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn face_camera_yaw_rotates_toward_camera_without_changing_height() {
        let mut plane = plane();
        plane.transform.translation =
            Some((Value::Static(0.0), Value::Static(2.0), Value::Static(0.0)));
        plane.transform.rotation = None;
        plane.orientation = ViewSpatialOrientationDef::FaceCameraYaw;
        let camera = GlobalTransform::from(Transform::from_translation(Vec3::new(5.0, 8.0, 5.0)));

        let transform = resolve_spatial_root_transform(&plane, None, Some(&camera));
        let normal = transform.rotation * Vec3::Z;

        assert_eq!(transform.translation.y, 2.0);
        assert!((normal.x - normal.z).abs() < 0.0001);
        assert!(normal.y.abs() < 0.0001);
    }
}

//! View camera target selection helpers.
//!
//! View 相机目标选择辅助逻辑。

use super::layout::{ViewCameraTargetDef, ViewLayoutAsset, ViewSpaceDef};
use bevy::prelude::*;

/// Selected camera information used while spawning or reconciling a View.
///
/// 生成或协调 View 时使用的相机选择结果。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ViewCameraTarget {
    /// Selected camera entity.
    ///
    /// 被选中的相机实体。
    pub(crate) entity: Entity,
    /// Visible 2D viewport size when it can be derived.
    ///
    /// 可推导时的二维可见视口尺寸。
    pub(crate) visible_size: Option<Vec2>,
    /// Parent entity for camera-relative View roots.
    ///
    /// 相机相对 View 根的父实体。
    pub(crate) camera_relative_parent: Option<Entity>,
}

/// Select the active camera target required by a specific View layout.
///
/// 选择指定 View 布局需要的活动相机目标。
pub(crate) fn select_view_camera_target<'a>(
    view_layout: &ViewLayoutAsset,
    cameras_2d: impl IntoIterator<Item = (Entity, &'a Camera, &'a Projection)>,
    cameras_3d: impl IntoIterator<Item = (Entity, &'a Camera)>,
) -> Option<ViewCameraTarget> {
    if let Some(ViewSpaceDef::World3dPlane(plane)) = &view_layout.space {
        return match plane.camera {
            ViewCameraTargetDef::Main => select_active_view_camera(cameras_2d, cameras_3d),
            ViewCameraTargetDef::Named(_) => None,
        };
    }

    active_2d_view_camera(cameras_2d)
}

/// Select the active main View camera, preferring 2D over 3D.
///
/// 选择活动主 View 相机，并优先使用 2D 相机。
pub(crate) fn select_active_view_camera<'a>(
    cameras_2d: impl IntoIterator<Item = (Entity, &'a Camera, &'a Projection)>,
    cameras_3d: impl IntoIterator<Item = (Entity, &'a Camera)>,
) -> Option<ViewCameraTarget> {
    active_2d_view_camera(cameras_2d).or_else(|| active_3d_view_camera(cameras_3d))
}

/// Select the active 2D View camera.
///
/// 选择活动 2D View 相机。
pub(crate) fn active_2d_view_camera<'a>(
    cameras: impl IntoIterator<Item = (Entity, &'a Camera, &'a Projection)>,
) -> Option<ViewCameraTarget> {
    cameras
        .into_iter()
        .find(|(_, camera, _)| camera.is_active)
        .map(|(entity, _, projection)| ViewCameraTarget {
            entity,
            visible_size: orthographic_visible_size(projection),
            camera_relative_parent: Some(entity),
        })
}

/// Select the active 3D View camera.
///
/// 选择活动 3D View 相机。
pub(crate) fn active_3d_view_camera<'a>(
    cameras: impl IntoIterator<Item = (Entity, &'a Camera)>,
) -> Option<ViewCameraTarget> {
    cameras
        .into_iter()
        .find(|(_, camera)| camera.is_active)
        .map(|(entity, _)| ViewCameraTarget {
            entity,
            visible_size: None,
            camera_relative_parent: None,
        })
}

/// Visible size for an orthographic 2D projection.
///
/// 正交 2D 投影的可见尺寸。
pub(crate) fn orthographic_visible_size(projection: &Projection) -> Option<Vec2> {
    let Projection::Orthographic(orthographic) = projection else {
        return None;
    };
    let size = Vec2::new(
        orthographic.area.width().abs(),
        orthographic.area.height().abs(),
    );
    if size.x > 0.0 && size.y > 0.0 {
        Some(size)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Rect;

    fn entity(row: u32) -> Entity {
        Entity::from_raw_u32(row).expect("test entity should be valid")
    }

    fn active_camera() -> Camera {
        Camera {
            is_active: true,
            ..Default::default()
        }
    }

    fn inactive_camera() -> Camera {
        Camera {
            is_active: false,
            ..Default::default()
        }
    }

    fn projection(width: f32, height: f32) -> Projection {
        Projection::Orthographic(OrthographicProjection {
            area: Rect::new(-width / 2.0, -height / 2.0, width / 2.0, height / 2.0),
            ..OrthographicProjection::default_2d()
        })
    }

    fn layout_with_space(space: ViewSpaceDef) -> ViewLayoutAsset {
        ViewLayoutAsset {
            roots: Vec::new(),
            requires: Vec::new(),
            facts: None,
            world_space: false,
            space: Some(space),
            coordinate_system: Default::default(),
            coordinate_space: None,
        }
    }

    #[test]
    fn selects_active_2d_camera_before_active_3d_camera() {
        let two_d_entity = entity(1);
        let three_d_entity = entity(2);
        let two_d_camera = active_camera();
        let three_d_camera = active_camera();
        let projection = projection(320.0, 240.0);

        let selected = select_active_view_camera(
            [(two_d_entity, &two_d_camera, &projection)],
            [(three_d_entity, &three_d_camera)],
        )
        .expect("active camera should be selected");

        assert_eq!(selected.entity, two_d_entity);
        assert_eq!(selected.camera_relative_parent, Some(two_d_entity));
        assert_eq!(selected.visible_size, Some(Vec2::new(320.0, 240.0)));
    }

    #[test]
    fn uses_3d_camera_when_no_active_2d_camera_exists() {
        let two_d_entity = entity(1);
        let three_d_entity = entity(2);
        let two_d_camera = inactive_camera();
        let three_d_camera = active_camera();
        let projection = projection(320.0, 240.0);

        let selected = select_active_view_camera(
            [(two_d_entity, &two_d_camera, &projection)],
            [(three_d_entity, &three_d_camera)],
        )
        .expect("active 3d camera should be selected");

        assert_eq!(selected.entity, three_d_entity);
        assert_eq!(selected.camera_relative_parent, None);
        assert_eq!(selected.visible_size, None);
    }

    #[test]
    fn named_3d_camera_target_does_not_fall_back_to_main_camera() {
        let three_d_entity = entity(2);
        let three_d_camera = active_camera();
        let layout = layout_with_space(ViewSpaceDef::World3dPlane(
            super::super::layout::ViewWorld3dPlaneDef {
                transform: Default::default(),
                rotation_degrees: None,
                plane_size: (1.0, 1.0),
                pixels_per_unit: 100.0,
                camera: ViewCameraTargetDef::Named("SideCamera".to_string()),
                anchor: Default::default(),
                orientation: Default::default(),
                depth: Default::default(),
                input: Default::default(),
            }
            .into(),
        ));

        let selected = select_view_camera_target(&layout, [], [(three_d_entity, &three_d_camera)]);

        assert_eq!(selected, None);
    }
}

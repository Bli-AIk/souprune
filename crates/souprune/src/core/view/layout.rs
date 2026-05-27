//! # layout.rs
//!
//! # layout.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! The View Layout module entry point.
//! This module organizes the View Schema definition and Serde helper types.
//! It corresponds to the `layout/` directory.
//!
//! 视图布局模块入口点。
//! 本模块组织了视图 Schema 定义和 Serde 辅助类型。
//! 它对应 `layout/` 目录。

pub mod coordinate_space;
pub(crate) mod placement;
pub mod serde_types;
pub mod slots;
pub mod taffy;
pub mod view_schema;

pub use coordinate_space::*;
pub use serde_types::*;
pub use slots::*;
pub use taffy::*;
pub use view_schema::*;

/// Convert the shared schema view layout into the runtime asset type.
pub fn runtime_view_layout_from_schema(
    schema: &souprune_schema::view::ViewLayoutAsset,
) -> Result<ViewLayoutAsset, String> {
    let serialized =
        ron::to_string(schema).map_err(|e| format!("failed to serialize view schema: {e}"))?;
    ron::from_str(&serialized)
        .map_err(|e| format!("failed to deserialize runtime view layout: {e}"))
}

/// Convert the shared schema SDF structure into the runtime asset type.
pub fn runtime_sdf_structure_from_schema(
    schema: &souprune_schema::view::SdfStructureAsset,
) -> Result<SdfStructureAsset, String> {
    let serialized =
        ron::to_string(schema).map_err(|e| format!("failed to serialize sdf schema: {e}"))?;
    ron::from_str(&serialized)
        .map_err(|e| format!("failed to deserialize runtime sdf structure: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sequencer::chapter_schema::Value;
    use souprune_schema::Val;
    use std::collections::HashMap;

    #[test]
    fn converts_shared_view_layout_into_runtime_asset() {
        let schema = souprune_schema::view::ViewLayoutAsset {
            roots: vec![souprune_schema::view::ViewNodeDef {
                name: "HudRoot".to_string(),
                tags: vec!["hud".to_string()],
                style: souprune_schema::view::StyleDef {
                    margin: Some(souprune_schema::view::SerializableRect {
                        left: souprune_schema::view::SerializableVal::Px(1.0),
                        right: souprune_schema::view::SerializableVal::Px(2.0),
                        top: souprune_schema::view::SerializableVal::Px(3.0),
                        bottom: souprune_schema::view::SerializableVal::Px(4.0),
                    }),
                    padding: Some(souprune_schema::view::SerializableRect {
                        left: souprune_schema::view::SerializableVal::Percent(5.0),
                        right: souprune_schema::view::SerializableVal::Percent(6.0),
                        top: souprune_schema::view::SerializableVal::Percent(7.0),
                        bottom: souprune_schema::view::SerializableVal::Percent(8.0),
                    }),
                    border: Some(souprune_schema::view::SerializableRect {
                        left: souprune_schema::view::SerializableVal::Px(11.0),
                        right: souprune_schema::view::SerializableVal::Px(12.0),
                        top: souprune_schema::view::SerializableVal::Px(13.0),
                        bottom: souprune_schema::view::SerializableVal::Px(14.0),
                    }),
                    gap: Some(souprune_schema::view::StyleGap {
                        row: souprune_schema::view::SerializableVal::Px(9.0),
                        column: souprune_schema::view::SerializableVal::Px(10.0),
                    }),
                    align_self: Some(souprune_schema::view::SerializableAlignSelf::Center),
                    display: Some(souprune_schema::view::SerializableDisplay::Flex),
                    overflow: Some(souprune_schema::view::ViewOverflowDef::Axes {
                        horizontal: souprune_schema::view::ViewOverflowAxisDef::Hidden,
                        vertical: souprune_schema::view::ViewOverflowAxisDef::Scroll,
                    }),
                    sizing: Some(souprune_schema::view::ViewSizingDef::Axes {
                        width: souprune_schema::view::ViewSizeAxisDef::Fill,
                        height: souprune_schema::view::ViewSizeAxisDef::Fit,
                    }),
                    ..Default::default()
                },
                transform: None,
                focus_policy: Some(souprune_schema::view::ViewFocusPolicyDef::Focusable),
                visible_when: Some("$show_hud".to_string()),
                background_color: Some((
                    Val::Static(0.1),
                    Val::Static(0.2),
                    Val::Static(0.3),
                    Val::Static(1.0),
                )),
                border_color: None,
                image: None,
                sprite: None,
                state_sprite: None,
                texts: vec![souprune_schema::view::TextDef {
                    id: "label".to_string(),
                    content: Some("HP".to_string()),
                    font: "hud".to_string(),
                    align: Some(souprune_schema::view::TextAlignDef::Center),
                    anchor: Some(souprune_schema::view::TextAnchorDef::TopLeft),
                    world_scale: (Val::Static(1.0), Val::Expr("$ui_scale".to_string())),
                    color: (
                        Val::Static(1.0),
                        Val::Static(1.0),
                        Val::Static(1.0),
                        Val::Static(1.0),
                    ),
                    transform: souprune_schema::view::SerializableTransform {
                        translation: Some((
                            Val::Static(8.0),
                            Val::Expr("4.0 + @i".to_string()),
                            Val::Static(2.0),
                        )),
                        rotation: None,
                        scale: Some((Val::Static(1.0), Val::Static(1.0), Val::Static(1.0))),
                    },
                    line_height: Some(12.0),
                    char_spacing: Some(1.5),
                    word_spacing: Some(3.0),
                    text_style: Some("battle_narration".to_string()),
                    conditional_style: None,
                    visible_when: Some("$show_label".to_string()),
                }],
                view_box: None,
                children: Vec::new(),
                repeat: None,
            }],
            requires: vec![souprune_schema::view::DataRequirement::Interface {
                interface: "player".to_string(),
                expects: vec!["player_hp".to_string()],
            }],
            facts: Some(HashMap::from([(
                "enemy_names".to_string(),
                souprune_schema::view::InitialFactValue::StringList(vec![
                    "Mush".to_string(),
                    "Soup".to_string(),
                ]),
            )])),
            space: Some(souprune_schema::view::ViewSpaceDef::World3dPlane(Box::new(
                souprune_schema::view::ViewWorld3dPlaneDef {
                    transform: souprune_schema::view::SerializableTransform {
                        translation: Some((Val::Static(1.0), Val::Static(2.0), Val::Static(3.0))),
                        rotation: Some(Val::Static(25.0)),
                        scale: Some((Val::Static(1.0), Val::Static(2.0), Val::Static(1.0))),
                    },
                    rotation_degrees: Some((
                        Val::Static(10.0),
                        Val::Static(20.0),
                        Val::Static(30.0),
                    )),
                    plane_size: (6.4, 4.8),
                    pixels_per_unit: 100.0,
                    camera: souprune_schema::view::ViewCameraTargetDef::Main,
                    anchor: souprune_schema::view::ViewSpatialAnchorDef::Named(
                        "SpatialAnchor".to_string(),
                    ),
                    orientation: souprune_schema::view::ViewSpatialOrientationDef::FaceCameraYaw,
                    depth: souprune_schema::view::ViewSpatialDepthDef::DistanceToCamera,
                    input: souprune_schema::view::ViewSpatialInputDef::PlaneRay,
                },
            ))),
            coordinate_system: souprune_schema::view::CoordinateSystem::Standard,
            coordinate_space: None,
        };

        let runtime = runtime_view_layout_from_schema(&schema).expect("conversion should succeed");
        let text = &runtime.roots[0].texts[0];

        let Some(ViewSpaceDef::World3dPlane(plane)) = runtime.space.as_ref() else {
            panic!("runtime space should be World3dPlane");
        };
        assert!(matches!(
            plane.rotation_degrees,
            Some((
                Value::Static(10.0),
                Value::Static(20.0),
                Value::Static(30.0)
            ))
        ));
        assert_eq!(plane.plane_size, (6.4, 4.8));
        assert_eq!(plane.pixels_per_unit, 100.0);
        assert!(matches!(plane.camera, ViewCameraTargetDef::Main));
        assert!(matches!(
            plane.anchor,
            ViewSpatialAnchorDef::Named(ref name) if name == "SpatialAnchor"
        ));
        assert!(matches!(
            plane.orientation,
            ViewSpatialOrientationDef::FaceCameraYaw
        ));
        assert!(matches!(plane.depth, ViewSpatialDepthDef::DistanceToCamera));
        assert!(matches!(plane.input, ViewSpatialInputDef::PlaneRay));
        let style = &runtime.roots[0].style;
        let margin = style.margin.as_ref().expect("margin should convert");
        assert!(matches!(margin.left, SerializableVal::Px(v) if (v - 1.0).abs() < f32::EPSILON));
        assert!(matches!(margin.bottom, SerializableVal::Px(v) if (v - 4.0).abs() < f32::EPSILON));
        let padding = style.padding.as_ref().expect("padding should convert");
        assert!(
            matches!(padding.top, SerializableVal::Percent(v) if (v - 7.0).abs() < f32::EPSILON)
        );
        let border = style.border.as_ref().expect("border should convert");
        assert!(matches!(border.left, SerializableVal::Px(v) if (v - 11.0).abs() < f32::EPSILON));
        assert!(matches!(border.bottom, SerializableVal::Px(v) if (v - 14.0).abs() < f32::EPSILON));
        let gap = style.gap.as_ref().expect("gap should convert");
        assert!(matches!(gap.row, SerializableVal::Px(v) if (v - 9.0).abs() < f32::EPSILON));
        assert!(matches!(
            style.align_self,
            Some(SerializableAlignSelf::Center)
        ));
        assert!(matches!(style.display, Some(SerializableDisplay::Flex)));
        assert!(matches!(
            style.overflow,
            Some(ViewOverflowDef::Axes {
                horizontal: ViewOverflowAxisDef::Hidden,
                vertical: ViewOverflowAxisDef::Scroll,
            })
        ));
        assert!(matches!(
            runtime.roots[0].focus_policy,
            Some(ViewFocusPolicyDef::Focusable)
        ));
        assert!(matches!(
            style.sizing,
            Some(ViewSizingDef::Axes {
                width: ViewSizeAxisDef::Fill,
                height: ViewSizeAxisDef::Fit
            })
        ));
        assert_eq!(text.char_spacing, Some(1.5));
        assert_eq!(text.word_spacing, Some(3.0));
        assert_eq!(text.text_style.as_deref(), Some("battle_narration"));
        assert!(matches!(text.align, Some(TextAlignDef::Center)));
        assert!(matches!(text.anchor, Some(TextAnchorDef::TopLeft)));
        assert!(matches!(text.world_scale.1, Value::Expr(ref expr) if expr == "$ui_scale"));
        assert!(matches!(
            text.transform.translation.as_ref().expect("translation").1,
            Value::Expr(ref expr) if expr == "4.0 + @i"
        ));
    }

    #[test]
    fn parses_world_3d_plane_view_space_from_ron() {
        let layout: souprune_schema::view::ViewLayoutAsset = ron::from_str(
            r#"
            (
                roots: [],
                space: Some(World3dPlane((
                    transform: (
                        translation: Some((1.0, 2.0, 3.0)),
                        rotation: Some(15.0),
                        scale: Some((1.0, 1.0, 1.0)),
                    ),
                    plane_size: (6.4, 4.8),
                    pixels_per_unit: 100.0,
                    camera: Main,
                    anchor: Named("SpatialAnchor"),
                    orientation: FaceCameraYaw,
                    depth: DistanceToCamera,
                    input: PlaneRay,
                ))),
            )
            "#,
        )
        .expect("world 3d plane space should parse");

        let Some(souprune_schema::view::ViewSpaceDef::World3dPlane(plane)) = layout.space.as_ref()
        else {
            panic!("schema space should be World3dPlane");
        };
        assert_eq!(plane.plane_size, (6.4, 4.8));
        assert_eq!(plane.pixels_per_unit, 100.0);
        assert!(matches!(
            plane.camera,
            souprune_schema::view::ViewCameraTargetDef::Main
        ));
        assert!(matches!(
            plane.anchor,
            souprune_schema::view::ViewSpatialAnchorDef::Named(ref name)
                if name == "SpatialAnchor"
        ));
        assert!(matches!(
            plane.orientation,
            souprune_schema::view::ViewSpatialOrientationDef::FaceCameraYaw
        ));
        assert!(matches!(
            plane.depth,
            souprune_schema::view::ViewSpatialDepthDef::DistanceToCamera
        ));
        assert!(matches!(
            plane.input,
            souprune_schema::view::ViewSpatialInputDef::PlaneRay
        ));
    }

    #[test]
    fn parses_view_layout_observer_example_asset() {
        let layout: ViewLayoutAsset = ron::from_str(include_str!(
            "../../../examples/assets/view/layout_observer_demo.view.ron"
        ))
        .expect("observer example asset should parse");

        assert_eq!(layout.roots.len(), 1);
        assert!(matches!(layout.space, Some(ViewSpaceDef::World2d)));
    }

    #[test]
    fn converts_shared_sdf_structure_into_runtime_asset() {
        let schema = souprune_schema::view::SdfStructureAsset {
            layer_count: 2,
            root: souprune_schema::view::SdfLayerDef {
                name: "frame".to_string(),
                sdf_type: souprune_schema::view::SdfShapeKind::Outer,
                color_source: souprune_schema::view::SdfColorSource::Custom((
                    Val::Static(1.0),
                    Val::Static(0.5),
                    Val::Static(0.25),
                    Val::Expr("$alpha".to_string()),
                )),
                z_offset: 0.25,
                is_filler: false,
                children: vec![souprune_schema::view::SdfLayerDef {
                    name: "fill".to_string(),
                    sdf_type: souprune_schema::view::SdfShapeKind::Inner,
                    color_source: souprune_schema::view::SdfColorSource::FillColor,
                    z_offset: 0.1,
                    is_filler: true,
                    children: Vec::new(),
                }],
            },
        };

        let runtime =
            runtime_sdf_structure_from_schema(&schema).expect("conversion should succeed");

        match &runtime.root.color_source {
            SdfColorSource::Custom(color) => {
                assert!(matches!(color.0, Value::Static(v) if (v - 1.0).abs() < f32::EPSILON));
                assert!(matches!(color.3, Value::Expr(ref expr) if expr == "$alpha"));
            }
            other => panic!("unexpected runtime color source: {other:?}"),
        }
    }
}

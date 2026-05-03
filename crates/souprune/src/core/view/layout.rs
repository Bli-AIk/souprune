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
pub mod serde_types;
pub mod view_schema;

pub use coordinate_space::*;
pub use serde_types::*;
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
                style: souprune_schema::view::StyleDef::default(),
                transform: None,
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
            world_space: true,
            coordinate_system: souprune_schema::view::CoordinateSystem::Standard,
            coordinate_space: None,
        };

        let runtime = runtime_view_layout_from_schema(&schema).expect("conversion should succeed");
        let text = &runtime.roots[0].texts[0];

        assert!(runtime.world_space);
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

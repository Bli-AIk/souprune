//! Tests for shared View schema parsing.
//!
//! 共享 View schema 解析测试。

use super::*;
use crate::val::Val;

#[test]
fn parses_view_layout_with_dynamic_values_and_text_spacing() {
    let ron = r#"(
        roots: [
            (
                name: "HudRoot",
                focus_policy: Some(Focusable),
                style: (
                    overflow: Some(Axes(horizontal: Hidden, vertical: Scroll)),
                ),
                background_color: Some((0.1, 0.2, 0.3, 1.0)),
                texts: [(
                    id: "label",
                    content: Some("HP"),
                    font: "hud",
                    align: Some(Center),
                    anchor: Some(TopLeft),
                    world_scale: (1.0, "$ui_scale"),
                    color: (1.0, 1.0, 1.0, 1.0),
                    transform: (
                        translation: Some((8.0, "4.0 + @i", 2.0)),
                        scale: Some((1.0, 1.0, 1.0)),
                    ),
                    line_height: Some(12.0),
                    char_spacing: Some(1.5),
                    word_spacing: Some(3.0),
                    text_style: Some("battle_narration"),
                )],
            ),
        ],
        facts: Some({
            "enemy_names": ["Mush", "Soup"],
        }),
    )"#;

    let layout: ViewLayoutAsset = ron::from_str(ron).expect("view layout should parse");
    let root = &layout.roots[0];
    let text = &layout.roots[0].texts[0];

    assert!(matches!(
        root.focus_policy,
        Some(ViewFocusPolicyDef::Focusable)
    ));
    assert!(matches!(
        root.style.overflow,
        Some(ViewOverflowDef::Axes {
            horizontal: ViewOverflowAxisDef::Hidden,
            vertical: ViewOverflowAxisDef::Scroll,
        })
    ));
    assert_eq!(text.char_spacing, Some(1.5));
    assert_eq!(text.word_spacing, Some(3.0));
    assert_eq!(text.text_style.as_deref(), Some("battle_narration"));
    assert!(matches!(text.align, Some(TextAlignDef::Center)));
    assert!(matches!(text.anchor, Some(TextAnchorDef::TopLeft)));
    assert!(matches!(text.world_scale.1, Val::Expr(ref expr) if expr == "$ui_scale"));
    assert!(matches!(
        text.transform.translation.as_ref().expect("translation").1,
        Val::Expr(ref expr) if expr == "4.0 + @i"
    ));
}

#[test]
fn parses_sdf_structure_with_custom_color_source() {
    let ron = r#"(
        layer_count: 2,
        root: (
            name: "frame",
            sdf_type: Outer,
            color_source: Custom((1.0, 0.5, 0.25, "$alpha")),
            children: [(
                name: "fill",
                sdf_type: Inner,
                is_filler: true,
            )],
        ),
    )"#;

    let sdf: SdfStructureAsset = ron::from_str(ron).expect("sdf structure should parse");

    match &sdf.root.color_source {
        SdfColorSource::Custom(color) => {
            assert!(matches!(color.0, Val::Static(v) if (v - 1.0).abs() < f32::EPSILON));
            assert!(matches!(color.3, Val::Expr(ref expr) if expr == "$alpha"));
        }
        other => panic!("unexpected color source parsed: {other:?}"),
    }
}

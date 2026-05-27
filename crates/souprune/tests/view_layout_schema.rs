//! View layout schema regression tests.
//!
//! View 布局 schema 回归测试。

use souprune::core::view::layout::{
    ViewLayoutAsset, ViewSpaceDef, ViewSpatialAnchorDef, ViewSpatialInputDef,
};

fn parse_runtime_view(asset: &str) -> ViewLayoutAsset {
    ron::from_str(asset).expect("view asset should parse as runtime schema")
}

#[test]
fn taffy_acceptance_asset_uses_current_view_schema() {
    let asset = include_str!("../examples/assets/view/taffy_minimal.view.ron");

    assert!(!asset.contains(concat!("world", "_space")));
    let layout = parse_runtime_view(asset);

    assert!(layout.space.is_none());
    assert!(!layout.roots.is_empty());
}

#[test]
fn spatial_acceptance_asset_uses_current_view_schema() {
    let asset = include_str!("../examples/assets/view/spatial_plane.view.ron");

    assert!(!asset.contains(concat!("world", "_space")));
    let layout = parse_runtime_view(asset);
    let Some(ViewSpaceDef::World3dPlane(plane)) = layout.space.as_ref() else {
        panic!("spatial acceptance asset should use World3dPlane");
    };

    assert!(matches!(
        plane.anchor,
        ViewSpatialAnchorDef::Named(ref name) if name == "SpatialAnchor"
    ));
    assert!(matches!(plane.input, ViewSpatialInputDef::PlaneRay));
}

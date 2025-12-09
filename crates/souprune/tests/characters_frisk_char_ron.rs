//! Character asset RON tests for Frisk.
//!
//! Frisk 角色资产 RON 测试。

#[path = "test_support.rs"]
mod test_support;

use souprune::{AnimationConfigAsset, CharacterAsset};

/// Ensure the `.char.ron` file deserializes to `CharacterAsset`.
///
/// 确保 `.char.ron` 文件能够正确反序列化为 `CharacterAsset`。
#[test]
fn frisk_character_asset_deserializes() {
    let asset: CharacterAsset = test_support::parse_project_ron("characters/frisk.char.ron");
    assert_eq!(asset.name, "Frisk");
    assert!(asset.base_speed > 0.0, "base speed should be positive");
}

/// Validate that the animation configuration referenced by the character asset exists and loads.
///
/// 验证角色资产引用的动画配置存在且可加载。
#[test]
fn frisk_character_animation_reference_is_loadable() {
    let asset: CharacterAsset = test_support::parse_project_ron("characters/frisk.char.ron");
    test_support::ensure_project_asset(&asset.animation_config);
    let config: AnimationConfigAsset = test_support::parse_project_ron(&asset.animation_config);
    assert!(
        config.states.keys().any(|state| state.as_str() == "Walk"),
        "animation config should expose Walk state"
    );
}

/// Rehearse collider logic by ensuring the hitbox encloses the sprite origin horizontally and sits below vertically.
///
/// 通过验证碰撞箱水平包裹角色原点且垂直位于角色下方，预演碰撞逻辑。
#[test]
fn frisk_character_collider_bounds_behave() {
    let asset: CharacterAsset = test_support::parse_project_ron("characters/frisk.char.ron");
    let half = asset.collider_size * 0.5;
    let min = asset.collider_offset - half;
    let max = asset.collider_offset + half;

    assert!(
        min.x <= 0.0 && max.x >= 0.0,
        "collider must wrap around sprite center on X axis"
    );
    assert!(
        max.y < 0.0,
        "collider should remain below sprite pivot for top-down gameplay"
    );
}

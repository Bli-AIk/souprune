//! Character `.character.ron` asset tests.
//!
//! `.character.ron` 角色资产测试。
//!
//! Two types of `.character.ron` files exist:
//! - Character definition files (e.g., `frisk.character.ron`) contain `CharacterAsset`
//! - Animation config files (e.g., `animations.character.ron`) contain `AnimationConfigAsset`

#[path = "test_support.rs"]
mod test_support;

use souprune::{AnimationConfigAsset, CharacterAsset};

const CHAR_DIR: &str = "states/overworld/characters";
const CHAR_SUFFIX: &str = ".character.ron";

/// Get character definition files (excluding animation configs).
/// 获取角色定义文件（排除动画配置文件）。
fn character_definition_files() -> Vec<String> {
    test_support::list_project_files_with_suffix(CHAR_DIR, CHAR_SUFFIX)
        .into_iter()
        .filter(|f| !f.contains("animations"))
        .collect()
}

/// Get animation config files.
/// 获取动画配置文件。
fn animation_config_files() -> Vec<String> {
    test_support::list_project_files_with_suffix(CHAR_DIR, CHAR_SUFFIX)
        .into_iter()
        .filter(|f| f.contains("animations"))
        .collect()
}

/// Ensure every character definition `.character.ron` file can be deserialized.
///
/// 确保每个角色定义 `.character.ron` 文件都能反序列化成功。
#[test]
fn character_assets_deserialize() {
    let files = character_definition_files();
    assert!(
        !files.is_empty(),
        "No character definition .character.ron files found"
    );
    for relative in files {
        let asset: CharacterAsset = test_support::parse_project_ron(&relative);
        assert!(
            !asset.name.is_empty(),
            "character name should not be empty for {}",
            relative
        );
        assert!(
            asset.base_speed > 0.0,
            "character base_speed should be positive for {}",
            relative
        );
    }
}

/// Validate that referenced animation configs exist and parse.
///
/// 验证角色引用的动画配置存在且可解析。
#[test]
fn character_animation_configs_exist() {
    for relative in character_definition_files() {
        let asset: CharacterAsset = test_support::parse_project_ron(&relative);
        test_support::ensure_project_asset(&asset.animation_config);
        let config: AnimationConfigAsset = test_support::parse_project_ron(&asset.animation_config);
        assert!(
            !config.states.is_empty(),
            "animation config {} used by {} should define states",
            asset.animation_config,
            relative
        );
    }
}

/// Ensure all animation config files can be deserialized.
///
/// 确保所有动画配置文件都能反序列化成功。
#[test]
fn animation_configs_deserialize() {
    let files = animation_config_files();
    assert!(
        !files.is_empty(),
        "No animation config .character.ron files found"
    );
    for relative in files {
        let config: AnimationConfigAsset = test_support::parse_project_ron(&relative);
        assert!(
            !config.states.is_empty(),
            "animation config {} should define states",
            relative
        );
        assert!(
            !config.sprite_source.is_empty(),
            "animation config {} should have sprite_source",
            relative
        );
    }
}

/// Rehearse collider logic by ensuring each collider encloses the sprite origin horizontally and remains below vertically.
///
/// 通过确认碰撞箱水平包裹角色原点且整体位于角色下方，预演碰撞逻辑。
#[test]
fn character_collider_bounds_cover_origin() {
    for relative in character_definition_files() {
        let asset: CharacterAsset = test_support::parse_project_ron(&relative);
        let half = asset.collider_size * 0.5;
        let min = asset.collider_offset - half;
        let max = asset.collider_offset + half;

        assert!(
            min.x <= 0.0 && max.x >= 0.0,
            "collider must wrap X origin for {}",
            relative
        );
        assert!(
            max.y <= 0.0,
            "collider should sit below sprite pivot for {}",
            relative
        );
    }
}

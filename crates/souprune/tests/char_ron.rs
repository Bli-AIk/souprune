//! Character `.character.ron` asset tests.
//!
//! `.character.ron` 角色资源测试。

#[path = "test_support.rs"]
mod test_support;

use souprune::{AnimationConfigAsset, CharacterAsset};

const CHAR_DIR: &str = "overworld/characters";
const CHAR_SUFFIX: &str = ".character.ron";

fn character_definition_files() -> Vec<String> {
    test_support::list_project_files_with_suffix(CHAR_DIR, CHAR_SUFFIX)
}

/// Ensure every character definition `.character.ron` file can be deserialized.
///
/// 确保每个角色定义 `.character.ron` 文件都能反序列化成功。
#[test]
fn character_assets_deserialize() {
    let files = character_definition_files();
    if files.is_empty() {
        // Skip if no character definitions exist (project assets may not be available in CI)
        return;
    }
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

/// Rehearse collider logic by ensuring each collider encloses the sprite origin horizontally and remains below vertically.
///
/// 通过确认碰撞箱水平包裹角色原点且整体位于角色下方，预演碰撞逻辑。
#[test]
fn character_collider_bounds_cover_origin() {
    for relative in character_definition_files() {
        let asset: CharacterAsset = test_support::parse_project_ron(&relative);
        let half_x = asset.collider_size.x * 0.5;
        let half_y = asset.collider_size.y * 0.5;
        let min_x = asset.collider_offset.x - half_x;
        let max_x = asset.collider_offset.x + half_x;
        let max_y = asset.collider_offset.y + half_y;

        assert!(
            min_x <= 0.0 && max_x >= 0.0,
            "collider must wrap X origin for {}",
            relative
        );
        assert!(
            max_y <= 0.0,
            "collider should sit below sprite pivot for {}",
            relative
        );
    }
}

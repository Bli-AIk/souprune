//! Tests for runtime project path configuration.
//!
//! 运行时项目路径配置的测试。

use super::*;
use std::path::PathBuf;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn desktop_projects_base_path_stays_relative() {
    let _guard = ENV_LOCK
        .lock()
        .expect("environment lock should be available");
    assert_eq!(get_projects_base_path(), PathBuf::from("projects"));
}

#[test]
fn android_private_base_path_points_to_app_storage() {
    let _guard = ENV_LOCK
        .lock()
        .expect("environment lock should be available");
    // SAFETY: These tests do not spawn threads that read this process
    // environment while the variable is being changed.
    unsafe {
        std::env::remove_var("SOUPRUNE_PRIVATE_ROOT");
    }
    assert_eq!(
        android_private_base_path(),
        PathBuf::from("/data/user/0/com.bliaik.souprune/files/SoupRune")
    );
    assert_eq!(
        android_private_base_path().join("projects"),
        PathBuf::from("/data/user/0/com.bliaik.souprune/files/SoupRune/projects")
    );
}

#[test]
fn android_private_base_path_can_come_from_environment() {
    let _guard = ENV_LOCK
        .lock()
        .expect("environment lock should be available");
    let root = PathBuf::from("/tmp/souprune-private-root-test");
    // SAFETY: These tests do not spawn threads that read this process
    // environment while the variable is being changed.
    unsafe {
        std::env::set_var("SOUPRUNE_PRIVATE_ROOT", &root);
    }
    assert_eq!(android_private_base_path(), root);
    // SAFETY: See safety note above.
    unsafe {
        std::env::remove_var("SOUPRUNE_PRIVATE_ROOT");
    }
}

#[test]
fn mod_game_config_accepts_project_declared_modes() {
    let parsed: ModConfigFile = toml::from_str(
        r#"
        [game]
        initial_mode = "field"

        [game.modes.field]
        primitives = ["top_down_map", "top_down_player", "interaction_zones"]
        entry_sequence = "maps/field.sequence.ron"
        rules = ["maps/field.fre.ron"]

        [game.modes.encounter]
        primitives = ["fixed_scene", "sequencer", "danmaku"]
        entry_sequence = "encounters/start.sequence.ron"
        fixed_camera_zoom = 1.5
        alight_motion_config = "encounters/alight_motion_config.ron"
        "#,
    )
    .expect("mode config should parse");

    let modes = parsed
        .game
        .expect("game config")
        .modes
        .expect("mode declarations");
    assert!(modes["field"].has_primitive(ModePrimitiveConfig::TopDownMap));
    assert!(modes["encounter"].has_primitive(ModePrimitiveConfig::FixedScene));
    assert_eq!(modes["encounter"].fixed_camera_zoom(), 1.5);
    assert_eq!(
        modes["encounter"].alight_motion_config.as_deref(),
        Some("encounters/alight_motion_config.ron")
    );
}

#[test]
fn mod_game_config_rejects_removed_mode_specific_fields() {
    let error = match toml::from_str::<ModConfigFile>(
        r#"
        [game]
        initial_battle_path = "battle/start.sequence.ron"
        "#,
    ) {
        Ok(_) => panic!("removed battle-specific config field should fail parsing"),
        Err(error) => error,
    };

    assert!(
        error.to_string().contains("initial_battle_path"),
        "error should name the removed field: {error}"
    );
}

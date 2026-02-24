//! Player behavior RON tests for overworld startup.
//!
//! Overworld 玩家行为 RON 测试。

#[path = "test_support.rs"]
mod test_support;

use ron::de::from_str;
use serde::Deserialize;
use souprune::{AnimationConfigAsset, CharacterAsset, Direction, PlayerBehavior};
use std::path::PathBuf;
use std::sync::Once;

/// Validate referenced assets (character + animation config) exist and parse.
///
/// 验证引用的资产（角色与动画配置）都存在并可解析。
#[test]
fn player_behavior_asset_references_are_valid() {
    let raw = load_raw_behavior();
    assert_eq!(raw.spawn_position.x, 0.0);
    assert_eq!(raw.spawn_position.y, 0.0);
    assert_eq!(raw.initial_state, "Walk");
    assert_eq!(raw.initial_facing, Direction::Down);
    if let Some(run) = &raw.run {
        assert_eq!(run.action, "Cancel");
        assert!((run.speed_multiplier - 2.0).abs() < f32::EPSILON);
    } else {
        panic!("run configuration should exist");
    }
    test_support::ensure_project_asset(&raw.character_asset);
    let character: CharacterAsset = test_support::parse_project_ron(&raw.character_asset);
    test_support::ensure_project_asset(&character.animation_config);
    let _animation: AnimationConfigAsset =
        test_support::parse_project_ron(&character.animation_config);
}

#[derive(Debug, Deserialize)]
struct RawPlayerBehavior {
    character_asset: String,
    #[serde(default = "default_spawn_position")]
    spawn_position: RawVec2,
    #[serde(default)]
    initial_facing: Direction,
    #[serde(default = "default_initial_state")]
    initial_state: String,
    #[serde(default)]
    run: Option<RawRunConfig>,
}

#[derive(Debug, Deserialize)]
struct RawRunConfig {
    /// Action name as string
    action: String,
    #[serde(default = "default_run_speed_multiplier")]
    speed_multiplier: f32,
}

#[derive(Debug, Deserialize)]
struct RawVec2 {
    x: f32,
    y: f32,
}

fn load_raw_behavior() -> RawPlayerBehavior {
    let path = test_support::project_root().join("states/overworld/players/player_behavior.ron");
    let contents = test_support::read_string(&path);
    from_str(&contents).expect("player behavior ron should parse")
}

fn default_spawn_position() -> RawVec2 {
    RawVec2 { x: 0.0, y: 0.0 }
}

fn default_initial_state() -> String {
    "Idle".to_string()
}

fn default_run_speed_multiplier() -> f32 {
    2.0
}

static WORKSPACE_DIR_SET: Once = Once::new();

fn ensure_workspace_dir() {
    WORKSPACE_DIR_SET.call_once(|| {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("workspace root");
        std::env::set_current_dir(workspace_root).expect("set workspace dir");
    });
}

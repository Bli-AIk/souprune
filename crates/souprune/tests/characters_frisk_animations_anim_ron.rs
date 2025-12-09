//! Animation configuration RON tests for Frisk.
//!
//! Frisk 的动画配置 RON 测试。

#[path = "test_support.rs"]
mod test_support;

use proptest::prelude::*;
use souprune::{AnimationConfigAsset, Direction, StateAnimationMapping};

/// Verify the `.anim.ron` file parses into `AnimationConfigAsset`.
///
/// 验证 `.anim.ron` 文件能解析为 `AnimationConfigAsset`。
#[test]
fn frisk_animation_config_deserializes() {
    let config: AnimationConfigAsset =
        test_support::parse_project_ron("characters/frisk/animations.anim.ron");
    assert_eq!(config.sprite_source, "overworld");
    assert_eq!(config.states.len(), 3);
}

/// Ensure all animation states provide directional clips with non-empty names.
///
/// 确保所有动画状态都提供非空的方向动画片段。
#[test]
fn frisk_animation_states_cover_all_directions() {
    let config: AnimationConfigAsset =
        test_support::parse_project_ron("characters/frisk/animations.anim.ron");
    let expected_states = ["Idle", "Walk", "Run"];
    for state in expected_states {
        let Some(StateAnimationMapping::Directional {
            up,
            down,
            left,
            right,
        }) = config.states.get(state)
        else {
            panic!("state {state} missing or not directional");
        };
        for clip in [up, down, left, right] {
            assert!(
                !clip.is_empty(),
                "directional clip for state {state} should not be empty"
            );
        }
    }
}

fn direction_strategy() -> impl Strategy<Value = Direction> {
    prop_oneof![
        Just(Direction::Up),
        Just(Direction::Down),
        Just(Direction::Left),
        Just(Direction::Right),
        Just(Direction::UpLeft),
        Just(Direction::UpRight),
        Just(Direction::DownLeft),
        Just(Direction::DownRight),
    ]
}

fn state_strategy() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just("Idle"), Just("Walk"), Just("Run")]
}

proptest! {
    /// Rehearse directional animation lookup logic using randomized states and facings.
    ///
    /// 使用随机状态与朝向预演方向动画查找逻辑。
    #[test]
    fn frisk_animation_directional_logic_behaves(state in state_strategy(), direction in direction_strategy()) {
        let config: AnimationConfigAsset =
            test_support::parse_project_ron("characters/frisk/animations.anim.ron");
        let mapping = config.states.get(state).expect("state must exist");
        let clip = mapping.get_clip_name(&direction);

        match direction {
            Direction::Up | Direction::UpLeft | Direction::UpRight => prop_assert!(clip.ends_with("_up")),
            Direction::Down | Direction::DownLeft | Direction::DownRight => prop_assert!(clip.ends_with("_down")),
            Direction::Left => prop_assert!(clip.ends_with("_left")),
            Direction::Right => prop_assert!(clip.ends_with("_right")),
        }
    }
}

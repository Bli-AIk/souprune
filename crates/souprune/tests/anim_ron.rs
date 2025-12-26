//! Animation `.anim.ron` asset tests.
//!
//! `.anim.ron` 动画资产测试。

#[path = "test_support.rs"]
mod test_support;

use proptest::prelude::*;
use proptest::test_runner::TestRunner;
use souprune::{AnimationConfigAsset, Direction, StateAnimationMapping};

const ANIM_DIR: &str = "overworld/characters";
const ANIM_SUFFIX: &str = ".animation.ron";

fn animation_files() -> Vec<String> {
    let files = test_support::list_project_files_with_suffix(ANIM_DIR, ANIM_SUFFIX);
    assert!(
        !files.is_empty(),
        "No .animation.ron files found under projects/example_mod/characters"
    );
    files
}

/// Ensure all `.anim.ron` files deserialize.
///
/// 确保所有 `.anim.ron` 文件都能被解析。
#[test]
fn animation_configs_deserialize() {
    for relative in animation_files() {
        let config: AnimationConfigAsset = test_support::parse_project_ron(&relative);
        assert!(
            !config.states.is_empty(),
            "animation config {} should contain states",
            relative
        );
        assert!(
            !config.sprite_source.is_empty(),
            "animation config {} should set sprite_source",
            relative
        );
    }
}

/// Validate that each state's clips are non-empty.
///
/// 验证每个状态的动画片段都非空。
#[test]
fn animation_states_have_clips() {
    for relative in animation_files() {
        let config: AnimationConfigAsset = test_support::parse_project_ron(&relative);
        for (state, mapping) in &config.states {
            match mapping {
                StateAnimationMapping::Directional {
                    up,
                    down,
                    left,
                    right,
                } => {
                    for (dir_name, clip) in
                        [("up", up), ("down", down), ("left", left), ("right", right)]
                    {
                        assert!(
                            !clip.is_empty(),
                            "{state} {dir_name} clip in {} should not be empty",
                            relative
                        );
                    }
                }
                StateAnimationMapping::Single(clip) => assert!(
                    !clip.is_empty(),
                    "{state} single clip in {} should not be empty",
                    relative
                ),
            }
        }
    }
}

#[derive(Clone)]
struct DirectionalCase {
    state: String,
    mapping: StateAnimationMapping,
}

fn gather_directional_cases() -> Vec<DirectionalCase> {
    let mut cases = Vec::new();
    for relative in animation_files() {
        let config: AnimationConfigAsset = test_support::parse_project_ron(&relative);
        for (state, mapping) in &config.states {
            if matches!(mapping, StateAnimationMapping::Directional { .. }) {
                cases.push(DirectionalCase {
                    state: format!("{relative}::{state}"),
                    mapping: mapping.clone(),
                });
            }
        }
    }
    cases
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

/// Rehearse directional animation lookup logic using randomized state/direction pairs.
///
/// 对随机状态与方向组合预演方向动画查找逻辑。
#[test]
fn animation_directional_lookup_behaves() {
    let cases = gather_directional_cases();
    assert!(
        !cases.is_empty(),
        "Expected at least one directional animation state"
    );
    let len = cases.len();
    let mut runner = TestRunner::default();
    let strategy = (0..len, direction_strategy());
    runner
        .run(&strategy, |(index, direction)| {
            let case = &cases[index];
            let StateAnimationMapping::Directional {
                up,
                down,
                left,
                right,
            } = &case.mapping
            else {
                return Err(TestCaseError::fail(
                    "Selected mapping should always be directional",
                ));
            };
            let clip = case.mapping.get_clip_name(&direction);
            match direction {
                Direction::Up | Direction::UpLeft | Direction::UpRight => {
                    prop_assert_eq!(clip, up, "{}", case.state);
                }
                Direction::Down | Direction::DownLeft | Direction::DownRight => {
                    prop_assert_eq!(clip, down, "{}", case.state);
                }
                Direction::Left => prop_assert_eq!(clip, left, "{}", case.state),
                Direction::Right => prop_assert_eq!(clip, right, "{}", case.state),
            }
            Ok(())
        })
        .expect("directional mapping rehearsal should pass");
}

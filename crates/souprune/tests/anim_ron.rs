//! Character animation `.character.ron` asset tests.
//!
//! `.character.ron` 角色动画资产测试。

#[path = "test_support.rs"]
mod test_support;

use proptest::prelude::*;
use proptest::test_runner::TestRunner;
use souprune::{AnimationConfigAsset, Direction, StateAnimationMapping};

const ANIM_DIR: &str = "states/overworld/characters";
const ANIM_SUFFIX: &str = ".character.ron";

fn animation_files() -> Vec<String> {
    // Filter to include only animation config files (contains "animations" in path)
    // 过滤只包含动画配置文件（路径包含 "animations"）
    test_support::list_project_files_with_suffix(ANIM_DIR, ANIM_SUFFIX)
        .into_iter()
        .filter(|f| f.contains("animations"))
        .collect()
}

/// Ensure all character animation config files deserialize.
///
/// 确保所有角色动画配置文件都能被解析。
#[test]
fn animation_configs_deserialize() {
    let files = animation_files();
    if files.is_empty() {
        // Skip if no animation configs exist (they might all be in character definitions)
        return;
    }
    for relative in files {
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

fn assert_mapping_clips_non_empty(state: &str, mapping: &StateAnimationMapping, file: &str) {
    match mapping {
        StateAnimationMapping::Directional {
            up,
            down,
            left,
            right,
        } => {
            for (dir_name, clip) in [("up", up), ("down", down), ("left", left), ("right", right)] {
                assert!(
                    !clip.is_empty(),
                    "{state} {dir_name} clip in {file} should not be empty",
                );
            }
        }
        StateAnimationMapping::Single(clip) => assert!(
            !clip.is_empty(),
            "{state} single clip in {file} should not be empty",
        ),
    }
}

/// Validate that each state's clips are non-empty.
///
/// 验证每个状态的动画片段都非空。
#[test]
fn animation_states_have_clips() {
    let files = animation_files();
    if files.is_empty() {
        return;
    }
    for relative in files {
        let config: AnimationConfigAsset = test_support::parse_project_ron(&relative);
        for (state, mapping) in &config.states {
            assert_mapping_clips_non_empty(state, mapping, &relative);
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

fn resolve_directional_clip<'a>(
    mapping: &'a StateAnimationMapping,
    direction: &Direction,
) -> &'a str {
    match mapping {
        StateAnimationMapping::Directional {
            up,
            down,
            left,
            right,
        } => match direction {
            Direction::Up | Direction::UpLeft | Direction::UpRight => up,
            Direction::Down | Direction::DownLeft | Direction::DownRight => down,
            Direction::Left => left,
            Direction::Right => right,
        },
        StateAnimationMapping::Single(clip) => clip,
    }
}

/// Rehearse directional animation lookup logic using randomized state/direction pairs.
///
/// 对随机状态与方向组合预演方向动画查找逻辑。
#[test]
fn animation_directional_lookup_behaves() {
    let cases = gather_directional_cases();
    if cases.is_empty() {
        // Skip if no animation configs exist (project assets may not be available in CI)
        return;
    }
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
            let expected_clip = match direction {
                Direction::Up | Direction::UpLeft | Direction::UpRight => up,
                Direction::Down | Direction::DownLeft | Direction::DownRight => down,
                Direction::Left => left,
                Direction::Right => right,
            };
            let clip = resolve_directional_clip(&case.mapping, &direction);
            prop_assert_eq!(clip, expected_clip, "{}", case.state);
            Ok(())
        })
        .expect("directional mapping rehearsal should pass");
}

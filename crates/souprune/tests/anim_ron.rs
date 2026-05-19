//! Character animation `.animation_config.ron` asset tests.
//!
//! `.animation_config.ron` 角色动画资源测试。

#[path = "test_support.rs"]
mod test_support;

use souprune::{AnimationConfigAsset, StateAnimationMapping};

const ANIM_DIR: &str = "overworld/characters";
const ANIM_SUFFIX: &str = ".animation_config.ron";

fn animation_files() -> Vec<String> {
    test_support::list_project_files_with_suffix(ANIM_DIR, ANIM_SUFFIX)
        .into_iter()
        .filter(|f| f.contains("animations"))
        .collect()
}

/// Ensure all character animation config files deserialize.
#[test]
fn animation_configs_deserialize() {
    let files = animation_files();
    if files.is_empty() {
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

fn assert_mapping_entries_non_empty(state: &str, mapping: &StateAnimationMapping, file: &str) {
    match mapping {
        StateAnimationMapping::Directional {
            up,
            down,
            left,
            right,
        } => {
            for (dir_name, entry) in [("up", up), ("down", down), ("left", left), ("right", right)]
            {
                assert!(
                    !entry.path().is_empty(),
                    "{state} {dir_name} path in {file} should not be empty",
                );
            }
        }
        StateAnimationMapping::Single(entry) => assert!(
            !entry.path().is_empty(),
            "{state} single path in {file} should not be empty",
        ),
    }
}

/// Validate that each state's entries have non-empty paths.
#[test]
fn animation_states_have_paths() {
    let files = animation_files();
    if files.is_empty() {
        return;
    }
    for relative in files {
        let config: AnimationConfigAsset = test_support::parse_project_ron(&relative);
        for (state, mapping) in &config.states {
            assert_mapping_entries_non_empty(state, mapping, &relative);
        }
    }
}

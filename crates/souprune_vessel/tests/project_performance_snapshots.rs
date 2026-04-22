//! Snapshot tests for all project-owned performance constructors.
//!
//! 所有项目侧 performance 构造器的快照测试。

#[path = "../../../projects/EVAERDRAD_storyspin_chara_fanmade/content/src/performances.rs"]
mod chara_mod_performances;
#[path = "../../../projects/epictale/content/src/performances.rs"]
mod epictale_performances;
#[path = "../../../projects/example_am_mod/content/src/performances.rs"]
mod example_am_mod_performances;
#[path = "../../../projects/example_battle_mod/content/src/performances.rs"]
mod example_battle_mod_performances;
#[path = "../../../projects/example_mod/content/src/demo_attack.rs"]
mod example_mod_demo_attack;
#[path = "../../../projects/example_mod/content/src/performances.rs"]
mod example_mod_performances;

use serde_json::Value;
use souprune_schema::danmaku::DanmakuPerformance;
use std::path::PathBuf;

fn workspace_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(relative)
}

fn parse_expected(path: &str) -> DanmakuPerformance {
    let ron_text = std::fs::read_to_string(workspace_path(path))
        .expect("expected performance file should exist");
    ron::from_str(&ron_text).expect("expected performance file should parse")
}

fn canonical_json(performance: &DanmakuPerformance) -> Value {
    sort_value(serde_json::to_value(performance).expect("performance should serialize to JSON"))
}

fn sort_value(value: Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.into_iter().map(sort_value).collect()),
        Value::Object(map) => {
            let mut entries = map.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            let mut sorted = serde_json::Map::new();
            for (key, value) in entries {
                sorted.insert(key, sort_value(value));
            }
            Value::Object(sorted)
        }
        other => other,
    }
}

fn assert_matches_asset(actual: DanmakuPerformance, path: &str) {
    let expected = parse_expected(path);
    assert_eq!(
        canonical_json(&actual),
        canonical_json(&expected),
        "performance constructor does not match asset {path}"
    );
}

#[test]
fn example_mod_performances_match_assets() {
    assert_matches_asset(
        example_mod_demo_attack::demo_attack(),
        "projects/example_mod/battle/danmaku/demo_attack.performance.ron",
    );
    assert_matches_asset(
        example_mod_demo_attack::demo_attack_overworld(),
        "projects/example_mod/overworld/danmaku/demo_attack_ow.performance.ron",
    );
    assert_matches_asset(
        example_mod_performances::cotton_top_sweep(),
        "projects/example_mod/battle/danmaku/cotton_top_sweep.performance.ron",
    );
    assert_matches_asset(
        example_mod_performances::cotton_surround(),
        "projects/example_mod/battle/danmaku/cotton_surround.performance.ron",
    );
    assert_matches_asset(
        example_mod_performances::cotton_side_pincer(),
        "projects/example_mod/battle/danmaku/cotton_side_pincer.performance.ron",
    );
    assert_matches_asset(
        example_mod_performances::cotton_bottom_wave(),
        "projects/example_mod/battle/danmaku/cotton_bottom_wave.performance.ron",
    );
    assert_matches_asset(
        example_mod_performances::cotton_first_turn(),
        "projects/example_mod/battle/danmaku/cotton_first_turn.performance.ron",
    );
}

#[test]
fn project_demo_attacks_match_assets() {
    assert_matches_asset(
        example_am_mod_performances::demo_attack(),
        "projects/example_am_mod/battle/danmaku/demo_attack.performance.ron",
    );
    assert_matches_asset(
        epictale_performances::demo_attack(),
        "projects/epictale/battle/danmaku/demo_attack.performance.ron",
    );
    assert_matches_asset(
        epictale_performances::demo_attack_overworld(),
        "projects/epictale/overworld/danmaku/demo_attack_ow.performance.ron",
    );
    assert_matches_asset(
        example_battle_mod_performances::demo_attack(),
        "projects/example_battle_mod/battle/danmaku/demo_attack.performance.ron",
    );
    assert_matches_asset(
        chara_mod_performances::demo_attack(),
        "projects/EVAERDRAD_storyspin_chara_fanmade/battle/danmaku/demo_attack.performance.ron",
    );
}

//! Snapshot test for the Rust builder output of `cotton_first_turn`.
//!
//! `cotton_first_turn` Rust 构造输出的快照测试。

#[path = "support/cotton_first_turn_assertions.rs"]
mod cotton_first_turn_assertions;

use cotton_first_turn_assertions::{assert_same_performance, parse_generated_ron};
use souprune_schema::danmaku::DanmakuPerformance;

#[test]
fn cotton_first_turn_fixture_round_trips_through_schema() {
    let original_ron = include_str!(
        "fixtures/project_ron_baselines/example_mod/battle/danmaku/cotton_first_turn.performance.ron"
    );
    let original: DanmakuPerformance =
        ron::from_str(original_ron).expect("hand-written cotton_first_turn should parse");

    let pretty_config = ron::ser::PrettyConfig::default()
        .struct_names(true)
        .enumerate_arrays(false);
    let generated_ron = ron::ser::to_string_pretty(&original, pretty_config)
        .expect("generated RON should serialize");
    let generated_round_trip = parse_generated_ron(&generated_ron);

    assert_same_performance(&generated_round_trip, &original);
}

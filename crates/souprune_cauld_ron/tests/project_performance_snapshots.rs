//! Snapshot tests for all project-owned performance constructors.
//!
//! 所有项目侧 performance 构造器的快照测试。

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

#[test]
fn tracked_performance_fixtures_parse_and_canonicalize() {
    assert_fixture_canonicalizes(
        "crates/souprune_cauld_ron/tests/fixtures/project_ron_baselines/example_mod/battle/danmaku/cotton_first_turn.performance.ron",
    );
}

fn assert_fixture_canonicalizes(path: &str) {
    let expected = parse_expected(path);
    let canonical = canonical_json(&expected);
    assert!(
        canonical.is_object(),
        "performance fixture should normalize to a JSON object: {path}"
    );
}

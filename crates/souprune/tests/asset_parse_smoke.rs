//! Smoke tests for high-value RON assets that guard refactors.
//!
//! 关键 RON 资产的烟测，用于给大重构兜底。

#[path = "test_support.rs"]
mod test_support;

use souprune::core;

fn read_project_file(project_name: &str, relative: &str) -> String {
    let path = test_support::named_project_root(project_name).join(relative);
    test_support::read_string(&path)
}

#[test]
fn input_config_assets_parse() {
    for project_name in ["example_mod", "example_am_mod"] {
        test_support::ensure_project_asset_from(project_name, "config/input.ron");
        let input: core::input::InputConfig =
            test_support::parse_project_ron_from(project_name, "config/input.ron");
        assert!(
            !input.actions.is_empty(),
            "input config should define actions for {project_name}"
        );
    }
}

#[test]
fn state_config_assets_parse() {
    for project_name in ["example_mod", "example_am_mod"] {
        test_support::ensure_project_asset_from(project_name, "config/states.ron");
        let state_config: core::state_config::StateConfig =
            test_support::parse_project_ron_from(project_name, "config/states.ron");
        assert!(
            !state_config.0.states.is_empty(),
            "states.ron should define at least one state for {project_name}"
        );
    }
}

#[test]
fn sequence_assets_parse_via_schema_and_runtime() {
    let files =
        test_support::list_project_files_with_suffix_from("example_mod", "states", ".sequence.ron");
    assert!(
        !files.is_empty(),
        "example_mod should contain sequence assets"
    );

    for relative in files {
        let contents = read_project_file("example_mod", &relative);
        let schema: souprune_schema::sequence::SequenceAsset =
            souprune_schema::from_ron_str(&contents)
                .unwrap_or_else(|err| panic!("Failed to parse schema {relative}: {err}"));
        let runtime = core::sequencer::runtime_asset_from_schema(&schema)
            .unwrap_or_else(|err| panic!("Failed to convert runtime {relative}: {err}"));
        assert!(
            !runtime.chapters.is_empty(),
            "sequence should contain chapters: {relative}"
        );
    }
}

#[test]
fn view_assets_parse_via_schema_and_runtime() {
    let files =
        test_support::list_project_files_with_suffix_from("example_mod", "states", ".view.ron");
    assert!(!files.is_empty(), "example_mod should contain view assets");

    for relative in files {
        let contents = read_project_file("example_mod", &relative);
        let schema: souprune_schema::view::ViewLayoutAsset =
            souprune_schema::from_ron_str(&contents)
                .unwrap_or_else(|err| panic!("Failed to parse schema {relative}: {err}"));
        let runtime = core::view::layout::runtime_view_layout_from_schema(&schema)
            .unwrap_or_else(|err| panic!("Failed to convert runtime {relative}: {err}"));
        assert!(
            !runtime.roots.is_empty(),
            "view should contain roots: {relative}"
        );
    }
}

#[test]
fn danmaku_performance_assets_parse() {
    for (project_name, relative) in [
        (
            "example_mod",
            "states/battle/danmaku/demo_attack.performance.ron",
        ),
        (
            "example_am_mod",
            "states/battle/danmaku/demo_attack.performance.ron",
        ),
    ] {
        test_support::ensure_project_asset_from(project_name, relative);
        let contents = read_project_file(project_name, relative);

        let runtime: core::danmaku::DanmakuPerformance =
            ron::from_str(&contents).unwrap_or_else(|err| {
                panic!("Failed to parse runtime {project_name}/{relative}: {err}")
            });
        let schema: souprune_schema::danmaku::DanmakuPerformance =
            souprune_schema::from_ron_str(&contents).unwrap_or_else(|err| {
                panic!("Failed to parse schema {project_name}/{relative}: {err}")
            });

        assert!(
            !runtime.timeline.is_empty(),
            "runtime danmaku timeline should not be empty: {project_name}/{relative}"
        );
        assert!(
            !schema.timeline.is_empty(),
            "schema danmaku timeline should not be empty: {project_name}/{relative}"
        );
    }
}

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
    let project_names = test_support::available_project_names(&[
        "example_mod",
        "undertale_preset",
        "example_am_mod",
    ]);
    if project_names.is_empty() {
        test_support::warn_missing_projects(
            "input_config_assets_parse",
            "example project worktrees are absent from this checkout",
        );
        return;
    }

    let mut parsed_any = false;
    for project_name in project_names {
        let root = test_support::named_project_root(project_name);
        if !root.join("config/input.ron").exists() {
            continue;
        }
        let input: core::input::InputConfig =
            test_support::parse_project_ron_from(project_name, "config/input.ron");
        assert!(
            !input.actions.is_empty(),
            "input config should define actions for {project_name}"
        );
        parsed_any = true;
    }
    assert!(
        parsed_any,
        "at least one project should have config/input.ron"
    );
}

#[test]
fn state_config_assets_parse() {
    let project_names = test_support::available_project_names(&[
        "example_mod",
        "undertale_preset",
        "example_am_mod",
    ]);
    if project_names.is_empty() {
        test_support::warn_missing_projects(
            "state_config_assets_parse",
            "example project worktrees are absent from this checkout",
        );
        return;
    }

    let mut parsed_any = false;
    for project_name in project_names {
        let root = test_support::named_project_root(project_name);
        if !root.join("config/states.ron").exists() {
            continue;
        }
        let state_config: core::state_config::StateConfig =
            test_support::parse_project_ron_from(project_name, "config/states.ron");
        assert!(
            !state_config.0.states.is_empty(),
            "states.ron should define at least one state for {project_name}"
        );
        parsed_any = true;
    }
    assert!(
        parsed_any,
        "at least one project should have config/states.ron"
    );
}

#[test]
fn sequence_assets_parse_via_schema_and_runtime() {
    if test_support::available_project_root("example_mod").is_none() {
        test_support::warn_missing_projects(
            "sequence_assets_parse_via_schema_and_runtime",
            "example_mod worktree is absent from this checkout",
        );
        return;
    }

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
    let candidates = ["undertale_preset", "example_mod"];
    let project_name = candidates
        .iter()
        .find(|name| {
            test_support::available_project_root(name).is_some()
                && !test_support::list_project_files_with_suffix_from(name, "states", ".view.ron")
                    .is_empty()
        })
        .copied();

    let Some(project_name) = project_name else {
        test_support::warn_missing_projects(
            "view_assets_parse_via_schema_and_runtime",
            "no project with view assets found in this checkout",
        );
        return;
    };

    let files =
        test_support::list_project_files_with_suffix_from(project_name, "states", ".view.ron");
    assert!(
        !files.is_empty(),
        "{project_name} should contain view assets"
    );

    for relative in files {
        let contents = read_project_file(project_name, &relative);
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
    let mut ran_any = false;

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
        if test_support::available_project_root(project_name).is_none() {
            continue;
        }

        ran_any = true;
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

    if !ran_any {
        test_support::warn_missing_projects(
            "danmaku_performance_assets_parse",
            "example project worktrees are absent from this checkout",
        );
    }
}

//! Architecture boundary tests for framework refactors.
//!
//! 框架重构的架构边界测试。

use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn forbidden_rust_source_hits(
    roots: &[PathBuf],
    display_root: &Path,
    forbidden: &[&str],
) -> Vec<String> {
    let mut hits = Vec::new();
    for root in roots {
        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
        {
            let path = entry.path();
            let text = fs::read_to_string(path).expect("read rust source");
            hits.extend(
                forbidden
                    .iter()
                    .filter(|token| text.contains(**token))
                    .map(|token| {
                        format!(
                            "{} contains `{}`",
                            path.strip_prefix(display_root).unwrap_or(path).display(),
                            token
                        )
                    }),
            );
        }
    }
    hits
}

fn forbidden_text_hits(
    roots: &[PathBuf],
    display_root: &Path,
    extensions: &[&str],
    forbidden: &[&str],
) -> Vec<String> {
    let mut hits = Vec::new();
    for root in roots {
        if !root.exists() {
            continue;
        }

        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.path().extension().is_some_and(|ext| {
                    extensions
                        .iter()
                        .any(|expected| ext.eq_ignore_ascii_case(expected))
                })
            })
        {
            let path = entry.path();
            let text = fs::read_to_string(path).expect("read text file");
            hits.extend(
                forbidden
                    .iter()
                    .filter(|token| text.contains(**token))
                    .map(|token| {
                        format!(
                            "{} contains `{}`",
                            path.strip_prefix(display_root).unwrap_or(path).display(),
                            token
                        )
                    }),
            );
        }
    }
    hits
}

#[test]
fn framework_no_longer_has_preset_layer_entrypoints() {
    let workspace = workspace_root();
    let removed_paths = [
        workspace.join("crates/souprune/src/preset"),
        workspace.join("crates/souprune/src/preset.rs"),
        workspace.join("crates/souprune/src/host_runtime"),
        workspace.join("crates/souprune/src/host_runtime.rs"),
    ];
    let existing: Vec<String> = removed_paths
        .iter()
        .filter(|path| path.exists())
        .map(|path| path.display().to_string())
        .collect();
    assert!(
        existing.is_empty(),
        "the framework must not keep compiled preset/host_runtime paths:\n{}",
        existing.join("\n")
    );

    let roots = [workspace.join("crates/souprune/src")];
    let forbidden = [
        "pub mod preset",
        "pub mod host_runtime",
        "PresetPlugin",
        "HostRuntimePlugin",
        "crate::preset::",
        "crate::host_runtime::",
        "super::preset::",
        "super::host_runtime::",
    ];
    let hits = forbidden_rust_source_hits(&roots, &workspace, &forbidden);

    assert!(
        hits.is_empty(),
        "preset layer entrypoints must be removed from framework source:\n{}",
        hits.join("\n")
    );
}

#[test]
fn view_authoring_surface_uses_space_not_legacy_boolean() {
    let workspace = workspace_root();
    let legacy_token = concat!("world", "_space");
    let roots = [
        workspace.join("crates/souprune/src/core/view"),
        workspace.join("crates/souprune/examples"),
        workspace.join("crates/souprune_schema/src"),
        workspace.join("crates/souprune_cauld_ron/tests"),
        workspace.join("projects"),
    ];
    let hits = forbidden_text_hits(&roots, &workspace, &["rs", "ron"], &[legacy_token]);

    assert!(
        hits.is_empty(),
        "View placement must use `space`, not the legacy world-space boolean:\n{}",
        hits.join("\n")
    );
}

#[test]
fn public_authoring_surface_does_not_add_ui_modules() {
    let workspace = workspace_root();
    let forbidden_paths = [
        workspace.join("crates/souprune/src/ui.rs"),
        workspace.join("crates/souprune/src/ui"),
        workspace.join("crates/souprune_schema/src/ui.rs"),
        workspace.join("crates/souprune_schema/src/ui"),
        workspace.join("crates/souprune_api/src/ui.rs"),
        workspace.join("crates/souprune_api/src/ui"),
        workspace.join("crates/souprune_sdk/src/ui.rs"),
        workspace.join("crates/souprune_sdk/src/ui"),
    ];
    let existing: Vec<String> = forbidden_paths
        .iter()
        .filter(|path| path.exists())
        .map(|path| path.display().to_string())
        .collect();

    assert!(
        existing.is_empty(),
        "author-facing public surfaces must use View naming, not new ui modules:\n{}",
        existing.join("\n")
    );

    let roots = [
        workspace.join("crates/souprune/src"),
        workspace.join("crates/souprune_schema/src"),
        workspace.join("crates/souprune_api/src"),
        workspace.join("crates/souprune_sdk/src"),
    ];
    let forbidden = [
        "pub mod ui",
        "pub use ui::",
        "pub use crate::ui::",
        "souprune::ui::",
    ];
    let hits = forbidden_rust_source_hits(&roots, &workspace, &forbidden);

    assert!(
        hits.is_empty(),
        "public authoring/API modules must expose View, not a second ui surface:\n{}",
        hits.join("\n")
    );
}

#[test]
fn public_docs_use_view_terminology_for_authoring_surface() {
    let workspace = workspace_root();
    let docs_roots = [
        workspace.join("doc/docs/en"),
        workspace.join("doc/docs/zh-hans"),
    ];
    let hits = forbidden_text_hits(
        &docs_roots,
        &workspace,
        &["md"],
        &["UI", "ui.md", "UI 设计", "UI 定制"],
    );

    assert!(
        hits.is_empty(),
        "author-facing docs must describe this surface as View:\n{}",
        hits.join("\n")
    );
}

#[test]
fn direct_view_acceptance_examples_do_not_double_register_fre_plugin() {
    let workspace = workspace_root();
    let example_paths = [
        workspace.join("crates/souprune/examples/view_taffy_layout.rs"),
        workspace.join("crates/souprune/examples/view_spatial_3d.rs"),
    ];
    let mut hits = Vec::new();

    for path in example_paths {
        let text = fs::read_to_string(&path).expect("read direct View acceptance example");
        if text.contains("FREPlugin::<GameActionDef>") {
            hits.push(format!(
                "{} explicitly adds `FREPlugin::<GameActionDef>`",
                path.strip_prefix(&workspace).unwrap_or(&path).display()
            ));
        }
    }

    assert!(
        hits.is_empty(),
        "direct View acceptance examples add `CorePlugin`, whose TopDownPlugin already registers FRE:\n{}",
        hits.join("\n")
    );
}

#[test]
fn framework_core_does_not_register_battle_or_overworld_as_first_class_modes() {
    let workspace = workspace_root();
    let roots = [workspace.join("crates/souprune/src/core")];
    let forbidden = [
        "pub mod battle_runtime",
        "pub mod overworld",
        "BattlePlugin",
        "OverworldPlugin",
        ".register(\"battle\")",
        ".register(\"overworld\")",
        ".register(\"top_down\")",
        "is_mode(\"battle\")",
        "is_mode(\"overworld\")",
        "is_mode(\"top_down\")",
        "InputContextId::Battle",
        "InputContextId::Overworld",
    ];
    let hits = forbidden_rust_source_hits(&roots, &workspace, &forbidden);

    assert!(
        hits.is_empty(),
        "core must provide primitives instead of first-class battle/overworld modes:\n{}",
        hits.join("\n")
    );
}

#[test]
fn framework_core_does_not_keep_project_path_or_old_specialized_names() {
    let workspace = workspace_root();
    let roots = [workspace.join("crates/souprune/src/core")];
    let forbidden = [
        "projects/undertale_preset",
        "projects/deltarune_preset",
        "BattleFREPlugin",
        "BattleFRESet",
        "BattleInputEntity",
        "BattleMenuStateTracker",
        "BattleSpeechBubblePlugin",
        "BattleSpeechBubbleRequest",
        "AlightMotionBattleConfig",
        "AlightMotionBattlePatterns",
        "battle/alight_motion_config.ron",
        "load_am_battle_config",
    ];
    let hits = forbidden_rust_source_hits(&roots, &workspace, &forbidden);

    assert!(
        hits.is_empty(),
        "core must not retain project paths or old specialized runtime names:\n{}",
        hits.join("\n")
    );
}

#[test]
fn preset_no_longer_defines_temporary_battle_box_runtime_types() {
    let workspace = workspace_root();
    let roots = [workspace.join("crates/souprune/src")];
    let forbidden = [
        "pub struct BattleBox",
        "pub struct BoundToBattleBox",
        "pub struct SplitBattleBox",
        "pub struct MergeBattleBoxes",
    ];
    let hits = forbidden_rust_source_hits(&roots, &workspace, &forbidden);

    assert!(
        hits.is_empty(),
        "temporary battle runtime types must live in project/prelude runtime, not preset:\n{}",
        hits.join("\n")
    );
}

#[test]
fn preset_no_longer_owns_battle_player_spawn_runtime() {
    let workspace = workspace_root();
    let roots = [workspace.join("crates/souprune/src/core/fixed_scene")];
    let forbidden = [
        ".battle_player.ron",
        "BattlePlayerConfig",
        "PlayerSpawnRequest",
        "process_battle_player_spawn_system",
        "Name::new(\"BattlePlayer\")",
    ];
    let hits = forbidden_rust_source_hits(&roots, &workspace, &forbidden);

    assert!(
        hits.is_empty(),
        "battle player spawn/config runtime must live in project/prelude runtime, not preset:\n{}",
        hits.join("\n")
    );
}

#[test]
fn framework_schema_no_longer_exposes_battle_player_config_files() {
    let workspace = workspace_root();
    let roots = [
        workspace.join("crates/souprune_schema/src"),
        workspace.join("crates/souprune_cauld_ron/src"),
        workspace.join("crates/souprune_lint/src"),
    ];
    let forbidden = [
        ".battle_player.ron",
        "BattlePlayerConfig",
        "BattleColliderShape",
        "BattleInvincibilityConfig",
    ];
    let hits = forbidden_rust_source_hits(&roots, &workspace, &forbidden);

    assert!(
        hits.is_empty(),
        "battle player config files must not remain a framework schema surface:\n{}",
        hits.join("\n")
    );
}

#[test]
fn framework_alight_motion_config_uses_generic_boundary_terms() {
    let workspace = workspace_root();
    let roots = [
        workspace.join("crates/souprune/src/core"),
        workspace.join("crates/souprune_schema/src"),
    ];
    let forbidden = [
        "battle_box_pattern",
        "default_battle_box_size",
        "AlightMotionBattleBoxMarker",
        "Battle box",
        "battle box",
    ];
    let hits = forbidden_rust_source_hits(&roots, &workspace, &forbidden);

    assert!(
        hits.is_empty(),
        "Alight Motion framework config must expose generic boundary terms, not project BattleBox semantics:\n{}",
        hits.join("\n")
    );
}

#[test]
fn framework_no_longer_owns_enemy_turn_selection_chapter() {
    let workspace = workspace_root();
    let roots = [
        workspace.join("crates/souprune/src"),
        workspace.join("crates/souprune_schema/src"),
    ];
    let forbidden = ["PickEnemyTurn", "preset/enemy_turn.rs"];
    let hits = forbidden_rust_source_hits(&roots, &workspace, &forbidden);

    assert!(
        hits.is_empty(),
        "enemy turn selection must be project/prelude logic, not framework schema or preset runtime:\n{}",
        hits.join("\n")
    );
}

#[test]
fn framework_preset_no_longer_owns_item_action_runtime() {
    let workspace = workspace_root();
    let roots = [workspace.join("crates/souprune/src")];
    let forbidden = [
        "item_actions",
        "UseItem",
        "CheckItem",
        "DropItem",
        "execute_use_item",
        "execute_check_item",
        "execute_drop_item",
    ];
    let hits = forbidden_rust_source_hits(&roots, &workspace, &forbidden);

    assert!(
        hits.is_empty(),
        "item use/check/drop actions must live in project/prelude runtime, not framework preset:\n{}",
        hits.join("\n")
    );
}

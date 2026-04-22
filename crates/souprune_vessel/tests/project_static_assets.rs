//! Regression tests for project-owned static canonical RON assets.
//!
//! 项目侧静态 canonical RON 资产的回归测试。

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use souprune_schema::RonFileKind;

#[test]
fn project_static_assets_canonicalize() {
    for (project_name, content_ron_dir) in [
        (
            "EVAERDRAD_storyspin_chara_fanmade",
            project_content_ron_dir("EVAERDRAD_storyspin_chara_fanmade"),
        ),
        ("epictale", project_content_ron_dir("epictale")),
        ("example_am_mod", project_content_ron_dir("example_am_mod")),
        (
            "example_battle_mod",
            project_content_ron_dir("example_battle_mod"),
        ),
        ("example_mod", project_content_ron_dir("example_mod")),
        (
            "undertale_preset",
            project_content_ron_dir("undertale_preset"),
        ),
    ] {
        let component_path = build_content_component(project_name);
        let generated_files = vessel::load_component_files(&component_path)
            .expect("project content guest should load through vessel host");
        assert!(
            !generated_files.is_empty(),
            "content guest should emit at least one file"
        );

        let source_paths = collect_relative_ron_paths(&content_ron_dir);
        let generated_by_path = generated_files
            .iter()
            .filter(|file| source_paths.contains(file.path.to_string_lossy().as_ref()))
            .map(|file| {
                (
                    file.path.to_string_lossy().replace('\\', "/"),
                    file.ron_text.as_str(),
                )
            })
            .collect::<HashMap<_, _>>();

        let generated_paths = generated_by_path.keys().cloned().collect::<BTreeSet<_>>();
        assert_eq!(
            generated_paths,
            source_paths,
            "content guest should emit every file under {} exactly once",
            content_ron_dir.display()
        );

        assert_project_specific_canonical_fields(project_name, &generated_by_path);
        assert_generated_assets_semantically_match_sources(&content_ron_dir, &generated_by_path);

        for (path, ron_text) in &generated_by_path {
            assert!(
                !ron_text.contains("#![enable("),
                "canonical output should not contain extension headers: {path}"
            );
        }
    }
}

fn build_content_component(project_name: &str) -> PathBuf {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let target_dir = workspace_root.join("target/content-wasm-tests");
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../projects")
        .join(project_name)
        .join("content/Cargo.toml");

    let build_status = Command::new("cargo")
        .env("CARGO_TARGET_DIR", &target_dir)
        .args([
            "build",
            "--manifest-path",
            &manifest_path.to_string_lossy(),
            "--target",
            "wasm32-wasip2",
        ])
        .status()
        .unwrap_or_else(|err| panic!("failed to invoke cargo build for {project_name}: {err}"));
    assert!(
        build_status.success(),
        "{project_name} content guest should build successfully"
    );

    target_dir.join("wasm32-wasip2/debug/content.wasm")
}

fn assert_generated_assets_semantically_match_sources(
    content_ron_dir: &Path,
    generated_by_path: &HashMap<String, &str>,
) {
    for (path, generated) in generated_by_path {
        let source_path = content_ron_dir.join(path);
        let source = fs::read_to_string(&source_path).unwrap_or_else(|err| {
            panic!(
                "source RON should be readable for {}: {err}",
                source_path.display()
            )
        });
        let kind = RonFileKind::from_path(path)
            .unwrap_or_else(|| panic!("unsupported RON kind for generated path: {path}"));
        let source_json = normalized_ron_json(kind, &source);
        let generated_json = normalized_ron_json(kind, generated);
        assert_eq!(
            generated_json, source_json,
            "canonical output should remain semantically identical for {path}"
        );
    }
}

fn assert_project_specific_canonical_fields(
    project_name: &str,
    generated_by_path: &HashMap<String, &str>,
) {
    match project_name {
        "undertale_preset" => {
            let menu_cancel = generated_by_path
                .get("battle/rules/menu_cancel.fre.ron")
                .expect("undertale preset should emit menu_cancel.fre.ron");
            assert!(
                menu_cancel.contains("enums: {"),
                "canonical FRE output should preserve enums in {project_name}"
            );
        }
        "example_mod" => {
            let mad_dummy = generated_by_path
                .get("battle/view/mad_dummy.view.ron")
                .expect("example_mod should emit mad_dummy.view.ron");
            assert!(
                mad_dummy.contains("coordinate_system: YDown"),
                "canonical view output should preserve coordinate_system in {project_name}"
            );
        }
        _ => {}
    }
}

fn normalized_ron_json(kind: RonFileKind, ron_text: &str) -> Value {
    match kind {
        RonFileKind::View => normalized_json::<souprune_schema::view::ViewLayoutAsset>(ron_text),
        RonFileKind::SdfStructure => {
            normalized_json::<souprune_schema::view::SdfStructureAsset>(ron_text)
        }
        RonFileKind::Performance => {
            normalized_json::<souprune_schema::danmaku::DanmakuPerformance>(ron_text)
        }
        RonFileKind::Sequence => {
            normalized_json::<souprune_schema::sequence::SequenceAsset>(ron_text)
        }
        RonFileKind::Enemy => normalized_json::<souprune_schema::enemy::EnemyDef>(ron_text),
        RonFileKind::Items => normalized_json::<souprune_schema::item::ItemListAsset>(ron_text),
        RonFileKind::BattlePlayer => {
            normalized_json::<souprune_schema::battle::BattlePlayerConfig>(ron_text)
        }
        RonFileKind::Fre => normalized_json::<souprune_schema::fre::FreAsset>(ron_text),
        RonFileKind::Dialogue => {
            normalized_json::<souprune_schema::dialogue::DialogueConfig>(ron_text)
        }
        RonFileKind::Input => normalized_json::<souprune_schema::config::InputConfig>(ron_text),
        RonFileKind::Flow => normalized_json::<souprune_schema::config::StateConfig>(ron_text),
        RonFileKind::TouchLayout => {
            normalized_json::<souprune_schema::config::TouchLayoutDef>(ron_text)
        }
        RonFileKind::AlightMotionConfig => {
            normalized_json::<souprune_schema::config::AlightMotionBattleConfig>(ron_text)
        }
        RonFileKind::Character => {
            normalized_json::<souprune_schema::character::CharacterAsset>(ron_text)
        }
        RonFileKind::AnimationConfig => {
            normalized_json::<souprune_schema::character::AnimationConfigAsset>(ron_text)
        }
        RonFileKind::PlayerBehavior => {
            normalized_json::<souprune_schema::overworld::PlayerBehaviorFile>(ron_text)
        }
        RonFileKind::ChaseConfig => {
            normalized_json::<souprune_schema::overworld::ChaseConfig>(ron_text)
        }
    }
}

fn normalized_json<T>(ron_text: &str) -> Value
where
    T: DeserializeOwned + Serialize,
{
    let value: T = ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(ron_text)
        .expect("RON should parse against canonical schema");
    serde_json::to_value(&value).expect("canonical value should serialize to JSON")
}

fn project_content_ron_dir(project_name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../projects")
        .join(project_name)
        .join("content/ron")
}

fn collect_relative_ron_paths(root: &Path) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    collect_relative_ron_paths_inner(root, root, &mut paths);
    paths
}

fn collect_relative_ron_paths_inner(root: &Path, current: &Path, out: &mut BTreeSet<String>) {
    for entry in fs::read_dir(current).expect("content/ron directory should be readable") {
        let entry = entry.expect("directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_relative_ron_paths_inner(root, &path, out);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "ron") {
            let relative = path
                .strip_prefix(root)
                .expect("file should be under content/ron")
                .to_string_lossy()
                .replace('\\', "/");
            out.insert(relative);
        }
    }
}

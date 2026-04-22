//! Regression tests for project-owned static canonical RON assets.
//!
//! 项目侧静态 canonical RON 资产的回归测试。

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

#[path = "../../../projects/EVAERDRAD_storyspin_chara_fanmade/content/src/static_assets.rs"]
mod chara_mod_static_assets;
#[path = "../../../projects/epictale/content/src/static_assets.rs"]
mod epictale_static_assets;
#[path = "../../../projects/example_am_mod/content/src/static_assets.rs"]
mod example_am_mod_static_assets;
#[path = "../../../projects/example_battle_mod/content/src/static_assets.rs"]
mod example_battle_mod_static_assets;
#[path = "../../../projects/example_mod/content/src/static_assets.rs"]
mod example_mod_static_assets;
#[path = "../../../projects/undertale_preset/content/src/static_assets.rs"]
mod undertale_preset_static_assets;

use souprune_schema::RonFileKind;
use souprune_vessel::guest::Registry;

#[test]
fn project_static_assets_canonicalize() {
    for (project_name, content_ron_dir, emit_all) in [
        (
            "EVAERDRAD_storyspin_chara_fanmade",
            project_content_ron_dir("EVAERDRAD_storyspin_chara_fanmade"),
            chara_mod_static_assets::emit_all as fn(&mut Registry) -> anyhow::Result<()>,
        ),
        (
            "epictale",
            project_content_ron_dir("epictale"),
            epictale_static_assets::emit_all,
        ),
        (
            "example_am_mod",
            project_content_ron_dir("example_am_mod"),
            example_am_mod_static_assets::emit_all,
        ),
        (
            "example_battle_mod",
            project_content_ron_dir("example_battle_mod"),
            example_battle_mod_static_assets::emit_all,
        ),
        (
            "example_mod",
            project_content_ron_dir("example_mod"),
            example_mod_static_assets::emit_all,
        ),
        (
            "undertale_preset",
            project_content_ron_dir("undertale_preset"),
            undertale_preset_static_assets::emit_all,
        ),
    ] {
        let mut registry = Registry::new();
        emit_all(&mut registry).expect("project static assets should canonicalize");
        let generated_files = registry.into_generated_files();
        assert!(
            !generated_files.is_empty(),
            "static asset registry should emit at least one file"
        );

        let generated_paths = generated_files
            .iter()
            .map(|file| file.path.clone())
            .collect::<BTreeSet<_>>();
        let source_paths = collect_relative_ron_paths(&content_ron_dir);
        assert_eq!(
            generated_paths,
            source_paths,
            "static asset registry should emit every file under {} exactly once",
            content_ron_dir.display()
        );

        let generated_by_path = generated_files
            .iter()
            .map(|file| (file.path.as_str(), file.ron_text.as_str()))
            .collect::<HashMap<_, _>>();

        assert_project_specific_canonical_fields(project_name, &generated_by_path);
        assert_generated_assets_semantically_match_sources(&content_ron_dir, &generated_by_path);

        for file in &generated_files {
            assert!(
                !file.ron_text.contains("#![enable("),
                "canonical output should not contain extension headers: {}",
                file.path
            );
        }
    }
}

fn assert_generated_assets_semantically_match_sources(
    content_ron_dir: &Path,
    generated_by_path: &HashMap<&str, &str>,
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
    generated_by_path: &HashMap<&str, &str>,
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

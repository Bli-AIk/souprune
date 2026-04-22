//! Regression tests for project-owned generated canonical RON assets.
//!
//! 项目侧生成的 canonical RON 资产回归测试。

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use souprune_schema::RonFileKind;

const EXPECTED_BASELINE_KINDS: &[&str] = &[
    "AlightMotionConfig",
    "AnimationConfig",
    "BattlePlayer",
    "Character",
    "ChaseConfig",
    "Dialogue",
    "Enemy",
    "Flow",
    "Fre",
    "Input",
    "Items",
    "Performance",
    "PlayerBehavior",
    "SdfStructure",
    "Sequence",
    "TouchLayout",
    "View",
];

#[derive(Debug)]
struct RonBaseline {
    project_name: String,
    relative_path: String,
    source_path: PathBuf,
}

#[test]
fn project_generated_assets_canonicalize_against_selected_baselines() {
    let baselines = collect_project_ron_baselines();
    assert_baselines_cover_each_ron_kind_once(&baselines);

    for (project_name, project_baselines) in baselines_by_project(&baselines) {
        let component_path = build_content_component(project_name);
        let generated_files = vessel::load_component_files(&component_path)
            .expect("project content guest should load through vessel host");
        assert!(
            !generated_files.is_empty(),
            "content guest should emit at least one file"
        );

        let generated_by_path = generated_files
            .iter()
            .map(|file| {
                (
                    file.path.to_string_lossy().replace('\\', "/"),
                    file.ron_text.as_str(),
                )
            })
            .collect::<HashMap<_, _>>();

        assert_generated_assets_parse(project_name, &generated_by_path);
        assert_generated_assets_semantically_match_baselines(
            project_name,
            &project_baselines,
            &generated_by_path,
        );
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

fn assert_generated_assets_semantically_match_baselines(
    project_name: &str,
    baselines: &[&RonBaseline],
    generated_by_path: &HashMap<String, &str>,
) {
    for baseline in baselines {
        let generated = generated_by_path
            .get(&baseline.relative_path)
            .unwrap_or_else(|| {
                panic!(
                    "{project_name} should emit baseline fixture path {}",
                    baseline.relative_path
                )
            });
        let source = fs::read_to_string(&baseline.source_path).unwrap_or_else(|err| {
            panic!(
                "source RON should be readable for {}: {err}",
                baseline.source_path.display()
            )
        });
        let kind = RonFileKind::from_path(&baseline.relative_path).unwrap_or_else(|| {
            panic!(
                "unsupported RON kind for baseline path: {}",
                baseline.relative_path
            )
        });
        let source_json = normalized_ron_json(kind, &source);
        let generated_json = normalized_ron_json(kind, generated);
        assert_eq!(
            generated_json, source_json,
            "canonical output should remain semantically identical for {}",
            baseline.relative_path
        );
    }
}

fn assert_generated_assets_parse(project_name: &str, generated_by_path: &HashMap<String, &str>) {
    for (path, ron_text) in generated_by_path {
        assert!(
            !ron_text.contains("#![enable("),
            "canonical output should not contain extension headers: {project_name}:{path}"
        );
        let kind = RonFileKind::from_path(path)
            .unwrap_or_else(|| panic!("unsupported generated RON kind: {project_name}:{path}"));
        let _ = normalized_ron_json(kind, ron_text);
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

fn collect_project_ron_baselines() -> Vec<RonBaseline> {
    let root = project_ron_baselines_dir();
    let mut baselines = Vec::new();
    for project_entry in fs::read_dir(&root).unwrap_or_else(|err| {
        panic!(
            "project RON baseline fixture directory should be readable: {}: {err}",
            root.display()
        )
    }) {
        let project_entry = project_entry.expect("project baseline entry should be readable");
        if !project_entry.path().is_dir() {
            continue;
        }
        let project_name = project_entry.file_name().to_string_lossy().into_owned();
        collect_project_ron_baselines_inner(
            &project_name,
            &project_entry.path(),
            &project_entry.path(),
            &mut baselines,
        );
    }
    baselines.sort_by(|left, right| {
        (&left.project_name, &left.relative_path).cmp(&(&right.project_name, &right.relative_path))
    });
    baselines
}

fn project_ron_baselines_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/project_ron_baselines")
}

fn collect_project_ron_baselines_inner(
    project_name: &str,
    root: &Path,
    current: &Path,
    out: &mut Vec<RonBaseline>,
) {
    for entry in fs::read_dir(current).expect("RON baseline directory should be readable") {
        let entry = entry.expect("directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_project_ron_baselines_inner(project_name, root, &path, out);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "ron") {
            let relative_path = path
                .strip_prefix(root)
                .expect("fixture file should be under project baseline root")
                .to_string_lossy()
                .replace('\\', "/");
            out.push(RonBaseline {
                project_name: project_name.to_owned(),
                relative_path,
                source_path: path,
            });
        }
    }
}

fn assert_baselines_cover_each_ron_kind_once(baselines: &[RonBaseline]) {
    let mut seen = BTreeSet::new();
    for baseline in baselines {
        let kind = RonFileKind::from_path(&baseline.relative_path).unwrap_or_else(|| {
            panic!(
                "unsupported RON kind for baseline path: {}:{}",
                baseline.project_name, baseline.relative_path
            )
        });
        let kind_name = format!("{kind:?}");
        assert!(
            seen.insert(kind_name.clone()),
            "duplicate baseline kind {kind_name} at {}:{}",
            baseline.project_name,
            baseline.relative_path
        );
    }
    let expected = EXPECTED_BASELINE_KINDS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let actual = seen.iter().map(String::as_str).collect::<BTreeSet<_>>();
    assert_eq!(
        actual, expected,
        "project RON baseline fixtures should contain exactly one sample per supported RON kind"
    );
}

fn baselines_by_project(baselines: &[RonBaseline]) -> BTreeMap<&str, Vec<&RonBaseline>> {
    let mut grouped = BTreeMap::new();
    for baseline in baselines {
        grouped
            .entry(baseline.project_name.as_str())
            .or_insert_with(Vec::new)
            .push(baseline);
    }
    grouped
}

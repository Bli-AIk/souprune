//! Host-side SoupRune integration for Vessel output.
//!
//! Vessel 输出的宿主侧 SoupRune 集成。

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use souprune_schema::RonFileKind;
use std::path::Path;

/// Build a Vessel component with SoupRune semantic RON preservation.
///
/// 使用 SoupRune 语义 RON 保留策略构建 Vessel component。
pub fn build_component(
    component_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<vessel::BuildSummary> {
    vessel::build_component_with_options(
        component_path,
        output_dir,
        souprune_write_generated_files_options(),
    )
}

/// Write generated files with SoupRune semantic RON preservation.
///
/// 使用 SoupRune 语义 RON 保留策略写入生成文件。
pub fn write_generated_files(files: &[vessel::GeneratedRonFile], output_dir: &Path) -> Result<()> {
    vessel::write_generated_files_with_options(
        files,
        output_dir,
        souprune_write_generated_files_options(),
    )
}

fn souprune_write_generated_files_options() -> vessel::WriteGeneratedFilesOptions<'static> {
    vessel::WriteGeneratedFilesOptions {
        semantic_equal: Some(&souprune_semantically_equal_ron),
    }
}

fn souprune_semantically_equal_ron(relative_path: &str, left: &str, right: &str) -> bool {
    let Some(kind) = RonFileKind::from_path(relative_path) else {
        return false;
    };

    match kind {
        RonFileKind::View => normalized_json::<souprune_schema::view::ViewLayoutAsset>(left, right),
        RonFileKind::SdfStructure => {
            normalized_json::<souprune_schema::view::SdfStructureAsset>(left, right)
        }
        RonFileKind::Performance => {
            normalized_json::<souprune_schema::danmaku::DanmakuPerformance>(left, right)
        }
        RonFileKind::Sequence => {
            normalized_json::<souprune_schema::sequence::SequenceAsset>(left, right)
        }
        RonFileKind::Enemy => normalized_json::<souprune_schema::enemy::EnemyDef>(left, right),
        RonFileKind::Items => normalized_json::<souprune_schema::item::ItemListAsset>(left, right),
        RonFileKind::BattlePlayer => {
            normalized_json::<souprune_schema::battle::BattlePlayerConfig>(left, right)
        }
        RonFileKind::Fre => normalized_json::<souprune_schema::fre::FreAsset>(left, right),
        RonFileKind::Dialogue => {
            normalized_json::<souprune_schema::dialogue::DialogueConfig>(left, right)
        }
        RonFileKind::Input => normalized_json::<souprune_schema::config::InputConfig>(left, right),
        RonFileKind::Flow => normalized_json::<souprune_schema::config::StateConfig>(left, right),
        RonFileKind::TouchLayout => {
            normalized_json::<souprune_schema::config::TouchLayoutDef>(left, right)
        }
        RonFileKind::AlightMotionConfig => {
            normalized_json::<souprune_schema::config::AlightMotionBattleConfig>(left, right)
        }
        RonFileKind::Character => {
            normalized_json::<souprune_schema::character::CharacterAsset>(left, right)
        }
        RonFileKind::AnimationConfig => {
            normalized_json::<souprune_schema::character::AnimationConfigAsset>(left, right)
        }
        RonFileKind::PlayerBehavior => {
            normalized_json::<souprune_schema::overworld::PlayerBehaviorFile>(left, right)
        }
        RonFileKind::ChaseConfig => {
            normalized_json::<souprune_schema::overworld::ChaseConfig>(left, right)
        }
    }
}

fn normalized_json<T>(left: &str, right: &str) -> bool
where
    T: DeserializeOwned + Serialize,
{
    let options =
        ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
    let Ok(left) = options.from_str::<T>(left) else {
        return false;
    };
    let Ok(right) = options.from_str::<T>(right) else {
        return false;
    };
    let Ok(left) = serde_json::to_value(left) else {
        return false;
    };
    let Ok(right) = serde_json::to_value(right) else {
        return false;
    };
    left == right
}

//! Guest-side helpers for Vessel content modules.
//!
//! Vessel 内容模块的 guest 侧辅助工具。

use anyhow::{Context, Result};
use serde::Serialize;
use serde::de::DeserializeOwned;
use souprune_schema::RonFileKind;

wit_bindgen::generate!({
    path: "../vessel/wit",
    world: "content-module",
    pub_export_macro: true,
});

/// Re-exports of generated WIT guest-side items.
///
/// WIT guest 侧生成项的重导出。
pub mod wit {
    pub use super::Guest;
    pub use super::export;
    pub use super::vessel::build::types::GeneratedFile;
}

/// Guest-side registry that collects generated RON files before export.
///
/// guest 侧注册表，用于在导出前收集生成的 RON 文件。
#[derive(Default)]
pub struct Registry {
    files: Vec<wit::GeneratedFile>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Emit a serializable value as pretty-printed RON.
    ///
    /// 将一个可序列化值发射为格式化 RON。
    pub fn emit_ron<T: Serialize>(&mut self, path: impl Into<String>, value: &T) -> Result<()> {
        let ron_str = format_pretty_ron(value)?;
        self.files.push(wit::GeneratedFile {
            path: path.into(),
            ron_text: format!("{ron_str}\n"),
        });
        Ok(())
    }

    /// Bootstrap a legacy source RON file into canonical output.
    ///
    /// This helper exists for migration, fixture generation, and regression baselines.
    /// New authoring code should prefer typed Rust construction plus `emit_ron`.
    ///
    /// 将旧的源 RON 文件 bootstrap 成 canonical 输出。
    ///
    /// 该 helper 仅用于迁移、fixture 生成与回归基线。
    /// 新的 authoring 代码应优先使用类型化 Rust 构造配合 `emit_ron`。
    pub fn emit_canonical_source(
        &mut self,
        path: impl Into<String>,
        ron_source: &str,
    ) -> Result<()> {
        let path = path.into();
        let ron_text = canonicalize_ron_source(&path, ron_source)
            .with_context(|| format!("failed to canonicalize source RON: {path}"))?;
        self.files.push(wit::GeneratedFile {
            path,
            ron_text: format!("{ron_text}\n"),
        });
        Ok(())
    }

    pub fn into_generated_files(self) -> Vec<wit::GeneratedFile> {
        self.files
    }
}

fn format_pretty_ron<T: Serialize>(value: &T) -> Result<String> {
    let config = ron::ser::PrettyConfig::default().enumerate_arrays(false);
    Ok(ron::ser::to_string_pretty(value, config)?)
}

fn canonicalize_ron_source(path: &str, ron_source: &str) -> Result<String> {
    let kind = RonFileKind::from_path(path)
        .ok_or_else(|| anyhow::anyhow!("unsupported RON file kind for path: {path}"))?;

    match kind {
        RonFileKind::View => canonicalize::<souprune_schema::view::ViewLayoutAsset>(ron_source),
        RonFileKind::SdfStructure => {
            canonicalize::<souprune_schema::view::SdfStructureAsset>(ron_source)
        }
        RonFileKind::Performance => {
            canonicalize::<souprune_schema::danmaku::DanmakuPerformance>(ron_source)
        }
        RonFileKind::Sequence => {
            canonicalize::<souprune_schema::sequence::SequenceAsset>(ron_source)
        }
        RonFileKind::Enemy => canonicalize::<souprune_schema::enemy::EnemyDef>(ron_source),
        RonFileKind::Items => canonicalize::<souprune_schema::item::ItemListAsset>(ron_source),
        RonFileKind::BattlePlayer => {
            canonicalize::<souprune_schema::battle::BattlePlayerConfig>(ron_source)
        }
        RonFileKind::Fre => canonicalize::<souprune_schema::fre::FreAsset>(ron_source),
        RonFileKind::Dialogue => {
            canonicalize::<souprune_schema::dialogue::DialogueConfig>(ron_source)
        }
        RonFileKind::Input => canonicalize::<souprune_schema::config::InputConfig>(ron_source),
        RonFileKind::Flow => canonicalize::<souprune_schema::config::StateConfig>(ron_source),
        RonFileKind::TouchLayout => {
            canonicalize::<souprune_schema::config::TouchLayoutDef>(ron_source)
        }
        RonFileKind::AlightMotionConfig => {
            canonicalize::<souprune_schema::config::AlightMotionBattleConfig>(ron_source)
        }
        RonFileKind::Character => {
            canonicalize::<souprune_schema::character::CharacterAsset>(ron_source)
        }
        RonFileKind::AnimationConfig => {
            canonicalize::<souprune_schema::character::AnimationConfigAsset>(ron_source)
        }
        RonFileKind::PlayerBehavior => {
            canonicalize::<souprune_schema::overworld::PlayerBehaviorFile>(ron_source)
        }
        RonFileKind::ChaseConfig => {
            canonicalize::<souprune_schema::overworld::ChaseConfig>(ron_source)
        }
    }
}

fn canonicalize<T: DeserializeOwned + Serialize>(ron_source: &str) -> Result<String> {
    let parsed: T = ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(ron_source)?;
    format_pretty_ron(&parsed)
}

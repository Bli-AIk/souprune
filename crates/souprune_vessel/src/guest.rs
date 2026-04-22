//! Guest-side helpers for Vessel content modules.
//!
//! Vessel 内容模块的 guest 侧辅助工具。

use anyhow::{Context, Result, anyhow, bail};
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

/// Output mapping override for auto-emitted assets.
///
/// 自动发射资产时的输出映射覆盖配置。
#[derive(Debug, Clone, Default)]
pub struct EmitPathConfig {
    output_path: Option<String>,
}

impl EmitPathConfig {
    /// Build a config that uses convention defaults.
    ///
    /// 构建一份使用约定默认值的配置。
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the inferred output path with an explicit canonical RON path.
    ///
    /// 用显式 canonical RON 路径覆盖自动推导的输出路径。
    pub fn output_path(mut self, output_path: impl Into<String>) -> Self {
        self.output_path = Some(output_path.into());
        self
    }

    fn output_path_override(&self) -> Option<&str> {
        self.output_path.as_deref()
    }
}

/// Trait implemented by typed canonical RON assets.
///
/// 由类型化 canonical RON 资产实现的 trait。
pub trait CanonicalRonAsset: Serialize {
    /// The canonical RON kind emitted by this typed value.
    ///
    /// 当前类型化值发射出的 canonical RON kind。
    fn ron_file_kind() -> RonFileKind;
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

    /// Emit an asset to its default canonical output path inferred from the source file.
    ///
    /// 将资产发射到由源码文件自动推导出的默认 canonical 输出路径。
    pub fn emit_auto<T: CanonicalRonAsset>(&mut self, source_file: &str, value: &T) -> Result<()> {
        self.emit_auto_with(source_file, value, EmitPathConfig::default())
    }

    /// Emit an asset using convention defaults, with optional explicit path override.
    ///
    /// 使用约定默认值发射资产，并允许显式路径覆盖。
    pub fn emit_auto_with<T: CanonicalRonAsset>(
        &mut self,
        source_file: &str,
        value: &T,
        config: EmitPathConfig,
    ) -> Result<()> {
        let output_path = infer_output_path::<T>(source_file, &config)?;
        self.emit_ron(output_path, value)
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

/// Infer the canonical output path for a typed asset from its authoring source file.
///
/// 根据 authoring 源文件为类型化资产推导 canonical 输出路径。
pub fn infer_output_path<T: CanonicalRonAsset>(
    source_file: &str,
    config: &EmitPathConfig,
) -> Result<String> {
    if let Some(output_path) = config.output_path_override() {
        validate_output_path_override(output_path)?;
        return Ok(output_path.to_string());
    }

    let relative_source = source_relative_to_content_root(source_file)?;
    let logical_stem = relative_source
        .strip_suffix(".rs")
        .ok_or_else(|| anyhow!("content source file must end with `.rs`: {source_file}"))?;

    if logical_stem == "lib" || logical_stem == "support" || logical_stem.starts_with("support/") {
        bail!("helper files cannot be auto-emitted as canonical assets: {source_file}");
    }

    Ok(format!(
        "{logical_stem}{}",
        ron_kind_suffix(T::ron_file_kind())
    ))
}

fn validate_output_path_override(output_path: &str) -> Result<()> {
    if output_path.is_empty() {
        bail!("output path override cannot be empty");
    }
    if output_path.starts_with('/') || output_path.starts_with('\\') {
        bail!("output path override must be relative: {output_path}");
    }
    if RonFileKind::from_path(output_path).is_none() {
        bail!("output path override must target a supported canonical RON path: {output_path}");
    }
    Ok(())
}

fn source_relative_to_content_root(source_file: &str) -> Result<String> {
    let normalized = source_file.replace('\\', "/");
    if let Some(stripped) = normalized.strip_prefix("src/") {
        return Ok(stripped.to_string());
    }
    if let Some((_, stripped)) = normalized.rsplit_once("/src/") {
        return Ok(stripped.to_string());
    }
    bail!("content source file must live under `src/`: {source_file}");
}

fn ron_kind_suffix(kind: RonFileKind) -> &'static str {
    match kind {
        RonFileKind::View => ".view.ron",
        RonFileKind::SdfStructure => ".sdf.ron",
        RonFileKind::Performance => ".performance.ron",
        RonFileKind::Sequence => ".sequence.ron",
        RonFileKind::Enemy => ".enemy.ron",
        RonFileKind::Items => ".items.ron",
        RonFileKind::BattlePlayer => ".battle_player.ron",
        RonFileKind::Fre => ".fre.ron",
        RonFileKind::Dialogue => ".ron",
        RonFileKind::Input => ".ron",
        RonFileKind::Flow => ".ron",
        RonFileKind::TouchLayout => ".ron",
        RonFileKind::AlightMotionConfig => ".ron",
        RonFileKind::Character => ".character.ron",
        RonFileKind::AnimationConfig => ".animation_config.ron",
        RonFileKind::PlayerBehavior => ".ron",
        RonFileKind::ChaseConfig => ".ron",
    }
}

macro_rules! impl_canonical_ron_asset {
    ($ty:path => $kind:expr) => {
        impl CanonicalRonAsset for $ty {
            fn ron_file_kind() -> RonFileKind {
                $kind
            }
        }
    };
}

impl_canonical_ron_asset!(souprune_schema::view::ViewLayout => RonFileKind::View);
impl_canonical_ron_asset!(souprune_schema::view::SdfStructure => RonFileKind::SdfStructure);
impl_canonical_ron_asset!(souprune_schema::danmaku::DanmakuPerformance => RonFileKind::Performance);
impl_canonical_ron_asset!(souprune_schema::sequence::SequenceAsset => RonFileKind::Sequence);
impl_canonical_ron_asset!(souprune_schema::enemy::EnemyDef => RonFileKind::Enemy);
impl_canonical_ron_asset!(souprune_schema::item::ItemListAsset => RonFileKind::Items);
impl_canonical_ron_asset!(souprune_schema::battle::BattlePlayerConfig => RonFileKind::BattlePlayer);
impl_canonical_ron_asset!(souprune_schema::fre::FreAsset => RonFileKind::Fre);
impl_canonical_ron_asset!(souprune_schema::dialogue::DialogueConfig => RonFileKind::Dialogue);
impl_canonical_ron_asset!(souprune_schema::config::InputConfig => RonFileKind::Input);
impl_canonical_ron_asset!(souprune_schema::config::StateConfig => RonFileKind::Flow);
impl_canonical_ron_asset!(souprune_schema::config::TouchLayoutDef => RonFileKind::TouchLayout);
impl_canonical_ron_asset!(
    souprune_schema::config::AlightMotionBattleConfig => RonFileKind::AlightMotionConfig
);
impl_canonical_ron_asset!(souprune_schema::character::CharacterAsset => RonFileKind::Character);
impl_canonical_ron_asset!(
    souprune_schema::character::AnimationConfigAsset => RonFileKind::AnimationConfig
);
impl_canonical_ron_asset!(souprune_schema::overworld::PlayerBehaviorFile => RonFileKind::PlayerBehavior);
impl_canonical_ron_asset!(souprune_schema::overworld::ChaseConfig => RonFileKind::ChaseConfig);

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

#[cfg(test)]
mod tests {
    use super::{EmitPathConfig, infer_output_path};

    #[test]
    fn infers_view_output_from_relative_source_path() {
        let path = infer_output_path::<souprune_schema::view::ViewLayout>(
            "src/battle/view/battle_bg.rs",
            &EmitPathConfig::default(),
        )
        .expect("view path should infer");
        assert_eq!(path, "battle/view/battle_bg.view.ron");
    }

    #[test]
    fn infers_performance_output_from_absolute_source_path() {
        let path = infer_output_path::<souprune_schema::danmaku::DanmakuPerformance>(
            "/tmp/example/content/src/battle/danmaku/cotton_first_turn.rs",
            &EmitPathConfig::default(),
        )
        .expect("performance path should infer");
        assert_eq!(path, "battle/danmaku/cotton_first_turn.performance.ron");
    }

    #[test]
    fn explicit_output_path_overrides_convention() {
        let config = EmitPathConfig::new().output_path("narrative/dialogue.fre.ron");
        let path = infer_output_path::<souprune_schema::fre::FreAsset>(
            "src/narrative/dialogue_rules.rs",
            &config,
        )
        .expect("override path should infer");
        assert_eq!(path, "narrative/dialogue.fre.ron");
    }

    #[test]
    fn rejects_helper_files_for_auto_emit() {
        let error = infer_output_path::<souprune_schema::fre::FreAsset>(
            "src/support/dialogue_rules.rs",
            &EmitPathConfig::default(),
        )
        .expect_err("support files must not auto emit");
        assert!(
            error
                .to_string()
                .contains("helper files cannot be auto-emitted"),
            "unexpected error: {error}"
        );
    }
}

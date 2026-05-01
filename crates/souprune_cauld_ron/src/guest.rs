//! Guest-side helpers for Cauld-ron content modules.
//!
//! Cauld-ron 内容模块的 guest 侧辅助工具。

use anyhow::{Result, anyhow, bail};
use serde::Serialize;
use souprune_schema::RonFileKind;
use std::collections::HashMap;

use crate::float_output::{FloatOutputConfig, normalize_ron_floats};

/// Re-exports of generated WIT guest-side items.
///
/// WIT guest 侧生成项的重导出。
pub mod wit {
    pub use cauld_ron::guest::wit::{GeneratedFile, Guest, export};
}

/// Guest-side registry that collects generated RON files before export.
///
/// guest 侧注册表，用于在导出前收集生成的 RON 文件。
#[derive(Default)]
pub struct Registry {
    files: Vec<wit::GeneratedFile>,
    float_output: FloatOutputConfig,
    path_float_outputs: HashMap<String, FloatOutputConfig>,
}

/// Output mapping override for auto-emitted assets.
///
/// 自动生成资源时的输出映射覆盖配置。
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
/// 由类型化 canonical RON 资源实现的 trait。
pub trait CanonicalRonAsset: Serialize {
    /// The canonical RON kind emitted by this typed value.
    ///
    /// 当前类型化值生成出的 canonical RON kind。
    fn ron_file_kind() -> RonFileKind;
}

impl Registry {
    /// Create a new registry.
    ///
    /// 创建一个新的注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// Set generated RON float output behavior.
    ///
    /// 设置生成 RON 时的浮点数输出行为。
    pub fn with_float_output(mut self, config: FloatOutputConfig) -> Self {
        self.float_output = config;
        self
    }

    /// Set generated RON float output behavior on an existing registry.
    ///
    /// 在已有注册表上设置生成 RON 时的浮点数输出行为。
    pub fn set_float_output(&mut self, config: FloatOutputConfig) {
        self.float_output = config;
    }

    /// Set generated RON float output behavior for one output path.
    ///
    /// 为单个输出路径设置生成 RON 时的浮点数输出行为。
    pub fn set_float_output_for_path(
        &mut self,
        path: impl Into<String>,
        config: FloatOutputConfig,
    ) {
        self.path_float_outputs.insert(path.into(), config);
    }

    /// Emit a serializable value as pretty-printed RON.
    ///
    /// 将一个可序列化值生成为格式化 RON。
    ///
    /// # Arguments
    /// - `path`: Destination path within the asset directory.
    /// - `value`: Serializable data to emit.
    pub fn emit_ron<T: Serialize>(&mut self, path: impl Into<String>, value: &T) -> Result<()> {
        let path = path.into();
        let ron_str = format_pretty_ron(value, self.float_output_for_path(&path))?;
        self.files.push(wit::GeneratedFile {
            path,
            ron_text: format!("{ron_str}\n"),
        });
        Ok(())
    }

    /// Emit an asset to its default canonical output path inferred from the source file.
    ///
    /// 将资源生成到由源码文件自动推导出的默认 canonical 输出路径。
    ///
    /// # Arguments
    /// - `source_file`: The `file!()` path of the caller.
    /// - `value`: The typed asset to emit.
    pub fn emit_auto<T: CanonicalRonAsset>(&mut self, source_file: &str, value: &T) -> Result<()> {
        self.emit_auto_with(source_file, value, EmitPathConfig::default())
    }

    /// Emit an asset using convention defaults, with optional explicit path override.
    ///
    /// 使用约定默认值生成资源，并允许显式路径覆盖。
    pub fn emit_auto_with<T: CanonicalRonAsset>(
        &mut self,
        source_file: &str,
        value: &T,
        config: EmitPathConfig,
    ) -> Result<()> {
        let output_path = infer_output_path::<T>(source_file, &config)?;
        self.emit_ron(output_path, value)
    }

    /// Finalize and return all collected files.
    ///
    /// 完成并返回所有收集到的文件。
    pub fn into_generated_files(self) -> Vec<wit::GeneratedFile> {
        self.files
    }

    fn float_output_for_path(&self, path: &str) -> FloatOutputConfig {
        self.path_float_outputs
            .get(path)
            .copied()
            .unwrap_or(self.float_output)
    }
}

/// Infer the canonical output path for a typed asset from its authoring source file.
///
/// 根据 authoring 源文件为类型化资源推导 canonical 输出路径。
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

fn format_pretty_ron<T: Serialize>(value: &T, float_output: FloatOutputConfig) -> Result<String> {
    let config = ron::ser::PrettyConfig::default().enumerate_arrays(false);
    let ron = ron::ser::to_string_pretty(value, config)?;
    Ok(normalize_ron_floats(&ron, float_output))
}

#[cfg(test)]
mod tests {
    use super::{EmitPathConfig, FloatOutputConfig, Registry, infer_output_path};
    use serde::Serialize;

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

    #[derive(Serialize)]
    struct NoisyFloatAsset {
        fire_time: f32,
    }

    #[test]
    fn registry_defaults_to_exact_float_output() {
        let mut registry = Registry::new();

        registry
            .emit_ron(
                "battle/danmaku/noisy.performance.ron",
                &NoisyFloatAsset {
                    fire_time: 3.8500001,
                },
            )
            .expect("asset should emit");

        let files = registry.into_generated_files();
        assert_eq!(files.len(), 1);
        assert!(files[0].ron_text.contains("fire_time: 3.8500001"));
    }

    #[test]
    fn registry_can_preserve_exact_float_output() {
        let mut registry = Registry::new().with_float_output(FloatOutputConfig::exact());

        registry
            .emit_ron(
                "battle/danmaku/noisy.performance.ron",
                &NoisyFloatAsset {
                    fire_time: 3.8500001,
                },
            )
            .expect("asset should emit");

        let files = registry.into_generated_files();
        assert_eq!(files.len(), 1);
        assert!(files[0].ron_text.contains("fire_time: 3.8500001"));
    }

    #[test]
    fn registry_can_use_fixed_fractional_digit_float_output() {
        let mut registry = Registry::new();
        registry.set_float_output(FloatOutputConfig::fixed_fractional_digits(2));

        registry
            .emit_ron(
                "battle/danmaku/noisy.performance.ron",
                &NoisyFloatAsset {
                    fire_time: 3.8500001,
                },
            )
            .expect("asset should emit");

        let files = registry.into_generated_files();
        assert_eq!(files.len(), 1);
        assert!(files[0].ron_text.contains("fire_time: 3.85"));
        assert!(!files[0].ron_text.contains("3.8500001"));
    }

    #[test]
    fn registry_can_scope_float_output_to_one_path() {
        let mut registry = Registry::new();
        registry.set_float_output_for_path(
            "battle/danmaku/noisy.performance.ron",
            FloatOutputConfig::fixed_fractional_digits(2),
        );

        registry
            .emit_ron(
                "battle/danmaku/noisy.performance.ron",
                &NoisyFloatAsset {
                    fire_time: 3.8500001,
                },
            )
            .expect("scoped asset should emit");
        registry
            .emit_ron(
                "battle/danmaku/exact.performance.ron",
                &NoisyFloatAsset {
                    fire_time: 3.8500001,
                },
            )
            .expect("exact asset should emit");

        let files = registry.into_generated_files();
        assert_eq!(files.len(), 2);
        assert!(files[0].ron_text.contains("fire_time: 3.85"));
        assert!(files[1].ron_text.contains("fire_time: 3.8500001"));
    }
}

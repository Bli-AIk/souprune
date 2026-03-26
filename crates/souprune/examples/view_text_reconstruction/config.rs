use crate::search::{
    CandidateSearchPlan, ConcreteTextParameters, OptionalTextFieldDefaults, parse_text_align,
    parse_text_anchor, parse_view_font,
};
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TaskConfig {
    pub workspace_root: PathBuf,
    pub image_path: PathBuf,
    pub text: String,
    pub bbox: Option<CropRect>,
    pub settle_frames: u32,
    pub target_similarity: f32,
    pub current_view_relative_path: String,
    pub current_view_absolute_path: PathBuf,
    pub best_view_absolute_path: PathBuf,
    pub current_summary_path: PathBuf,
    pub best_summary_path: PathBuf,
    pub current_diff_path: PathBuf,
    pub best_diff_path: PathBuf,
    pub property_defaults: OptionalTextFieldDefaults,
    pub search_plan: CandidateSearchPlan,
}

impl TaskConfig {
    pub fn load(config_path: &Path, workspace_root: &Path) -> Result<Self> {
        let raw = fs::read_to_string(config_path)
            .with_context(|| format!("failed to read task config: {}", config_path.display()))?;
        let parsed: TaskConfigFile = toml::from_str(&raw)
            .with_context(|| format!("failed to parse TOML: {}", config_path.display()))?;
        let config_dir = config_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        let image_path = resolve_path(&config_dir, &parsed.image);
        if !image_path.exists() {
            bail!("reference image does not exist: {}", image_path.display());
        }

        let bbox = parsed
            .bbox
            .map(|bbox| CropRect {
                x: bbox[0],
                y: bbox[1],
                width: bbox[2],
                height: bbox[3],
            })
            .filter(|bbox| bbox.width > 0 && bbox.height > 0);

        let current_view_relative_path = parsed.generated_view_path;
        if Path::new(&current_view_relative_path).is_absolute() {
            bail!("`generated_view_path` must be relative to the workspace root");
        }
        let current_view_absolute_path = workspace_root.join(&current_view_relative_path);
        let generated_dir = current_view_absolute_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| workspace_root.to_path_buf());
        let current_view_file_name = current_view_absolute_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("current.view.ron");
        let best_view_file_name = current_view_file_name.replace("current", "best");
        let best_view_absolute_path = generated_dir.join(best_view_file_name);

        let current_summary_path = generated_dir.join("current.json");
        let best_summary_path = generated_dir.join("best.json");
        let current_diff_path = generated_dir.join("current.diff.png");
        let best_diff_path = generated_dir.join("best.diff.png");

        let property_defaults = OptionalTextFieldDefaults {
            align_uses_default: matches!(parsed.properties.align.mode, PropertyMode::Default),
            anchor_uses_default: matches!(parsed.properties.anchor.mode, PropertyMode::Default),
            line_height_uses_default: matches!(
                parsed.properties.line_height.mode,
                PropertyMode::Default
            ),
            char_spacing_uses_default: matches!(
                parsed.properties.char_spacing.mode,
                PropertyMode::Default
            ),
            word_spacing_uses_default: matches!(
                parsed.properties.word_spacing.mode,
                PropertyMode::Default
            ),
        };

        let search_plan = CandidateSearchPlan::build(
            parsed.assume_single_line,
            &parsed.properties,
            ConcreteTextParameters::default(),
        )?;

        Ok(Self {
            workspace_root: workspace_root.to_path_buf(),
            image_path,
            text: parsed.text,
            bbox,
            settle_frames: parsed.settle_frames.unwrap_or(3),
            target_similarity: validate_target_similarity(parsed.target_similarity)?,
            current_view_relative_path,
            current_view_absolute_path,
            best_view_absolute_path,
            current_summary_path,
            best_summary_path,
            current_diff_path,
            best_diff_path,
            property_defaults,
            search_plan,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CropRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
struct TaskConfigFile {
    image: PathBuf,
    text: String,
    #[serde(default)]
    bbox: Option<[u32; 4]>,
    #[serde(default)]
    assume_single_line: bool,
    #[serde(default)]
    settle_frames: Option<u32>,
    #[serde(default = "default_target_similarity")]
    target_similarity: f32,
    #[serde(default = "default_generated_view_path")]
    generated_view_path: String,
    #[serde(default)]
    properties: PropertyTable,
}

fn default_generated_view_path() -> String {
    "generated/view_text_reconstruction/current.view.ron".to_string()
}

fn default_target_similarity() -> f32 {
    0.98
}

fn validate_target_similarity(value: f32) -> Result<f32> {
    if (0.0..=1.0).contains(&value) {
        Ok(value)
    } else {
        bail!("`target_similarity` must be in [0.0, 1.0], got {value}");
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PropertyTable {
    #[serde(default)]
    pub font: EnumPropertyConfig,
    #[serde(default)]
    pub align: EnumPropertyConfig,
    #[serde(default)]
    pub anchor: EnumPropertyConfig,
    #[serde(default)]
    pub translation_x: NumericPropertyConfig,
    #[serde(default)]
    pub translation_y: NumericPropertyConfig,
    #[serde(default)]
    pub world_scale_x: NumericPropertyConfig,
    #[serde(default)]
    pub world_scale_y: NumericPropertyConfig,
    #[serde(default)]
    pub line_height: NumericPropertyConfig,
    #[serde(default)]
    pub char_spacing: NumericPropertyConfig,
    #[serde(default)]
    pub word_spacing: NumericPropertyConfig,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PropertyMode {
    #[default]
    Default,
    Fixed,
    Search,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct EnumPropertyConfig {
    #[serde(default)]
    pub mode: PropertyMode,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub candidates: Option<Vec<String>>,
}

impl EnumPropertyConfig {
    pub fn resolve_font_values(
        &self,
        default_value: souprune_schema::view::ViewFontDef,
    ) -> Result<Vec<souprune_schema::view::ViewFontDef>> {
        resolve_enum_values(
            self,
            default_value,
            parse_view_font,
            &["DeterminationMono", "DeterminationSans", "Hud", "BattleHud"],
        )
    }

    pub fn resolve_align_values(
        &self,
        default_value: souprune_schema::view::TextAlignDef,
    ) -> Result<Vec<souprune_schema::view::TextAlignDef>> {
        resolve_enum_values(
            self,
            default_value,
            parse_text_align,
            &["Left", "Center", "Right"],
        )
    }

    pub fn resolve_anchor_values(
        &self,
        default_value: souprune_schema::view::TextAnchorDef,
    ) -> Result<Vec<souprune_schema::view::TextAnchorDef>> {
        resolve_enum_values(
            self,
            default_value,
            parse_text_anchor,
            &[
                "TopLeft",
                "TopCenter",
                "TopRight",
                "CenterLeft",
                "Center",
                "CenterRight",
                "BottomLeft",
                "BottomCenter",
                "BottomRight",
            ],
        )
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct NumericPropertyConfig {
    #[serde(default)]
    pub mode: PropertyMode,
    #[serde(default)]
    pub value: Option<f32>,
    #[serde(default)]
    pub candidates: Option<Vec<f32>>,
    #[serde(default)]
    pub range: Option<[f32; 2]>,
    #[serde(default)]
    pub step: Option<f32>,
}

impl NumericPropertyConfig {
    pub fn resolve_values(&self, label: &str, default_value: f32) -> Result<Vec<f32>> {
        match self.mode {
            PropertyMode::Default => Ok(vec![default_value]),
            PropertyMode::Fixed => Ok(vec![self.value.with_context(|| {
                format!("`properties.{label}.value` is required in fixed mode")
            })?]),
            PropertyMode::Search => {
                if let Some(candidates) = &self.candidates {
                    if candidates.is_empty() {
                        bail!("`properties.{label}.candidates` must not be empty");
                    }
                    return Ok(candidates.clone());
                }

                let [start, end] = self.range.with_context(|| {
                    format!("`properties.{label}.range` is required in search mode")
                })?;
                let step = self.step.with_context(|| {
                    format!("`properties.{label}.step` is required in search mode")
                })?;
                if step <= 0.0 {
                    bail!("`properties.{label}.step` must be positive");
                }
                if end < start {
                    bail!("`properties.{label}.range` must be ascending");
                }

                let mut values = Vec::new();
                let mut current = start;
                while current <= end + step * 0.25 {
                    values.push(round_numeric_candidate(current));
                    current += step;
                }
                Ok(values)
            }
        }
    }
}

fn round_numeric_candidate(value: f32) -> f32 {
    (value * 1000.0).round() / 1000.0
}

fn resolve_enum_values<T>(
    config: &EnumPropertyConfig,
    default_value: T,
    parse: fn(&str) -> Result<T>,
    default_candidates: &[&str],
) -> Result<Vec<T>>
where
    T: Clone,
{
    match config.mode {
        PropertyMode::Default => Ok(vec![default_value]),
        PropertyMode::Fixed => {
            Ok(vec![parse(config.value.as_deref().context(
                "`value` is required for fixed enum properties",
            )?)?])
        }
        PropertyMode::Search => {
            let candidates = config
                .candidates
                .as_ref()
                .map(|values| {
                    values
                        .iter()
                        .map(|value| value.as_str())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| default_candidates.to_vec());
            candidates.into_iter().map(parse).collect()
        }
    }
}

fn resolve_path(base_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}

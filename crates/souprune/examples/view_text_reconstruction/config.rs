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
    pub capture_reference_absolute_path: Option<PathBuf>,
    pub text: String,
    pub bbox: Option<CropRect>,
    pub settle_frames: u32,
    pub target_similarity: f32,
    pub exit_on_completion: bool,
    pub current_view_relative_path: String,
    pub current_view_absolute_path: PathBuf,
    pub best_view_absolute_path: PathBuf,
    pub current_summary_path: PathBuf,
    pub best_summary_path: PathBuf,
    pub current_render_path: PathBuf,
    pub best_render_path: PathBuf,
    pub current_diff_path: PathBuf,
    pub best_diff_path: PathBuf,
    pub world_scale_bound: bool,
    pub manual_steps: ManualAdjustmentSteps,
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
        let capture_reference_absolute_path = parsed
            .capture_reference_path
            .as_ref()
            .map(|path| resolve_path(&config_dir, path));

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
        let current_render_path = generated_dir.join("current.render.png");
        let best_render_path = generated_dir.join("best.render.png");
        let current_diff_path = generated_dir.join("current.diff.png");
        let best_diff_path = generated_dir.join("best.diff.png");
        let target_similarity = validate_target_similarity(parsed.target_similarity)?;

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
        let world_scale_bound = parsed.bindings.world_scale.is_bound();
        let manual_steps = ManualAdjustmentSteps {
            translation_x: parsed.properties.translation_x.preferred_step(1.0),
            translation_y: parsed.properties.translation_y.preferred_step(1.0),
            world_scale: parsed.properties.world_scale_x.preferred_step(0.25),
            line_height: parsed.properties.line_height.preferred_step(0.05),
            char_spacing: parsed.properties.char_spacing.preferred_step(0.25),
            word_spacing: parsed.properties.word_spacing.preferred_step(0.25),
        };
        let search_defaults = parsed
            .properties
            .resolve_search_defaults(&parsed.bindings, ConcreteTextParameters::default())?;

        let search_plan = CandidateSearchPlan::build(
            parsed.assume_single_line,
            &parsed.properties,
            &parsed.bindings,
            search_defaults,
            target_similarity,
            parsed.search_budget,
            parsed.population_size,
        )?;

        Ok(Self {
            workspace_root: workspace_root.to_path_buf(),
            image_path,
            capture_reference_absolute_path,
            text: parsed.text,
            bbox,
            settle_frames: parsed.settle_frames.unwrap_or(3),
            target_similarity,
            exit_on_completion: parsed.exit_on_completion,
            current_view_relative_path,
            current_view_absolute_path,
            best_view_absolute_path,
            current_summary_path,
            best_summary_path,
            current_render_path,
            best_render_path,
            current_diff_path,
            best_diff_path,
            world_scale_bound,
            manual_steps,
            property_defaults,
            search_plan,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ManualAdjustmentSteps {
    pub translation_x: f32,
    pub translation_y: f32,
    pub world_scale: f32,
    pub line_height: f32,
    pub char_spacing: f32,
    pub word_spacing: f32,
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
    #[serde(default)]
    capture_reference_path: Option<PathBuf>,
    text: String,
    #[serde(default)]
    bbox: Option<[u32; 4]>,
    #[serde(default)]
    assume_single_line: bool,
    #[serde(default)]
    settle_frames: Option<u32>,
    #[serde(default)]
    search_budget: Option<usize>,
    #[serde(default)]
    population_size: Option<usize>,
    #[serde(default = "default_target_similarity")]
    target_similarity: f32,
    #[serde(default)]
    exit_on_completion: bool,
    #[serde(default = "default_generated_view_path")]
    generated_view_path: String,
    #[serde(default)]
    properties: PropertyTable,
    #[serde(default)]
    bindings: BindingTable,
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

impl PropertyTable {
    fn resolve_search_defaults(
        &self,
        bindings: &BindingTable,
        base_defaults: ConcreteTextParameters,
    ) -> Result<ConcreteTextParameters> {
        let mut defaults = ConcreteTextParameters {
            font: self.font.resolve_font_seed(base_defaults.font)?,
            align: self.align.resolve_align_seed(base_defaults.align)?,
            anchor: self.anchor.resolve_anchor_seed(base_defaults.anchor)?,
            translation_x: self
                .translation_x
                .resolve_seed_value("translation_x", base_defaults.translation_x)?,
            translation_y: self
                .translation_y
                .resolve_seed_value("translation_y", base_defaults.translation_y)?,
            world_scale_x: self
                .world_scale_x
                .resolve_seed_value("world_scale_x", base_defaults.world_scale_x)?,
            world_scale_y: self
                .world_scale_y
                .resolve_seed_value("world_scale_y", base_defaults.world_scale_y)?,
            line_height: self
                .line_height
                .resolve_seed_value("line_height", base_defaults.line_height)?,
            char_spacing: self
                .char_spacing
                .resolve_seed_value("char_spacing", base_defaults.char_spacing)?,
            word_spacing: self
                .word_spacing
                .resolve_seed_value("word_spacing", base_defaults.word_spacing)?,
        };

        if bindings.world_scale.is_bound() {
            defaults.world_scale_y = defaults.world_scale_x;
        }

        Ok(defaults)
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BindingTable {
    #[serde(default)]
    pub world_scale: AxisBindingMode,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AxisBindingMode {
    #[default]
    Independent,
    #[serde(alias = "linked")]
    Bound,
}

impl AxisBindingMode {
    pub fn is_bound(self) -> bool {
        matches!(self, Self::Bound)
    }
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

    fn resolve_font_seed(
        &self,
        default_value: souprune_schema::view::ViewFontDef,
    ) -> Result<souprune_schema::view::ViewFontDef> {
        resolve_enum_seed(self, default_value, parse_view_font)
    }

    fn resolve_align_seed(
        &self,
        default_value: souprune_schema::view::TextAlignDef,
    ) -> Result<souprune_schema::view::TextAlignDef> {
        resolve_enum_seed(self, default_value, parse_text_align)
    }

    fn resolve_anchor_seed(
        &self,
        default_value: souprune_schema::view::TextAnchorDef,
    ) -> Result<souprune_schema::view::TextAnchorDef> {
        resolve_enum_seed(self, default_value, parse_text_anchor)
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

    fn resolve_seed_value(&self, label: &str, default_value: f32) -> Result<f32> {
        match self.mode {
            PropertyMode::Default => Ok(default_value),
            PropertyMode::Fixed => self
                .value
                .with_context(|| format!("`properties.{label}.value` is required in fixed mode")),
            PropertyMode::Search => {
                if let Some(candidates) = &self.candidates {
                    if candidates.is_empty() {
                        bail!("`properties.{label}.candidates` must not be empty");
                    }
                    return Ok(candidates[candidates.len() / 2]);
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

                let midpoint = (start + end) * 0.5;
                let step_index = ((midpoint - start) / step).round();
                let aligned = start + step * step_index;
                Ok(round_numeric_candidate(aligned.clamp(start, end)))
            }
        }
    }

    fn preferred_step(&self, default_step: f32) -> f32 {
        self.step.filter(|step| *step > 0.0).unwrap_or(default_step)
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

fn resolve_enum_seed<T>(
    config: &EnumPropertyConfig,
    default_value: T,
    parse: fn(&str) -> Result<T>,
) -> Result<T>
where
    T: Clone,
{
    match config.mode {
        PropertyMode::Default => Ok(default_value),
        PropertyMode::Fixed => parse(
            config
                .value
                .as_deref()
                .context("`value` is required for fixed enum properties")?,
        ),
        PropertyMode::Search => {
            let Some(candidates) = &config.candidates else {
                return Ok(default_value);
            };
            if candidates.is_empty() {
                bail!("enum search candidates must not be empty");
            }

            let parsed_candidates = candidates
                .iter()
                .map(|value| parse(value))
                .collect::<Result<Vec<_>>>()?;
            Ok(parsed_candidates
                .iter()
                .find(|value| same_enum_variant(*value, &default_value))
                .cloned()
                .unwrap_or_else(|| parsed_candidates[0].clone()))
        }
    }
}

fn same_enum_variant<T>(left: &T, right: &T) -> bool {
    std::mem::discriminant(left) == std::mem::discriminant(right)
}

fn resolve_path(base_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}

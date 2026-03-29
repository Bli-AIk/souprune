#![expect(clippy::excessive_nesting)]

use crate::search::{
    find_target_text_def, parse_text_align, parse_text_anchor, parse_view_font,
    text_parameters_from_text_def, CandidateSearchPlan, ConcreteTextParameters,
    OptionalTextFieldDefaults, TextFieldOverridePolicy,
};
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use souprune_schema::view::ViewLayoutAsset as SchemaViewLayoutAsset;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum LoadedConfig {
    Single(TaskConfig),
    Session(SessionConfig),
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub cases: Vec<SessionCaseConfig>,
    pub initial_task: TaskConfig,
    pub final_view_absolute_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SessionCaseConfig {
    pub id: String,
    pub stage_one_path: PathBuf,
    pub stage_two_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum StageKind {
    Single,
    AlignFirstGlyph,
    RefineSpacing,
}

impl StageKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::AlignFirstGlyph => "align_first_glyph",
            Self::RefineSpacing => "refine_spacing",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskConfig {
    pub workspace_root: PathBuf,
    pub stage_kind: StageKind,
    pub image_path: PathBuf,
    pub capture_reference_absolute_path: Option<PathBuf>,
    pub text: String,
    pub bbox: Option<CropRect>,
    pub settle_frames: u32,
    pub target_similarity: f32,
    pub exit_on_completion: bool,
    pub current_view_absolute_path: PathBuf,
    pub runtime_view_relative_path: String,
    pub runtime_view_absolute_path: PathBuf,
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
    pub field_override_policy: TextFieldOverridePolicy,
    pub host_view: Option<HostViewTemplate>,
    pub search_plan: CandidateSearchPlan,
}

pub fn load_config(config_path: &Path, workspace_root: &Path) -> Result<LoadedConfig> {
    let extension = config_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match extension.as_str() {
        "toml" => Ok(LoadedConfig::Single(TaskConfig::load_legacy_toml(
            config_path,
            workspace_root,
        )?)),
        "ron" => {
            let raw = fs::read_to_string(config_path)
                .with_context(|| format!("failed to read config: {}", config_path.display()))?;
            if let Ok(session_file) = ron::from_str::<SessionConfigFile>(&raw) {
                return Ok(LoadedConfig::Session(load_session_config(
                    config_path,
                    workspace_root,
                    session_file,
                )?));
            }

            Ok(LoadedConfig::Single(TaskConfig::load_stage_ron(
                config_path,
                workspace_root,
                None,
            )?))
        }
        _ => bail!(
            "unsupported config extension for `{}`; expected .toml or .ron",
            config_path.display()
        ),
    }
}

impl TaskConfig {
    pub fn load_legacy_toml(config_path: &Path, workspace_root: &Path) -> Result<Self> {
        let raw = fs::read_to_string(config_path)
            .with_context(|| format!("failed to read task config: {}", config_path.display()))?;
        let parsed: TaskConfigFile = toml::from_str(&raw)
            .with_context(|| format!("failed to parse TOML: {}", config_path.display()))?;
        Self::from_source(
            config_path,
            workspace_root,
            TaskConfigSource {
                stage_kind: StageKind::Single,
                image: parsed.image,
                capture_reference_path: parsed.capture_reference_path,
                host_view: parsed.host_view,
                text: parsed.text,
                bbox: parsed.bbox,
                assume_single_line: parsed.assume_single_line,
                settle_frames: parsed.settle_frames,
                search_budget: parsed.search_budget,
                population_size: parsed.population_size,
                target_similarity: parsed.target_similarity,
                exit_on_completion: parsed.exit_on_completion,
                generated_view_path: Some(parsed.generated_view_path),
                properties: parsed.properties,
                bindings: parsed.bindings,
            },
            None,
        )
    }

    pub fn load_stage_ron(
        config_path: &Path,
        workspace_root: &Path,
        inherited_seed: Option<&ConcreteTextParameters>,
    ) -> Result<Self> {
        let raw = fs::read_to_string(config_path)
            .with_context(|| format!("failed to read stage config: {}", config_path.display()))?;
        let parsed: StageConfigFile = ron::from_str(&raw)
            .with_context(|| format!("failed to parse RON: {}", config_path.display()))?;
        let stage_text = resolve_stage_text(parsed.stage_kind, &parsed.text, parsed.first_glyph)?;
        let properties = parsed
            .properties
            .into_runtime_table(inherited_seed)
            .context("failed to resolve stage properties")?;
        Self::from_source(
            config_path,
            workspace_root,
            TaskConfigSource {
                stage_kind: parsed.stage_kind,
                image: parsed.image,
                capture_reference_path: parsed.capture_reference_path,
                host_view: parsed.host_view,
                text: stage_text,
                bbox: parsed.bbox,
                assume_single_line: parsed.assume_single_line,
                settle_frames: parsed.settle_frames,
                search_budget: parsed.search_budget,
                population_size: parsed.population_size,
                target_similarity: parsed.target_similarity,
                exit_on_completion: parsed.exit_on_completion,
                generated_view_path: parsed.generated_view_path,
                properties,
                bindings: parsed.bindings,
            },
            inherited_seed,
        )
    }

    fn from_source(
        config_path: &Path,
        workspace_root: &Path,
        parsed: TaskConfigSource,
        inherited_seed: Option<&ConcreteTextParameters>,
    ) -> Result<Self> {
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
        let host_view = parsed
            .host_view
            .as_ref()
            .map(|host_view| load_host_view_template(host_view, workspace_root, &config_dir))
            .transpose()?;

        let bbox = parsed
            .bbox
            .map(|bbox| CropRect {
                x: bbox[0],
                y: bbox[1],
                width: bbox[2],
                height: bbox[3],
            })
            .filter(|bbox| bbox.width > 0 && bbox.height > 0);

        let current_view_relative_path =
            resolve_generated_view_path(config_path, workspace_root, parsed.generated_view_path)?;
        if Path::new(&current_view_relative_path).is_absolute() {
            bail!("`generated_view_path` must be relative to the workspace root");
        }
        let current_view_absolute_path = workspace_root.join(&current_view_relative_path);
        let current_output_dir = current_view_absolute_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| workspace_root.to_path_buf());
        let current_view_file_name = current_view_absolute_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("view.ron");
        let runtime_view_relative_path = current_output_dir
            .strip_prefix(workspace_root)
            .ok()
            .map(|relative_dir| relative_dir.join("runtime.view.ron"))
            .unwrap_or_else(|| PathBuf::from("generated/view_text_reconstruction/runtime.view.ron"))
            .to_string_lossy()
            .replace('\\', "/");
        let runtime_view_absolute_path = current_output_dir.join("runtime.view.ron");
        let best_output_dir = current_output_dir
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| *name == "current")
            .and_then(|_| current_output_dir.parent().map(|path| path.join("best")))
            .unwrap_or_else(|| current_output_dir.join("best"));
        let best_view_absolute_path = best_output_dir.join(current_view_file_name);

        let current_summary_path = current_output_dir.join("summary.json");
        let best_summary_path = best_output_dir.join("summary.json");
        let current_render_path = current_output_dir.join("render.png");
        let best_render_path = best_output_dir.join("render.png");
        let current_diff_path = current_output_dir.join("diff.png");
        let best_diff_path = best_output_dir.join("diff.png");
        let target_similarity = validate_target_similarity(parsed.target_similarity)?;
        let base_defaults = if let Some(host_view) = &host_view {
            text_parameters_from_text_def(find_target_text_def(&host_view.layout, Some(host_view))?)
        } else {
            ConcreteTextParameters::default()
        };
        let base_defaults = inherited_seed.cloned().unwrap_or(base_defaults);

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
        let field_override_policy = parsed.properties.text_field_override_policy();
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
            .resolve_search_defaults(&parsed.bindings, base_defaults)?;

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
            stage_kind: parsed.stage_kind,
            image_path,
            capture_reference_absolute_path,
            text: parsed.text,
            bbox,
            settle_frames: parsed.settle_frames.unwrap_or(3),
            target_similarity,
            exit_on_completion: parsed.exit_on_completion,
            current_view_absolute_path,
            runtime_view_relative_path,
            runtime_view_absolute_path,
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
            field_override_policy,
            host_view,
            search_plan,
        })
    }
}

#[derive(Debug)]
struct TaskConfigSource {
    stage_kind: StageKind,
    image: PathBuf,
    capture_reference_path: Option<PathBuf>,
    host_view: Option<HostViewConfigFile>,
    text: String,
    bbox: Option<[u32; 4]>,
    assume_single_line: bool,
    settle_frames: Option<u32>,
    search_budget: Option<usize>,
    population_size: Option<usize>,
    target_similarity: f32,
    exit_on_completion: bool,
    generated_view_path: Option<String>,
    properties: PropertyTable,
    bindings: BindingTable,
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
    #[serde(default)]
    host_view: Option<HostViewConfigFile>,
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

#[derive(Debug, Deserialize)]
struct SessionConfigFile {
    #[serde(default)]
    final_view_path: Option<PathBuf>,
    cases: Vec<SessionCaseFile>,
}

#[derive(Debug, Deserialize)]
struct SessionCaseFile {
    id: Option<String>,
    stage_one: PathBuf,
    stage_two: PathBuf,
}

#[derive(Debug, Deserialize)]
struct StageConfigFile {
    stage_kind: StageKind,
    image: PathBuf,
    #[serde(default)]
    capture_reference_path: Option<PathBuf>,
    #[serde(default)]
    host_view: Option<HostViewConfigFile>,
    text: String,
    #[serde(default)]
    first_glyph: Option<String>,
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
    #[serde(default)]
    generated_view_path: Option<String>,
    #[serde(default)]
    properties: StagePropertyTable,
    #[serde(default)]
    bindings: BindingTable,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct StagePropertyTable {
    #[serde(default)]
    font: StageEnumPropertyConfig,
    #[serde(default)]
    align: StageEnumPropertyConfig,
    #[serde(default)]
    anchor: StageEnumPropertyConfig,
    #[serde(default)]
    translation_x: StageNumericPropertyConfig,
    #[serde(default)]
    translation_y: StageNumericPropertyConfig,
    #[serde(default)]
    world_scale_x: StageNumericPropertyConfig,
    #[serde(default)]
    world_scale_y: StageNumericPropertyConfig,
    #[serde(default)]
    line_height: StageNumericPropertyConfig,
    #[serde(default)]
    char_spacing: StageNumericPropertyConfig,
    #[serde(default)]
    word_spacing: StageNumericPropertyConfig,
}

impl StagePropertyTable {
    fn into_runtime_table(
        self,
        inherited_seed: Option<&ConcreteTextParameters>,
    ) -> Result<PropertyTable> {
        Ok(PropertyTable {
            font: self.font.into_runtime_enum(
                "font",
                inherited_seed.map(|seed| format!("{:?}", seed.font)),
            )?,
            align: self.align.into_runtime_enum(
                "align",
                inherited_seed.map(|seed| format!("{:?}", seed.align)),
            )?,
            anchor: self.anchor.into_runtime_enum(
                "anchor",
                inherited_seed.map(|seed| format!("{:?}", seed.anchor)),
            )?,
            translation_x: self.translation_x.into_runtime_numeric(
                "translation_x",
                inherited_seed.map(|seed| seed.translation_x),
            )?,
            translation_y: self.translation_y.into_runtime_numeric(
                "translation_y",
                inherited_seed.map(|seed| seed.translation_y),
            )?,
            world_scale_x: self.world_scale_x.into_runtime_numeric(
                "world_scale_x",
                inherited_seed.map(|seed| seed.world_scale_x),
            )?,
            world_scale_y: self.world_scale_y.into_runtime_numeric(
                "world_scale_y",
                inherited_seed.map(|seed| seed.world_scale_y),
            )?,
            line_height: self
                .line_height
                .into_runtime_numeric("line_height", inherited_seed.map(|seed| seed.line_height))?,
            char_spacing: self.char_spacing.into_runtime_numeric(
                "char_spacing",
                inherited_seed.map(|seed| seed.char_spacing),
            )?,
            word_spacing: self.word_spacing.into_runtime_numeric(
                "word_spacing",
                inherited_seed.map(|seed| seed.word_spacing),
            )?,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
enum StageEnumPropertyConfig {
    #[default]
    Default,
    Fixed(String),
    Search(Vec<String>),
    InheritFixed,
}

impl StageEnumPropertyConfig {
    fn into_runtime_enum(
        self,
        label: &str,
        inherited_value: Option<String>,
    ) -> Result<EnumPropertyConfig> {
        Ok(match self {
            Self::Default => EnumPropertyConfig::default(),
            Self::Fixed(value) => EnumPropertyConfig {
                mode: PropertyMode::Fixed,
                value: Some(value),
                candidates: None,
            },
            Self::Search(candidates) => EnumPropertyConfig {
                mode: PropertyMode::Search,
                value: None,
                candidates: Some(candidates),
            },
            Self::InheritFixed => EnumPropertyConfig {
                mode: PropertyMode::Fixed,
                value: Some(inherited_value.with_context(|| {
                    format!(
                        "`{label}` requested InheritFixed but no previous-stage seed was provided"
                    )
                })?),
                candidates: None,
            },
        })
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
enum StageNumericPropertyConfig {
    #[default]
    Default,
    Fixed(f32),
    SearchCandidates(Vec<f32>),
    SearchRange((f32, f32, f32)),
    InheritFixed,
}

impl StageNumericPropertyConfig {
    fn into_runtime_numeric(
        self,
        label: &str,
        inherited_value: Option<f32>,
    ) -> Result<NumericPropertyConfig> {
        Ok(match self {
            Self::Default => NumericPropertyConfig::default(),
            Self::Fixed(value) => NumericPropertyConfig {
                mode: PropertyMode::Fixed,
                value: Some(value),
                candidates: None,
                range: None,
                step: None,
            },
            Self::SearchCandidates(candidates) => NumericPropertyConfig {
                mode: PropertyMode::Search,
                value: None,
                candidates: Some(candidates),
                range: None,
                step: None,
            },
            Self::SearchRange((start, end, step)) => NumericPropertyConfig {
                mode: PropertyMode::Search,
                value: None,
                candidates: None,
                range: Some([start, end]),
                step: Some(step),
            },
            Self::InheritFixed => NumericPropertyConfig {
                mode: PropertyMode::Fixed,
                value: Some(inherited_value.with_context(|| {
                    format!(
                        "`{label}` requested InheritFixed but no previous-stage seed was provided"
                    )
                })?),
                candidates: None,
                range: None,
                step: None,
            },
        })
    }
}

#[derive(Debug, Clone)]
pub struct HostViewTemplate {
    pub source_path: PathBuf,
    pub layout: SchemaViewLayoutAsset,
    pub node_path: Vec<String>,
    pub text_id: String,
    pub show_parent_boxes: bool,
}

#[derive(Debug, Deserialize)]
struct HostViewConfigFile {
    path: PathBuf,
    node_path: Vec<String>,
    text_id: String,
    #[serde(default)]
    show_parent_boxes: bool,
}

fn default_generated_view_path() -> String {
    "generated/view_text_reconstruction/current.view.ron".to_string()
}

fn load_session_config(
    session_path: &Path,
    workspace_root: &Path,
    parsed: SessionConfigFile,
) -> Result<SessionConfig> {
    if parsed.cases.is_empty() {
        bail!("session must contain at least one text case");
    }

    let session_dir = session_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let mut cases = Vec::with_capacity(parsed.cases.len());
    for (index, case) in parsed.cases.into_iter().enumerate() {
        let case_id = case.id.unwrap_or_else(|| format!("case_{index:03}"));
        let stage_one_path =
            resolve_workspace_or_config_path(workspace_root, &session_dir, &case.stage_one);
        let stage_two_path =
            resolve_workspace_or_config_path(workspace_root, &session_dir, &case.stage_two);
        cases.push(SessionCaseConfig {
            id: case_id,
            stage_one_path,
            stage_two_path,
        });
    }

    let initial_task = TaskConfig::load_stage_ron(&cases[0].stage_one_path, workspace_root, None)?;
    let final_view_absolute_path = if let Some(final_view_path) = parsed.final_view_path {
        resolve_workspace_or_config_path(workspace_root, &session_dir, &final_view_path)
    } else if let Some(host_view) = initial_task.host_view.as_ref() {
        session_dir.join("final").join(
            host_view
                .source_path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("final.view.ron")),
        )
    } else {
        session_dir.join("final.view.ron")
    };
    Ok(SessionConfig {
        cases,
        initial_task,
        final_view_absolute_path,
    })
}

fn resolve_generated_view_path(
    config_path: &Path,
    workspace_root: &Path,
    configured_path: Option<String>,
) -> Result<String> {
    if let Some(configured_path) = configured_path {
        return Ok(configured_path);
    }

    let absolute_config_path = if config_path.is_absolute() {
        config_path.to_path_buf()
    } else {
        workspace_root.join(config_path)
    };

    let config_relative_dir = absolute_config_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .strip_prefix(workspace_root)
        .with_context(|| {
            format!(
                "config path `{}` must live inside the workspace root to derive output paths",
                config_path.display()
            )
        })?;
    let stage_dir = config_relative_dir.join(
        config_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("task"),
    );
    Ok(stage_dir
        .join("current/view.ron")
        .to_string_lossy()
        .replace('\\', "/"))
}

fn resolve_stage_text(
    stage_kind: StageKind,
    full_text: &str,
    first_glyph_override: Option<String>,
) -> Result<String> {
    match stage_kind {
        StageKind::AlignFirstGlyph => {
            if let Some(first_glyph_override) = first_glyph_override {
                if first_glyph_override.is_empty() {
                    bail!("`first_glyph` must not be empty");
                }
                Ok(first_glyph_override)
            } else {
                full_text
                    .chars()
                    .next()
                    .map(|character| character.to_string())
                    .with_context(|| {
                        "stage `AlignFirstGlyph` requires `text` to contain at least one character"
                    })
            }
        }
        StageKind::Single | StageKind::RefineSpacing => Ok(full_text.to_string()),
    }
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

    fn text_field_override_policy(&self) -> TextFieldOverridePolicy {
        TextFieldOverridePolicy {
            font: self.font.is_overridden(),
            align: self.align.is_overridden(),
            anchor: self.anchor.is_overridden(),
            translation_x: self.translation_x.is_overridden(),
            translation_y: self.translation_y.is_overridden(),
            world_scale_x: self.world_scale_x.is_overridden(),
            world_scale_y: self.world_scale_y.is_overridden(),
            line_height: self.line_height.is_overridden(),
            char_spacing: self.char_spacing.is_overridden(),
            word_spacing: self.word_spacing.is_overridden(),
        }
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

    fn is_overridden(&self) -> bool {
        !matches!(self.mode, PropertyMode::Default)
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

    fn is_overridden(&self) -> bool {
        !matches!(self.mode, PropertyMode::Default)
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

fn load_host_view_template(
    parsed: &HostViewConfigFile,
    workspace_root: &Path,
    config_dir: &Path,
) -> Result<HostViewTemplate> {
    if parsed.node_path.is_empty() {
        bail!("`host_view.node_path` must contain at least one node name");
    }

    let absolute_path = resolve_workspace_or_config_path(workspace_root, config_dir, &parsed.path);
    if !absolute_path.exists() {
        bail!("host view file does not exist: {}", absolute_path.display());
    }

    let raw = fs::read_to_string(&absolute_path)
        .with_context(|| format!("failed to read host view file: {}", absolute_path.display()))?;
    let layout: SchemaViewLayoutAsset = ron::from_str(&raw)
        .with_context(|| format!("failed to parse host view RON: {}", absolute_path.display()))?;
    let host_view = HostViewTemplate {
        source_path: absolute_path,
        layout,
        node_path: parsed.node_path.clone(),
        text_id: parsed.text_id.clone(),
        show_parent_boxes: parsed.show_parent_boxes,
    };
    let _ = find_target_text_def(&host_view.layout, Some(&host_view))?;
    Ok(host_view)
}

fn resolve_workspace_or_config_path(
    workspace_root: &Path,
    config_dir: &Path,
    path: &Path,
) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    let workspace_candidate = workspace_root.join(path);
    if workspace_candidate.exists() {
        workspace_candidate
    } else {
        config_dir.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn stage_align_first_glyph_uses_stage_directory_output_and_first_character() {
        let workspace_root = create_test_workspace("stage_align_first_glyph");
        let stage_dir = workspace_root.join("generated/view_text_reconstruction/demo_case");
        let config_path = stage_dir.join("stage_1_align_first_glyph.ron");
        fs::create_dir_all(&stage_dir).expect("stage dir should be created");
        write_test_reference_image(&stage_dir.join("reference.png"));
        fs::write(
            &config_path,
            r#"
(
    stage_kind: AlignFirstGlyph,
    image: "reference.png",
    text: "CHARA",
    assume_single_line: true,
    target_similarity: 0.95,
    properties: (
        char_spacing: Fixed(0.0),
        word_spacing: Fixed(0.0),
    ),
    bindings: (
        world_scale: bound,
    ),
)
"#,
        )
        .expect("stage config should be written");

        let task = TaskConfig::load_stage_ron(&config_path, &workspace_root, None)
            .expect("stage config should load");

        assert_eq!(task.stage_kind, StageKind::AlignFirstGlyph);
        assert_eq!(task.text, "C");
        assert_eq!(
            task.current_view_absolute_path,
            workspace_root.join(
                "generated/view_text_reconstruction/demo_case/stage_1_align_first_glyph/current/view.ron"
            )
        );
        assert_eq!(
            task.runtime_view_relative_path,
            "generated/view_text_reconstruction/demo_case/stage_1_align_first_glyph/current/runtime.view.ron"
        );
    }

    #[test]
    fn session_loads_cases_and_initial_stage_from_relative_paths() {
        let workspace_root = create_test_workspace("session_load");
        let session_dir = workspace_root.join("generated/view_text_reconstruction/demo_case");
        fs::create_dir_all(&session_dir).expect("session dir should be created");
        write_test_reference_image(&session_dir.join("reference.png"));
        fs::write(
            session_dir.join("stage_1_align_first_glyph.ron"),
            r#"
(
    stage_kind: AlignFirstGlyph,
    image: "reference.png",
    text: "CHARA",
    first_glyph: Some("C"),
    target_similarity: 0.95,
)
"#,
        )
        .expect("stage one config should be written");
        fs::write(
            session_dir.join("stage_2_refine_spacing.ron"),
            r#"
(
    stage_kind: RefineSpacing,
    image: "reference.png",
    text: "CHARA",
    target_similarity: 0.95,
    properties: (
        font: InheritFixed,
        translation_x: InheritFixed,
        translation_y: InheritFixed,
        world_scale_x: InheritFixed,
        world_scale_y: InheritFixed,
        line_height: InheritFixed,
        char_spacing: SearchRange((-2.0, 2.0, 0.5)),
        word_spacing: SearchRange((-2.0, 2.0, 0.5)),
    ),
    bindings: (
        world_scale: bound,
    ),
)
"#,
        )
        .expect("stage two config should be written");
        fs::write(
            session_dir.join("session.ron"),
            r#"
(
    cases: [
        (
            id: Some("demo_case"),
            stage_one: "stage_1_align_first_glyph.ron",
            stage_two: "stage_2_refine_spacing.ron",
        ),
    ],
)
"#,
        )
        .expect("session config should be written");

        let loaded = load_config(&session_dir.join("session.ron"), &workspace_root)
            .expect("session config should load");
        let LoadedConfig::Session(session) = loaded else {
            panic!("expected session config");
        };

        assert_eq!(session.cases.len(), 1);
        assert_eq!(session.cases[0].id, "demo_case");
        assert_eq!(session.initial_task.stage_kind, StageKind::AlignFirstGlyph);
        assert_eq!(session.initial_task.text, "C");
        assert_eq!(
            session.initial_task.current_view_absolute_path,
            workspace_root.join(
                "generated/view_text_reconstruction/demo_case/stage_1_align_first_glyph/current/view.ron"
            )
        );
    }

    #[test]
    fn stage_two_inherit_fixed_uses_previous_stage_seed() {
        let workspace_root = create_test_workspace("stage_two_inherit_fixed");
        let stage_dir = workspace_root.join("generated/view_text_reconstruction/demo_case");
        let config_path = stage_dir.join("stage_2_refine_spacing.ron");
        fs::create_dir_all(&stage_dir).expect("stage dir should be created");
        write_test_reference_image(&stage_dir.join("reference.png"));
        fs::write(
            &config_path,
            r#"
(
    stage_kind: RefineSpacing,
    image: "reference.png",
    text: "CHARA",
    target_similarity: 0.95,
    properties: (
        font: InheritFixed,
        align: InheritFixed,
        anchor: InheritFixed,
        translation_x: InheritFixed,
        translation_y: InheritFixed,
        world_scale_x: InheritFixed,
        world_scale_y: InheritFixed,
        line_height: InheritFixed,
        char_spacing: SearchRange((-2.0, 2.0, 0.5)),
        word_spacing: SearchRange((-2.0, 2.0, 0.5)),
    ),
    bindings: (
        world_scale: bound,
    ),
)
"#,
        )
        .expect("stage config should be written");

        let inherited_seed = ConcreteTextParameters {
            font: souprune_schema::view::ViewFontDef::DeterminationSans,
            align: souprune_schema::view::TextAlignDef::Center,
            anchor: souprune_schema::view::TextAnchorDef::BottomLeft,
            translation_x: -28.5,
            translation_y: 22.0,
            world_scale_x: 13.0,
            world_scale_y: 13.0,
            line_height: 1.125,
            char_spacing: 0.0,
            word_spacing: 0.0,
        };

        let task = TaskConfig::load_stage_ron(&config_path, &workspace_root, Some(&inherited_seed))
            .expect("stage two config should load with inherited seed");
        let seed = task.search_plan.seed_parameters();

        assert_eq!(task.stage_kind, StageKind::RefineSpacing);
        assert_eq!(task.text, "CHARA");
        assert!(same_enum_variant(&seed.font, &inherited_seed.font));
        assert!(same_enum_variant(&seed.align, &inherited_seed.align));
        assert!(same_enum_variant(&seed.anchor, &inherited_seed.anchor));
        assert_eq!(seed.translation_x, inherited_seed.translation_x);
        assert_eq!(seed.translation_y, inherited_seed.translation_y);
        assert_eq!(seed.world_scale_x, inherited_seed.world_scale_x);
        assert_eq!(seed.world_scale_y, inherited_seed.world_scale_y);
        assert_eq!(seed.line_height, inherited_seed.line_height);
    }

    fn create_test_workspace(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();
        let workspace_root = std::env::temp_dir().join(format!(
            "souprune_view_text_reconstruction_{label}_{unique}"
        ));
        fs::create_dir_all(&workspace_root).expect("workspace root should be created");
        workspace_root
    }

    fn write_test_reference_image(path: &Path) {
        fs::create_dir_all(path.parent().expect("image parent should exist"))
            .expect("image parent should be created");
        RgbaImage::new(2, 2)
            .save(path)
            .expect("reference image should be written");
    }
}

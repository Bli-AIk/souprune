//! Drives the interactive reconstruction runtime for the text reconstruction example.
//!
//! 负责文本重建示例的交互式运行时。
//!
//! This file owns window setup, real View spawning, screenshot evaluation, and the preview HUD.
//! It keeps reconstruction feedback inside the actual Bevy + SoupRune runtime instead of using a
//! separate mock renderer.
//!
//! 这个文件负责窗口初始化、真实 View 生成、截图评分，以及预览 HUD。
//! 它把重建反馈放在真实的 Bevy + SoupRune 运行时里，而不是维护一套假的渲染实现。
use crate::config::{
    CropRect, LoadedConfig, SessionCaseConfig, SessionConfig, StageKind, TaskConfig,
};
use crate::search::{
    CandidateSearchPlan, ConcreteTextParameters, RestartSearchResult, SearchParameterField,
    apply_export_text_patch, build_export_view_layout, build_runtime_view_layout,
    find_target_text_def, parse_text_align, parse_text_anchor, parse_view_font,
};
use anyhow::{Context, Result};
use bevy::app::AppExit;
use bevy::asset::RenderAssetUsages;
use bevy::asset::io::file::FileAssetReader;
use bevy::asset::io::{AssetSourceBuilder, AssetSourceId};
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::image::Image;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::{PrimaryWindow, WindowPlugin};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use souprune::ViewLayoutAsset;
use souprune::config::SoupruneConfig;
use souprune::core::camera::MainGameCamera;
use souprune::core::game_action::GameActionDef;
use souprune::core::view::{CoreViewPlugin, DespawnViewRequest, SpawnViewRequest};
use souprune::extra::multi_source::MultiSourceAssetReader;
use souprune_schema::Val;
use souprune_schema::view::ViewLayoutAsset as SchemaViewLayoutAsset;
use std::fs;
use std::path::{Path, PathBuf};

const PREVIEW_LAYER: RenderLayers = RenderLayers::layer(2);
const PREVIEW_TEXT_MARGIN: f32 = 18.0;
const MANUAL_STEP_MULTIPLIERS: [f32; 7] = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0];
const GUIDE_THICKNESS: f32 = 2.0;
const GUIDE_HIT_RADIUS: f32 = 10.0;

pub fn configure_app(
    app: &mut App,
    souprune_config: SoupruneConfig,
    loaded_config: LoadedConfig,
) -> Result<()> {
    let (task, session) = match loaded_config {
        LoadedConfig::Single(task) => (task, None),
        LoadedConfig::Session(session) => (session.initial_task.clone(), Some(session)),
    };
    let current_project_name = souprune_config.project.mod_name.clone();
    let workspace_root = task.workspace_root.clone();
    let reference_image = load_reference_image(&task.image_path)?;

    if let Some(bbox) = task.bbox {
        validate_bbox(&reference_image, bbox)?;
    }

    let search_plan = task.search_plan.clone();
    let restored_current = load_saved_resume_state(&task, &search_plan);
    let restored_best = load_saved_best_score(&task, &search_plan);
    let (initial_current, initial_text, skip_initial_snap) =
        if let Some(restored_current) = restored_current {
            (restored_current.parameters, restored_current.text, true)
        } else {
            (search_plan.seed_parameters(), task.text.clone(), false)
        };

    app.insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.11)))
        .insert_resource(souprune::app_state::app_setup::ResolutionScale(
            souprune_config.window.resolution_scale,
        ))
        .insert_resource(souprune::extra::mortar::CurrentLocale(
            souprune_config.project.language.clone(),
        ))
        .insert_resource(souprune_config.clone())
        .register_asset_source(
            AssetSourceId::Default,
            AssetSourceBuilder::new(move || {
                let roots = souprune::config::get_asset_roots(&current_project_name);
                let readers = roots
                    .into_iter()
                    .chain(std::iter::once(workspace_root.clone()))
                    .map(FileAssetReader::new)
                    .collect();
                Box::new(MultiSourceAssetReader::new(readers))
            }),
        )
        .add_plugins(
            DefaultPlugins
                .set(bevy::image::ImagePlugin::default_nearest())
                .set(bevy::asset::AssetPlugin {
                    unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(bevy::window::Window {
                        title: "View Text Reconstruction".into(),
                        resolution: (1280, 960).into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, enter_running_state)
        .add_plugins((
            souprune::get_file_importer_plugins(),
            souprune::get_third_plugins(),
            bevy_fact_rule_event::FREPlugin::<GameActionDef>::default(),
            souprune::core::CorePlugin,
            CoreViewPlugin,
            souprune::core::mod_system::ModPlugin,
        ))
        .insert_resource(TaskResource(task.clone()))
        .insert_resource(OptionalSessionController::from_loaded_session(session))
        .insert_resource(RenderTargetImage::default())
        .insert_resource(ReferenceImages {
            original: reference_image.clone(),
            compare_masked: apply_bbox_mask(&reference_image, task.bbox),
            width: reference_image.width(),
            height: reference_image.height(),
            reference_handle: Handle::default(),
            diff_handle: Handle::default(),
        })
        .insert_resource(ReferenceMaskState::new(
            reference_image.width(),
            reference_image.height(),
        ))
        .insert_resource(SearchController {
            plan: search_plan,
            total_candidates: task.search_plan.total_candidates(),
        })
        .insert_resource(ReconstructionState {
            phase: EvaluationPhase::Ready,
            display_mode: DisplayMode::Overlay,
            auto_search: false,
            total_candidates: task.search_plan.total_candidates(),
            target_similarity: task.target_similarity,
            current_candidate_index: None,
            current_parameters: initial_current,
            current_text: initial_text,
            current_score: None,
            best_score: restored_best,
            latest_render_image: None,
            latest_diff_image: None,
            persist_current_after_evaluation: false,
            capture_after_apply: true,
            manual_adjustment_kind: ManualAdjustmentKind::initial_for_task(&task),
            manual_step_multiplier_index: 0,
            snap_to_grid: true,
            text_edit_mode: false,
            show_detailed_status: false,
            skip_snap_on_next_apply: skip_initial_snap,
            pending_apply: true,
            awaiting_user_step: None,
        })
        .insert_resource(CurrentViewAssetHandle::default())
        .add_systems(Startup, setup_runtime)
        .add_systems(
            Update,
            (
                handle_keyboard_input,
                apply_pending_session_transition,
                handle_inspector_interactions,
                handle_reference_mask_interactions,
                apply_pending_candidate,
                drive_capture_state,
                update_preview_scene,
                update_inspector_panel,
            )
                .chain(),
        );
    souprune::init_game_state(app);
    souprune::insert_font_resources(app);
    souprune::insert_input_resources(app);

    Ok(())
}

#[derive(Resource, Clone)]
struct TaskResource(TaskConfig);

#[derive(Resource, Default)]
struct OptionalSessionController(Option<SessionController>);

impl OptionalSessionController {
    fn from_loaded_session(session: Option<SessionConfig>) -> Self {
        Self(session.map(SessionController::new))
    }
}

#[derive(Debug, Clone)]
struct SessionController {
    cases: Vec<SessionCaseConfig>,
    current_case_index: usize,
    current_stage: SessionStageSlot,
    pending_transition: Option<PendingSessionTransition>,
    final_view_absolute_path: PathBuf,
}

impl SessionController {
    fn new(session: SessionConfig) -> Self {
        Self {
            cases: session.cases,
            current_case_index: 0,
            current_stage: SessionStageSlot::StageOne,
            pending_transition: None,
            final_view_absolute_path: session.final_view_absolute_path,
        }
    }

    fn current_case(&self) -> &SessionCaseConfig {
        &self.cases[self.current_case_index]
    }

    fn current_case_label(&self) -> &str {
        &self.current_case().id
    }

    fn has_next_text(&self) -> bool {
        self.current_case_index + 1 < self.cases.len()
    }

    fn current_stage_path(&self) -> &PathBuf {
        match self.current_stage {
            SessionStageSlot::StageOne => &self.current_case().stage_one_path,
            SessionStageSlot::StageTwo => &self.current_case().stage_two_path,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionStageSlot {
    StageOne,
    StageTwo,
}

impl SessionStageSlot {
    fn label(self) -> &'static str {
        match self {
            Self::StageOne => "stage_1",
            Self::StageTwo => "stage_2",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingSessionTransition {
    NextStage,
    NextText,
}

#[derive(Resource, Default)]
struct CurrentViewAssetHandle {
    handle: Handle<ViewLayoutAsset>,
}

#[derive(Resource, Default)]
struct RenderTargetImage(Handle<Image>);

#[derive(Resource)]
struct ReferenceImages {
    original: image::RgbaImage,
    compare_masked: image::RgbaImage,
    width: u32,
    height: u32,
    reference_handle: Handle<Image>,
    diff_handle: Handle<Image>,
}

#[derive(Resource)]
struct SearchController {
    plan: CandidateSearchPlan,
    total_candidates: usize,
}

#[derive(Resource)]
struct ReferenceMaskState {
    vertical_split_x: f32,
    vertical_side: MaskOcclusionSide,
    horizontal_split_y: f32,
    horizontal_side: MaskOcclusionSide,
    dragging: Option<MaskGuideKind>,
}

#[derive(Resource)]
struct ReconstructionState {
    phase: EvaluationPhase,
    display_mode: DisplayMode,
    auto_search: bool,
    total_candidates: usize,
    target_similarity: f32,
    current_candidate_index: Option<usize>,
    current_parameters: ConcreteTextParameters,
    current_text: String,
    current_score: Option<ScoredCandidate>,
    best_score: Option<ScoredCandidate>,
    latest_render_image: Option<image::RgbaImage>,
    latest_diff_image: Option<image::RgbaImage>,
    persist_current_after_evaluation: bool,
    capture_after_apply: bool,
    manual_adjustment_kind: ManualAdjustmentKind,
    manual_step_multiplier_index: usize,
    snap_to_grid: bool,
    text_edit_mode: bool,
    show_detailed_status: bool,
    skip_snap_on_next_apply: bool,
    pending_apply: bool,
    awaiting_user_step: Option<AwaitingUserStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AwaitingUserStep {
    NextStage,
    NextText,
    SessionComplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EvaluationPhase {
    Ready,
    WaitingForSettle { remaining_frames: u32 },
    WaitingForScreenshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisplayMode {
    Reference,
    Render,
    Overlay,
    Diff,
}

impl DisplayMode {
    fn next(self) -> Self {
        match self {
            Self::Reference => Self::Render,
            Self::Render => Self::Overlay,
            Self::Overlay => Self::Diff,
            Self::Diff => Self::Reference,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MaskGuideKind {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MaskOcclusionSide {
    Negative,
    Positive,
}

impl MaskOcclusionSide {
    fn toggle(self) -> Self {
        match self {
            Self::Negative => Self::Positive,
            Self::Positive => Self::Negative,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManualAdjustmentKind {
    Content,
    Font,
    Align,
    Anchor,
    Translation,
    WorldScale,
    WorldScaleX,
    WorldScaleY,
    LineHeight,
    CharSpacing,
    WordSpacing,
}

impl ManualAdjustmentKind {
    fn initial_for_task(_task: &TaskConfig) -> Self {
        Self::Content
    }

    fn next_for_task(self, task: &TaskConfig) -> Self {
        let kinds = self.cycle_for_task(task);
        let current_index = kinds.iter().position(|kind| *kind == self).unwrap_or(0);
        kinds[(current_index + 1) % kinds.len()]
    }

    fn cycle_for_task(self, task: &TaskConfig) -> &'static [ManualAdjustmentKind] {
        if task.world_scale_bound {
            &[
                Self::Content,
                Self::Font,
                Self::Align,
                Self::Anchor,
                Self::Translation,
                Self::WorldScale,
                Self::LineHeight,
                Self::CharSpacing,
                Self::WordSpacing,
            ]
        } else {
            &[
                Self::Content,
                Self::Font,
                Self::Align,
                Self::Anchor,
                Self::Translation,
                Self::WorldScaleX,
                Self::WorldScaleY,
                Self::LineHeight,
                Self::CharSpacing,
                Self::WordSpacing,
            ]
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Content => "content",
            Self::Font => "font",
            Self::Align => "align",
            Self::Anchor => "anchor",
            Self::Translation => "translation",
            Self::WorldScale => "world_scale",
            Self::WorldScaleX => "world_scale_x",
            Self::WorldScaleY => "world_scale_y",
            Self::LineHeight => "line_height",
            Self::CharSpacing => "char_spacing",
            Self::WordSpacing => "word_spacing",
        }
    }

    fn base_step(self, task: &TaskConfig) -> f32 {
        match self {
            Self::Content | Self::Font | Self::Align | Self::Anchor => 1.0,
            Self::Translation => task
                .manual_steps
                .translation_x
                .max(task.manual_steps.translation_y),
            Self::WorldScale | Self::WorldScaleX | Self::WorldScaleY => {
                task.manual_steps.world_scale
            }
            Self::LineHeight => task.manual_steps.line_height,
            Self::CharSpacing => task.manual_steps.char_spacing,
            Self::WordSpacing => task.manual_steps.word_spacing,
        }
    }

    fn current_value(self, parameters: &ConcreteTextParameters) -> f32 {
        match self {
            Self::Content | Self::Font | Self::Align | Self::Anchor => 0.0,
            Self::Translation => parameters.translation_x,
            Self::WorldScale | Self::WorldScaleX => parameters.world_scale_x,
            Self::WorldScaleY => parameters.world_scale_y,
            Self::LineHeight => parameters.line_height,
            Self::CharSpacing => parameters.char_spacing,
            Self::WordSpacing => parameters.word_spacing,
        }
    }
}

impl ReferenceMaskState {
    fn new(_width: u32, _height: u32) -> Self {
        Self {
            vertical_split_x: 0.0,
            vertical_side: MaskOcclusionSide::Negative,
            horizontal_split_y: 0.0,
            horizontal_side: MaskOcclusionSide::Negative,
            dragging: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ScoredCandidate {
    candidate_index: Option<usize>,
    total_candidates: usize,
    text: String,
    parameters: ConcreteTextParameters,
    fitness_score: f32,
    global_similarity: f32,
    content_similarity: f32,
    pixel_match_rate: f32,
    content_mask_f1: f32,
    content_bbox_iou: f32,
    content_size_similarity: f32,
    content_center_similarity: f32,
    differing_pixels: u64,
}

#[derive(Debug, Clone)]
struct RestoredCurrentState {
    text: String,
    parameters: ConcreteTextParameters,
}

#[derive(Component)]
struct PreviewReferenceSprite;

#[derive(Component)]
struct PreviewRenderSprite;

#[derive(Component)]
struct PreviewDiffSprite;

#[derive(Component)]
struct PreviewCameraMarker;

#[derive(Component)]
struct PreviewVerticalOcclusionMask;

#[derive(Component)]
struct PreviewHorizontalOcclusionMask;

#[derive(Component)]
struct PreviewVerticalGuide;

#[derive(Component)]
struct PreviewHorizontalGuide;

#[derive(Component)]
struct PreviewStatusPanel;

#[derive(Component)]
struct PreviewStatusText;

#[derive(Component)]
struct PreviewControlsText;

#[derive(Component, Clone, Copy)]
struct InspectorFieldButton {
    field: ManualAdjustmentKind,
}

#[derive(Component, Clone, Copy)]
struct InspectorFieldText {
    field: ManualAdjustmentKind,
}

#[derive(Component)]
struct InspectorSnapButton;

#[derive(Component)]
struct InspectorSnapText;

#[derive(Component)]
struct PreviewDetailsText;

fn enter_running_state(mut next_state: ResMut<NextState<souprune::app_state::AppState>>) {
    next_state.set(souprune::app_state::AppState::Running);
}

fn build_reconstruction_game_projection(souprune_config: &SoupruneConfig) -> Projection {
    Projection::Orthographic(OrthographicProjection {
        scaling_mode: bevy::camera::ScalingMode::Fixed {
            width: souprune_config.render.base_resolution_width as f32,
            height: souprune_config.render.base_resolution_height as f32,
        },
        ..OrthographicProjection::default_2d()
    })
}

fn setup_runtime(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut reference_images: ResMut<ReferenceImages>,
    mut render_target: ResMut<RenderTargetImage>,
    mut current_view_handle: ResMut<CurrentViewAssetHandle>,
    state: Res<ReconstructionState>,
    task: Res<TaskResource>,
    asset_server: Res<AssetServer>,
    souprune_config: Res<SoupruneConfig>,
) {
    if let Some(parent_dir) = task.0.current_view_absolute_path.parent() {
        fs::create_dir_all(parent_dir).expect("failed to create generated view directory");
    }
    write_candidate_view_files(&task.0, &state.current_text, &state.current_parameters)
        .expect("initial view layouts should serialize to RON");

    let preview_reference = Image::from_dynamic(
        DynamicImage::ImageRgba8(reference_images.original.clone()),
        true,
        RenderAssetUsages::all(),
    );
    let blank_diff = Image::from_dynamic(
        DynamicImage::ImageRgba8(image::RgbaImage::new(
            reference_images.width,
            reference_images.height,
        )),
        true,
        RenderAssetUsages::all(),
    );
    let render_texture = Image::new_target_texture(
        reference_images.width,
        reference_images.height,
        TextureFormat::Rgba8UnormSrgb,
        None,
    );

    reference_images.reference_handle = images.add(preview_reference);
    reference_images.diff_handle = images.add(blank_diff);
    render_target.0 = images.add(render_texture);
    current_view_handle.handle =
        asset_server.load::<ViewLayoutAsset>(task.0.runtime_view_relative_path.clone());

    let game_projection = build_reconstruction_game_projection(&souprune_config);

    commands.spawn((
        Camera2d,
        Camera {
            clear_color: bevy::camera::ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        game_projection,
        RenderTarget::Image(render_target.0.clone().into()),
        MainGameCamera,
    ));
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: bevy::camera::ClearColorConfig::Custom(Color::srgb(0.08, 0.09, 0.11)),
            order: 1,
            ..default()
        },
        PREVIEW_LAYER,
        PreviewCameraMarker,
    ));

    commands.spawn((
        Sprite::from_image(reference_images.reference_handle.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        PREVIEW_LAYER,
        PreviewReferenceSprite,
    ));
    commands.spawn((
        Sprite::from_image(render_target.0.clone()),
        Transform::from_xyz(0.0, 0.0, 0.1),
        PREVIEW_LAYER,
        PreviewRenderSprite,
    ));
    commands.spawn((
        Sprite::from_image(reference_images.diff_handle.clone()),
        Transform::from_xyz(0.0, 0.0, 0.2),
        PREVIEW_LAYER,
        PreviewDiffSprite,
    ));
    commands.spawn((
        Sprite::from_color(Color::srgba(0.0, 0.0, 0.0, 0.92), Vec2::new(1.0, 1.0)),
        Transform::from_xyz(0.0, 0.0, 0.05),
        PREVIEW_LAYER,
        PreviewVerticalOcclusionMask,
    ));
    commands.spawn((
        Sprite::from_color(Color::srgba(0.0, 0.0, 0.0, 0.92), Vec2::new(1.0, 1.0)),
        Transform::from_xyz(0.0, 0.0, 0.06),
        PREVIEW_LAYER,
        PreviewHorizontalOcclusionMask,
    ));
    commands.spawn((
        Sprite::from_color(
            Color::srgb(1.0, 0.82, 0.18),
            Vec2::new(GUIDE_THICKNESS, 1.0),
        ),
        Transform::from_xyz(0.0, 0.0, 0.25),
        PREVIEW_LAYER,
        PreviewVerticalGuide,
    ));
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.22, 0.86, 1.0),
            Vec2::new(1.0, GUIDE_THICKNESS),
        ),
        Transform::from_xyz(0.0, 0.0, 0.26),
        PREVIEW_LAYER,
        PreviewHorizontalGuide,
    ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(PREVIEW_TEXT_MARGIN),
                right: px(PREVIEW_TEXT_MARGIN),
                width: px(320.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(8.0),
                padding: UiRect::axes(px(12.0), px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.88)),
            BorderColor::all(Color::srgba(0.82, 0.84, 0.9, 0.32)),
            PreviewStatusPanel,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Initializing view reconstruction..."),
                TextLayout::new_with_justify(Justify::Left),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                PreviewStatusText,
            ));
            parent
                .spawn((
                    Button,
                    Node {
                        width: percent(100.0),
                        justify_content: JustifyContent::Center,
                        padding: UiRect::axes(px(10.0), px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.14, 0.17, 0.22, 0.95)),
                    InspectorSnapButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Snap: On"),
                        TextLayout::new_with_justify(Justify::Left),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        InspectorSnapText,
                    ));
                });
            for field in ManualAdjustmentKind::initial_for_task(&task.0).cycle_for_task(&task.0) {
                spawn_inspector_field_row(parent, *field);
            }
            parent.spawn((
                Text::new(""),
                TextLayout::new_with_justify(Justify::Left),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.74, 0.8)),
                PreviewDetailsText,
            ));
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(PREVIEW_TEXT_MARGIN),
                right: px(PREVIEW_TEXT_MARGIN),
                bottom: px(PREVIEW_TEXT_MARGIN),
                padding: UiRect::axes(px(12.0), px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.84)),
            BorderColor::all(Color::srgba(0.82, 0.84, 0.9, 0.26)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(
                    "Tab: display  Space: evolve  C: tweak target  M: step x1/x2/x4/x8/x16/x32/x64",
                ),
                TextLayout::new_with_justify(Justify::Left),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.84, 0.9)),
                PreviewControlsText,
            ));
        });
}

fn replace_reference_images(
    images: &mut Assets<Image>,
    reference_images: &mut ReferenceImages,
    render_target_handle: &Handle<Image>,
    task: &TaskConfig,
) -> Result<()> {
    let reference_image = load_reference_image(&task.image_path)
        .with_context(|| "failed to open reference image for task switch")?;
    if let Some(bbox) = task.bbox {
        validate_bbox(&reference_image, bbox)?;
    }

    reference_images.original = reference_image.clone();
    reference_images.compare_masked = apply_bbox_mask(&reference_image, task.bbox);
    reference_images.width = reference_image.width();
    reference_images.height = reference_image.height();

    if let Some(reference_asset) = images.get_mut(&reference_images.reference_handle) {
        *reference_asset = Image::from_dynamic(
            DynamicImage::ImageRgba8(reference_image),
            true,
            RenderAssetUsages::all(),
        );
    }
    if let Some(diff_asset) = images.get_mut(&reference_images.diff_handle) {
        *diff_asset = Image::from_dynamic(
            DynamicImage::ImageRgba8(image::RgbaImage::new(
                reference_images.width,
                reference_images.height,
            )),
            true,
            RenderAssetUsages::all(),
        );
    }
    if let Some(render_target_asset) = images.get_mut(render_target_handle) {
        *render_target_asset = Image::new_target_texture(
            reference_images.width,
            reference_images.height,
            TextureFormat::Rgba8UnormSrgb,
            None,
        );
    }

    Ok(())
}

fn spawn_inspector_field_row(parent: &mut ChildSpawnerCommands, field: ManualAdjustmentKind) {
    parent
        .spawn((
            Button,
            Node {
                width: percent(100.0),
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::axes(px(10.0), px(7.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.12, 0.16, 0.88)),
            InspectorFieldButton { field },
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(field.label()),
                TextLayout::new_with_justify(Justify::Left),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                InspectorFieldText { field },
            ));
        });
}

fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    mut state: ResMut<ReconstructionState>,
    mut search: ResMut<SearchController>,
    task: Res<TaskResource>,
    mut session: ResMut<OptionalSessionController>,
) {
    if state.text_edit_mode {
        let text_changed = apply_text_input(&mut keyboard_inputs, &mut state);
        if text_changed {
            search.plan.clear_evaluation_history();
            mark_text_changed(&mut state);
        }
        return;
    }

    if keyboard.just_pressed(KeyCode::Tab) {
        state.display_mode = state.display_mode.next();
    }

    if keyboard.just_pressed(KeyCode::KeyS)
        && let (Some(current_score), Some(render_image), Some(diff_image)) = (
            state.current_score.as_ref(),
            state.latest_render_image.as_ref(),
            state.latest_diff_image.as_ref(),
        )
    {
        persist_current_snapshot(&task.0, current_score, render_image, diff_image);
    }

    if keyboard.just_pressed(KeyCode::KeyC) {
        state.manual_adjustment_kind = state.manual_adjustment_kind.next_for_task(&task.0);
    }

    if keyboard.just_pressed(KeyCode::KeyM) {
        state.manual_step_multiplier_index =
            (state.manual_step_multiplier_index + 1) % MANUAL_STEP_MULTIPLIERS.len();
    }

    if keyboard.just_pressed(KeyCode::KeyG) {
        state.snap_to_grid = !state.snap_to_grid;
    }

    if keyboard.just_pressed(KeyCode::KeyI) {
        state.show_detailed_status = !state.show_detailed_status;
    }

    if keyboard.just_pressed(KeyCode::KeyN)
        && matches!(state.phase, EvaluationPhase::Ready)
        && let Some(awaiting_user_step) = state.awaiting_user_step
        && let Some(session_controller) = session.0.as_mut()
    {
        session_controller.pending_transition = match awaiting_user_step {
            AwaitingUserStep::NextStage => Some(PendingSessionTransition::NextStage),
            AwaitingUserStep::NextText => Some(PendingSessionTransition::NextText),
            AwaitingUserStep::SessionComplete => None,
        };
        if session_controller.pending_transition.is_some() {
            state.awaiting_user_step = None;
        }
        return;
    }

    if keyboard.just_pressed(KeyCode::Enter)
        && state.manual_adjustment_kind == ManualAdjustmentKind::Content
    {
        state.text_edit_mode = true;
        return;
    }

    if keyboard.just_pressed(KeyCode::Space) && state.auto_search {
        state.auto_search = false;
        return;
    }

    if !matches!(state.phase, EvaluationPhase::Ready) {
        return;
    }

    if keyboard.just_pressed(KeyCode::Space) {
        match search
            .plan
            .restart_from_parameters(&state.current_parameters)
        {
            RestartSearchResult::SearchCandidate {
                candidate_index,
                parameters,
            } => {
                state.auto_search = search.total_candidates > 1;
                state.current_candidate_index = Some(candidate_index);
                state.current_parameters = parameters;
            }
            RestartSearchResult::ReevaluateCurrent { parameters } => {
                state.auto_search = false;
                state.current_candidate_index = None;
                state.current_parameters = parameters;
            }
        }
        state.total_candidates = search.total_candidates;
        state.target_similarity = task.0.target_similarity;
        state.current_score = None;
        state.awaiting_user_step = None;
        state.capture_after_apply = true;
        state.pending_apply = true;
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyR)
        && let Some(best_score) = state.best_score.clone()
    {
        state.auto_search = false;
        state.current_candidate_index = best_score.candidate_index;
        state.current_text = best_score.text.clone();
        state.current_parameters = best_score.parameters.clone();
        state.current_score = Some(best_score.clone());
        state.awaiting_user_step = None;
        state.capture_after_apply = false;
        state.pending_apply = true;
        return;
    }

    let changed = apply_keyboard_adjustment(&keyboard, &mut state, &search, &task.0);

    if changed {
        if state.snap_to_grid {
            search
                .plan
                .constrain_parameters(&mut state.current_parameters);
        }
        mark_parameter_changed(&mut state);
    }
}

fn mark_parameter_changed(state: &mut ReconstructionState) {
    state.auto_search = false;
    state.current_candidate_index = None;
    state.current_score = None;
    state.awaiting_user_step = None;
    state.persist_current_after_evaluation = true;
    state.capture_after_apply = true;
    state.pending_apply = true;
}

fn mark_text_changed(state: &mut ReconstructionState) {
    state.auto_search = false;
    state.current_candidate_index = None;
    state.current_score = None;
    state.best_score = None;
    state.awaiting_user_step = None;
    state.persist_current_after_evaluation = true;
    state.capture_after_apply = true;
    state.pending_apply = true;
}

fn mark_reference_mask_changed(state: &mut ReconstructionState, task: &TaskConfig) {
    state.auto_search = false;
    state.current_candidate_index = None;
    state.current_score = None;
    state.best_score = None;
    state.awaiting_user_step = None;
    state.persist_current_after_evaluation = true;
    state.phase = EvaluationPhase::WaitingForSettle {
        remaining_frames: task.settle_frames,
    };
}

fn apply_text_input(
    keyboard_inputs: &mut MessageReader<KeyboardInput>,
    state: &mut ReconstructionState,
) -> bool {
    let mut changed = false;
    for keyboard_input in keyboard_inputs.read() {
        if !keyboard_input.state.is_pressed() {
            continue;
        }

        match (&keyboard_input.logical_key, &keyboard_input.text) {
            (Key::Enter, _) | (Key::Escape, _) => {
                state.text_edit_mode = false;
            }
            (Key::Backspace, _) => {
                changed |= state.current_text.pop().is_some();
            }
            (_, Some(inserted_text)) => {
                if inserted_text.chars().all(is_printable_char) {
                    state.current_text.push_str(inserted_text);
                    changed = true;
                }
            }
            _ => {}
        }
    }
    changed
}

fn handle_inspector_interactions(
    buttons: Query<
        (
            &Interaction,
            Option<&InspectorFieldButton>,
            Has<InspectorSnapButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<ReconstructionState>,
) {
    for (interaction, row_button, is_snap_button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(row_button) = row_button {
            if row_button.field == state.manual_adjustment_kind
                && row_button.field == ManualAdjustmentKind::Content
            {
                state.text_edit_mode = true;
            } else {
                state.manual_adjustment_kind = row_button.field;
                state.text_edit_mode = false;
            }
        } else if is_snap_button {
            state.snap_to_grid = !state.snap_to_grid;
        }
    }
}

fn handle_reference_mask_interactions(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    preview_camera: Query<(&Camera, &GlobalTransform), With<PreviewCameraMarker>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    reference_images: Res<ReferenceImages>,
    task: Res<TaskResource>,
    mut search: ResMut<SearchController>,
    mut mask_state: ResMut<ReferenceMaskState>,
    mut state: ResMut<ReconstructionState>,
) {
    if mouse_buttons.just_released(MouseButton::Left) {
        mask_state.dragging = None;
    }

    if !matches!(state.phase, EvaluationPhase::Ready) {
        return;
    }

    let Ok(window) = primary_window.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = preview_camera.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok(cursor_world) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let geometry = compute_preview_geometry(window, &reference_images);
    let Some(cursor_local) = cursor_world_to_preview_local(cursor_world, &geometry) else {
        return;
    };
    let hit_guide = pick_mask_guide(cursor_local, &geometry, &mask_state);

    if mouse_buttons.just_pressed(MouseButton::Left) {
        mask_state.dragging = hit_guide;
    }

    let mut changed = false;
    if mouse_buttons.pressed(MouseButton::Left)
        && let Some(dragging) = mask_state.dragging
    {
        changed |=
            update_mask_split_from_cursor(dragging, cursor_local, &geometry, &mut mask_state);
    }

    if mouse_buttons.just_pressed(MouseButton::Right)
        && let Some(hit_guide) = hit_guide
    {
        match hit_guide {
            MaskGuideKind::Vertical => {
                mask_state.vertical_side = mask_state.vertical_side.toggle();
            }
            MaskGuideKind::Horizontal => {
                mask_state.horizontal_side = mask_state.horizontal_side.toggle();
            }
        }
        changed = true;
    }

    if changed {
        search.plan.clear_evaluation_history();
        mark_reference_mask_changed(&mut state, &task.0);
    }
}

fn apply_pending_session_transition(
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut despawn_writer: MessageWriter<DespawnViewRequest>,
    mut task: ResMut<TaskResource>,
    mut session: ResMut<OptionalSessionController>,
    mut search: ResMut<SearchController>,
    mut state: ResMut<ReconstructionState>,
    mut reference_images: ResMut<ReferenceImages>,
    mut reference_mask_state: ResMut<ReferenceMaskState>,
    mut current_view_handle: ResMut<CurrentViewAssetHandle>,
    render_target: Res<RenderTargetImage>,
) {
    let Some(session_controller) = session.0.as_mut() else {
        return;
    };
    let Some(pending_transition) = session_controller.pending_transition.take() else {
        return;
    };

    let next_task = match pending_transition {
        PendingSessionTransition::NextStage => {
            session_controller.current_stage = SessionStageSlot::StageTwo;
            TaskConfig::load_stage_ron(
                session_controller.current_stage_path(),
                &task.0.workspace_root,
                Some(&state.current_parameters),
            )
        }
        PendingSessionTransition::NextText => {
            if !session_controller.has_next_text() {
                state.awaiting_user_step = Some(AwaitingUserStep::SessionComplete);
                return;
            }
            session_controller.current_case_index += 1;
            session_controller.current_stage = SessionStageSlot::StageOne;
            TaskConfig::load_stage_ron(
                session_controller.current_stage_path(),
                &task.0.workspace_root,
                None,
            )
        }
    };

    let Ok(next_task) = next_task else {
        session_controller.pending_transition = Some(pending_transition);
        return;
    };

    let previous_runtime_path = task.0.runtime_view_relative_path.clone();
    despawn_writer.write(DespawnViewRequest {
        path: Some(previous_runtime_path),
    });

    replace_reference_images(
        &mut images,
        &mut reference_images,
        &render_target.0,
        &next_task,
    )
    .expect("next stage reference image should load");
    *reference_mask_state =
        ReferenceMaskState::new(reference_images.width, reference_images.height);

    current_view_handle.handle =
        asset_server.load::<ViewLayoutAsset>(next_task.runtime_view_relative_path.clone());
    let next_search_plan = next_task.search_plan.clone();
    let restored_current = load_saved_resume_state(&next_task, &next_search_plan);
    let restored_best = load_saved_best_score(&next_task, &next_search_plan);
    let (initial_parameters, initial_text, skip_initial_snap) =
        if let Some(restored_current) = restored_current {
            (restored_current.parameters, restored_current.text, true)
        } else {
            (
                next_search_plan.seed_parameters(),
                next_task.text.clone(),
                false,
            )
        };
    search.plan = next_search_plan;
    search.total_candidates = next_task.search_plan.total_candidates();

    task.0 = next_task.clone();
    state.phase = EvaluationPhase::Ready;
    state.display_mode = DisplayMode::Overlay;
    state.auto_search = false;
    state.total_candidates = search.total_candidates;
    state.target_similarity = next_task.target_similarity;
    state.current_candidate_index = None;
    state.current_parameters = initial_parameters;
    state.current_text = initial_text;
    state.current_score = None;
    state.best_score = restored_best;
    state.latest_render_image = None;
    state.latest_diff_image = None;
    state.persist_current_after_evaluation = false;
    state.capture_after_apply = true;
    state.manual_adjustment_kind = ManualAdjustmentKind::initial_for_task(&next_task);
    state.manual_step_multiplier_index = 0;
    state.snap_to_grid = true;
    state.text_edit_mode = false;
    state.show_detailed_status = false;
    state.skip_snap_on_next_apply = skip_initial_snap;
    state.pending_apply = true;
    state.awaiting_user_step = None;
}

fn apply_keyboard_adjustment(
    keyboard: &ButtonInput<KeyCode>,
    state: &mut ReconstructionState,
    search: &SearchController,
    task: &TaskConfig,
) -> bool {
    match state.manual_adjustment_kind {
        ManualAdjustmentKind::Content => false,
        ManualAdjustmentKind::Font => {
            apply_discrete_field_adjustment(keyboard, state, search, SearchParameterField::Font)
        }
        ManualAdjustmentKind::Align => {
            apply_discrete_field_adjustment(keyboard, state, search, SearchParameterField::Align)
        }
        ManualAdjustmentKind::Anchor => {
            apply_discrete_field_adjustment(keyboard, state, search, SearchParameterField::Anchor)
        }
        ManualAdjustmentKind::Translation => {
            apply_translation_adjustment(keyboard, state, search, task)
        }
        ManualAdjustmentKind::WorldScale => apply_scalar_field_adjustment(
            keyboard,
            state,
            search,
            task,
            SearchParameterField::WorldScaleX,
            |parameters, delta| {
                parameters.world_scale_x = round3(parameters.world_scale_x + delta);
                parameters.world_scale_y = round3(parameters.world_scale_y + delta);
            },
        ),
        ManualAdjustmentKind::WorldScaleX => apply_scalar_field_adjustment(
            keyboard,
            state,
            search,
            task,
            SearchParameterField::WorldScaleX,
            |parameters, delta| parameters.world_scale_x = round3(parameters.world_scale_x + delta),
        ),
        ManualAdjustmentKind::WorldScaleY => apply_scalar_field_adjustment(
            keyboard,
            state,
            search,
            task,
            SearchParameterField::WorldScaleY,
            |parameters, delta| parameters.world_scale_y = round3(parameters.world_scale_y + delta),
        ),
        ManualAdjustmentKind::LineHeight => apply_scalar_field_adjustment(
            keyboard,
            state,
            search,
            task,
            SearchParameterField::LineHeight,
            |parameters, delta| parameters.line_height = round3(parameters.line_height + delta),
        ),
        ManualAdjustmentKind::CharSpacing => apply_scalar_field_adjustment(
            keyboard,
            state,
            search,
            task,
            SearchParameterField::CharSpacing,
            |parameters, delta| parameters.char_spacing = round3(parameters.char_spacing + delta),
        ),
        ManualAdjustmentKind::WordSpacing => apply_scalar_field_adjustment(
            keyboard,
            state,
            search,
            task,
            SearchParameterField::WordSpacing,
            |parameters, delta| parameters.word_spacing = round3(parameters.word_spacing + delta),
        ),
    }
}

fn apply_discrete_field_adjustment(
    keyboard: &ButtonInput<KeyCode>,
    state: &mut ReconstructionState,
    search: &SearchController,
    field: SearchParameterField,
) -> bool {
    let direction = scalar_direction_from_keyboard(keyboard);
    if direction == 0 {
        return false;
    }
    let multiplier = state.current_manual_multiplier();
    search
        .plan
        .nudge_parameter(&mut state.current_parameters, field, direction * multiplier);
    true
}

fn apply_translation_adjustment(
    keyboard: &ButtonInput<KeyCode>,
    state: &mut ReconstructionState,
    search: &SearchController,
    task: &TaskConfig,
) -> bool {
    let mut changed = false;
    let multiplier = state.current_manual_multiplier_value();
    let discrete_multiplier = state.current_manual_multiplier();

    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        changed = true;
        if state.snap_to_grid {
            search.plan.nudge_parameter(
                &mut state.current_parameters,
                SearchParameterField::TranslationX,
                -discrete_multiplier,
            );
        } else {
            state.current_parameters.translation_x = round3(
                state.current_parameters.translation_x
                    - task.manual_steps.translation_x * multiplier,
            );
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        changed = true;
        if state.snap_to_grid {
            search.plan.nudge_parameter(
                &mut state.current_parameters,
                SearchParameterField::TranslationX,
                discrete_multiplier,
            );
        } else {
            state.current_parameters.translation_x = round3(
                state.current_parameters.translation_x
                    + task.manual_steps.translation_x * multiplier,
            );
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        changed = true;
        if state.snap_to_grid {
            search.plan.nudge_parameter(
                &mut state.current_parameters,
                SearchParameterField::TranslationY,
                discrete_multiplier,
            );
        } else {
            state.current_parameters.translation_y = round3(
                state.current_parameters.translation_y
                    + task.manual_steps.translation_y * multiplier,
            );
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        changed = true;
        if state.snap_to_grid {
            search.plan.nudge_parameter(
                &mut state.current_parameters,
                SearchParameterField::TranslationY,
                -discrete_multiplier,
            );
        } else {
            state.current_parameters.translation_y = round3(
                state.current_parameters.translation_y
                    - task.manual_steps.translation_y * multiplier,
            );
        }
    }

    changed
}

fn apply_scalar_field_adjustment(
    keyboard: &ButtonInput<KeyCode>,
    state: &mut ReconstructionState,
    search: &SearchController,
    task: &TaskConfig,
    field: SearchParameterField,
    raw_adjust: impl FnOnce(&mut ConcreteTextParameters, f32),
) -> bool {
    let direction = scalar_direction_from_keyboard(keyboard);
    if direction == 0 {
        return false;
    }
    if state.snap_to_grid {
        let multiplier = state.current_manual_multiplier();
        search
            .plan
            .nudge_parameter(&mut state.current_parameters, field, direction * multiplier);
    } else {
        let signed_step = state.current_manual_step(task) * direction as f32;
        raw_adjust(&mut state.current_parameters, signed_step);
    }
    true
}

fn scalar_direction_from_keyboard(keyboard: &ButtonInput<KeyCode>) -> i32 {
    let mut direction = 0i32;
    if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::ArrowDown) {
        direction -= 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::ArrowUp) {
        direction += 1;
    }
    direction
}

fn apply_pending_candidate(
    mut view_layouts: ResMut<Assets<ViewLayoutAsset>>,
    mut despawn_writer: MessageWriter<DespawnViewRequest>,
    mut spawn_writer: MessageWriter<SpawnViewRequest>,
    search: Res<SearchController>,
    mut state: ResMut<ReconstructionState>,
    task: Res<TaskResource>,
    current_view_handle: Res<CurrentViewAssetHandle>,
) {
    if !state.pending_apply || !matches!(state.phase, EvaluationPhase::Ready) {
        return;
    }

    if state.snap_to_grid && !state.skip_snap_on_next_apply {
        search
            .plan
            .constrain_parameters(&mut state.current_parameters);
    }
    state.skip_snap_on_next_apply = false;

    write_candidate_view_files(&task.0, &state.current_text, &state.current_parameters)
        .expect("current candidate view layouts should serialize to RON");

    let runtime_layout: ViewLayoutAsset = ron::from_str(
        &fs::read_to_string(&task.0.runtime_view_absolute_path)
            .expect("generated runtime view RON file should be readable immediately after writing"),
    )
    .expect("generated runtime view RON should deserialize into runtime asset");
    view_layouts
        .insert(current_view_handle.handle.id(), runtime_layout)
        .expect("current view handle should accept asset replacement");

    despawn_writer.write(DespawnViewRequest {
        path: Some(task.0.runtime_view_relative_path.clone()),
    });
    spawn_writer.write(SpawnViewRequest {
        path: task.0.runtime_view_relative_path.clone(),
        mode_scope: None,
        bindings: None,
    });

    state.pending_apply = false;
    state.phase = if state.capture_after_apply {
        state.capture_after_apply = false;
        EvaluationPhase::WaitingForSettle {
            remaining_frames: task.0.settle_frames,
        }
    } else {
        EvaluationPhase::Ready
    };
}

fn load_saved_resume_state(
    task: &TaskConfig,
    search_plan: &CandidateSearchPlan,
) -> Option<RestoredCurrentState> {
    for summary_path in [&task.best_summary_path, &task.current_summary_path] {
        if !summary_path.exists() {
            continue;
        }
        match load_resume_state_from_summary(task, search_plan, summary_path) {
            Ok(Some(restored_state)) => return Some(restored_state),
            Ok(None) => {}
            Err(error) => {
                eprintln!(
                    "[view_text_reconstruction] failed to restore {}: {error:#}",
                    summary_path.display()
                );
            }
        }
    }

    for view_path in [
        &task.best_view_absolute_path,
        &task.current_view_absolute_path,
    ] {
        if !view_path.exists() {
            continue;
        }
        match load_resume_state_from_view_ron(task, search_plan, view_path) {
            Ok(Some(restored_state)) => return Some(restored_state),
            Ok(None) => {}
            Err(error) => {
                eprintln!(
                    "[view_text_reconstruction] failed to restore {}: {error:#}",
                    view_path.display()
                );
            }
        }
    }

    None
}

fn load_saved_best_score(
    task: &TaskConfig,
    search_plan: &CandidateSearchPlan,
) -> Option<ScoredCandidate> {
    if !task.best_summary_path.exists() {
        return None;
    }

    match load_scored_candidate_from_summary(task, search_plan, &task.best_summary_path) {
        Ok(Some(score)) => Some(score),
        Ok(None) => None,
        Err(error) => {
            eprintln!(
                "[view_text_reconstruction] failed to restore {}: {error:#}",
                task.best_summary_path.display()
            );
            None
        }
    }
}

fn load_resume_state_from_summary(
    task: &TaskConfig,
    search_plan: &CandidateSearchPlan,
    summary_path: &Path,
) -> Result<Option<RestoredCurrentState>> {
    if !summary_path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(summary_path)
        .with_context(|| format!("failed to read saved summary: {}", summary_path.display()))?;
    let persisted: PersistedScoredCandidate = serde_json::from_str(&raw).with_context(|| {
        format!(
            "failed to parse saved summary JSON: {}",
            summary_path.display()
        )
    })?;

    Ok(Some(RestoredCurrentState {
        text: persisted.text,
        parameters: decode_persisted_parameters(task, search_plan, &persisted.parameters)?,
    }))
}

fn load_scored_candidate_from_summary(
    task: &TaskConfig,
    search_plan: &CandidateSearchPlan,
    summary_path: &Path,
) -> Result<Option<ScoredCandidate>> {
    if !summary_path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(summary_path)
        .with_context(|| format!("failed to read saved summary: {}", summary_path.display()))?;
    let persisted: PersistedScoredCandidate = serde_json::from_str(&raw).with_context(|| {
        format!(
            "failed to parse saved summary JSON: {}",
            summary_path.display()
        )
    })?;

    Ok(Some(ScoredCandidate {
        candidate_index: persisted.candidate_index,
        total_candidates: persisted.total_candidates,
        text: persisted.text,
        parameters: decode_persisted_parameters(task, search_plan, &persisted.parameters)?,
        fitness_score: persisted.fitness_score,
        global_similarity: persisted.global_similarity,
        content_similarity: persisted.content_similarity,
        pixel_match_rate: persisted.pixel_match_rate,
        content_mask_f1: persisted.content_mask_f1,
        content_bbox_iou: persisted.content_bbox_iou,
        content_size_similarity: persisted.content_size_similarity,
        content_center_similarity: persisted.content_center_similarity,
        differing_pixels: persisted.differing_pixels,
    }))
}

fn load_resume_state_from_view_ron(
    task: &TaskConfig,
    search_plan: &CandidateSearchPlan,
    view_path: &Path,
) -> Result<Option<RestoredCurrentState>> {
    if !view_path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(view_path)
        .with_context(|| format!("failed to read saved view RON: {}", view_path.display()))?;
    let layout: SchemaViewLayoutAsset = ron::from_str(&raw)
        .with_context(|| format!("failed to parse saved view RON: {}", view_path.display()))?;
    let text_def = find_target_text_def(&layout, task.host_view.as_ref())?;

    let mut parameters = search_plan.seed_parameters();
    parameters.font = text_def.font.clone();
    parameters.align = text_def.align.unwrap_or(parameters.align);
    parameters.anchor = text_def.anchor.unwrap_or(parameters.anchor);
    parameters.world_scale_x =
        extract_static_val(&text_def.world_scale.0, parameters.world_scale_x);
    parameters.world_scale_y =
        extract_static_val(&text_def.world_scale.1, parameters.world_scale_y);
    if let Some((translation_x, translation_y, _)) = &text_def.transform.translation {
        parameters.translation_x = extract_static_val(translation_x, parameters.translation_x);
        parameters.translation_y = extract_static_val(translation_y, parameters.translation_y);
    }
    parameters.line_height = text_def.line_height.unwrap_or(parameters.line_height);
    parameters.char_spacing = text_def.char_spacing.unwrap_or(parameters.char_spacing);
    parameters.word_spacing = text_def.word_spacing.unwrap_or(parameters.word_spacing);
    search_plan.constrain_parameters(&mut parameters);

    Ok(Some(RestoredCurrentState {
        text: if task.host_view.is_some() {
            task.text.clone()
        } else {
            text_def
                .content
                .clone()
                .unwrap_or_else(|| task.text.clone())
        },
        parameters,
    }))
}

fn decode_persisted_parameters(
    task: &TaskConfig,
    search_plan: &CandidateSearchPlan,
    persisted: &PersistedTextParameters,
) -> Result<ConcreteTextParameters> {
    let mut parameters = search_plan.seed_parameters();
    parameters.font = parse_view_font(&persisted.font)?;
    parameters.align = parse_optional_align_label(
        &persisted.align,
        task.property_defaults.align_uses_default,
        parameters.align,
    )?;
    parameters.anchor = parse_optional_anchor_label(
        &persisted.anchor,
        task.property_defaults.anchor_uses_default,
        parameters.anchor,
    )?;
    parameters.translation_x = persisted.translation_x;
    parameters.translation_y = persisted.translation_y;
    parameters.world_scale_x = persisted.world_scale_x;
    parameters.world_scale_y = persisted.world_scale_y;
    parameters.line_height = persisted.line_height;
    parameters.char_spacing = persisted.char_spacing;
    parameters.word_spacing = persisted.word_spacing;
    search_plan.constrain_parameters(&mut parameters);
    Ok(parameters)
}

fn parse_optional_align_label(
    value: &str,
    default_allowed: bool,
    default_value: souprune_schema::view::TextAlignDef,
) -> Result<souprune_schema::view::TextAlignDef> {
    if default_allowed && value.eq_ignore_ascii_case("default") {
        Ok(default_value)
    } else {
        parse_text_align(value)
    }
}

fn parse_optional_anchor_label(
    value: &str,
    default_allowed: bool,
    default_value: souprune_schema::view::TextAnchorDef,
) -> Result<souprune_schema::view::TextAnchorDef> {
    if default_allowed && value.eq_ignore_ascii_case("default") {
        Ok(default_value)
    } else {
        parse_text_anchor(value)
    }
}

fn extract_static_val(value: &Val<f32>, default_value: f32) -> f32 {
    match value {
        Val::Static(number) => *number,
        _ => default_value,
    }
}

fn write_candidate_view_files(
    task: &TaskConfig,
    text: &str,
    parameters: &ConcreteTextParameters,
) -> Result<()> {
    ensure_parent_directory(&task.current_view_absolute_path);
    ensure_parent_directory(&task.runtime_view_absolute_path);

    let export_layout = build_export_view_layout(
        text,
        parameters,
        task.property_defaults,
        task.field_override_policy,
        task.host_view.as_ref(),
    )?;
    let runtime_layout = build_runtime_view_layout(
        text,
        parameters,
        task.property_defaults,
        task.field_override_policy,
        task.host_view.as_ref(),
    )?;
    let pretty_config = ron::ser::PrettyConfig::new();
    let export_ron = ron::ser::to_string_pretty(&export_layout, pretty_config.clone())
        .context("export schema should serialize to RON")?;
    let runtime_ron = ron::ser::to_string_pretty(&runtime_layout, pretty_config)
        .context("runtime schema should serialize to RON")?;

    fs::write(&task.current_view_absolute_path, export_ron)
        .context("failed to write export view RON file")?;
    fs::write(&task.runtime_view_absolute_path, runtime_ron)
        .context("failed to write runtime view RON file")?;
    Ok(())
}

fn drive_capture_state(
    mut commands: Commands,
    render_target: Res<RenderTargetImage>,
    mut state: ResMut<ReconstructionState>,
) {
    match state.phase {
        EvaluationPhase::Ready => {}
        EvaluationPhase::WaitingForSettle {
            ref mut remaining_frames,
        } => {
            if *remaining_frames > 0 {
                *remaining_frames -= 1;
                return;
            }

            commands
                .spawn(Screenshot::image(render_target.0.clone()))
                .observe(handle_screenshot_captured);
            state.phase = EvaluationPhase::WaitingForScreenshot;
        }
        EvaluationPhase::WaitingForScreenshot => {}
    }
}

fn handle_screenshot_captured(
    trigger: On<ScreenshotCaptured>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
    mut state: ResMut<ReconstructionState>,
    mut search: ResMut<SearchController>,
    task: Res<TaskResource>,
    session: Res<OptionalSessionController>,
    reference_images: Res<ReferenceImages>,
    mask_state: Res<ReferenceMaskState>,
    mut images: ResMut<Assets<Image>>,
) {
    let screenshot_image = trigger
        .event()
        .image
        .clone()
        .try_into_dynamic()
        .expect("screenshot image should convert to a DynamicImage")
        .to_rgba8();
    if let Some(capture_path) = &task.0.capture_reference_absolute_path {
        if let Some(parent_dir) = capture_path.parent() {
            fs::create_dir_all(parent_dir).expect("failed to create capture reference directory");
        }
        screenshot_image
            .save(capture_path)
            .expect("failed to save captured reference image");
        commands.entity(trigger.entity).despawn();
        exit.write(AppExit::Success);
        return;
    }
    let masked_screenshot = apply_bbox_mask(&screenshot_image, task.0.bbox);
    let masked_reference =
        apply_reference_occlusion_mask(&reference_images.compare_masked, &mask_state);
    let (comparison, diff_image) =
        bevy_alight_motion::image_comparison::compare_images(&masked_screenshot, &masked_reference);

    if let Some(diff_asset) = images.get_mut(&reference_images.diff_handle) {
        *diff_asset = Image::from_dynamic(
            DynamicImage::ImageRgba8(diff_image.clone()),
            true,
            RenderAssetUsages::all(),
        );
    }
    state.latest_diff_image = Some(diff_image.clone());
    state.latest_render_image = Some(screenshot_image.clone());
    let fitness_score = compute_candidate_fitness(comparison);
    search.plan.record_fitness(fitness_score);

    let scored_candidate = ScoredCandidate {
        candidate_index: state.current_candidate_index,
        total_candidates: search.total_candidates,
        text: state.current_text.clone(),
        parameters: state.current_parameters.clone(),
        fitness_score,
        global_similarity: comparison.global_similarity,
        content_similarity: comparison.content_similarity,
        pixel_match_rate: comparison.pixel_match_rate,
        content_mask_f1: comparison.content_mask_f1,
        content_bbox_iou: comparison.content_bbox_iou,
        content_size_similarity: comparison.content_size_similarity,
        content_center_similarity: comparison.content_center_similarity,
        differing_pixels: comparison.differing_pixels,
    };
    state.current_score = Some(scored_candidate.clone());

    let is_new_best = state
        .best_score
        .as_ref()
        .map(|best| scored_candidate.fitness_score > best.fitness_score)
        .unwrap_or(true);
    if is_new_best {
        state.best_score = Some(scored_candidate.clone());
        persist_best_candidate(&task.0, &scored_candidate, &screenshot_image, &diff_image);
        if let Some(session_controller) = session.0.as_ref() {
            persist_session_final_view(
                &session_controller.final_view_absolute_path,
                &task.0,
                &scored_candidate.parameters,
            );
        }
    }

    let target_reached = scored_candidate.fitness_score >= state.target_similarity;
    let mut search_exhausted = false;

    state.phase = EvaluationPhase::Ready;
    state.awaiting_user_step = None;
    if state.auto_search && target_reached {
        state.auto_search = false;
    } else if state.auto_search {
        if let Some((candidate_index, parameters)) = search.plan.next_candidate() {
            state.current_candidate_index = Some(candidate_index);
            state.current_parameters = parameters;
            state.capture_after_apply = true;
            state.pending_apply = true;
        } else {
            state.auto_search = false;
            search_exhausted = true;
        }
    }

    if target_reached && let Some(session_controller) = session.0.as_ref() {
        state.awaiting_user_step = Some(match task.0.stage_kind {
            StageKind::AlignFirstGlyph => AwaitingUserStep::NextStage,
            StageKind::Single | StageKind::RefineSpacing => {
                if session_controller.has_next_text() {
                    AwaitingUserStep::NextText
                } else {
                    AwaitingUserStep::SessionComplete
                }
            }
        });
    }

    if state.persist_current_after_evaluation || target_reached || !state.auto_search {
        persist_current_snapshot(&task.0, &scored_candidate, &screenshot_image, &diff_image);
        state.persist_current_after_evaluation = false;
    }

    if task.0.exit_on_completion
        && (target_reached || search_exhausted || state.total_candidates <= 1)
    {
        if target_reached {
            exit.write(AppExit::Success);
        } else {
            exit.write(AppExit::Error(std::num::NonZero::new(1).unwrap()));
        }
    }
}

fn update_preview_scene(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut preview_nodes: ParamSet<(
        Query<(&mut Sprite, &mut Transform), With<PreviewReferenceSprite>>,
        Query<(&mut Sprite, &mut Transform), With<PreviewRenderSprite>>,
        Query<(&mut Sprite, &mut Transform), With<PreviewDiffSprite>>,
        Query<(&mut Sprite, &mut Transform), With<PreviewVerticalOcclusionMask>>,
        Query<(&mut Sprite, &mut Transform), With<PreviewHorizontalOcclusionMask>>,
        Query<(&mut Sprite, &mut Transform), With<PreviewVerticalGuide>>,
        Query<(&mut Sprite, &mut Transform), With<PreviewHorizontalGuide>>,
    )>,
    reference_images: Res<ReferenceImages>,
    mask_state: Res<ReferenceMaskState>,
    state: Res<ReconstructionState>,
) {
    let Ok(window) = primary_window.single() else {
        return;
    };

    let geometry = compute_preview_geometry(window, &reference_images);
    let preview_center = geometry.center.extend(0.0);
    let preview_scale = Vec3::splat(geometry.scale);
    let (reference_alpha, render_alpha, diff_alpha) = match state.display_mode {
        DisplayMode::Reference => (1.0, 0.0, 0.0),
        DisplayMode::Render => (0.0, 1.0, 0.0),
        DisplayMode::Overlay => (1.0, 0.55, 0.0),
        DisplayMode::Diff => (0.0, 0.0, 1.0),
    };

    {
        let mut reference_query = preview_nodes.p0();
        let Ok((mut sprite, mut transform)) = reference_query.single_mut() else {
            return;
        };
        transform.translation = preview_center;
        transform.scale = preview_scale;
        sprite.color = Color::WHITE.with_alpha(reference_alpha);
    }
    {
        let mut render_query = preview_nodes.p1();
        let Ok((mut sprite, mut transform)) = render_query.single_mut() else {
            return;
        };
        transform.translation = preview_center + Vec3::new(0.0, 0.0, 0.1);
        transform.scale = preview_scale;
        sprite.color = Color::WHITE.with_alpha(render_alpha);
    }
    {
        let mut diff_query = preview_nodes.p2();
        let Ok((mut sprite, mut transform)) = diff_query.single_mut() else {
            return;
        };
        transform.translation = preview_center + Vec3::new(0.0, 0.0, 0.2);
        transform.scale = preview_scale;
        sprite.color = Color::WHITE.with_alpha(diff_alpha);
    }
    {
        let mut vertical_mask_query = preview_nodes.p3();
        if let Ok((mut sprite, mut transform)) = vertical_mask_query.single_mut() {
            apply_vertical_mask_visual(
                &mut sprite,
                &mut transform,
                &geometry,
                &mask_state,
                matches!(
                    state.display_mode,
                    DisplayMode::Reference | DisplayMode::Overlay
                ),
            );
        }
    }
    {
        let mut horizontal_mask_query = preview_nodes.p4();
        if let Ok((mut sprite, mut transform)) = horizontal_mask_query.single_mut() {
            apply_horizontal_mask_visual(
                &mut sprite,
                &mut transform,
                &geometry,
                &mask_state,
                matches!(
                    state.display_mode,
                    DisplayMode::Reference | DisplayMode::Overlay
                ),
            );
        }
    }
    {
        let mut vertical_guide_query = preview_nodes.p5();
        if let Ok((mut sprite, mut transform)) = vertical_guide_query.single_mut() {
            apply_vertical_guide_visual(&mut sprite, &mut transform, &geometry, &mask_state);
        }
    }
    {
        let mut horizontal_guide_query = preview_nodes.p6();
        if let Ok((mut sprite, mut transform)) = horizontal_guide_query.single_mut() {
            apply_horizontal_guide_visual(&mut sprite, &mut transform, &geometry, &mask_state);
        }
    }
}

fn update_inspector_panel(
    mut ui_queries: ParamSet<(
        Query<&mut Node, With<PreviewStatusPanel>>,
        Query<(
            &mut Text,
            Has<PreviewStatusText>,
            Has<PreviewDetailsText>,
            Has<PreviewControlsText>,
            Has<InspectorSnapText>,
            Option<&InspectorFieldText>,
        )>,
        Query<(&InspectorFieldButton, &mut BackgroundColor)>,
    )>,
    task: Res<TaskResource>,
    session: Res<OptionalSessionController>,
    state: Res<ReconstructionState>,
    mask_state: Res<ReferenceMaskState>,
) {
    let current_similarity = state
        .current_score
        .as_ref()
        .map(|score| score.fitness_score)
        .unwrap_or(0.0);

    if let Ok(mut panel) = ui_queries.p0().single_mut() {
        panel.width = if state.show_detailed_status {
            px(420.0)
        } else {
            px(320.0)
        };
    }

    {
        let mut row_button_query = ui_queries.p2();
        for (row_button, mut background) in &mut row_button_query {
            *background = if row_button.field == state.manual_adjustment_kind {
                BackgroundColor(Color::srgba(0.20, 0.28, 0.38, 0.98))
            } else {
                BackgroundColor(Color::srgba(0.10, 0.12, 0.16, 0.88))
            };
        }
    }

    {
        let mut text_query = ui_queries.p1();
        for (mut text, is_status, is_details, is_controls, is_snap, row_text) in &mut text_query {
            if is_status {
                let session_label = session
                    .0
                    .as_ref()
                    .map(|session_controller| {
                        format!(
                            "case: {} ({}/{})\nstage: {}",
                            session_controller.current_case_label(),
                            session_controller.current_case_index + 1,
                            session_controller.cases.len(),
                            session_controller.current_stage.label(),
                        )
                    })
                    .unwrap_or_else(|| format!("stage: {}", task.0.stage_kind.label()));
                let advance_label = match state.awaiting_user_step {
                    Some(AwaitingUserStep::NextStage) => "ready: press N for stage 2",
                    Some(AwaitingUserStep::NextText) => "ready: press N for next text",
                    Some(AwaitingUserStep::SessionComplete) => "session complete",
                    None => "ready: keep tuning or press Space",
                };
                *text = Text::new(format!(
                    "{session_label}\nsimilarity: {:.4}\n{advance_label}\nselected: {}\nsnap: {}\nmultiplier: {}x\ntext_edit: {}",
                    current_similarity,
                    state.manual_adjustment_kind.label(),
                    if state.snap_to_grid { "on" } else { "off" },
                    state.current_manual_multiplier(),
                    if state.text_edit_mode { "on" } else { "off" },
                ));
                continue;
            }

            if is_details {
                if state.show_detailed_status {
                    let best_similarity = state
                        .best_score
                        .as_ref()
                        .map(|score| score.fitness_score)
                        .unwrap_or(0.0);
                    *text = Text::new(format!(
                        "mode: {:?}\nphase: {:?}\nauto_search: {}\ncandidate: {}/{}\ntarget: {:.4}\nbest: {:.4}\nstep: {}\nselected_value: {}\nvertical_mask: {} @ {:.3}\nhorizontal_mask: {} @ {:.3}",
                        state.display_mode,
                        state.phase,
                        state.auto_search,
                        state
                            .current_candidate_index
                            .map(|index| index + 1)
                            .unwrap_or(0),
                        state.total_candidates.max(1),
                        state.target_similarity,
                        best_similarity,
                        state.current_manual_step_label(&task.0),
                        state.current_selected_value_label(),
                        mask_side_label(mask_state.vertical_side, MaskGuideKind::Vertical),
                        mask_state.vertical_split_x,
                        mask_side_label(mask_state.horizontal_side, MaskGuideKind::Horizontal),
                        mask_state.horizontal_split_y,
                    ));
                } else {
                    *text = Text::new("");
                }
                continue;
            }

            if is_controls {
                *text = Text::new(
                    "Tab: display mode  Space: evolve/cancel search  N: next stage/text  C: next property  G: snap on/off\nEnter/click content: edit text  M: step multiplier  I: toggle details\nArrows: adjust selected  LMB: drag guide  RMB: flip mask side  R: use best  S: save current",
                );
                continue;
            }

            if is_snap {
                *text = Text::new(format!(
                    "Snap: {} (G)",
                    if state.snap_to_grid { "On" } else { "Off" }
                ));
                continue;
            }

            if let Some(row_text) = row_text {
                let prefix = if row_text.field == state.manual_adjustment_kind {
                    ">"
                } else {
                    " "
                };
                *text = Text::new(format!(
                    "{prefix} {:<14} {}",
                    row_text.field.label(),
                    inspector_field_value_label(row_text.field, &state, &task.0),
                ));
            }
        }
    }
}

impl ReconstructionState {
    fn current_manual_multiplier(&self) -> i32 {
        MANUAL_STEP_MULTIPLIERS[self.manual_step_multiplier_index] as i32
    }

    fn current_manual_multiplier_value(&self) -> f32 {
        MANUAL_STEP_MULTIPLIERS[self.manual_step_multiplier_index]
    }

    fn current_manual_step(&self, task: &TaskConfig) -> f32 {
        self.manual_adjustment_kind.base_step(task) * self.current_manual_multiplier_value()
    }

    fn current_manual_step_label(&self, task: &TaskConfig) -> String {
        match self.manual_adjustment_kind {
            ManualAdjustmentKind::Content => "text".to_string(),
            ManualAdjustmentKind::Font
            | ManualAdjustmentKind::Align
            | ManualAdjustmentKind::Anchor => "discrete".to_string(),
            ManualAdjustmentKind::Translation => format!(
                "x={:.3}, y={:.3}",
                task.manual_steps.translation_x * self.current_manual_multiplier_value(),
                task.manual_steps.translation_y * self.current_manual_multiplier_value(),
            ),
            _ => format!("{:.3}", self.current_manual_step(task)),
        }
    }

    fn current_selected_value_label(&self) -> String {
        match self.manual_adjustment_kind {
            ManualAdjustmentKind::Content => {
                if self.text_edit_mode {
                    format!("\"{}\" [editing]", self.current_text)
                } else {
                    format!("\"{}\"", self.current_text)
                }
            }
            ManualAdjustmentKind::Font => format!("{:?}", self.current_parameters.font),
            ManualAdjustmentKind::Align => format!("{:?}", self.current_parameters.align),
            ManualAdjustmentKind::Anchor => format!("{:?}", self.current_parameters.anchor),
            ManualAdjustmentKind::Translation => format!(
                "x={:.3}, y={:.3}",
                self.current_parameters.translation_x, self.current_parameters.translation_y,
            ),
            _ => format!(
                "{:.3}",
                self.manual_adjustment_kind
                    .current_value(&self.current_parameters)
            ),
        }
    }
}

fn inspector_field_value_label(
    field: ManualAdjustmentKind,
    state: &ReconstructionState,
    task: &TaskConfig,
) -> String {
    match field {
        ManualAdjustmentKind::Content => {
            if state.text_edit_mode {
                format!("\"{}\"  [editing]", state.current_text)
            } else {
                format!("\"{}\"", state.current_text)
            }
        }
        ManualAdjustmentKind::Font => format!("{:?}", state.current_parameters.font),
        ManualAdjustmentKind::Align => optional_enum_label(
            task.property_defaults.align_uses_default,
            state.current_parameters.align,
        ),
        ManualAdjustmentKind::Anchor => optional_enum_label(
            task.property_defaults.anchor_uses_default,
            state.current_parameters.anchor,
        ),
        ManualAdjustmentKind::Translation => format!(
            "x={:.3}, y={:.3}",
            state.current_parameters.translation_x, state.current_parameters.translation_y,
        ),
        ManualAdjustmentKind::WorldScale => format!(
            "x={:.3}, y={:.3}",
            state.current_parameters.world_scale_x, state.current_parameters.world_scale_y,
        ),
        ManualAdjustmentKind::WorldScaleX => {
            format!("{:.3}", state.current_parameters.world_scale_x)
        }
        ManualAdjustmentKind::WorldScaleY => {
            format!("{:.3}", state.current_parameters.world_scale_y)
        }
        ManualAdjustmentKind::LineHeight => format!("{:.3}", state.current_parameters.line_height),
        ManualAdjustmentKind::CharSpacing => {
            format!("{:.3}", state.current_parameters.char_spacing)
        }
        ManualAdjustmentKind::WordSpacing => {
            format!("{:.3}", state.current_parameters.word_spacing)
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PreviewGeometry {
    center: Vec2,
    scale: f32,
    image_size: Vec2,
}

fn compute_preview_geometry(
    window: &Window,
    reference_images: &ReferenceImages,
) -> PreviewGeometry {
    let scale = compute_preview_scale(
        window.width(),
        window.height(),
        reference_images.width as f32,
        reference_images.height as f32,
    );
    PreviewGeometry {
        center: Vec2::new(0.0, -20.0),
        scale,
        image_size: Vec2::new(
            reference_images.width as f32,
            reference_images.height as f32,
        ),
    }
}

fn cursor_world_to_preview_local(cursor_world: Vec2, geometry: &PreviewGeometry) -> Option<Vec2> {
    if geometry.scale <= f32::EPSILON {
        return None;
    }
    let local = (cursor_world - geometry.center) / geometry.scale;
    let half_size = geometry.image_size * 0.5;
    if local.x.abs() <= half_size.x && local.y.abs() <= half_size.y {
        Some(local)
    } else {
        None
    }
}

fn pick_mask_guide(
    cursor_local: Vec2,
    geometry: &PreviewGeometry,
    mask_state: &ReferenceMaskState,
) -> Option<MaskGuideKind> {
    let vertical_x = mask_state.vertical_split_x - geometry.image_size.x * 0.5;
    let horizontal_y = geometry.image_size.y * 0.5 - mask_state.horizontal_split_y;
    let hit_radius = GUIDE_HIT_RADIUS / geometry.scale.max(0.01);

    let vertical_distance = (cursor_local.x - vertical_x).abs();
    let horizontal_distance = (cursor_local.y - horizontal_y).abs();

    let vertical_hit = vertical_distance <= hit_radius;
    let horizontal_hit = horizontal_distance <= hit_radius;

    match (vertical_hit, horizontal_hit) {
        (true, true) => {
            if vertical_distance <= horizontal_distance {
                Some(MaskGuideKind::Vertical)
            } else {
                Some(MaskGuideKind::Horizontal)
            }
        }
        (true, false) => Some(MaskGuideKind::Vertical),
        (false, true) => Some(MaskGuideKind::Horizontal),
        (false, false) => None,
    }
}

fn update_mask_split_from_cursor(
    guide: MaskGuideKind,
    cursor_local: Vec2,
    geometry: &PreviewGeometry,
    mask_state: &mut ReferenceMaskState,
) -> bool {
    match guide {
        MaskGuideKind::Vertical => {
            let next = round3(
                (cursor_local.x + geometry.image_size.x * 0.5).clamp(0.0, geometry.image_size.x),
            );
            if (next - mask_state.vertical_split_x).abs() > 0.0001 {
                mask_state.vertical_split_x = next;
                true
            } else {
                false
            }
        }
        MaskGuideKind::Horizontal => {
            let next = round3(
                (geometry.image_size.y * 0.5 - cursor_local.y).clamp(0.0, geometry.image_size.y),
            );
            if (next - mask_state.horizontal_split_y).abs() > 0.0001 {
                mask_state.horizontal_split_y = next;
                true
            } else {
                false
            }
        }
    }
}

fn apply_vertical_mask_visual(
    sprite: &mut Sprite,
    transform: &mut Transform,
    geometry: &PreviewGeometry,
    mask_state: &ReferenceMaskState,
    visible: bool,
) {
    let width = match mask_state.vertical_side {
        MaskOcclusionSide::Negative => mask_state.vertical_split_x,
        MaskOcclusionSide::Positive => geometry.image_size.x - mask_state.vertical_split_x,
    }
    .max(0.0);
    let local_center_x = match mask_state.vertical_side {
        MaskOcclusionSide::Negative => -geometry.image_size.x * 0.5 + width * 0.5,
        MaskOcclusionSide::Positive => mask_state.vertical_split_x * 0.5,
    };
    let alpha = if visible && width > 0.0 { 0.92 } else { 0.0 };

    sprite.custom_size = Some(Vec2::new(width.max(1.0), geometry.image_size.y));
    sprite.color = Color::srgba(0.0, 0.0, 0.0, alpha);
    transform.translation = Vec3::new(
        geometry.center.x + local_center_x * geometry.scale,
        geometry.center.y,
        0.05,
    );
    transform.scale = Vec3::splat(geometry.scale);
}

fn apply_horizontal_mask_visual(
    sprite: &mut Sprite,
    transform: &mut Transform,
    geometry: &PreviewGeometry,
    mask_state: &ReferenceMaskState,
    visible: bool,
) {
    let height = match mask_state.horizontal_side {
        MaskOcclusionSide::Negative => mask_state.horizontal_split_y,
        MaskOcclusionSide::Positive => geometry.image_size.y - mask_state.horizontal_split_y,
    }
    .max(0.0);
    let local_center_y = match mask_state.horizontal_side {
        MaskOcclusionSide::Negative => geometry.image_size.y * 0.5 - height * 0.5,
        MaskOcclusionSide::Positive => -mask_state.horizontal_split_y * 0.5,
    };
    let alpha = if visible && height > 0.0 { 0.92 } else { 0.0 };

    sprite.custom_size = Some(Vec2::new(geometry.image_size.x, height.max(1.0)));
    sprite.color = Color::srgba(0.0, 0.0, 0.0, alpha);
    transform.translation = Vec3::new(
        geometry.center.x,
        geometry.center.y + local_center_y * geometry.scale,
        0.06,
    );
    transform.scale = Vec3::splat(geometry.scale);
}

fn apply_vertical_guide_visual(
    sprite: &mut Sprite,
    transform: &mut Transform,
    geometry: &PreviewGeometry,
    mask_state: &ReferenceMaskState,
) {
    let local_x = mask_state.vertical_split_x - geometry.image_size.x * 0.5;
    sprite.custom_size = Some(Vec2::new(GUIDE_THICKNESS, geometry.image_size.y));
    sprite.color = match mask_state.vertical_side {
        MaskOcclusionSide::Negative => Color::srgb(1.0, 0.82, 0.18),
        MaskOcclusionSide::Positive => Color::srgb(1.0, 0.45, 0.18),
    };
    transform.translation = Vec3::new(
        geometry.center.x + local_x * geometry.scale,
        geometry.center.y,
        0.25,
    );
    transform.scale = Vec3::splat(geometry.scale);
}

fn apply_horizontal_guide_visual(
    sprite: &mut Sprite,
    transform: &mut Transform,
    geometry: &PreviewGeometry,
    mask_state: &ReferenceMaskState,
) {
    let local_y = geometry.image_size.y * 0.5 - mask_state.horizontal_split_y;
    sprite.custom_size = Some(Vec2::new(geometry.image_size.x, GUIDE_THICKNESS));
    sprite.color = match mask_state.horizontal_side {
        MaskOcclusionSide::Negative => Color::srgb(0.22, 0.86, 1.0),
        MaskOcclusionSide::Positive => Color::srgb(0.12, 0.64, 0.88),
    };
    transform.translation = Vec3::new(
        geometry.center.x,
        geometry.center.y + local_y * geometry.scale,
        0.26,
    );
    transform.scale = Vec3::splat(geometry.scale);
}

fn apply_reference_occlusion_mask(
    base_image: &image::RgbaImage,
    mask_state: &ReferenceMaskState,
) -> image::RgbaImage {
    let mut masked = base_image.clone();
    let width = masked.width();
    let height = masked.height();

    let vertical_split = mask_state.vertical_split_x.clamp(0.0, width as f32) as u32;
    match mask_state.vertical_side {
        MaskOcclusionSide::Negative => {
            for y in 0..height {
                for x in 0..vertical_split {
                    masked.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
                }
            }
        }
        MaskOcclusionSide::Positive => {
            for y in 0..height {
                for x in vertical_split..width {
                    masked.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
                }
            }
        }
    }

    let horizontal_split = mask_state.horizontal_split_y.clamp(0.0, height as f32) as u32;
    match mask_state.horizontal_side {
        MaskOcclusionSide::Negative => {
            for y in 0..horizontal_split {
                for x in 0..width {
                    masked.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
                }
            }
        }
        MaskOcclusionSide::Positive => {
            for y in horizontal_split..height {
                for x in 0..width {
                    masked.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
                }
            }
        }
    }

    masked
}

fn validate_bbox(reference_image: &image::RgbaImage, bbox: CropRect) -> Result<()> {
    if bbox.x + bbox.width > reference_image.width()
        || bbox.y + bbox.height > reference_image.height()
    {
        anyhow::bail!(
            "bbox ({}, {}, {}, {}) exceeds reference image bounds {}x{}",
            bbox.x,
            bbox.y,
            bbox.width,
            bbox.height,
            reference_image.width(),
            reference_image.height(),
        );
    }
    Ok(())
}

fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}

fn compute_preview_scale(
    window_width: f32,
    window_height: f32,
    image_width: f32,
    image_height: f32,
) -> f32 {
    let max_width = window_width * 0.9;
    let max_height = window_height * 0.72;
    (max_width / image_width)
        .min(max_height / image_height)
        .max(0.1)
}

fn apply_bbox_mask(image: &image::RgbaImage, bbox: Option<CropRect>) -> image::RgbaImage {
    let Some(bbox) = bbox else {
        return image.clone();
    };

    let mut masked = image::RgbaImage::new(image.width(), image.height());
    for y in bbox.y..bbox.y + bbox.height {
        for x in bbox.x..bbox.x + bbox.width {
            masked.put_pixel(x, y, *image.get_pixel(x, y));
        }
    }
    masked
}

fn load_reference_image(path: &std::path::Path) -> Result<image::RgbaImage> {
    let reference_image = image::open(path)
        .with_context(|| format!("failed to open reference image: {}", path.display()))?
        .to_rgba8();
    Ok(flatten_reference_alpha_against_black(&reference_image))
}

fn flatten_reference_alpha_against_black(image: &image::RgbaImage) -> image::RgbaImage {
    let mut flattened = image::RgbaImage::new(image.width(), image.height());
    for (x, y, pixel) in image.enumerate_pixels() {
        let alpha = pixel[3] as u16;
        let red = ((pixel[0] as u16 * alpha) / 255) as u8;
        let green = ((pixel[1] as u16 * alpha) / 255) as u8;
        let blue = ((pixel[2] as u16 * alpha) / 255) as u8;
        flattened.put_pixel(x, y, image::Rgba([red, green, blue, 255]));
    }
    flattened
}

fn compute_candidate_fitness(
    comparison: bevy_alight_motion::image_comparison::ImageComparisonResult,
) -> f32 {
    comparison.content_mask_f1 * 0.45
        + comparison.content_bbox_iou * 0.25
        + comparison.content_size_similarity * 0.15
        + comparison.content_center_similarity * 0.10
        + comparison.content_similarity * 0.05
}

fn persist_best_candidate(
    task: &TaskConfig,
    score: &ScoredCandidate,
    render_image: &image::RgbaImage,
    diff_image: &image::RgbaImage,
) {
    ensure_parent_directory(&task.best_view_absolute_path);
    ensure_parent_directory(&task.best_summary_path);
    ensure_parent_directory(&task.best_render_path);
    ensure_parent_directory(&task.best_diff_path);
    let schema_layout = build_export_view_layout(
        &score.text,
        &score.parameters,
        task.property_defaults,
        task.field_override_policy,
        task.host_view.as_ref(),
    )
    .expect("best schema should build from task configuration");
    let pretty_config = ron::ser::PrettyConfig::new();
    let ron_text = ron::ser::to_string_pretty(&schema_layout, pretty_config)
        .expect("best schema should serialize to RON");
    fs::write(&task.best_view_absolute_path, ron_text).expect("failed to write best view file");
    fs::write(
        &task.best_summary_path,
        serde_json::to_vec_pretty(&to_persisted_candidate(task, score))
            .expect("best score should serialize to JSON"),
    )
    .expect("failed to write best summary");
    render_image
        .save(&task.best_render_path)
        .expect("failed to save best render image");
    diff_image
        .save(&task.best_diff_path)
        .expect("failed to save best diff image");
}

fn persist_session_final_view(
    final_view_absolute_path: &Path,
    task: &TaskConfig,
    parameters: &ConcreteTextParameters,
) {
    let Some(host_view) = task.host_view.as_ref() else {
        return;
    };

    ensure_parent_directory(final_view_absolute_path);
    let mut layout = if final_view_absolute_path.exists() {
        let raw = fs::read_to_string(final_view_absolute_path)
            .expect("session final view should be readable");
        ron::from_str(&raw).expect("session final view should deserialize")
    } else {
        host_view.layout.clone()
    };
    apply_export_text_patch(
        &mut layout,
        parameters,
        task.field_override_policy,
        host_view,
    )
    .expect("session final view should accept host text patch");
    let ron_text = ron::ser::to_string_pretty(&layout, ron::ser::PrettyConfig::new())
        .expect("session final view should serialize to RON");
    fs::write(final_view_absolute_path, ron_text).expect("failed to write session final view");
}

fn persist_current_snapshot(
    task: &TaskConfig,
    score: &ScoredCandidate,
    render_image: &image::RgbaImage,
    diff_image: &image::RgbaImage,
) {
    ensure_parent_directory(&task.current_summary_path);
    ensure_parent_directory(&task.current_render_path);
    ensure_parent_directory(&task.current_diff_path);
    fs::write(
        &task.current_summary_path,
        serde_json::to_vec_pretty(&to_persisted_candidate(task, score))
            .expect("current score should serialize to JSON"),
    )
    .expect("failed to write current summary");
    render_image
        .save(&task.current_render_path)
        .expect("failed to save current render image");
    diff_image
        .save(&task.current_diff_path)
        .expect("failed to save current diff image");
}

fn ensure_parent_directory(path: &std::path::Path) {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir).expect("failed to create output directory");
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct PersistedScoredCandidate {
    candidate_index: Option<usize>,
    total_candidates: usize,
    text: String,
    parameters: PersistedTextParameters,
    fitness_score: f32,
    global_similarity: f32,
    content_similarity: f32,
    pixel_match_rate: f32,
    content_mask_f1: f32,
    content_bbox_iou: f32,
    content_size_similarity: f32,
    content_center_similarity: f32,
    differing_pixels: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct PersistedTextParameters {
    font: String,
    align: String,
    anchor: String,
    translation_x: f32,
    translation_y: f32,
    world_scale_x: f32,
    world_scale_y: f32,
    line_height: f32,
    char_spacing: f32,
    word_spacing: f32,
}

fn to_persisted_candidate(task: &TaskConfig, score: &ScoredCandidate) -> PersistedScoredCandidate {
    PersistedScoredCandidate {
        candidate_index: score.candidate_index,
        total_candidates: score.total_candidates,
        text: score.text.clone(),
        parameters: PersistedTextParameters {
            font: format!("{:?}", score.parameters.font),
            align: optional_enum_label(
                task.property_defaults.align_uses_default,
                score.parameters.align,
            ),
            anchor: optional_enum_label(
                task.property_defaults.anchor_uses_default,
                score.parameters.anchor,
            ),
            translation_x: score.parameters.translation_x,
            translation_y: score.parameters.translation_y,
            world_scale_x: score.parameters.world_scale_x,
            world_scale_y: score.parameters.world_scale_y,
            line_height: score.parameters.line_height,
            char_spacing: score.parameters.char_spacing,
            word_spacing: score.parameters.word_spacing,
        },
        fitness_score: score.fitness_score,
        global_similarity: score.global_similarity,
        content_similarity: score.content_similarity,
        pixel_match_rate: score.pixel_match_rate,
        content_mask_f1: score.content_mask_f1,
        content_bbox_iou: score.content_bbox_iou,
        content_size_similarity: score.content_size_similarity,
        content_center_similarity: score.content_center_similarity,
        differing_pixels: score.differing_pixels,
    }
}

fn optional_enum_label<T>(uses_default: bool, value: T) -> String
where
    T: std::fmt::Debug,
{
    if uses_default {
        "default".to_string()
    } else {
        format!("{value:?}")
    }
}

fn mask_side_label(side: MaskOcclusionSide, guide: MaskGuideKind) -> &'static str {
    match (guide, side) {
        (MaskGuideKind::Vertical, MaskOcclusionSide::Negative) => "left",
        (MaskGuideKind::Vertical, MaskOcclusionSide::Positive) => "right",
        (MaskGuideKind::Horizontal, MaskOcclusionSide::Negative) => "top",
        (MaskGuideKind::Horizontal, MaskOcclusionSide::Positive) => "bottom",
    }
}

fn round3(value: f32) -> f32 {
    (value * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn flatten_reference_alpha_against_black_turns_transparent_into_black() {
        let mut image = image::RgbaImage::new(2, 1);
        image.put_pixel(0, 0, image::Rgba([255, 255, 255, 0]));
        image.put_pixel(1, 0, image::Rgba([200, 100, 50, 128]));

        let flattened = flatten_reference_alpha_against_black(&image);

        assert_eq!(flattened.get_pixel(0, 0), &image::Rgba([0, 0, 0, 255]));
        assert_eq!(flattened.get_pixel(1, 0), &image::Rgba([100, 50, 25, 255]));
    }

    #[test]
    fn load_saved_resume_state_prefers_best_over_current() {
        let workspace_root = create_test_workspace("resume_prefers_best");
        let stage_dir = workspace_root.join("generated/view_text_reconstruction/demo_case");
        let config_path = stage_dir.join("stage.ron");
        fs::create_dir_all(&stage_dir).expect("stage dir should be created");
        write_test_reference_image(&stage_dir.join("reference.png"));
        fs::write(
            &config_path,
            r#"
(
    stage_kind: Single,
    image: "reference.png",
    text: "CHARA",
    target_similarity: 0.95,
    properties: (
        translation_x: SearchRange((-50.0, 50.0, 0.25)),
        translation_y: SearchRange((-50.0, 50.0, 0.25)),
        world_scale_x: SearchRange((1.0, 30.0, 0.25)),
        world_scale_y: SearchRange((1.0, 30.0, 0.25)),
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
        let search_plan = task.search_plan.clone();

        write_persisted_candidate(
            &task.current_summary_path,
            "CURRENT",
            ConcreteTextParameters {
                font: souprune_schema::view::ViewFontDef::DeterminationSans,
                align: souprune_schema::view::TextAlignDef::Left,
                anchor: souprune_schema::view::TextAnchorDef::BottomRight,
                translation_x: -11.0,
                translation_y: 3.0,
                world_scale_x: 12.0,
                world_scale_y: 12.0,
                line_height: 1.0,
                char_spacing: 0.0,
                word_spacing: 0.0,
            },
        );
        write_persisted_candidate(
            &task.best_summary_path,
            "BEST",
            ConcreteTextParameters {
                font: souprune_schema::view::ViewFontDef::DeterminationSans,
                align: souprune_schema::view::TextAlignDef::Left,
                anchor: souprune_schema::view::TextAnchorDef::BottomRight,
                translation_x: 7.25,
                translation_y: 9.5,
                world_scale_x: 19.0,
                world_scale_y: 19.0,
                line_height: 1.0,
                char_spacing: 0.0,
                word_spacing: 0.0,
            },
        );

        let restored =
            load_saved_resume_state(&task, &search_plan).expect("saved state should restore");

        assert_eq!(restored.text, "BEST");
        assert_eq!(restored.parameters.translation_x, 7.25);
        assert_eq!(restored.parameters.translation_y, 9.5);
        assert_eq!(restored.parameters.world_scale_x, 19.0);
    }

    #[test]
    fn load_saved_best_score_reads_persisted_best_summary() {
        let workspace_root = create_test_workspace("load_saved_best_score");
        let stage_dir = workspace_root.join("generated/view_text_reconstruction/demo_case");
        let config_path = stage_dir.join("stage.ron");
        fs::create_dir_all(&stage_dir).expect("stage dir should be created");
        write_test_reference_image(&stage_dir.join("reference.png"));
        fs::write(
            &config_path,
            r#"
(
    stage_kind: Single,
    image: "reference.png",
    text: "CHARA",
    target_similarity: 0.95,
    properties: (
        translation_x: SearchRange((-50.0, 50.0, 0.25)),
        translation_y: SearchRange((-50.0, 50.0, 0.25)),
        world_scale_x: SearchRange((1.0, 30.0, 0.25)),
        world_scale_y: SearchRange((1.0, 30.0, 0.25)),
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
        let search_plan = task.search_plan.clone();

        write_persisted_candidate(
            &task.best_summary_path,
            "BEST",
            ConcreteTextParameters {
                font: souprune_schema::view::ViewFontDef::DeterminationSans,
                align: souprune_schema::view::TextAlignDef::Left,
                anchor: souprune_schema::view::TextAnchorDef::BottomRight,
                translation_x: 4.5,
                translation_y: -7.25,
                world_scale_x: 18.0,
                world_scale_y: 18.0,
                line_height: 1.0,
                char_spacing: 0.0,
                word_spacing: 0.0,
            },
        );

        let best = load_saved_best_score(&task, &search_plan).expect("best score should restore");

        assert_eq!(best.text, "BEST");
        assert_eq!(best.fitness_score, 1.0);
        assert_eq!(best.parameters.translation_x, 4.5);
        assert_eq!(best.parameters.translation_y, -7.25);
        assert_eq!(best.parameters.world_scale_x, 18.0);
    }

    #[test]
    fn load_saved_resume_state_reconstrains_stale_stage_two_geometry_to_inherited_seed() {
        let workspace_root = create_test_workspace("stage_two_reconstrains_restored_geometry");
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
            align: souprune_schema::view::TextAlignDef::Left,
            anchor: souprune_schema::view::TextAnchorDef::BottomRight,
            translation_x: -28.5,
            translation_y: 23.25,
            world_scale_x: 13.0,
            world_scale_y: 13.0,
            line_height: 1.125,
            char_spacing: 0.0,
            word_spacing: 0.0,
        };

        let task = TaskConfig::load_stage_ron(&config_path, &workspace_root, Some(&inherited_seed))
            .expect("stage config should load");
        let search_plan = task.search_plan.clone();

        write_persisted_candidate(
            &task.current_summary_path,
            "CHARA",
            ConcreteTextParameters {
                translation_x: -11.0,
                translation_y: 6.5,
                world_scale_x: 22.0,
                world_scale_y: 22.0,
                char_spacing: 1.5,
                word_spacing: -1.0,
                ..inherited_seed.clone()
            },
        );

        let restored =
            load_saved_resume_state(&task, &search_plan).expect("saved state should restore");

        assert_eq!(
            restored.parameters.translation_x,
            inherited_seed.translation_x
        );
        assert_eq!(
            restored.parameters.translation_y,
            inherited_seed.translation_y
        );
        assert_eq!(
            restored.parameters.world_scale_x,
            inherited_seed.world_scale_x
        );
        assert_eq!(
            restored.parameters.world_scale_y,
            inherited_seed.world_scale_y
        );
        assert_eq!(restored.parameters.char_spacing, 1.5);
        assert_eq!(restored.parameters.word_spacing, -1.0);
    }

    fn create_test_workspace(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();
        let workspace_root = std::env::temp_dir().join(format!(
            "souprune_view_text_reconstruction_runtime_{label}_{unique}"
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

    fn write_persisted_candidate(path: &Path, text: &str, parameters: ConcreteTextParameters) {
        ensure_parent_directory(path);
        let persisted = PersistedScoredCandidate {
            candidate_index: Some(0),
            total_candidates: 1,
            text: text.to_string(),
            parameters: PersistedTextParameters {
                font: format!("{:?}", parameters.font),
                align: format!("{:?}", parameters.align),
                anchor: format!("{:?}", parameters.anchor),
                translation_x: parameters.translation_x,
                translation_y: parameters.translation_y,
                world_scale_x: parameters.world_scale_x,
                world_scale_y: parameters.world_scale_y,
                line_height: parameters.line_height,
                char_spacing: parameters.char_spacing,
                word_spacing: parameters.word_spacing,
            },
            fitness_score: 1.0,
            global_similarity: 1.0,
            content_similarity: 1.0,
            pixel_match_rate: 1.0,
            content_mask_f1: 1.0,
            content_bbox_iou: 1.0,
            content_size_similarity: 1.0,
            content_center_similarity: 1.0,
            differing_pixels: 0,
        };
        fs::write(
            path,
            serde_json::to_string(&persisted).expect("persisted candidate should serialize"),
        )
        .expect("persisted candidate should be written");
    }
}

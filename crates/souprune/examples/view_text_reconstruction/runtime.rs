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
use crate::config::{CropRect, TaskConfig};
use crate::search::{CandidateSearchPlan, ConcreteTextParameters, build_view_layout};
use anyhow::{Context, Result};
use bevy::app::AppExit;
use bevy::asset::RenderAssetUsages;
use bevy::asset::io::file::FileAssetReader;
use bevy::asset::io::{AssetSourceBuilder, AssetSourceId};
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::{PrimaryWindow, WindowPlugin};
use image::DynamicImage;
use serde::Serialize;
use souprune::ViewLayoutAsset;
use souprune::config::SoupruneConfig;
use souprune::core::camera::MainGameCamera;
use souprune::core::game_action::GameActionDef;
use souprune::core::view::{CoreViewPlugin, DespawnViewRequest, SpawnViewRequest};
use souprune::extra::multi_source::MultiSourceAssetReader;
use std::fs;

const PREVIEW_LAYER: RenderLayers = RenderLayers::layer(2);
const PREVIEW_TEXT_MARGIN: f32 = 18.0;
const MANUAL_STEP_MULTIPLIERS: [f32; 7] = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0];

pub fn configure_app(
    app: &mut App,
    souprune_config: SoupruneConfig,
    task: TaskConfig,
) -> Result<()> {
    let current_project_name = souprune_config.project.mod_name.clone();
    let workspace_root = task.workspace_root.clone();
    let reference_image = image::open(&task.image_path)
        .with_context(|| {
            format!(
                "failed to open reference image: {}",
                task.image_path.display()
            )
        })?
        .to_rgba8();

    if let Some(bbox) = task.bbox {
        validate_bbox(&reference_image, bbox)?;
    }

    let search_plan = task.search_plan.clone();
    let initial_current = search_plan.seed_parameters();

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
        .insert_resource(RenderTargetImage::default())
        .insert_resource(ReferenceImages {
            original: reference_image.clone(),
            compare_masked: apply_bbox_mask(&reference_image, task.bbox),
            width: reference_image.width(),
            height: reference_image.height(),
            reference_handle: Handle::default(),
            diff_handle: Handle::default(),
        })
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
            current_score: None,
            best_score: None,
            latest_render_image: None,
            latest_diff_image: None,
            persist_current_after_evaluation: false,
            capture_after_apply: false,
            manual_adjustment_kind: ManualAdjustmentKind::initial_for_task(&task),
            manual_step_multiplier_index: 0,
            show_detailed_status: false,
            pending_apply: true,
        })
        .insert_resource(CurrentViewAssetHandle::default())
        .add_systems(Startup, setup_runtime)
        .add_systems(
            Update,
            (
                handle_keyboard_input,
                apply_pending_candidate,
                drive_capture_state,
                update_preview_scene,
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
struct ReconstructionState {
    phase: EvaluationPhase,
    display_mode: DisplayMode,
    auto_search: bool,
    total_candidates: usize,
    target_similarity: f32,
    current_candidate_index: Option<usize>,
    current_parameters: ConcreteTextParameters,
    current_score: Option<ScoredCandidate>,
    best_score: Option<ScoredCandidate>,
    latest_render_image: Option<image::RgbaImage>,
    latest_diff_image: Option<image::RgbaImage>,
    persist_current_after_evaluation: bool,
    capture_after_apply: bool,
    manual_adjustment_kind: ManualAdjustmentKind,
    manual_step_multiplier_index: usize,
    show_detailed_status: bool,
    pending_apply: bool,
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
enum ManualAdjustmentKind {
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
        Self::Translation
    }

    fn next_for_task(self, task: &TaskConfig) -> Self {
        let kinds = self.cycle_for_task(task);
        let current_index = kinds.iter().position(|kind| *kind == self).unwrap_or(0);
        kinds[(current_index + 1) % kinds.len()]
    }

    fn cycle_for_task(self, task: &TaskConfig) -> &'static [ManualAdjustmentKind] {
        if task.world_scale_bound {
            &[
                Self::Translation,
                Self::WorldScale,
                Self::LineHeight,
                Self::CharSpacing,
                Self::WordSpacing,
            ]
        } else {
            &[
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
            Self::Translation => parameters.translation_x,
            Self::WorldScale | Self::WorldScaleX => parameters.world_scale_x,
            Self::WorldScaleY => parameters.world_scale_y,
            Self::LineHeight => parameters.line_height,
            Self::CharSpacing => parameters.char_spacing,
            Self::WordSpacing => parameters.word_spacing,
        }
    }

    fn apply_delta(self, parameters: &mut ConcreteTextParameters, task: &TaskConfig, delta: f32) {
        match self {
            Self::Translation => {
                parameters.translation_x = round3(parameters.translation_x + delta)
            }
            Self::WorldScale => {
                parameters.world_scale_x = round3(parameters.world_scale_x + delta);
                parameters.world_scale_y = round3(parameters.world_scale_y + delta);
            }
            Self::WorldScaleX => {
                parameters.world_scale_x = round3(parameters.world_scale_x + delta)
            }
            Self::WorldScaleY => {
                parameters.world_scale_y = round3(parameters.world_scale_y + delta)
            }
            Self::LineHeight => parameters.line_height = round3(parameters.line_height + delta),
            Self::CharSpacing => parameters.char_spacing = round3(parameters.char_spacing + delta),
            Self::WordSpacing => parameters.word_spacing = round3(parameters.word_spacing + delta),
        }

        if task.world_scale_bound {
            parameters.world_scale_y = parameters.world_scale_x;
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ScoredCandidate {
    candidate_index: Option<usize>,
    total_candidates: usize,
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

#[derive(Component)]
struct PreviewReferenceSprite;

#[derive(Component)]
struct PreviewRenderSprite;

#[derive(Component)]
struct PreviewDiffSprite;

#[derive(Component)]
struct PreviewStatusPanel;

#[derive(Component)]
struct PreviewStatusText;

#[derive(Component)]
struct PreviewControlsText;

fn enter_running_state(mut next_state: ResMut<NextState<souprune::app_state::AppState>>) {
    next_state.set(souprune::app_state::AppState::Running);
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
) {
    if let Some(parent_dir) = task.0.current_view_absolute_path.parent() {
        fs::create_dir_all(parent_dir).expect("failed to create generated view directory");
    }
    let initial_ron = serialize_candidate_view_ron(&task.0, &state.current_parameters);
    fs::write(&task.0.current_view_absolute_path, initial_ron)
        .expect("failed to write initial generated view RON file");

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
        asset_server.load::<ViewLayoutAsset>(task.0.current_view_relative_path.clone());

    commands.spawn((
        Camera2d,
        Camera {
            clear_color: bevy::camera::ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
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
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(PREVIEW_TEXT_MARGIN),
                right: px(PREVIEW_TEXT_MARGIN),
                width: px(280.0),
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

fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ReconstructionState>,
    mut search: ResMut<SearchController>,
    task: Res<TaskResource>,
) {
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

    if keyboard.just_pressed(KeyCode::KeyI) {
        state.show_detailed_status = !state.show_detailed_status;
    }

    if keyboard.just_pressed(KeyCode::Space) && state.auto_search {
        state.auto_search = false;
        return;
    }

    if !matches!(state.phase, EvaluationPhase::Ready) {
        return;
    }

    if keyboard.just_pressed(KeyCode::Space) {
        let (candidate_index, parameters) = search.plan.restart();
        state.auto_search = search.total_candidates > 1;
        state.total_candidates = search.total_candidates;
        state.target_similarity = task.0.target_similarity;
        state.current_candidate_index = Some(candidate_index);
        state.current_parameters = parameters;
        state.current_score = None;
        state.capture_after_apply = true;
        state.pending_apply = true;
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyR)
        && let Some(best_score) = state.best_score.clone()
    {
        state.auto_search = false;
        state.current_candidate_index = best_score.candidate_index;
        state.current_parameters = best_score.parameters.clone();
        state.current_score = Some(best_score.clone());
        state.capture_after_apply = false;
        state.pending_apply = true;
        return;
    }

    let changed = match state.manual_adjustment_kind {
        ManualAdjustmentKind::Translation => {
            let mut changed = false;
            let multiplier = state.current_manual_multiplier_value();
            let x_step = task.0.manual_steps.translation_x * multiplier;
            let y_step = task.0.manual_steps.translation_y * multiplier;

            if keyboard.just_pressed(KeyCode::ArrowLeft) {
                state.current_parameters.translation_x =
                    round3(state.current_parameters.translation_x - x_step);
                changed = true;
            }
            if keyboard.just_pressed(KeyCode::ArrowRight) {
                state.current_parameters.translation_x =
                    round3(state.current_parameters.translation_x + x_step);
                changed = true;
            }
            if keyboard.just_pressed(KeyCode::ArrowUp) {
                state.current_parameters.translation_y =
                    round3(state.current_parameters.translation_y + y_step);
                changed = true;
            }
            if keyboard.just_pressed(KeyCode::ArrowDown) {
                state.current_parameters.translation_y =
                    round3(state.current_parameters.translation_y - y_step);
                changed = true;
            }
            changed
        }
        _ => {
            let mut direction = 0i32;
            if keyboard.just_pressed(KeyCode::ArrowLeft)
                || keyboard.just_pressed(KeyCode::ArrowDown)
            {
                direction -= 1;
            }
            if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::ArrowUp)
            {
                direction += 1;
            }

            if direction != 0 {
                let signed_step = state.current_manual_step(&task.0) * direction as f32;
                state.manual_adjustment_kind.apply_delta(
                    &mut state.current_parameters,
                    &task.0,
                    signed_step,
                );
                true
            } else {
                false
            }
        }
    };

    if changed {
        search
            .plan
            .constrain_parameters(&mut state.current_parameters);
        state.auto_search = false;
        state.current_candidate_index = None;
        state.current_score = None;
        state.persist_current_after_evaluation = true;
        state.capture_after_apply = true;
        state.pending_apply = true;
    }
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

    search
        .plan
        .constrain_parameters(&mut state.current_parameters);

    let ron_text = serialize_candidate_view_ron(&task.0, &state.current_parameters);
    fs::write(&task.0.current_view_absolute_path, ron_text)
        .expect("failed to write current view RON file");

    let runtime_layout: ViewLayoutAsset = ron::from_str(
        &fs::read_to_string(&task.0.current_view_absolute_path)
            .expect("generated view RON file should be readable immediately after writing"),
    )
    .expect("generated view RON should deserialize into runtime asset");
    view_layouts
        .insert(current_view_handle.handle.id(), runtime_layout)
        .expect("current view handle should accept asset replacement");

    despawn_writer.write(DespawnViewRequest {
        path: Some(task.0.current_view_relative_path.clone()),
    });
    spawn_writer.write(SpawnViewRequest {
        path: task.0.current_view_relative_path.clone(),
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

fn serialize_candidate_view_ron(task: &TaskConfig, parameters: &ConcreteTextParameters) -> String {
    let schema_layout = build_view_layout(&task.text, parameters, task.property_defaults);
    let pretty_config = ron::ser::PrettyConfig::new();
    ron::ser::to_string_pretty(&schema_layout, pretty_config)
        .expect("generated schema should serialize to RON")
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
    reference_images: Res<ReferenceImages>,
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
    let (comparison, diff_image) = bevy_alight_motion::image_comparison::compare_images(
        &masked_screenshot,
        &reference_images.compare_masked,
    );

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
    }

    let target_reached = scored_candidate.fitness_score >= state.target_similarity;
    let mut search_exhausted = false;

    state.phase = EvaluationPhase::Ready;
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
        Query<&mut Node, With<PreviewStatusPanel>>,
        Query<&mut Text, With<PreviewStatusText>>,
        Query<&mut Text, With<PreviewControlsText>>,
    )>,
    task: Res<TaskResource>,
    reference_images: Res<ReferenceImages>,
    state: Res<ReconstructionState>,
) {
    let Ok(window) = primary_window.single() else {
        return;
    };

    let fit_scale = compute_preview_scale(
        window.width(),
        window.height(),
        reference_images.width as f32,
        reference_images.height as f32,
    );
    let preview_center = Vec3::new(0.0, -20.0, 0.0);
    let preview_scale = Vec3::splat(fit_scale);
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

    let current_similarity = state
        .current_score
        .as_ref()
        .map(|score| score.fitness_score)
        .unwrap_or(0.0);
    let mut status_value = format!(
        "adjust: {}\nmultiplier: {}x\nstep: {}\nvalue: {}\nsimilarity: {:.4}",
        state.manual_adjustment_kind.label(),
        state.current_manual_multiplier(),
        state.current_manual_step_label(&task.0),
        state.current_selected_value_label(),
        current_similarity,
    );
    if state.show_detailed_status {
        let best_similarity = state
            .best_score
            .as_ref()
            .map(|score| score.fitness_score)
            .unwrap_or(0.0);
        let detail_block = format!(
            "\n\nmode: {:?}\nphase: {:?}\nauto_search: {}\ncandidate: {}/{}\ntarget: {:.4}\nbest: {:.4}\nfont: {:?}\nalign: {}\nanchor: {}\npos: ({:.3}, {:.3})\nscale: ({:.3}, {:.3})\nline: {:.3}\nchar: {:.3}\nword: {:.3}",
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
            state.current_parameters.font,
            optional_enum_label(
                task.0.property_defaults.align_uses_default,
                state.current_parameters.align,
            ),
            optional_enum_label(
                task.0.property_defaults.anchor_uses_default,
                state.current_parameters.anchor,
            ),
            state.current_parameters.translation_x,
            state.current_parameters.translation_y,
            state.current_parameters.world_scale_x,
            state.current_parameters.world_scale_y,
            state.current_parameters.line_height,
            state.current_parameters.char_spacing,
            state.current_parameters.word_spacing,
        );
        status_value.push_str(&detail_block);
    }
    let controls_value = format!(
        "Tab: display mode  Space: evolve/cancel search  C: cycle tweak target  M: cycle step multiplier  I: toggle details\nArrows: adjust selected value  Left/Down = -step  Right/Up = +step  R: use best  S: save current"
    );
    {
        let mut panel_query = preview_nodes.p3();
        let Ok(mut panel) = panel_query.single_mut() else {
            return;
        };
        panel.width = if state.show_detailed_status {
            px(420.0)
        } else {
            px(280.0)
        };
    }
    {
        let mut status_query = preview_nodes.p4();
        let Ok(mut text) = status_query.single_mut() else {
            return;
        };
        *text = Text::new(status_value);
    }
    {
        let mut controls_query = preview_nodes.p5();
        let Ok(mut text) = controls_query.single_mut() else {
            return;
        };
        *text = Text::new(controls_value);
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
    let schema_layout = build_view_layout(&task.text, &score.parameters, task.property_defaults);
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

fn persist_current_snapshot(
    task: &TaskConfig,
    score: &ScoredCandidate,
    render_image: &image::RgbaImage,
    diff_image: &image::RgbaImage,
) {
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

#[derive(Debug, Serialize)]
struct PersistedScoredCandidate {
    candidate_index: Option<usize>,
    total_candidates: usize,
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

#[derive(Debug, Serialize)]
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

fn round3(value: f32) -> f32 {
    (value * 1000.0).round() / 1000.0
}

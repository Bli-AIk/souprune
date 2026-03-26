use crate::config::{CropRect, TaskConfig};
use crate::search::{CandidateSearchPlan, ConcreteTextParameters, build_view_layout};
use anyhow::{Context, Result};
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

    let mut search_plan = task.search_plan.clone();
    let (initial_candidate_index, initial_current) = search_plan.restart();

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
            auto_search: task.search_plan.total_candidates() > 1,
            total_candidates: task.search_plan.total_candidates(),
            target_similarity: task.target_similarity,
            current_candidate_index: Some(initial_candidate_index),
            current_parameters: initial_current,
            current_score: None,
            best_score: None,
            latest_diff_image: None,
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
    latest_diff_image: Option<image::RgbaImage>,
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

#[derive(Debug, Clone, Serialize)]
struct ScoredCandidate {
    candidate_index: Option<usize>,
    total_candidates: usize,
    parameters: ConcreteTextParameters,
    fitness_score: f32,
    global_similarity: f32,
    content_similarity: f32,
    pixel_match_rate: f32,
    differing_pixels: u64,
}

#[derive(Component)]
struct PreviewReferenceSprite;

#[derive(Component)]
struct PreviewRenderSprite;

#[derive(Component)]
struct PreviewDiffSprite;

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
    commands.spawn((
        Text2d::new("Initializing view reconstruction..."),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 0.0, 10.0),
        PREVIEW_LAYER,
        PreviewStatusText,
    ));
    commands.spawn((
        Text2d::new(
            "Tab: mode  Space: restart search  R: use best  Arrows: move  [ ]: scale  ; ': char  , .: word  - =: line  S: save",
        ),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.82, 0.84, 0.9)),
        Transform::from_xyz(0.0, 0.0, 10.0),
        PREVIEW_LAYER,
        PreviewControlsText,
    ));
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
        && let (Some(current_score), Some(diff_image)) = (
            state.current_score.as_ref(),
            state.latest_diff_image.as_ref(),
        )
    {
        persist_current_snapshot(&task.0, current_score, diff_image);
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
        state.pending_apply = true;
        return;
    }

    let translation_step =
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            4.0
        } else {
            1.0
        };

    let mut changed = false;
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        state.current_parameters.translation_x -= translation_step;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        state.current_parameters.translation_x += translation_step;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        state.current_parameters.translation_y += translation_step;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        state.current_parameters.translation_y -= translation_step;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        state.current_parameters.world_scale_x -= 0.25;
        state.current_parameters.world_scale_y -= 0.25;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::BracketRight) {
        state.current_parameters.world_scale_x += 0.25;
        state.current_parameters.world_scale_y += 0.25;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Semicolon) {
        state.current_parameters.char_spacing -= 0.25;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Quote) {
        state.current_parameters.char_spacing += 0.25;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Comma) {
        state.current_parameters.word_spacing -= 0.25;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Period) {
        state.current_parameters.word_spacing += 0.25;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Minus) {
        state.current_parameters.line_height -= 0.05;
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Equal) {
        state.current_parameters.line_height += 0.05;
        changed = true;
    }

    if changed {
        state.auto_search = false;
        state.current_score = None;
        state.pending_apply = true;
    }
}

fn apply_pending_candidate(
    mut view_layouts: ResMut<Assets<ViewLayoutAsset>>,
    mut despawn_writer: MessageWriter<DespawnViewRequest>,
    mut spawn_writer: MessageWriter<SpawnViewRequest>,
    mut state: ResMut<ReconstructionState>,
    task: Res<TaskResource>,
    current_view_handle: Res<CurrentViewAssetHandle>,
) {
    if !state.pending_apply || !matches!(state.phase, EvaluationPhase::Ready) {
        return;
    }

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
    state.phase = EvaluationPhase::WaitingForSettle {
        remaining_frames: task.0.settle_frames,
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
        persist_best_candidate(&task.0, &scored_candidate, &diff_image);
    }

    state.phase = EvaluationPhase::Ready;
    if state.auto_search && scored_candidate.fitness_score >= state.target_similarity {
        state.auto_search = false;
    } else if state.auto_search {
        if let Some((candidate_index, parameters)) = search.plan.next_candidate() {
            state.current_candidate_index = Some(candidate_index);
            state.current_parameters = parameters;
            state.pending_apply = true;
        } else {
            state.auto_search = false;
        }
    }
}

fn update_preview_scene(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut preview_nodes: ParamSet<(
        Query<(&mut Sprite, &mut Transform), With<PreviewReferenceSprite>>,
        Query<(&mut Sprite, &mut Transform), With<PreviewRenderSprite>>,
        Query<(&mut Sprite, &mut Transform), With<PreviewDiffSprite>>,
        Query<(&mut Text2d, &mut Transform), With<PreviewStatusText>>,
        Query<&mut Transform, With<PreviewControlsText>>,
    )>,
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

    let best_similarity = state
        .best_score
        .as_ref()
        .map(|score| score.fitness_score)
        .unwrap_or(0.0);
    let current_similarity = state
        .current_score
        .as_ref()
        .map(|score| score.fitness_score)
        .unwrap_or(0.0);
    let mode_label = match state.display_mode {
        DisplayMode::Reference => "reference",
        DisplayMode::Render => "render",
        DisplayMode::Overlay => "overlay",
        DisplayMode::Diff => "diff",
    };
    let status_translation = Vec3::new(
        -window.width() * 0.5 + 18.0,
        window.height() * 0.5 - 28.0,
        10.0,
    );
    let controls_translation = Vec3::new(
        -window.width() * 0.5 + 18.0,
        -window.height() * 0.5 + 28.0,
        10.0,
    );
    let status_value = format!(
        "mode={mode_label}  phase={:?}  auto_search={}  candidate={}/{}  current={:.4}  best={:.4}  target={:.4}\nfont={:?}  align={:?}  anchor={:?}  pos=({:.2}, {:.2})  scale=({:.2}, {:.2})  line={:.2}  char={:.2}  word={:.2}",
        state.phase,
        state.auto_search,
        state
            .current_candidate_index
            .map(|index| index + 1)
            .unwrap_or(0),
        state.total_candidates.max(1),
        current_similarity,
        best_similarity,
        state.target_similarity,
        state.current_parameters.font,
        state.current_parameters.align,
        state.current_parameters.anchor,
        state.current_parameters.translation_x,
        state.current_parameters.translation_y,
        state.current_parameters.world_scale_x,
        state.current_parameters.world_scale_y,
        state.current_parameters.line_height,
        state.current_parameters.char_spacing,
        state.current_parameters.word_spacing,
    );
    {
        let mut status_query = preview_nodes.p3();
        let Ok((mut text, mut transform)) = status_query.single_mut() else {
            return;
        };
        *text = Text2d::new(status_value);
        transform.translation = status_translation;
    }
    {
        let mut controls_query = preview_nodes.p4();
        let Ok(mut transform) = controls_query.single_mut() else {
            return;
        };
        transform.translation = controls_translation;
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
    comparison.content_similarity * 0.65
        + comparison.pixel_match_rate * 0.30
        + comparison.global_similarity * 0.05
}

fn persist_best_candidate(
    task: &TaskConfig,
    score: &ScoredCandidate,
    diff_image: &image::RgbaImage,
) {
    let schema_layout = build_view_layout(&task.text, &score.parameters, task.property_defaults);
    let pretty_config = ron::ser::PrettyConfig::new();
    let ron_text = ron::ser::to_string_pretty(&schema_layout, pretty_config)
        .expect("best schema should serialize to RON");
    fs::write(&task.best_view_absolute_path, ron_text).expect("failed to write best view file");
    fs::write(
        &task.best_summary_path,
        serde_json::to_vec_pretty(score).expect("best score should serialize to JSON"),
    )
    .expect("failed to write best summary");
    diff_image
        .save(&task.best_diff_path)
        .expect("failed to save best diff image");
}

fn persist_current_snapshot(
    task: &TaskConfig,
    score: &ScoredCandidate,
    diff_image: &image::RgbaImage,
) {
    fs::write(
        &task.current_summary_path,
        serde_json::to_vec_pretty(score).expect("current score should serialize to JSON"),
    )
    .expect("failed to write current summary");
    diff_image
        .save(&task.current_diff_path)
        .expect("failed to save current diff image");
}

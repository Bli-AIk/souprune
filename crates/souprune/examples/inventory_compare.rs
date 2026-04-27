//! # inventory_compare
//!
//! Builds the phase-one comparison harness for
//! `undertale_backpack.view.ron`.
//!
//! 为 `undertale_backpack.view.ron` 建立阶段一比对工具。
//!
//! ## Usage
//!
//! ## 运行方式
//!
//! ```bash
//! # 1. Export the LÖVE reference frames.
//! # 1. 导出 LÖVE 参考帧。
//! cd dev/inventory_reference && love . --export 3
//!
//! # 2. Run the comparison.
//! # 2. 运行对比。
//! cargo run -p souprune --example inventory_compare
//! ```
//!
//! ## Behavior
//!
//! ## 功能
//!
//! - Loads and renders `overworld/view/undertale_backpack.view.ron`
//! - Compares menu, item-list, and status states against `dev/inventory_reference/frames/`
//! - Writes comparison metrics and diff images to `generated/inventory_compare/`
//! - Exits with an error when the reference is missing or similarity is below
//!   the phase-two acceptance thresholds
//! - 加载并渲染 `overworld/view/undertale_backpack.view.ron`
//! - 与 `dev/inventory_reference/frames/` 中的主菜单、物品列表、状态页参考帧对比
//! - 输出指标和 diff 图到 `generated/inventory_compare/`
//! - 当参考帧缺失，或相似度低于阶段二验收阈值时以错误状态退出

use bevy::app::AppExit;
use bevy::asset::io::file::FileAssetReader;
use bevy::asset::io::{AssetSourceBuilder, AssetSourceId};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::WindowResolution;
use bevy_fact_rule_event::LayeredFactDatabase;
use souprune::core::camera::MainGameCamera;
use souprune::core::game_action::GameActionDef;
use souprune::core::view::CoreViewPlugin;
use souprune::core::view::SpawnViewRequest;
use souprune::core::view::components::ViewRoot;
use souprune::extra::multi_source::MultiSourceAssetReader;

const OUTPUT_DIR: &str = "generated/inventory_compare";
const VIEW_PATH: &str = "overworld/view/undertale_backpack.view.ron";
const REFERENCE_DIR: &str = "dev/inventory_reference/frames";
const INITIAL_SETTLE_FRAMES: u32 = 240;
const CASE_SETTLE_FRAMES: u32 = 30;
const MIN_CONTENT_SIMILARITY: f32 = 0.90;
const MIN_GLOBAL_SIMILARITY: f32 = 0.95;
const MIN_TEXT_HEAVY_GLOBAL_SIMILARITY: f32 = 0.98;
const MIN_TEXT_HEAVY_PIXEL_MATCH: f32 = 0.97;
const MIN_GEOMETRY_SIMILARITY: f32 = 0.99;

#[derive(Clone, Copy)]
struct CompareCase {
    key: &'static str,
    label: &'static str,
    depth: i64,
    selection: i64,
    reference_frame: &'static str,
}

const COMPARE_CASES: [CompareCase; 3] = [
    CompareCase {
        key: "menu",
        label: "Menu Layer",
        depth: 0,
        selection: 0,
        reference_frame: "frame_000.png",
    },
    CompareCase {
        key: "item",
        label: "Item Layer",
        depth: 1,
        selection: 0,
        reference_frame: "frame_001.png",
    },
    CompareCase {
        key: "status",
        label: "Status Layer",
        depth: 3,
        selection: 0,
        reference_frame: "frame_002.png",
    },
];

#[derive(Resource)]
struct CompareState {
    frames_remaining: u32,
    case_index: usize,
    capture_pending: bool,
    failed: bool,
}

fn main() {
    let souprune_config = souprune::config::load_config();

    let mut app = App::new();

    // Register all project and dependency asset roots before DefaultPlugins.
    // 在 DefaultPlugins 之前注册当前项目和依赖项目的所有资源根目录。
    app.register_asset_source(
        AssetSourceId::Default,
        AssetSourceBuilder::new(move || {
            let roots = souprune::config::get_all_asset_roots();
            let readers = roots.into_iter().map(FileAssetReader::new).collect();
            Box::new(MultiSourceAssetReader::new(readers))
        }),
    );

    app.add_plugins(
        DefaultPlugins
            .set(bevy::image::ImagePlugin::default_nearest())
            .set(bevy::asset::AssetPlugin {
                unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Inventory Compare - SoupRune".into(),
                    resolution: WindowResolution::new(640, 480),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
    );

    app.add_plugins((
        souprune::get_file_importer_plugins(),
        souprune::get_third_plugins(),
        bevy_fact_rule_event::FREPlugin::<GameActionDef>::default(),
        souprune::core::CorePlugin,
        CoreViewPlugin,
    ));

    app.insert_non_send_resource(souprune::core::mod_system::LoadedMods::default())
        .init_resource::<souprune::core::mod_system::BehaviorRegistry>()
        .init_resource::<souprune::core::mod_system::DanmakuRegistry>()
        .init_resource::<souprune::core::mod_system::SpawnPatternRegistry>();

    app.insert_resource(souprune_config);
    souprune::init_game_state(&mut app);
    souprune::insert_input_resources(&mut app);
    souprune::insert_font_resources(&mut app);

    app.insert_resource(ClearColor(Color::BLACK));
    app.insert_resource(CompareState {
        frames_remaining: INITIAL_SETTLE_FRAMES,
        case_index: 0,
        capture_pending: false,
        failed: false,
    });

    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (seed_compare_facts, auto_capture, screenshot_on_key),
    );

    app.run();
}

fn setup(
    mut commands: Commands,
    mut next_state: ResMut<NextState<souprune::app_state::AppState>>,
    mut sequence_mode: ResMut<souprune::app_state::SequenceMode>,
    mut spawn_writer: MessageWriter<SpawnViewRequest>,
) {
    commands.spawn((
        Name::new("Inventory Compare Camera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: 640.0,
                height: 480.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::default(),
        MainGameCamera,
    ));

    sequence_mode.0 = Some("overworld".to_string());
    next_state.set(souprune::app_state::AppState::Running);

    spawn_writer.write(SpawnViewRequest {
        path: VIEW_PATH.to_string(),
        mode_scope: Some("overworld".to_string()),
        bindings: None,
    });

    let _ = std::fs::create_dir_all(OUTPUT_DIR);
    info!(
        "Inventory Compare: settling {} frames, then comparing {} cases",
        INITIAL_SETTLE_FRAMES,
        COMPARE_CASES.len()
    );
}

fn seed_compare_facts(mut facts: ResMut<LayeredFactDatabase>) {
    facts.set_global_if_changed(
        "player:inventory",
        vec![
            "Monster Candy",
            "Monster Candy",
            "Monster Candy",
            "Monster Candy",
            "Monster Candy",
            "UNDEFINED (UNDEFITEM)",
        ],
    );
    facts.set_global_if_changed("player:weapon", "Stick");
    facts.set_global_if_changed("player:armor", "Bandage");
    facts.set_global_if_changed("player:total_attack", 0i64);
    facts.set_global_if_changed("player:weapon_atk", 0i64);
    facts.set_global_if_changed("player:total_defense", 0i64);
    facts.set_global_if_changed("player:armor_def", 0i64);
}

fn auto_capture(
    mut commands: Commands,
    mut state: ResMut<CompareState>,
    mut view_roots: Query<&mut ViewRoot>,
) {
    let Some(case) = COMPARE_CASES.get(state.case_index) else {
        return;
    };

    if state.capture_pending {
        return;
    }

    {
        let Some(mut view_root) = view_roots
            .iter_mut()
            .find(|view_root| view_root.layout_path == VIEW_PATH)
        else {
            return;
        };

        view_root.local_facts.set("depth", case.depth);
        view_root.local_facts.set("selection", case.selection);
    }

    if state.frames_remaining > 0 {
        state.frames_remaining -= 1;
        return;
    }

    state.capture_pending = true;
    info!("Auto-capturing {} for comparison", case.label);
    commands
        .spawn(Screenshot::primary_window())
        .observe(handle_comparison);
}

fn handle_comparison(
    trigger: On<ScreenshotCaptured>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
    mut state: ResMut<CompareState>,
) {
    let Some(case) = COMPARE_CASES.get(state.case_index) else {
        error!("No comparison case available for captured screenshot");
        commands.entity(trigger.entity).despawn();
        exit_error(&mut exit);
        return;
    };

    let screenshot_rgba = trigger
        .event()
        .image
        .clone()
        .try_into_dynamic()
        .expect("screenshot should convert to DynamicImage")
        .to_rgba8();

    let render_path = format!("{}/souprune_render_{}.png", OUTPUT_DIR, case.key);
    screenshot_rgba
        .save(&render_path)
        .expect("failed to save render");
    info!("Saved SoupRune render -> {}", render_path);

    let ref_path = std::path::Path::new(REFERENCE_DIR).join(case.reference_frame);
    if !ref_path.exists() {
        error!(
            "Reference frame not found: {}\n  -> Run: cd dev/inventory_reference && love . --export 3",
            ref_path.display()
        );
        commands.entity(trigger.entity).despawn();
        exit_error(&mut exit);
        return;
    }

    let reference_rgba = image::open(ref_path)
        .expect("failed to open reference image")
        .to_rgba8();
    let shot = flatten_alpha_black(&screenshot_rgba);
    let refe = flatten_alpha_black(&reference_rgba);

    let (result, diff_img) = bevy_alight_motion::image_comparison::compare_images(&shot, &refe);

    let diff_path = format!("{}/diff_{}.png", OUTPUT_DIR, case.key);
    let reference_path = format!("{}/reference_{}.png", OUTPUT_DIR, case.key);
    diff_img.save(&diff_path).expect("save diff");
    reference_rgba
        .save(&reference_path)
        .expect("save reference");

    if case.key == "menu" {
        screenshot_rgba
            .save(format!("{}/souprune_render.png", OUTPUT_DIR))
            .expect("save legacy render");
        diff_img
            .save(format!("{}/diff.png", OUTPUT_DIR))
            .expect("save legacy diff");
        reference_rgba
            .save(format!("{}/reference.png", OUTPUT_DIR))
            .expect("save legacy reference");
    }

    info!("==================================================");
    info!("  Inventory View Comparison: {}", case.label);
    info!("==================================================");
    info!("  Global similarity:  {:.4}", result.global_similarity);
    info!("  Content similarity: {:.4}", result.content_similarity);
    info!("  Pixel match rate:   {:.4}", result.pixel_match_rate);
    info!("  Content mask F1:    {:.4}", result.content_mask_f1);
    info!("  Content bbox IoU:   {:.4}", result.content_bbox_iou);
    info!(
        "  Size similarity:    {:.4}",
        result.content_size_similarity
    );
    info!(
        "  Center similarity:  {:.4}",
        result.content_center_similarity
    );
    info!("  Differing pixels:   {}", result.differing_pixels);
    info!("==================================================");
    info!("  Render -> {}", render_path);
    info!("  Diff   -> {}", diff_path);
    info!("==================================================");

    let strict_pixel_pass = result.content_similarity >= MIN_CONTENT_SIMILARITY
        && result.global_similarity >= MIN_GLOBAL_SIMILARITY;
    let coordinate_pass = result.global_similarity >= MIN_TEXT_HEAVY_GLOBAL_SIMILARITY
        && result.pixel_match_rate >= MIN_TEXT_HEAVY_PIXEL_MATCH
        && result.content_bbox_iou >= MIN_GEOMETRY_SIMILARITY
        && result.content_size_similarity >= MIN_GEOMETRY_SIMILARITY
        && result.content_center_similarity >= MIN_GEOMETRY_SIMILARITY;

    if strict_pixel_pass {
        info!("Inventory View comparison case passed strict pixel thresholds");
    } else if coordinate_pass {
        info!(
            "Inventory View comparison case passed coordinate thresholds; content similarity reflects font rasterization differences"
        );
    } else {
        warn!("Inventory View comparison case is below phase-two coordinate thresholds");
        state.failed = true;
    }

    state.case_index += 1;
    state.capture_pending = false;
    state.frames_remaining = CASE_SETTLE_FRAMES;

    if state.case_index >= COMPARE_CASES.len() {
        if state.failed {
            warn!("Inventory View comparison finished with failing cases");
            exit_error(&mut exit);
        } else {
            info!("Inventory View comparison passed for all cases");
            info!("Output -> {}/", OUTPUT_DIR);
            exit.write(AppExit::Success);
        }
    }

    commands.entity(trigger.entity).despawn();
}

fn exit_error(exit: &mut MessageWriter<AppExit>) {
    exit.write(AppExit::Error(
        std::num::NonZero::new(1).expect("exit code is non-zero"),
    ));
    std::process::exit(1);
}

fn flatten_alpha_black(img: &image::RgbaImage) -> image::RgbaImage {
    let mut out = image::RgbaImage::new(img.width(), img.height());
    for (x, y, px) in img.enumerate_pixels() {
        let a = px[3] as u16;
        let r = ((px[0] as u16 * a) / 255) as u8;
        let g = ((px[1] as u16 * a) / 255) as u8;
        let b = ((px[2] as u16 * a) / 255) as u8;
        out.put_pixel(x, y, image::Rgba([r, g, b, 255]));
    }
    out
}

fn screenshot_on_key(
    input: Res<ButtonInput<KeyCode>>,
    mut counter: Local<u32>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::KeyP) {
        let path = format!("{}/manual_{:03}.png", OUTPUT_DIR, *counter);
        *counter += 1;
        commands
            .spawn(Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(path.clone()));
        info!("Manual screenshot -> {}", path);
    }
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

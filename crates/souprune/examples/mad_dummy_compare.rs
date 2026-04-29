//! # mad_dummy_compare
//!
//! Validates the UT-original coordinate-space conversion in `mad_dummy.view.ron`.
//! Compares the SoupRune render against a LÖVE reference frame pixel by pixel.
//!
//! 验证 `mad_dummy.view.ron` 的 UT 原始坐标空间转换是否正确。
//! 自动将 SoupRune 渲染结果与 LÖVE 参考帧进行像素级对比。
//!
//! ## Usage
//!
//! ## 运行方式
//!
//! ```bash
//! # 1. Export the LÖVE reference frame once.
//! # 1. 导出 LÖVE 参考帧（仅需一次）。
//! cd dev/mad_dummy_reference && love . --export
//!
//! # 2. Run the comparison.
//! # 2. 运行对比。
//! cargo run -p souprune --example mad_dummy_compare
//! ```
//!
//! ## Behavior
//!
//! ## 功能
//!
//! - Automatically loads and renders `mad_dummy.view.ron`
//! - Compares against `dev/mad_dummy_reference/frames/frame_000.png`
//! - Writes comparison metrics and diff images to `generated/mad_dummy_compare/`
//! - Exits successfully when the automatic comparison passes
//! - Press `P` for a manual screenshot, or `Escape` to quit before auto-capture
//! - 自动加载 `mad_dummy.view.ron` 并渲染
//! - 与 LÖVE 参考帧 (`dev/mad_dummy_reference/frames/frame_000.png`) 对比
//! - 输出对比指标和 diff 图像到 `generated/mad_dummy_compare/`
//! - 自动对比通过后以成功状态退出
//! - 按 `P` 手动截图，或在自动截图前按 `Escape` 退出

use bevy::app::AppExit;
use bevy::asset::io::file::FileAssetReader;
use bevy::asset::io::{AssetSourceBuilder, AssetSourceId};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::WindowResolution;
use souprune::core::camera::MainGameCamera;
use souprune::core::game_action::GameActionDef;
use souprune::core::view::CoreViewPlugin;
use souprune::core::view::SpawnViewRequest;
use souprune::extra::multi_source::MultiSourceAssetReader;

const OUTPUT_DIR: &str = "generated/mad_dummy_compare";
const VIEW_PATH: &str = "battle/view/mad_dummy.view.ron";
const REFERENCE_FRAME: &str = "dev/mad_dummy_reference/frames/frame_000.png";

/// Frames to wait for the view to load and settle before auto-capture.
///
/// 自动截图前等待 View 加载和稳定的帧数。
const SETTLE_FRAMES: u32 = 120;

#[derive(Resource)]
struct CompareState {
    frames_remaining: u32,
    captured: bool,
}

fn main() {
    let souprune_config = souprune::config::load_config();
    let project_name = souprune_config.project.mod_name.clone();

    let mut app = App::new();

    // Register multi-source asset reader BEFORE DefaultPlugins.
    // This cascades asset lookups through project directories.
    app.register_asset_source(
        AssetSourceId::Default,
        AssetSourceBuilder::new(move || {
            let roots = souprune::config::get_asset_roots(&project_name);
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
                    title: "Mad Dummy Compare — SoupRune".into(),
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
        souprune::core::mod_system::ModPlugin,
    ));

    app.insert_resource(souprune_config);
    souprune::init_game_state(&mut app);
    souprune::insert_input_resources(&mut app);

    app.insert_resource(ClearColor(Color::BLACK));
    app.insert_resource(CompareState {
        frames_remaining: SETTLE_FRAMES,
        captured: false,
    });

    app.add_systems(Startup, setup);
    app.add_systems(Update, (auto_capture, screenshot_on_key));

    app.run();
}

fn setup(
    mut commands: Commands,
    mut next_state: ResMut<NextState<souprune::app_state::AppState>>,
    mut spawn_writer: MessageWriter<SpawnViewRequest>,
) {
    // Camera: 640×480 orthographic, centered on Bevy world origin.
    // The View coordinate_space maps UT 640×480 top-left coordinates into this
    // camera space: UT drawer point (320, 90) becomes Bevy world (0, 150).
    //
    // 相机使用 640×480 正交视图，并居中于 Bevy 世界原点。
    // View 的 coordinate_space 会把 UT 640×480 左上原点坐标映射到该相机空间：
    // UT drawer 点 (320, 90) 变为 Bevy 世界坐标 (0, 150)。
    commands.spawn((
        Name::new("Mad Dummy Camera"),
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

    next_state.set(souprune::app_state::AppState::Running);

    spawn_writer.write(SpawnViewRequest {
        path: VIEW_PATH.to_string(),
        mode_scope: None,
        pre_spawn_events: Vec::new(),
        bindings: None,
    });

    let _ = std::fs::create_dir_all(OUTPUT_DIR);
    info!(
        "Mad Dummy Compare: settling {} frames, then auto-capture",
        SETTLE_FRAMES
    );
}

fn auto_capture(mut commands: Commands, mut state: ResMut<CompareState>) {
    if state.captured {
        return;
    }
    if state.frames_remaining > 0 {
        state.frames_remaining -= 1;
        return;
    }
    state.captured = true;
    info!("Auto-capturing screenshot for comparison…");
    commands
        .spawn(Screenshot::primary_window())
        .observe(handle_comparison);
}

fn handle_comparison(
    trigger: On<ScreenshotCaptured>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    let screenshot_rgba = trigger
        .event()
        .image
        .clone()
        .try_into_dynamic()
        .expect("screenshot should convert to DynamicImage")
        .to_rgba8();

    // Save SoupRune render
    let render_path = format!("{}/souprune_render.png", OUTPUT_DIR);
    screenshot_rgba
        .save(&render_path)
        .expect("failed to save render");
    info!("Saved SoupRune render → {}", render_path);

    // Load LÖVE reference
    let ref_path = std::path::Path::new(REFERENCE_FRAME);
    if !ref_path.exists() {
        error!(
            "Reference frame not found: {}\n  → Run: cd dev/mad_dummy_reference && love . --export",
            REFERENCE_FRAME
        );
        commands.entity(trigger.entity).despawn();
        exit.write(AppExit::Error(
            std::num::NonZero::new(1).expect("exit code is non-zero"),
        ));
        return;
    }
    let reference_rgba = image::open(ref_path)
        .expect("failed to open reference image")
        .to_rgba8();

    // Flatten alpha against black for both (both render on black BG)
    let shot = flatten_alpha_black(&screenshot_rgba);
    let refe = flatten_alpha_black(&reference_rgba);

    // Compare
    let (result, diff_img) = bevy_alight_motion::image_comparison::compare_images(&shot, &refe);

    // Save artefacts
    diff_img
        .save(format!("{}/diff.png", OUTPUT_DIR))
        .expect("save diff");
    reference_rgba
        .save(format!("{}/reference.png", OUTPUT_DIR))
        .expect("save ref");

    // Report
    info!("════════════════════════════════════════════════════");
    info!("  Mad Dummy UT Coordinate-Space Comparison");
    info!("════════════════════════════════════════════════════");
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
    info!("════════════════════════════════════════════════════");
    info!("  Output → {}/", OUTPUT_DIR);
    info!("════════════════════════════════════════════════════");

    if result.content_similarity > 0.8 {
        info!("UT coordinate-space conversion appears correct");
        exit.write(AppExit::Success);
    } else {
        warn!("Low similarity: coordinate-space conversion may have issues");
        exit.write(AppExit::Error(
            std::num::NonZero::new(1).expect("exit code is non-zero"),
        ));
    }

    commands.entity(trigger.entity).despawn();
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
        info!("Manual screenshot → {}", path);
    }
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

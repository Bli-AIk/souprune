//! # image_overlay.rs
//!
//! # image_overlay.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Provides a debug tool to overlay a reference image on the screen and generate a diff overlay
//! against the current frame.
//!
//! 提供一个调试工具，可在屏幕上覆盖参考图，并把当前帧与参考图生成 diff 覆盖层。

#[cfg(feature = "debug")]
pub mod debug_image_overlay {
    use bevy::asset::RenderAssetUsages;
    use bevy::image::Image;
    use bevy::prelude::*;
    use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
    use image::DynamicImage;
    use image::imageops::FilterType;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::SystemTime;

    /// Resource to control image overlay visibility.
    ///
    /// 控制图像覆盖可见性的调试资源。
    #[derive(Resource, Default)]
    pub struct ImageOverlaySettings {
        pub show_reference_overlay: bool,
        pub show_diff_overlay: bool,
        pending_diff_capture: Option<PendingDiffCapture>,
    }

    #[derive(Clone)]
    struct DebugImageInfo {
        asset_path: String,
        filesystem_path: PathBuf,
    }

    #[derive(Clone)]
    struct PendingDiffCapture {
        reference: DebugImageInfo,
    }

    /// Component for the reference overlay image entity.
    ///
    /// 参考图覆盖实体的组件。
    #[derive(Component)]
    pub struct DebugReferenceOverlay;

    /// Component for the diff overlay image entity.
    ///
    /// diff 覆盖实体的组件。
    #[derive(Component)]
    pub struct DebugDiffOverlay;

    /// Set up the image overlay debug systems.
    ///
    /// 设置图像覆盖调试系统。
    pub fn setup_image_overlay_debug(app: &mut App) {
        app.init_resource::<ImageOverlaySettings>().add_systems(
            Update,
            (
                toggle_reference_overlay_system,
                toggle_diff_overlay_system,
                maintain_overlay_system,
            ),
        );
    }

    /// Toggle the reference image overlay with the F5 key (debug only).
    ///
    /// F5 键切换参考图覆盖的系统（仅调试模式）。
    fn toggle_reference_overlay_system(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut settings: ResMut<ImageOverlaySettings>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        reference_overlay_query: Query<Entity, With<DebugReferenceOverlay>>,
        diff_overlay_query: Query<Entity, With<DebugDiffOverlay>>,
        window_query: Query<&Window>,
        mut toast_events: MessageWriter<super::super::DebugToastEvent>,
    ) {
        if !keyboard.just_pressed(KeyCode::F5) {
            return;
        }

        settings.show_reference_overlay = !settings.show_reference_overlay;
        settings.show_diff_overlay = false;
        settings.pending_diff_capture = None;

        for entity in reference_overlay_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in diff_overlay_query.iter() {
            commands.entity(entity).despawn();
        }

        if !settings.show_reference_overlay {
            info!("Image Overlay: OFF");
            toast_events.write(super::super::DebugToastEvent {
                message: "Image Overlay: OFF".into(),
            });
            return;
        }

        let Some(debug_image) = find_latest_debug_image_info() else {
            settings.show_reference_overlay = false;
            toast_events.write(super::super::DebugToastEvent {
                message: "Image Overlay: no debug image found".into(),
            });
            return;
        };
        spawn_reference_overlay(&mut commands, &asset_server, &window_query, &debug_image);
        info!("Image Overlay: ON");
        toast_events.write(super::super::DebugToastEvent {
            message: "Image Overlay: ON".into(),
        });
    }

    /// Trigger a diff capture with the F10 key and show the resulting diff overlay.
    ///
    /// F10 触发当前帧与参考图的 diff 捕获，并显示 diff 覆盖层。
    fn toggle_diff_overlay_system(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut settings: ResMut<ImageOverlaySettings>,
        mut commands: Commands,
        reference_overlay_query: Query<Entity, With<DebugReferenceOverlay>>,
        diff_overlay_query: Query<Entity, With<DebugDiffOverlay>>,
        mut toast_events: MessageWriter<super::super::DebugToastEvent>,
    ) {
        if !keyboard.just_pressed(KeyCode::F10) {
            return;
        }

        if settings.pending_diff_capture.is_some() {
            settings.pending_diff_capture = None;
            settings.show_diff_overlay = false;
            for entity in diff_overlay_query.iter() {
                commands.entity(entity).despawn();
            }
            info!("Image Diff: canceled");
            toast_events.write(super::super::DebugToastEvent {
                message: "Image Diff: canceled".into(),
            });
            return;
        }

        if settings.show_diff_overlay {
            settings.show_diff_overlay = false;
            for entity in diff_overlay_query.iter() {
                commands.entity(entity).despawn();
            }
            info!("Image Diff: OFF");
            toast_events.write(super::super::DebugToastEvent {
                message: "Image Diff: OFF".into(),
            });
            return;
        }

        let Some(reference) = find_latest_debug_image_info() else {
            toast_events.write(super::super::DebugToastEvent {
                message: "Image Diff: no debug image found".into(),
            });
            return;
        };

        settings.show_reference_overlay = false;
        settings.show_diff_overlay = false;
        settings.pending_diff_capture = Some(PendingDiffCapture { reference });

        for entity in reference_overlay_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in diff_overlay_query.iter() {
            commands.entity(entity).despawn();
        }

        commands
            .spawn(Screenshot::primary_window())
            .observe(handle_diff_screenshot_captured);
    }

    /// Spawn a reference overlay image entity.
    fn spawn_reference_overlay(
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        window_query: &Query<&Window>,
        debug_image: &DebugImageInfo,
    ) {
        info!("Loading debug overlay image: {}", debug_image.asset_path);
        let image_handle: Handle<Image> = asset_server.load(&debug_image.asset_path);

        let Ok(window) = window_query.single() else {
            return;
        };
        spawn_overlay_node(
            commands,
            Name::new("DebugReferenceOverlay"),
            DebugReferenceOverlay,
            image_handle,
            window.width(),
            window.height(),
            Color::srgba(1.0, 1.0, 1.0, 0.7),
            Color::srgba(0.0, 0.0, 0.0, 0.3),
        );
    }

    fn spawn_diff_overlay(
        commands: &mut Commands,
        window_query: &Query<&Window>,
        image_handle: Handle<Image>,
    ) {
        let Ok(window) = window_query.single() else {
            return;
        };
        spawn_overlay_node(
            commands,
            Name::new("DebugDiffOverlay"),
            DebugDiffOverlay,
            image_handle,
            window.width(),
            window.height(),
            Color::WHITE,
            Color::NONE,
        );
    }

    fn spawn_overlay_node<T: Component>(
        commands: &mut Commands,
        name: Name,
        marker: T,
        image_handle: Handle<Image>,
        window_width: f32,
        window_height: f32,
        image_color: Color,
        background_color: Color,
    ) {
        commands
            .spawn((
                name,
                marker,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(background_color),
                ZIndex(1000),
            ))
            .with_children(|parent| {
                parent.spawn((
                    ImageNode {
                        image: image_handle,
                        color: image_color,
                        ..default()
                    },
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        max_width: Val::Px(window_width),
                        max_height: Val::Px(window_height),
                        ..default()
                    },
                ));
            });
    }

    fn handle_diff_screenshot_captured(
        trigger: On<ScreenshotCaptured>,
        mut commands: Commands,
        mut settings: ResMut<ImageOverlaySettings>,
        mut images: ResMut<Assets<Image>>,
        window_query: Query<&Window>,
        mut toast_events: MessageWriter<super::super::DebugToastEvent>,
    ) {
        let Some(pending_capture) = settings.pending_diff_capture.take() else {
            commands.entity(trigger.entity).despawn();
            return;
        };

        let screenshot_image = match trigger.event().image.clone().try_into_dynamic() {
            Ok(dynamic_image) => dynamic_image.to_rgba8(),
            Err(error) => {
                commands.entity(trigger.entity).despawn();
                warn!("Image Diff: failed to convert screenshot image: {error}");
                toast_events.write(super::super::DebugToastEvent {
                    message: "Image Diff: failed to capture screenshot".into(),
                });
                return;
            }
        };

        let reference_image = match load_reference_image(&pending_capture.reference.filesystem_path)
        {
            Ok(reference_image) => reference_image,
            Err(error) => {
                commands.entity(trigger.entity).despawn();
                warn!(
                    "Image Diff: failed to load reference image {}: {error}",
                    pending_capture.reference.filesystem_path.display()
                );
                toast_events.write(super::super::DebugToastEvent {
                    message: "Image Diff: failed to load reference".into(),
                });
                return;
            }
        };

        let fitted_reference_image = fit_image_to_canvas(
            &reference_image,
            screenshot_image.width(),
            screenshot_image.height(),
        );
        let (comparison, diff_image) = bevy_alight_motion::image_comparison::compare_images(
            &screenshot_image,
            &fitted_reference_image,
        );
        let diff_handle = images.add(Image::from_dynamic(
            DynamicImage::ImageRgba8(diff_image),
            true,
            RenderAssetUsages::all(),
        ));

        spawn_diff_overlay(&mut commands, &window_query, diff_handle);
        settings.show_diff_overlay = true;

        info!(
            "Image Diff: ON (content={:.4}, global={:.4}, differing_pixels={})",
            comparison.content_similarity,
            comparison.global_similarity,
            comparison.differing_pixels
        );
        toast_events.write(super::super::DebugToastEvent {
            message: format!(
                "Image Diff: ON | content {:.4} | global {:.4}",
                comparison.content_similarity, comparison.global_similarity
            ),
        });
        commands.entity(trigger.entity).despawn();
    }

    /// Maintain the overlay entity and remove it when needed.
    ///
    /// 维护覆盖层的系统（如需要则移除）。
    fn maintain_overlay_system(
        settings: Res<ImageOverlaySettings>,
        mut commands: Commands,
        reference_overlay_query: Query<Entity, With<DebugReferenceOverlay>>,
        diff_overlay_query: Query<Entity, With<DebugDiffOverlay>>,
    ) {
        if !settings.show_reference_overlay {
            for entity in reference_overlay_query.iter() {
                commands.entity(entity).despawn();
            }
        }

        if !settings.show_diff_overlay {
            for entity in diff_overlay_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }

    /// Scan directory entries for the latest image file by modification time.
    fn scan_for_latest_image(
        entries: fs::ReadDir,
        extensions: &[&str],
        latest_file: &mut Option<(DebugImageInfo, SystemTime)>,
    ) {
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_file() {
                continue;
            }
            let file_name_os = entry.file_name();
            let Some(file_name) = file_name_os.to_str() else {
                continue;
            };
            let Some(extension) = file_name.split('.').next_back() else {
                continue;
            };
            if !extensions.contains(&extension.to_lowercase().as_str()) {
                continue;
            }
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            let Ok(modified) = metadata.modified() else {
                continue;
            };

            let entry_path = entry.path();
            let Some(parent_dir) = entry_path.parent() else {
                continue;
            };
            let relative_path = format!("debug/{}", file_name);
            let filesystem_path = parent_dir.join(file_name);
            if latest_file.is_none() || latest_file.as_ref().unwrap().1 < modified {
                *latest_file = Some((
                    DebugImageInfo {
                        asset_path: relative_path,
                        filesystem_path,
                    },
                    modified,
                ));
            }
        }
    }

    /// Find the most recently modified image in the debug folder.
    ///
    /// 查找 debug 文件夹中最近修改的图像。
    fn find_latest_debug_image_info() -> Option<DebugImageInfo> {
        let config = crate::config::load_config();
        let project_debug_path = format!("projects/{}/assets/debug", config.project.mod_name);
        let possible_paths = [project_debug_path.as_str(), "assets/debug"];
        let extensions = ["png", "jpg", "jpeg", "gif", "bmp", "tiff"];

        let mut latest_file: Option<(DebugImageInfo, SystemTime)> = None;
        let mut found_debug_folder = false;

        for debug_path in &possible_paths {
            if !Path::new(debug_path).exists() {
                continue;
            }

            found_debug_folder = true;

            let Ok(entries) = fs::read_dir(debug_path) else {
                continue;
            };
            scan_for_latest_image(entries, &extensions, &mut latest_file);
            if latest_file.is_some() {
                break;
            }
        }

        if !found_debug_folder {
            warn!("Debug folder not found in any of the expected locations");
            return None;
        }

        if let Some((debug_image, _)) = latest_file {
            info!("Selected latest debug image: {}", debug_image.asset_path);
            Some(debug_image)
        } else {
            warn!("No image files found in debug folder");
            None
        }
    }

    fn load_reference_image(path: &Path) -> anyhow::Result<image::RgbaImage> {
        let reference_image = image::open(path)
            .map_err(|error| {
                anyhow::anyhow!("failed to open reference image {}: {error}", path.display())
            })?
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

    fn fit_image_to_canvas(
        image: &image::RgbaImage,
        canvas_width: u32,
        canvas_height: u32,
    ) -> image::RgbaImage {
        if image.width() == canvas_width && image.height() == canvas_height {
            return image.clone();
        }

        let scale_x = canvas_width as f32 / image.width() as f32;
        let scale_y = canvas_height as f32 / image.height() as f32;
        let scale = scale_x.min(scale_y);

        let scaled_width = (image.width() as f32 * scale).round().max(1.0) as u32;
        let scaled_height = (image.height() as f32 * scale).round().max(1.0) as u32;
        let resized =
            image::imageops::resize(image, scaled_width, scaled_height, FilterType::Nearest);

        let mut canvas =
            image::RgbaImage::from_pixel(canvas_width, canvas_height, image::Rgba([0, 0, 0, 255]));
        let offset_x = ((canvas_width - scaled_width) / 2) as i64;
        let offset_y = ((canvas_height - scaled_height) / 2) as i64;
        image::imageops::overlay(&mut canvas, &resized, offset_x, offset_y);
        canvas
    }
}

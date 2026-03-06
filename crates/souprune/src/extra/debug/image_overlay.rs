//! # image_overlay.rs
//!
//! # image_overlay.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Provides a debug tool to overlay an image on the screen (toggled via F4), useful for comparing in-game visuals with reference assets.
//!
//! 提供一个在屏幕上覆盖图像的调试工具（通过 F4 切换），用于将游戏内视觉效果与参考资产进行对比。

#[cfg(feature = "debug")]
pub mod debug_image_overlay {
    use bevy::prelude::*;
    use std::fs;
    use std::path::Path;
    use std::time::SystemTime;

    /// Resource to control image overlay visibility.
    ///
    /// 控制图像覆盖可见性的调试资源。
    #[derive(Resource, Default)]
    pub struct ImageOverlaySettings {
        pub show_overlay: bool,
    }

    /// Component for the overlay image entity.
    ///
    /// 覆盖图像实体的组件。
    #[derive(Component)]
    pub struct DebugImageOverlay;

    /// Set up the image overlay debug systems.
    ///
    /// 设置图像覆盖调试系统。
    pub fn setup_image_overlay_debug(app: &mut App) {
        app.init_resource::<ImageOverlaySettings>().add_systems(
            Update,
            (toggle_image_overlay_system, maintain_overlay_system),
        );
    }

    /// Toggle the image overlay with the F4 key (debug only).
    ///
    /// F4 键切换图像覆盖的系统（仅调试模式）。
    fn toggle_image_overlay_system(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut settings: ResMut<ImageOverlaySettings>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        overlay_query: Query<Entity, With<DebugImageOverlay>>,
        window_query: Query<&Window>,
    ) {
        if !keyboard.just_pressed(KeyCode::F5) {
            return;
        }

        settings.show_overlay = !settings.show_overlay;

        // Remove any existing overlay entity.
        for entity in overlay_query.iter() {
            commands.entity(entity).despawn();
        }

        if !settings.show_overlay {
            info!("Debug image overlay: OFF");
            return;
        }

        spawn_debug_overlay(&mut commands, &asset_server, &window_query);
    }

    /// Spawn a debug overlay image entity.
    fn spawn_debug_overlay(
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        window_query: &Query<&Window>,
    ) {
        let Some(latest_image_path) = find_latest_debug_image() else {
            return;
        };
        info!("Loading debug overlay image: {}", latest_image_path);

        let image_handle: Handle<Image> = asset_server.load(&latest_image_path);

        let Ok(window) = window_query.single() else {
            return;
        };
        let window_width = window.width();
        let window_height = window.height();

        commands
            .spawn((
                Name::new("DebugImageOverlay"),
                DebugImageOverlay,
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
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                ZIndex(1000),
            ))
            .with_children(|parent| {
                parent.spawn((
                    ImageNode {
                        image: image_handle,
                        color: Color::srgba(1.0, 1.0, 1.0, 0.7),
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

        info!("Debug image overlay: ON");
    }

    /// Maintain the overlay entity and remove it when needed.
    ///
    /// 维护覆盖层的系统（如需要则移除）。
    fn maintain_overlay_system(
        settings: Res<ImageOverlaySettings>,
        mut commands: Commands,
        overlay_query: Query<Entity, With<DebugImageOverlay>>,
    ) {
        if !settings.show_overlay {
            for entity in overlay_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }

    /// Scan directory entries for the latest image file by modification time.
    fn scan_for_latest_image(
        entries: fs::ReadDir,
        extensions: &[&str],
        latest_file: &mut Option<(String, SystemTime)>,
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

            let relative_path = format!("debug/{}", file_name);
            if latest_file.is_none() || latest_file.as_ref().unwrap().1 < modified {
                *latest_file = Some((relative_path, modified));
            }
        }
    }

    /// Find the most recently modified image in the debug folder.
    ///
    /// 查找 debug 文件夹中最近修改的图像。
    fn find_latest_debug_image() -> Option<String> {
        let config = crate::config::load_config();
        let project_debug_path = format!("projects/{}/assets/debug", config.project.mod_name);
        let possible_paths = [project_debug_path.as_str(), "assets/debug"];
        let extensions = ["png", "jpg", "jpeg", "gif", "bmp", "tiff"];

        let mut latest_file: Option<(String, SystemTime)> = None;
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

        if let Some((path, _)) = latest_file {
            info!("Selected latest debug image: {}", path);
            Some(path)
        } else {
            warn!("No image files found in debug folder");
            None
        }
    }
}

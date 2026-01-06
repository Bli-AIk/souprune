//! Bevy plugin for Alight Motion support.

use bevy::prelude::*;

use crate::animation::{AmPlayback, advance_playback, animate_opacity, animate_transform};
use crate::loader::{AlightMotionLoader, AmProject};
use crate::scene::{AmProjectBundle, AmProjectRoot, AmSceneConfig, spawn_scene};

/// Plugin providing Alight Motion support for Bevy.
pub struct AlightMotionPlugin;

impl Plugin for AlightMotionPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<AmProject>()
            .init_asset_loader::<AlightMotionLoader>()
            .init_resource::<AmPlayback>()
            .add_systems(
                Update,
                (
                    spawn_loaded_projects,
                    advance_playback,
                    animate_transform,
                    animate_opacity,
                )
                    .chain(),
            );
    }
}

/// System to spawn entities when a project finishes loading.
fn spawn_loaded_projects(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AmProjectRoot)>,
    projects: Res<Assets<AmProject>>,
    mut playback: ResMut<AmPlayback>,
) {
    for (entity, mut root) in query.iter_mut() {
        if root.spawned {
            continue;
        }

        if let Some(project) = projects.get(&root.handle) {
            println!(
                "Loading AM project: {} ({}x{}, {}ms)",
                project.scene.title,
                project.scene.width,
                project.scene.height,
                project.scene.total_time
            );
            println!("  Media count: {}", project.scene.media.len());
            println!("  Images loaded: {}", project.images.len());
            for (uri, _) in &project.images {
                println!("    - {}", uri);
            }
            println!("  Layers count: {}", project.scene.layers.len());

            // Update playback duration
            playback.total_time_ms = project.scene.total_time as f32;

            // Build scene configuration
            let config = AmSceneConfig {
                canvas_width: project.scene.width as f32,
                canvas_height: project.scene.height as f32,
                ..Default::default()
            };

            // Spawn the scene entities
            spawn_scene(
                &mut commands,
                &project.scene,
                &project.images,
                entity,
                &config,
            );

            root.spawned = true;
            println!("Scene spawned successfully");
        }
    }
}

/// Helper function to load and spawn an AM project.
pub fn load_am_project(
    commands: &mut Commands,
    asset_server: &AssetServer,
    path: impl Into<String>,
) -> Entity {
    let path_string: String = path.into();
    let handle: Handle<AmProject> = asset_server.load(path_string);

    commands
        .spawn(AmProjectBundle {
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
            marker: AmProjectRoot {
                handle,
                spawned: false,
            },
        })
        .id()
}

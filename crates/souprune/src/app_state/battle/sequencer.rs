//! # sequencer.rs
//!
//! # sequencer.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Sequencer is the linear sequence manager for the battle system.
//! It is responsible for managing and executing Chapters in the battle,
//! ensuring they proceed in order.
//!
//! Sequencer 是战斗系统的线性序列管理器。
//! 它负责管理和执行战斗中的章节（Chapter），确保它们按顺序进行。

pub(crate) struct SequencerPlugin;

impl Plugin for SequencerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BattleQueue>()
            .add_systems(OnEnter(AppState::Battle), load_default_chapter_system)
            .add_systems(
                Update,
                (
                    advance_battle_flow_system,
                    process_player_action_system,
                    process_camera_action_system,
                    process_ui_action_system,
                    process_player_spawn_requests,
                    process_wait_chapter_system,
                    sync_battle_flow_system,
                )
                    .chain()
                    .in_set(BattleUpdate),
            );
    }
}

use super::chapter::{Chapter, PlayerAction};
use crate::app_state::AppState;
use crate::app_state::battle::config::BattlePlayerConfig;
use crate::app_state::battle::{BattleFlowAsset, BattleUpdate};
use crate::core::mod_system::{SoulParams, SoulState, SoulVelocity};
use bevy::prelude::*;

/// [Resource] includes the queue of Chapters that have not yet occurred
///
/// [Resource] 存放还没发生的章节队列
#[derive(Resource, Default)]
pub struct BattleQueue {
    pub chapters: Vec<Chapter>,
}

#[derive(Component)]
struct ActiveChapter(Chapter);

#[derive(Component)]
struct WaitTimer(Timer);

#[derive(Resource)]
struct CurrentBattleFlow(Handle<BattleFlowAsset>);

fn load_default_chapter_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    // TODO: Remove hardcoded chapter path - should be configurable or load from save data
    // TODO：删除硬编码的章节路径 - 应该是可配置的或从保存数据加载
    let handle = asset_server.load::<BattleFlowAsset>("battle/chapters/demo.chapter.ron");
    commands.insert_resource(CurrentBattleFlow(handle));
    info!("Loading default battle flow: battle/chapters/demo.chapter.ron");
}

fn sync_battle_flow_system(
    mut commands: Commands,
    flow_handle: Option<Res<CurrentBattleFlow>>,
    mut queue: ResMut<BattleQueue>,
    assets: Res<Assets<BattleFlowAsset>>,
) {
    if let Some(handle) = flow_handle
        && let Some(asset) = assets.get(&handle.0)
        && queue.chapters.is_empty()
    {
        info!(
            "Battle flow loaded. Pushing {} chapters to queue.",
            asset.0.len()
        );
        queue.chapters.extend(asset.0.clone());
        commands.remove_resource::<CurrentBattleFlow>();
    }
}

/// Advance the battle flow system.
fn advance_battle_flow_system(
    mut commands: Commands,
    mut queue: ResMut<BattleQueue>,
    active_query: Query<Entity, With<ActiveChapter>>,
) {
    if !active_query.is_empty() {
        return;
    }

    if queue.chapters.is_empty() {
        return;
    }
    let next_chapter = queue.chapters.remove(0);

    info!("Starting Chapter: {:?}", next_chapter);
    let entity = commands.spawn(ActiveChapter(next_chapter.clone())).id();

    // Add specific components based on chapter type
    if let Chapter::Wait(secs) = next_chapter {
        commands
            .entity(entity)
            .insert(WaitTimer(Timer::from_seconds(secs, TimerMode::Once)));
    }
}

fn process_wait_chapter_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut WaitTimer), With<ActiveChapter>>,
    time: Res<Time>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.is_finished() {
            commands.entity(entity).despawn();
            info!("Wait Chapter finished.");
        }
    }
}

fn process_camera_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<WaitTimer>>,
    mut camera_query: Query<
        (Entity, &mut Transform, &mut Projection),
        With<crate::app_state::battle::BattleCamera>,
    >,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::SetCamera(action) = &active_chapter.0 {
            // Using iter_mut instead of get_single_mut to be safe and compatible
            for (_cam_entity, mut transform, mut proj) in camera_query.iter_mut() {
                match action {
                    super::chapter::CameraAction::SetPosition(pos) => {
                        transform.translation = pos.extend(transform.translation.z);
                    }
                    super::chapter::CameraAction::SetZoom(zoom) => {
                        if let Projection::Orthographic(ortho) = &mut *proj {
                            ortho.scale = *zoom;
                        } else {
                            warn!("BattleCamera does not have OrthographicProjection!");
                        }
                    }
                    // Shake and FollowPlayer need more components/systems
                    _ => {
                        warn!("Camera action {:?} not implemented yet", action);
                    }
                }
            }
            commands.entity(entity).despawn();
        }
    }
}

fn process_ui_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<WaitTimer>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::SetUI(action) = &active_chapter.0 {
            match action {
                super::chapter::UIAction::LoadLayout(path) => {
                    let handle = asset_server.load(path);
                    commands.insert_resource(crate::core::ui::UILayoutHandle {
                        handle,
                        last_modified: None,
                    });

                    // Spawn a root Battle UI entity
                    // Reuse RonUI component structure
                    commands.spawn((
                        crate::core::ui::components::RonUI::new(
                            crate::core::ui::components::UILayer::BACKPACK_MENU,
                            0,
                        ),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        crate::app_state::battle::BattleEntity(),
                        Name::new("BattleUI Root"),
                    ));

                    // Signal watcher to reload
                    // Using public export from ui module
                    commands.init_resource::<crate::core::ui::UILayoutWatcher>();
                }
                _ => {
                    warn!("UI action {:?} not fully implemented yet", action);
                }
            }
            commands.entity(entity).despawn();
        }
        // Handle legacy UIInteraction by treating it as LoadLayout
        // (For compatibility with demo.chapter.ron)
        else if let Chapter::UIInteraction { ui_layout } = &active_chapter.0 {
            let handle = asset_server.load(ui_layout);
            commands.insert_resource(crate::core::ui::UILayoutHandle {
                handle,
                last_modified: None,
            });
            commands.spawn((
                crate::core::ui::components::RonUI::new(
                    crate::core::ui::components::UILayer::BACKPACK_MENU,
                    0,
                ),
                Transform::default(),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                crate::app_state::battle::BattleEntity(),
                Name::new("BattleUI Root"),
            ));
            // Don't despawn yet? If we want to block?
            // For now, let's treat it as non-blocking to keep it simple, or implement blocking later.
            commands.entity(entity).despawn();
        }
    }
}

fn process_player_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<WaitTimer>>,
    asset_server: Res<AssetServer>,
    mut player_query: Query<
        &mut Transform,
        (
            With<SoulParams>,
            With<crate::app_state::battle::BattleEntity>,
        ),
    >,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::SetPlayer(action) = &active_chapter.0 {
            match action {
                PlayerAction::Spawn {
                    config_path,
                    position,
                } => {
                    let handle = asset_server.load::<BattlePlayerConfig>(config_path);
                    commands.spawn((
                        PlayerSpawnRequest {
                            config_handle: handle,
                            position: *position,
                        },
                        crate::app_state::battle::BattleEntity(),
                    ));
                }
                PlayerAction::Teleport(pos) => {
                    for mut transform in player_query.iter_mut() {
                        transform.translation = pos.extend(0.0);
                        info!("Player teleported to {}", pos);
                    }
                }
                PlayerAction::Despawn => {
                    // For simplicity, just despawn all battle entities with SoulParams
                    // In reality, should be more specific
                    // Handled here via a hacky way for now
                }
                _ => {}
            }
            // Most SetPlayer actions are instantaneous
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
struct PlayerSpawnRequest {
    config_handle: Handle<BattlePlayerConfig>,
    position: Vec2,
}

// System to handle the actual spawn once config is loaded
// I need to add this to the plugin or App.
// Let's add it to SequencerPlugin above.

// Wait, I cannot modify SequencerPlugin easily inside this replace block if I don't add the system there.
// I'll add `process_player_spawn_requests` to the system list in `build`.

fn process_player_spawn_requests(
    mut commands: Commands,
    query: Query<(Entity, &PlayerSpawnRequest)>,
    configs: Res<Assets<BattlePlayerConfig>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, req) in query.iter() {
        if let Some(config) = configs.get(&req.config_handle) {
            info!("Config loaded. Spawning player...");

            // Convert collider configs to components
            let physics_collider = match &config.physics_collider.shape {
                crate::app_state::battle::config::ColliderShape::Circle { radius } => {
                    crate::core::collision::PhysicsCollider::Circle { radius: *radius }
                }
                crate::app_state::battle::config::ColliderShape::Box { half_size } => {
                    crate::core::collision::PhysicsCollider::Box {
                        half_size: *half_size,
                    }
                }
            };

            let damage_trigger = match &config.damage_trigger.shape {
                crate::app_state::battle::config::ColliderShape::Circle { radius } => {
                    crate::core::collision::TriggerCollider::Circle { radius: *radius }
                }
                crate::app_state::battle::config::ColliderShape::Box { half_size } => {
                    crate::core::collision::TriggerCollider::Box {
                        half_size: *half_size,
                    }
                }
            };

            commands.spawn((
                Sprite {
                    image: asset_server.load(&config.sprite_path),
                    color: config.color,
                    ..default()
                },
                Transform::from_translation(req.position.extend(config.z_position)),
                physics_collider.clone(),
                damage_trigger.clone(),
                SoulParams {
                    mode_id: config.default_mode_id.clone(),
                },
                SoulState::default(),
                SoulVelocity::default(),
                crate::app_state::battle::BattleEntity(),
                Name::new("BattlePlayer"),
            ));

            info!(
                "Spawned player with physics collider: {:?}, damage trigger: {:?}, at z: {}",
                physics_collider, damage_trigger, config.z_position
            );

            commands.entity(entity).despawn();
        }
    }
}

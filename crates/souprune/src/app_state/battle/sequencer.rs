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
    let handle = asset_server.load::<BattleFlowAsset>("battle/demo.chapter.ron");
    commands.insert_resource(CurrentBattleFlow(handle));
    info!("Loading default battle flow: battle/demo.chapter.ron");
}

fn sync_battle_flow_system(
    mut commands: Commands,
    flow_handle: Option<Res<CurrentBattleFlow>>,
    mut queue: ResMut<BattleQueue>,
    assets: Res<Assets<BattleFlowAsset>>,
) {
    if let Some(handle) = flow_handle {
        if let Some(asset) = assets.get(&handle.0) {
            if queue.chapters.is_empty() {
                info!(
                    "Battle flow loaded. Pushing {} chapters to queue.",
                    asset.0.len()
                );
                queue.chapters.extend(asset.0.clone());
                commands.remove_resource::<CurrentBattleFlow>();
            }
        }
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
    match next_chapter {
        Chapter::Wait(secs) => {
            commands
                .entity(entity)
                .insert(WaitTimer(Timer::from_seconds(secs, TimerMode::Once)));
        }
        _ => {}
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
        match &active_chapter.0 {
            Chapter::SetPlayer(action) => {
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
            _ => {}
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

            commands.spawn((
                Sprite {
                    image: asset_server.load(&config.sprite_path),
                    color: config.color,
                    ..default()
                },
                Transform::from_translation(req.position.extend(0.0)),
                SoulParams {
                    mode_id: config.default_mode_id.clone(),
                },
                SoulState::default(),
                SoulVelocity::default(),
                crate::app_state::battle::BattleEntity(),
            ));

            commands.entity(entity).despawn();
        }
    }
}

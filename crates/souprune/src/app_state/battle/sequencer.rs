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
        app.init_resource::<BattleContext>()
            .add_systems(OnEnter(AppState::Battle), load_default_chapter_system)
            .add_systems(
                Update,
                (
                    advance_battle_flow_system,
                    process_player_action_system,
                    process_camera_action_system,
                    process_ui_action_system,
                    process_bullet_pattern_system,
                    process_danmaku_performance_system,
                    process_player_spawn_requests,
                    process_wait_chapter_system,
                    process_parallel_chapter_system,
                    cleanup_finished_chapters_system,
                    sync_battle_flow_system,
                )
                    .chain()
                    .in_set(BattleUpdate),
            );
    }
}

use super::chapter::{Chapter, PlayerAction};
use super::danmaku::{PlayPerformanceEvent, SpawnPatternEvent};
use crate::app_state::AppState;
use crate::app_state::battle::config::BattlePlayerConfig;
use crate::app_state::battle::{BattleAsset, BattleUpdate};
use crate::core::mod_system::{BehaviorParams, BehaviorVelocity};
use bevy::prelude::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleExecutionState {
    #[default]
    Idle,
    Processing,
    Waiting,
}

/// [Resource] includes the queue of Chapters that have not yet occurred
///
/// [Resource] 存放还没发生的章节队列
#[derive(Resource, Default)]
pub struct BattleContext {
    pub chapters: Vec<Chapter>,
    pub state: BattleExecutionState,
}

#[derive(Component)]
struct ActiveChapter {
    chapter: Chapter,
    parent: Option<Entity>,
}

#[derive(Component)]
struct WaitTimer(Timer);

#[derive(Component)]
struct ParallelTracker {
    pending_count: usize,
}

#[derive(Resource)]
struct CurrentBattleFlow(Handle<BattleAsset>);

fn load_default_chapter_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    // TODO: Remove hardcoded chapter path - should be configurable or load from save data
    // TODO：删除硬编码的章节路径 - 应该是可配置的或从保存数据加载
    let handle = asset_server.load::<BattleAsset>("battle/chapters/demo.battle.ron");
    commands.insert_resource(CurrentBattleFlow(handle));
    info!("Loading default battle flow: battle/chapters/demo.battle.ron");
}

fn sync_battle_flow_system(
    mut commands: Commands,
    flow_handle: Option<Res<CurrentBattleFlow>>,
    mut context: ResMut<BattleContext>,
    assets: Res<Assets<BattleAsset>>,
) {
    if let Some(handle) = flow_handle
        && let Some(asset) = assets.get(&handle.0)
        && context.chapters.is_empty()
    {
        info!(
            "Battle flow loaded. Pushing {} chapters to queue.",
            asset.0.len()
        );
        context.chapters.extend(asset.0.clone());
        commands.remove_resource::<CurrentBattleFlow>();
    }
}

// Helper to spawn chapters
fn spawn_chapter(commands: &mut Commands, chapter: Chapter, parent: Option<Entity>) {
    let entity = commands
        .spawn(ActiveChapter {
            chapter: chapter.clone(),
            parent,
        })
        .id();

    match chapter {
        Chapter::Wait(secs) => {
            commands
                .entity(entity)
                .insert(WaitTimer(Timer::from_seconds(secs, TimerMode::Once)));
        }
        Chapter::Parallel(children) => {
            commands.entity(entity).insert(ParallelTracker {
                pending_count: children.len(),
            });
            for child in children {
                spawn_chapter(commands, child, Some(entity));
            }
        }
        Chapter::Sequence(mut children) => {
            if parent.is_none() {
                // If parent is none, handled in advance_battle_flow_system.
                // But if we reach here, it means we spawned it as an entity, which shouldn't happen for root sequence.
                // Or maybe we treat it as Parallel for now if somehow spawned.
            } else {
                warn!("Nested Sequence not fully implemented yet, treating as Parallel for now");
                commands.entity(entity).insert(ParallelTracker {
                    pending_count: children.len(),
                });
                for child in children {
                    spawn_chapter(commands, child, Some(entity));
                }
            }
        }
        _ => {}
    }
}

// Let's rewrite `advance_battle_flow_system` to use the helper and handle Sequence properly
fn advance_battle_flow_system(
    mut commands: Commands,
    mut context: ResMut<BattleContext>,
    active_chapters: Query<&ActiveChapter>,
) {
    // Check if any root-level chapter is active
    for chapter in active_chapters.iter() {
        if chapter.parent.is_none() {
            return; // Busy
        }
    }

    if context.chapters.is_empty() {
        return;
    }

    let next_chapter = context.chapters.remove(0);

    match next_chapter {
        Chapter::Sequence(sub_chapters) => {
            // Unpack sequence to the front of the queue
            let mut new_queue = sub_chapters;
            new_queue.append(&mut context.chapters);
            context.chapters = new_queue;
            // Loop again next frame to pick up the first item
        }
        _ => {
            info!("Starting Root Chapter: {:?}", next_chapter);
            spawn_chapter(&mut commands, next_chapter, None);
        }
    }
}

fn process_parallel_chapter_system(
    _commands: Commands,
    _parents: Query<(Entity, &mut ParallelTracker)>,
) {
    // Placeholder to keep the system chain happy if needed, or remove it.
    // Logic moved to cleanup_finished_chapters_system
}

#[derive(Component)]
struct ChapterFinished;

fn cleanup_finished_chapters_system(
    mut commands: Commands,
    finished_query: Query<(Entity, &ActiveChapter), With<ChapterFinished>>,
    mut parallel_parents: Query<&mut ParallelTracker>,
) {
    for (entity, chapter) in finished_query.iter() {
        if let Some(parent_entity) = chapter.parent {
            if let Ok(mut tracker) = parallel_parents.get_mut(parent_entity) {
                tracker.pending_count = tracker.pending_count.saturating_sub(1);
                if tracker.pending_count == 0 {
                    // Parent finished!
                    commands.entity(parent_entity).insert(ChapterFinished);
                }
            }
        }

        // Use despawn_recursive from Bevy's hierarchy extension
        // Since I cannot easily import it here without changing prelude usage,
        // and despawn_recursive is a trait method on EntityCommands.
        // It requires `bevy::hierarchy::DespawnRecursiveExt`.
        //
        // However, a simpler way in standard Bevy usage is usually commands.entity(e).despawn_recursive().
        // If it's not found, maybe I should just use despawn() if I don't expect children?
        // But Parallel chapters have children (though children despawn themselves).
        // The Parallel parent itself doesn't "own" children in ECS hierarchy (Transform parent),
        // it just tracks them via Entity ID.
        // So despawn() is fine.
        commands.entity(entity).despawn();
    }
}

fn process_wait_chapter_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut WaitTimer), Without<ChapterFinished>>,
    time: Res<Time>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.is_finished() {
            commands.entity(entity).insert(ChapterFinished);
            info!("Wait Chapter finished.");
        }
    }
}

fn process_camera_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    mut camera_query: Query<
        (Entity, &mut Transform, &mut Projection),
        With<crate::app_state::battle::BattleCamera>,
    >,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::SetCamera(action) = &active_chapter.chapter {
            for (_cam_entity, mut transform, mut proj) in camera_query.iter_mut() {
                match action {
                    super::chapter::CameraAction::SetPosition(pos) => {
                        transform.translation = pos.extend(transform.translation.z);
                    }
                    super::chapter::CameraAction::SetZoom(zoom) => {
                        if let Projection::Orthographic(ortho) = &mut *proj {
                            ortho.scale = *zoom;
                        }
                    }
                    _ => {
                        warn!("Camera action {:?} not implemented yet", action);
                    }
                }
            }
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

fn process_ui_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::SetUI(action) = &active_chapter.chapter {
            match action {
                super::chapter::UIAction::LoadLayout(path) => {
                    let handle = asset_server.load(path);
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
                    commands.init_resource::<crate::core::ui::UILayoutWatcher>();
                }
                _ => {
                    warn!("UI action {:?} not fully implemented yet", action);
                }
            }
            commands.entity(entity).insert(ChapterFinished);
        } else if let Chapter::UIInteraction { ui_layout } = &active_chapter.chapter {
            info!("[Battle] Loading UI layout for battle: {}", ui_layout);
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
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

fn process_bullet_pattern_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    mut pattern_events: bevy::ecs::message::MessageWriter<SpawnPatternEvent>,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::BulletPattern { blueprints, count } = &active_chapter.chapter {
            for blueprint_path in blueprints {
                info!("[Battle] Spawning bullet pattern from: {}", blueprint_path);
                let mut event = SpawnPatternEvent::new(blueprint_path.clone());
                if let Some(c) = count {
                    event = event.with_count(*c);
                }
                pattern_events.write(event);
            }
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

/// V2: System to process DanmakuPerformance chapters.
///
/// V2: 处理弹幕演出章节的系统。
fn process_danmaku_performance_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    mut performance_events: bevy::ecs::message::MessageWriter<PlayPerformanceEvent>,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::DanmakuPerformance {
            performance,
            position,
        } = &active_chapter.chapter
        {
            info!(
                "[Battle] Starting danmaku performance from: {}",
                performance
            );
            let mut event = PlayPerformanceEvent::new(performance.clone());
            if let Some((x, y)) = position {
                event = event.at_position(Vec2::new(*x, *y));
            }
            performance_events.write(event);
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

fn process_player_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    asset_server: Res<AssetServer>,
    mut player_query: Query<
        &mut Transform,
        (
            With<BehaviorParams>,
            With<crate::app_state::battle::BattleEntity>,
        ),
    >,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::SetPlayer(action) = &active_chapter.chapter {
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
                _ => {}
            }
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

#[derive(Component)]
struct PlayerSpawnRequest {
    config_handle: Handle<BattlePlayerConfig>,
    position: Vec2,
}

fn process_player_spawn_requests(
    mut commands: Commands,
    query: Query<(Entity, &PlayerSpawnRequest)>,
    configs: Res<Assets<BattlePlayerConfig>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, req) in query.iter() {
        if let Some(config) = configs.get(&req.config_handle) {
            info!("Config loaded. Spawning player...");

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
                BehaviorParams {
                    mode_id: config.default_mode_id.clone(),
                },
                BehaviorVelocity::default(),
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

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

/// Module for the battle sequencer.
///
/// 战斗系统的线性序列管理器。
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
                    process_modify_view_element_system,
                    process_danmaku_performance_system,
                    process_am_performance_system,
                    process_player_spawn_requests,
                    process_wait_chapter_system,
                    process_am_wait_chapter_system,
                    process_parallel_chapter_system,
                    cleanup_finished_chapters_system,
                    sync_battle_flow_system,
                )
                    .chain()
                    .in_set(BattleUpdate),
            );
    }
}

use super::am_integration::{AmPerformanceState, PlayAmPerformanceEvent};
use super::chapter_schema::{Chapter, PlayerAction};
use super::danmaku::PlayPerformanceEvent;
use crate::app_state::AppState;
use crate::app_state::battle::player_config_schema::{BattlePlayerConfig, ColliderShape};
use crate::app_state::battle::{BattleAsset, BattleUpdate};
use crate::core::collision::PhysicsCollider;
use crate::core::danmaku::BulletTarget;
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

/// System to load the default chapter resource.
///
/// 加载默认章节资源的系统。
fn load_default_chapter_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    souprune_config: Res<crate::config::SoupruneConfig>,
) {
    let chapter_path = &souprune_config.game.initial_battle_path;
    let handle = asset_server.load::<BattleAsset>(chapter_path);
    commands.insert_resource(CurrentBattleFlow(handle));
    info!("Loading default battle flow: {}", chapter_path);
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
        Chapter::Sequence(children) => {
            if parent.is_some() {
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

/// System to advance the battle flow.
///
/// 推进战斗流程系统。
fn advance_battle_flow_system(
    mut commands: Commands,
    mut context: ResMut<BattleContext>,
    active_chapters: Query<&ActiveChapter>,
) {
    // Check if any root-level chapter is active
    // 检查是否有任何根级章节处于活动状态
    for chapter in active_chapters.iter() {
        if chapter.parent.is_none() {
            return;
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
        if let Some(parent_entity) = chapter.parent
            && let Ok(mut tracker) = parallel_parents.get_mut(parent_entity)
        {
            tracker.pending_count = tracker.pending_count.saturating_sub(1);
            if tracker.pending_count == 0 {
                // Parent finished!
                commands.entity(parent_entity).insert(ChapterFinished);
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
#[allow(clippy::type_complexity)]
fn process_camera_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    mut camera_query: Query<
        (Entity, &mut Transform, &mut Projection),
        With<crate::app_state::battle::BattleCamera>,
    >,
    resolution_scale: Res<crate::app_state::app_setup::ResolutionScale>,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::SetCamera(action) = &active_chapter.chapter {
            for (_cam_entity, mut transform, mut proj) in camera_query.iter_mut() {
                match action {
                    super::chapter_schema::CameraAction::SetPosition(pos) => {
                        transform.translation = pos.extend(transform.translation.z);
                    }
                    super::chapter_schema::CameraAction::SetZoom(zoom) => {
                        if let Projection::Orthographic(ortho) = &mut *proj {
                            // Apply zoom relative to base resolution scale
                            // 相对于基础分辨率缩放应用缩放
                            ortho.scale = *zoom / resolution_scale.get() as f32;
                            info!(
                                "[Battle] SetZoom: requested={}, actual={}",
                                zoom, ortho.scale
                            );
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

#[allow(clippy::type_complexity)]
fn process_ui_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::SetUI(action) = &active_chapter.chapter {
            match action {
                super::chapter_schema::UIAction::LoadLayout(path) => {
                    let handle = asset_server.load(path);
                    commands.insert_resource(crate::core::ui::UILayoutHandle {
                        handle,
                        last_modified: None,
                        path: path.clone(),
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
                        crate::app_state::battle::BattleEntity,
                        Name::new("BattleUI Root"),
                    ));
                    commands.init_resource::<crate::core::ui::UILayoutWatcher>();
                }
                _ => {
                    warn!("UI action {:?} not fully implemented yet", action);
                }
            }
            commands.entity(entity).insert(ChapterFinished);
        } else if let Chapter::ViewInteraction { view_layout }
        | Chapter::ViewInteraction { view_layout } = &active_chapter.chapter
        {
            info!("[Battle] Loading view layout for battle: {}", view_layout);
            let handle = asset_server.load(view_layout);
            commands.insert_resource(crate::core::ui::UILayoutHandle {
                handle,
                last_modified: None,
                path: view_layout.clone(),
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
                crate::app_state::battle::BattleEntity,
                Name::new("BattleUI Root"),
            ));
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

/// System to process DanmakuPerformance chapters.
///
/// 处理弹幕演出章节的系统。
#[allow(clippy::type_complexity)]
fn process_danmaku_performance_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    mut performance_events: MessageWriter<PlayPerformanceEvent>,
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

#[allow(clippy::type_complexity)]
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
                        crate::app_state::battle::BattleEntity,
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
                ColliderShape::Circle { radius } => PhysicsCollider::Circle { radius: *radius },
                ColliderShape::Box { half_size } => PhysicsCollider::Box {
                    half_size: *half_size,
                },
            };

            let damage_trigger = match &config.damage_trigger.shape {
                ColliderShape::Circle { radius } => {
                    crate::core::collision::TriggerCollider::Circle { radius: *radius }
                }
                ColliderShape::Box { half_size } => crate::core::collision::TriggerCollider::Box {
                    half_size: *half_size,
                },
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
                BulletTarget::new(),
                crate::app_state::battle::BattleEntity,
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

/// Marker component for AM performance chapter tracking
#[derive(Component)]
struct AmPerformanceTracker {
    wait_for_completion: bool,
    /// Whether we've seen the performance start (is_playing became true)
    started: bool,
}

/// System to process AmPerformance chapters.
///
/// 处理 AM 演出章节的系统。
#[allow(clippy::type_complexity)]
fn process_am_performance_system(
    mut commands: Commands,
    query: Query<
        (Entity, &ActiveChapter),
        (
            Without<WaitTimer>,
            Without<ChapterFinished>,
            Without<AmPerformanceTracker>,
        ),
    >,
    mut performance_events: MessageWriter<PlayAmPerformanceEvent>,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::AmPerformance {
            amproj_path,
            wait_for_completion,
        } = &active_chapter.chapter
        {
            info!("[Battle] Starting AM performance from: {}", amproj_path);

            // Send event to start the AM performance
            performance_events.write(PlayAmPerformanceEvent::new(amproj_path.clone()));

            if *wait_for_completion {
                // Add tracker component to wait for completion
                commands.entity(entity).insert(AmPerformanceTracker {
                    wait_for_completion: true,
                    started: false,
                });
            } else {
                // Not waiting, mark as finished immediately
                commands.entity(entity).insert(ChapterFinished);
            }
        }
    }
}

/// System to check if AM performance has completed and finish the chapter.
///
/// 检查 AM 演出是否完成并结束章节的系统。
fn process_am_wait_chapter_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AmPerformanceTracker), Without<ChapterFinished>>,
    am_state: Res<AmPerformanceState>,
) {
    for (entity, mut tracker) in query.iter_mut() {
        if !tracker.wait_for_completion {
            continue;
        }

        // Wait for performance to start first
        if am_state.is_playing {
            tracker.started = true;
        }

        // Only mark finished after performance has started and then stopped
        if tracker.started && !am_state.is_playing {
            info!("[Battle] AM performance chapter finished");
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

// ============================================================================
// Phase 3 & 4: Process ModifyViewElement and TweenViewElement
// Phase 3 & 4: 处理 ModifyViewElement 和 TweenViewElement
// ============================================================================

fn process_modify_view_element_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    active_chapters: Query<
        (Entity, &ActiveChapter),
        (Without<WaitTimer>, Without<ChapterFinished>),
    >,
    view_elements: Query<(Entity, &crate::core::ui::components::ViewElement)>,
    mut transforms: Query<&mut Transform>,
    mut sprites: Query<&mut Sprite>,
    mut visibilities: Query<&mut Visibility>,
    mut histories: Query<&mut crate::core::ui::ViewElementHistory>,
) {
    for (chapter_entity, active_chapter) in active_chapters.iter() {
        if let Chapter::ModifyViewElement {
            selector,
            modification,
        } = &active_chapter.chapter
        {
            info!(
                "[ModifyViewElement] Processing: selector={:?}, modification={:?}",
                selector, modification
            );

            // Resolve the selector to get target entities
            // 解析选择器以获取目标实体
            let target_entities = match selector {
                super::chapter_schema::ElementSelector::FullName(full_name) => {
                    if let Some(entity) =
                        crate::core::ui::find_element_by_full_name(&view_elements, full_name)
                    {
                        info!(
                            "[ModifyViewElement] Found element: {:?} (full_name={})",
                            entity, full_name
                        );
                        vec![entity]
                    } else {
                        warn!(
                            "[ModifyViewElement] Element not found (full name): {}",
                            full_name
                        );
                        vec![]
                    }
                }
                super::chapter_schema::ElementSelector::LocalName(local_name) => {
                    // For simplicity, search in all namespaces
                    // 为简单起见，在所有命名空间中搜索
                    view_elements
                        .iter()
                        .filter(|(_, elem)| elem.local_name == *local_name)
                        .map(|(entity, _)| entity)
                        .collect()
                }
                super::chapter_schema::ElementSelector::Tag(tag) => {
                    crate::core::ui::find_elements_by_tag(&view_elements, tag)
                }
            };

            // Apply the modification to all target entities
            // 对所有目标实体应用修改
            for entity in target_entities {
                info!(
                    "[ModifyViewElement] Applying modification to entity {:?}",
                    entity
                );

                match modification {
                    super::chapter_schema::ElementModification::SetTexture(path) => {
                        if let Ok(mut sprite) = sprites.get_mut(entity) {
                            let texture_path = if path.starts_with("textures/") {
                                path.clone()
                            } else {
                                format!("textures/{}", path)
                            };
                            sprite.image = asset_server.load(&texture_path);
                            info!("Set texture for entity {:?}: {}", entity, texture_path);
                        }
                    }
                    super::chapter_schema::ElementModification::SetPosition(x, y, z) => {
                        if let Ok(mut transform) = transforms.get_mut(entity) {
                            // Ensure history exists or create it
                            // 确保历史存在或创建它
                            let history_exists = histories.get_mut(entity).is_ok();
                            if !history_exists {
                                let original_state = crate::core::ui::ElementState::capture(
                                    Some(&*transform),
                                    sprites.get(entity).ok(),
                                    visibilities.get(entity).ok(),
                                );
                                commands.entity(entity).insert(
                                    crate::core::ui::ViewElementHistory::new(original_state),
                                );
                            }

                            // Apply modification
                            // 应用修改
                            transform.translation = Vec3::new(*x, *y, *z);

                            // Push NEW state to history AFTER modification
                            // 在修改后将新状态推送到历史
                            if let Ok(mut history) = histories.get_mut(entity) {
                                let new_state = crate::core::ui::ElementState::capture(
                                    Some(&*transform),
                                    sprites.get(entity).ok(),
                                    visibilities.get(entity).ok(),
                                );
                                history.push(new_state);
                            }
                        }
                    }
                    super::chapter_schema::ElementModification::SetScale(x, y, z) => {
                        if let Ok(mut transform) = transforms.get_mut(entity) {
                            transform.scale = Vec3::new(*x, *y, *z);
                            info!("Set scale for entity {:?}: ({}, {}, {})", entity, x, y, z);
                        }
                    }
                    super::chapter_schema::ElementModification::SetColor(r, g, b, a) => {
                        if let Ok(mut sprite) = sprites.get_mut(entity) {
                            sprite.color = Color::srgba(*r, *g, *b, *a);
                            info!(
                                "Set color for entity {:?}: ({}, {}, {}, {})",
                                entity, r, g, b, a
                            );
                        }
                    }
                    super::chapter_schema::ElementModification::SetVisibility(visible) => {
                        if let Ok(mut visibility) = visibilities.get_mut(entity) {
                            *visibility = if *visible {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            };
                            info!("Set visibility for entity {:?}: {}", entity, visible);
                        }
                    }
                    super::chapter_schema::ElementModification::SetPositionRandom(
                        base_y,
                        base_z,
                        range,
                    ) => {
                        // Generate random offset using current time as seed
                        // 使用当前时间作为种子生成随机偏移
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let nanos = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .subsec_nanos();

                        // Simple pseudo-random using nanos (only for Y axis)
                        // 使用纳秒进行简单伪随机（仅用于 Y 轴）
                        let rand_y = ((nanos % 1000) as f32 / 1000.0) * 2.0 - 1.0; // -1.0 to 1.0

                        let final_y = base_y + rand_y * range;
                        let final_z = *base_z;

                        if let Ok(mut transform) = transforms.get_mut(entity) {
                            // X coordinate uses current value
                            // X 坐标使用当前值
                            let final_x = transform.translation.x;

                            // Ensure history exists or create it
                            // 确保历史存在或创建它
                            let history_exists = histories.get_mut(entity).is_ok();
                            if !history_exists {
                                let original_state = crate::core::ui::ElementState::capture(
                                    Some(&*transform),
                                    sprites.get(entity).ok(),
                                    visibilities.get(entity).ok(),
                                );
                                commands.entity(entity).insert(
                                    crate::core::ui::ViewElementHistory::new(original_state),
                                );
                            }

                            // Apply modification
                            // 应用修改
                            transform.translation = Vec3::new(final_x, final_y, final_z);

                            // Push NEW state to history AFTER modification
                            // 在修改后将新状态推送到历史
                            if let Ok(mut history) = histories.get_mut(entity) {
                                let new_state = crate::core::ui::ElementState::capture(
                                    Some(&*transform),
                                    sprites.get(entity).ok(),
                                    visibilities.get(entity).ok(),
                                );
                                history.push(new_state);
                            }
                        }
                    }
                    super::chapter_schema::ElementModification::Undo => {
                        if let Ok(mut history) = histories.get_mut(entity) {
                            if let Some(previous_state) = history.undo() {
                                // Apply previous state
                                // 应用之前的状态
                                if let Some((trans, rot, scale)) = previous_state.transform {
                                    if let Ok(mut transform) = transforms.get_mut(entity) {
                                        transform.translation = trans;
                                        transform.rotation = rot;
                                        transform.scale = scale;
                                    }
                                }
                                if let Some(color) = previous_state.color {
                                    if let Ok(mut sprite) = sprites.get_mut(entity) {
                                        sprite.color = color;
                                    }
                                }
                                if let Some(vis) = previous_state.visibility {
                                    if let Ok(mut visibility) = visibilities.get_mut(entity) {
                                        *visibility = vis;
                                    }
                                }
                            }
                        }
                    }
                    super::chapter_schema::ElementModification::Redo => {
                        if let Ok(mut history) = histories.get_mut(entity) {
                            if let Some(next_state) = history.redo() {
                                // Apply next state
                                // 应用下一个状态
                                if let Some((trans, rot, scale)) = next_state.transform {
                                    if let Ok(mut transform) = transforms.get_mut(entity) {
                                        transform.translation = trans;
                                        transform.rotation = rot;
                                        transform.scale = scale;
                                    }
                                }
                                if let Some(color) = next_state.color {
                                    if let Ok(mut sprite) = sprites.get_mut(entity) {
                                        sprite.color = color;
                                    }
                                }
                                if let Some(vis) = next_state.visibility {
                                    if let Ok(mut visibility) = visibilities.get_mut(entity) {
                                        *visibility = vis;
                                    }
                                }
                            }
                        }
                    }
                    super::chapter_schema::ElementModification::Reset => {
                        if let Ok(mut history) = histories.get_mut(entity) {
                            let original_state = history.reset();
                            // Apply original state
                            // 应用原始状态
                            if let Some((trans, rot, scale)) = original_state.transform {
                                if let Ok(mut transform) = transforms.get_mut(entity) {
                                    transform.translation = trans;
                                    transform.rotation = rot;
                                    transform.scale = scale;
                                }
                            }
                            if let Some(color) = original_state.color {
                                if let Ok(mut sprite) = sprites.get_mut(entity) {
                                    sprite.color = color;
                                }
                            }
                            if let Some(vis) = original_state.visibility {
                                if let Ok(mut visibility) = visibilities.get_mut(entity) {
                                    *visibility = vis;
                                }
                            }
                        }
                    }
                }
            }

            // Mark chapter as finished
            // 标记章节为完成
            commands.entity(chapter_entity).insert(ChapterFinished);
        }
    }
}

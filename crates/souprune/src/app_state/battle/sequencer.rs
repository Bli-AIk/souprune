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
                    process_await_view_interaction_system,
                    process_modify_view_element_system,
                    process_tween_view_element_system,
                    process_danmaku_performance_system,
                    process_am_performance_system,
                    process_player_spawn_requests,
                    process_wait_chapter_system,
                    process_tween_wait_chapter_system,
                    process_am_wait_chapter_system,
                    process_parallel_chapter_system,
                    check_await_interaction_completion_system,
                    update_interactive_layer_sprites_system,
                    cleanup_finished_chapters_system,
                    sync_battle_flow_system,
                )
                    .chain()
                    .in_set(BattleUpdate),
            );
    }
}

use super::am_integration::{AmPerformanceState, PlayAmPerformanceEvent};
use super::chapter_schema::{Chapter, EasingFunction, PlayerAction, TweenTarget, Val};
use super::danmaku::PlayPerformanceEvent;
use crate::app_state::AppState;
use crate::app_state::battle::player_config_schema::{BattlePlayerConfig, ColliderShape};
use crate::app_state::battle::{BattleAsset, BattleUpdate};
use crate::core::collision::PhysicsCollider;
use crate::core::danmaku::BulletTarget;
use crate::core::mod_system::{BehaviorParams, BehaviorVelocity};
use crate::core::view::components::{
    AwaitingInteraction, InteractionResult, InteractiveLayer, UIBox,
};
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
                    commands.insert_resource(crate::core::view::UILayoutHandle {
                        handle,
                        last_modified: None,
                        path: path.clone(),
                    });
                    commands.spawn((
                        crate::core::view::components::RonUI::new(
                            crate::core::view::components::UILayer::BACKPACK_MENU,
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
                    commands.init_resource::<crate::core::view::UILayoutWatcher>();
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
            commands.insert_resource(crate::core::view::UILayoutHandle {
                handle,
                last_modified: None,
                path: view_layout.clone(),
            });
            commands.spawn((
                crate::core::view::components::RonUI::new(
                    crate::core::view::components::UILayer::BACKPACK_MENU,
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
            translation,
        } = &active_chapter.chapter
        {
            info!(
                "[Battle] Starting danmaku performance from: {}",
                performance
            );
            let mut event = PlayPerformanceEvent::new(performance.clone());
            if let Some((x, y)) = translation {
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
// ModifyViewElement and TweenViewElement Systems
// ModifyViewElement 和 TweenViewElement 系统
// ============================================================================

fn process_modify_view_element_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    active_chapters: Query<
        (Entity, &ActiveChapter),
        (Without<WaitTimer>, Without<ChapterFinished>),
    >,
    view_elements: Query<(Entity, &crate::core::view::components::ViewElement)>,
    mut transforms: Query<&mut Transform>,
    mut sprites: Query<&mut Sprite>,
    mut visibilities: Query<&mut Visibility>,
    mut histories: Query<&mut crate::core::view::ViewElementHistory>,
    mut ui_boxes: Query<&mut UIBox>,
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
                        crate::core::view::find_element_by_full_name(&view_elements, full_name)
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
                    crate::core::view::find_elements_by_tag(&view_elements, tag)
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
                            use crate::core::view::ron_view::parsing::resolve_val_f32;
                            let player_data = crate::core::data::PlayerData::default();

                            let final_x = resolve_val_f32(
                                x,
                                Some(transform.translation.x),
                                &player_data,
                                None,
                            );
                            let final_y = resolve_val_f32(
                                y,
                                Some(transform.translation.y),
                                &player_data,
                                None,
                            );
                            let final_z = resolve_val_f32(
                                z,
                                Some(transform.translation.z),
                                &player_data,
                                None,
                            );

                            // Ensure history exists or create it
                            // 确保历史存在或创建它
                            let history_exists = histories.get_mut(entity).is_ok();
                            if !history_exists {
                                let original_state = crate::core::view::ElementState::capture(
                                    Some(&*transform),
                                    sprites.get(entity).ok(),
                                    visibilities.get(entity).ok(),
                                );
                                commands.entity(entity).insert(
                                    crate::core::view::ViewElementHistory::new(original_state),
                                );
                            }

                            // Apply modification
                            // 应用修改
                            transform.translation = Vec3::new(final_x, final_y, final_z);
                            info!(
                                "Set position for entity {:?}: ({}, {}, {})",
                                entity, final_x, final_y, final_z
                            );

                            // Push NEW state to history AFTER modification
                            // 在修改后将新状态推送到历史
                            if let Ok(mut history) = histories.get_mut(entity) {
                                let new_state = crate::core::view::ElementState::capture(
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
                            use crate::core::view::ron_view::parsing::resolve_val_f32;
                            let player_data = crate::core::data::PlayerData::default();

                            let final_x =
                                resolve_val_f32(x, Some(transform.scale.x), &player_data, None);
                            let final_y =
                                resolve_val_f32(y, Some(transform.scale.y), &player_data, None);
                            let final_z =
                                resolve_val_f32(z, Some(transform.scale.z), &player_data, None);

                            transform.scale = Vec3::new(final_x, final_y, final_z);
                            info!(
                                "Set scale for entity {:?}: ({}, {}, {})",
                                entity, final_x, final_y, final_z
                            );
                        }
                    }
                    super::chapter_schema::ElementModification::SetColor(r, g, b, a) => {
                        if let Ok(mut sprite) = sprites.get_mut(entity) {
                            use crate::core::view::ron_view::parsing::resolve_val_f32;
                            let player_data = crate::core::data::PlayerData::default();
                            let color = sprite.color;

                            let final_r =
                                resolve_val_f32(r, Some(color.to_srgba().red), &player_data, None);
                            let final_g = resolve_val_f32(
                                g,
                                Some(color.to_srgba().green),
                                &player_data,
                                None,
                            );
                            let final_b =
                                resolve_val_f32(b, Some(color.to_srgba().blue), &player_data, None);
                            let final_a = resolve_val_f32(
                                a,
                                Some(color.to_srgba().alpha),
                                &player_data,
                                None,
                            );

                            sprite.color = Color::srgba(final_r, final_g, final_b, final_a);
                            info!(
                                "Set color for entity {:?}: ({}, {}, {}, {})",
                                entity, final_r, final_g, final_b, final_a
                            );
                        }
                    }
                    super::chapter_schema::ElementModification::SetVisibility(visible) => {
                        if let Ok(mut visibility) = visibilities.get_mut(entity) {
                            use crate::core::view::ron_view::parsing::resolve_val_bool;
                            let player_data = crate::core::data::PlayerData::default();

                            let is_visible = resolve_val_bool(visible, &player_data);
                            *visibility = if is_visible {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            };
                            info!("Set visibility for entity {:?}: {}", entity, is_visible);
                        }
                    }
                    super::chapter_schema::ElementModification::SetBoxSize(width, height) => {
                        if let Ok(mut ui_box) = ui_boxes.get_mut(entity) {
                            use crate::core::view::ron_view::parsing::resolve_val_f32;
                            let player_data = crate::core::data::PlayerData::default();

                            let w = resolve_val_f32(width, None, &player_data, None);
                            let h = resolve_val_f32(height, None, &player_data, None);
                            ui_box.width = w;
                            ui_box.height = h;
                            info!("Set box size for entity {:?}: {}x{}", entity, w, h);
                        }
                    }
                    super::chapter_schema::ElementModification::Undo => {
                        if let Ok(mut history) = histories.get_mut(entity)
                            && let Some(previous_state) = history.undo()
                        {
                            // Apply previous state
                            // 应用之前的状态
                            if let Some((trans, rot, scale)) = previous_state.transform
                                && let Ok(mut transform) = transforms.get_mut(entity)
                            {
                                transform.translation = trans;
                                transform.rotation = rot;
                                transform.scale = scale;
                            }
                            if let Some(color) = previous_state.color
                                && let Ok(mut sprite) = sprites.get_mut(entity)
                            {
                                sprite.color = color;
                            }
                            if let Some(vis) = previous_state.visibility
                                && let Ok(mut visibility) = visibilities.get_mut(entity)
                            {
                                *visibility = vis;
                            }
                        }
                    }
                    super::chapter_schema::ElementModification::Redo => {
                        if let Ok(mut history) = histories.get_mut(entity)
                            && let Some(next_state) = history.redo()
                        {
                            // Apply next state
                            // 应用下一个状态
                            if let Some((trans, rot, scale)) = next_state.transform
                                && let Ok(mut transform) = transforms.get_mut(entity)
                            {
                                transform.translation = trans;
                                transform.rotation = rot;
                                transform.scale = scale;
                            }
                            if let Some(color) = next_state.color
                                && let Ok(mut sprite) = sprites.get_mut(entity)
                            {
                                sprite.color = color;
                            }
                            if let Some(vis) = next_state.visibility
                                && let Ok(mut visibility) = visibilities.get_mut(entity)
                            {
                                *visibility = vis;
                            }
                        }
                    }
                    super::chapter_schema::ElementModification::Reset => {
                        if let Ok(mut history) = histories.get_mut(entity) {
                            let original_state = history.reset();
                            // Apply original state
                            // 应用原始状态
                            if let Some((trans, rot, scale)) = original_state.transform
                                && let Ok(mut transform) = transforms.get_mut(entity)
                            {
                                transform.translation = trans;
                                transform.rotation = rot;
                                transform.scale = scale;
                            }
                            if let Some(color) = original_state.color
                                && let Ok(mut sprite) = sprites.get_mut(entity)
                            {
                                sprite.color = color;
                            }
                            if let Some(vis) = original_state.visibility
                                && let Ok(mut visibility) = visibilities.get_mut(entity)
                            {
                                *visibility = vis;
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

// =============================================================================
// Tween View Element System
// 补间视图元素系统
// =============================================================================

/// Component to track an active tween animation.
///
/// 跟踪活动补间动画的组件。
#[derive(Component)]
struct ActiveTween {
    /// Target entity being animated.
    target_entity: Entity,
    /// Start values for interpolation.
    start_value: TweenValue,
    /// End values for interpolation.
    end_value: TweenValue,
    /// Animation timer.
    timer: Timer,
    /// Easing function.
    easing: EasingFunction,
    /// Whether to wait for completion.
    wait_for_completion: bool,
}

/// Interpolatable value types for tween animations.
///
/// 用于补间动画的可插值值类型。
#[derive(Debug, Clone)]
enum TweenValue {
    /// Box size (width, height).
    BoxSize(f32, f32),
    /// Position (x, y, z).
    Position(Vec3),
    /// Scale (x, y, z).
    Scale(Vec3),
    /// Color (r, g, b, a).
    Color(Vec4),
    /// Rotation (radians).
    Rotation(f32),
    /// Alpha only.
    Alpha(f32),
}

impl TweenValue {
    /// Interpolate between two values.
    fn lerp(&self, other: &TweenValue, t: f32) -> TweenValue {
        match (self, other) {
            (TweenValue::BoxSize(w1, h1), TweenValue::BoxSize(w2, h2)) => {
                TweenValue::BoxSize(w1 + (w2 - w1) * t, h1 + (h2 - h1) * t)
            }
            (TweenValue::Position(v1), TweenValue::Position(v2)) => {
                TweenValue::Position(v1.lerp(*v2, t))
            }
            (TweenValue::Scale(v1), TweenValue::Scale(v2)) => TweenValue::Scale(v1.lerp(*v2, t)),
            (TweenValue::Color(v1), TweenValue::Color(v2)) => TweenValue::Color(v1.lerp(*v2, t)),
            (TweenValue::Rotation(r1), TweenValue::Rotation(r2)) => {
                TweenValue::Rotation(r1 + (r2 - r1) * t)
            }
            (TweenValue::Alpha(a1), TweenValue::Alpha(a2)) => TweenValue::Alpha(a1 + (a2 - a1) * t),
            _ => self.clone(), // Mismatched types, return start value
        }
    }
}

/// Apply easing function to a linear progress value (0.0 to 1.0).
///
/// 对线性进度值（0.0 到 1.0）应用缓动函数。
fn apply_easing(t: f32, easing: EasingFunction) -> f32 {
    match easing {
        EasingFunction::Linear => t,
        EasingFunction::QuadIn => t * t,
        EasingFunction::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
        EasingFunction::QuadInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
        EasingFunction::CubicIn => t * t * t,
        EasingFunction::CubicOut => 1.0 - (1.0 - t).powi(3),
        EasingFunction::CubicInOut => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
            }
        }
        EasingFunction::SineIn => 1.0 - (t * std::f32::consts::FRAC_PI_2).cos(),
        EasingFunction::SineOut => (t * std::f32::consts::FRAC_PI_2).sin(),
        EasingFunction::SineInOut => -(((t * std::f32::consts::PI).cos() - 1.0) / 2.0),
        EasingFunction::ExpoIn => {
            if t == 0.0 {
                0.0
            } else {
                2.0_f32.powf(10.0 * t - 10.0)
            }
        }
        EasingFunction::ExpoOut => {
            if t == 1.0 {
                1.0
            } else {
                1.0 - 2.0_f32.powf(-10.0 * t)
            }
        }
        EasingFunction::ExpoInOut => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else if t < 0.5 {
                2.0_f32.powf(20.0 * t - 10.0) / 2.0
            } else {
                (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
            }
        }
        EasingFunction::ElasticIn => {
            let c4 = (2.0 * std::f32::consts::PI) / 3.0;
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                -2.0_f32.powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * c4).sin()
            }
        }
        EasingFunction::ElasticOut => {
            let c4 = (2.0 * std::f32::consts::PI) / 3.0;
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
            }
        }
        EasingFunction::ElasticInOut => {
            let c5 = (2.0 * std::f32::consts::PI) / 4.5;
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else if t < 0.5 {
                -(2.0_f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
            } else {
                (2.0_f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0 + 1.0
            }
        }
        EasingFunction::BounceIn => 1.0 - apply_easing(1.0 - t, EasingFunction::BounceOut),
        EasingFunction::BounceOut => {
            let n1 = 7.5625;
            let d1 = 2.75;
            if t < 1.0 / d1 {
                n1 * t * t
            } else if t < 2.0 / d1 {
                let t = t - 1.5 / d1;
                n1 * t * t + 0.75
            } else if t < 2.5 / d1 {
                let t = t - 2.25 / d1;
                n1 * t * t + 0.9375
            } else {
                let t = t - 2.625 / d1;
                n1 * t * t + 0.984375
            }
        }
        EasingFunction::BounceInOut => {
            if t < 0.5 {
                (1.0 - apply_easing(1.0 - 2.0 * t, EasingFunction::BounceOut)) / 2.0
            } else {
                (1.0 + apply_easing(2.0 * t - 1.0, EasingFunction::BounceOut)) / 2.0
            }
        }
        EasingFunction::BackIn => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            c3 * t * t * t - c1 * t * t
        }
        EasingFunction::BackOut => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
        }
        EasingFunction::BackInOut => {
            let c1 = 1.70158;
            let c2 = c1 * 1.525;
            if t < 0.5 {
                ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
            } else {
                ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
            }
        }
    }
}

/// Helper to resolve a Val<f32> to an f32 value using the unified expression system.
/// Uses the `resolve_val_f32` function from `parsing.rs` for full expression support.
///
/// 使用统一的表达式系统解析 Val<f32> 为 f32 值。
/// 使用 `parsing.rs` 中的 `resolve_val_f32` 函数以支持完整的表达式功能。
fn resolve_tween_val_f32(
    val: &Val<f32>,
    current: f32,
    player_data: &crate::core::data::PlayerData,
    time: Option<f64>,
) -> f32 {
    crate::core::view::ron_view::parsing::resolve_val_f32(val, Some(current), player_data, time)
}

/// System to process TweenViewElement chapters.
///
/// 处理 TweenViewElement 章节的系统。
#[allow(clippy::type_complexity)]
fn process_tween_view_element_system(
    mut commands: Commands,
    active_chapters: Query<
        (Entity, &ActiveChapter),
        (
            Without<ActiveTween>,
            Without<WaitTimer>,
            Without<ChapterFinished>,
        ),
    >,
    view_elements: Query<(Entity, &crate::core::view::components::ViewElement)>,
    transforms: Query<&Transform>,
    sprites: Query<&Sprite>,
    ui_boxes: Query<&UIBox>,
    player_data: Res<crate::core::data::PlayerData>,
    time: Res<Time>,
) {
    // Get current elapsed time for expression evaluation
    let current_time = time.elapsed_secs_f64();

    for (chapter_entity, active_chapter) in active_chapters.iter() {
        if let Chapter::TweenViewElement {
            selector,
            target,
            duration,
            easing,
            wait_for_completion,
        } = &active_chapter.chapter
        {
            info!(
                "[TweenViewElement] Processing: selector={:?}, target={:?}, duration={}s",
                selector, target, duration
            );

            // Resolve the selector to get target entity
            let target_entity = match selector {
                super::chapter_schema::ElementSelector::FullName(full_name) => {
                    crate::core::view::find_element_by_full_name(&view_elements, full_name)
                }
                super::chapter_schema::ElementSelector::LocalName(local_name) => view_elements
                    .iter()
                    .find(|(_, elem)| elem.local_name == *local_name)
                    .map(|(entity, _)| entity),
                super::chapter_schema::ElementSelector::Tag(tag) => {
                    crate::core::view::find_elements_by_tag(&view_elements, tag)
                        .into_iter()
                        .next()
                }
            };

            let Some(target_entity) = target_entity else {
                warn!(
                    "[TweenViewElement] Target element not found: {:?}",
                    selector
                );
                commands.entity(chapter_entity).insert(ChapterFinished);
                continue;
            };

            // Get start and end values based on target type
            let (start_value, end_value) = match target {
                TweenTarget::BoxSize { from, to } => {
                    if let Ok(ui_box) = ui_boxes.get(target_entity) {
                        let current_w = ui_box.width();
                        let current_h = ui_box.height();

                        // Handle optional from value
                        let start = if let Some(from) = from {
                            TweenValue::BoxSize(
                                resolve_tween_val_f32(
                                    &from.0,
                                    current_w,
                                    &player_data,
                                    Some(current_time),
                                ),
                                resolve_tween_val_f32(
                                    &from.1,
                                    current_h,
                                    &player_data,
                                    Some(current_time),
                                ),
                            )
                        } else {
                            TweenValue::BoxSize(current_w, current_h)
                        };

                        let end = TweenValue::BoxSize(
                            resolve_tween_val_f32(
                                &to.0,
                                current_w,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &to.1,
                                current_h,
                                &player_data,
                                Some(current_time),
                            ),
                        );
                        (start, end)
                    } else {
                        warn!("[TweenViewElement] Target has no UIBox component");
                        commands.entity(chapter_entity).insert(ChapterFinished);
                        continue;
                    }
                }
                TweenTarget::Position { from, to } => {
                    if let Ok(transform) = transforms.get(target_entity) {
                        let current = transform.translation;

                        let start = if let Some(from) = from {
                            TweenValue::Position(Vec3::new(
                                resolve_tween_val_f32(
                                    &from.0,
                                    current.x,
                                    &player_data,
                                    Some(current_time),
                                ),
                                resolve_tween_val_f32(
                                    &from.1,
                                    current.y,
                                    &player_data,
                                    Some(current_time),
                                ),
                                resolve_tween_val_f32(
                                    &from.2,
                                    current.z,
                                    &player_data,
                                    Some(current_time),
                                ),
                            ))
                        } else {
                            TweenValue::Position(current)
                        };

                        let end = TweenValue::Position(Vec3::new(
                            resolve_tween_val_f32(
                                &to.0,
                                current.x,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &to.1,
                                current.y,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &to.2,
                                current.z,
                                &player_data,
                                Some(current_time),
                            ),
                        ));
                        (start, end)
                    } else {
                        warn!("[TweenViewElement] Target has no Transform component");
                        commands.entity(chapter_entity).insert(ChapterFinished);
                        continue;
                    }
                }
                TweenTarget::Scale { from, to } => {
                    if let Ok(transform) = transforms.get(target_entity) {
                        let current = transform.scale;

                        let start = if let Some(from) = from {
                            TweenValue::Scale(Vec3::new(
                                resolve_tween_val_f32(
                                    &from.0,
                                    current.x,
                                    &player_data,
                                    Some(current_time),
                                ),
                                resolve_tween_val_f32(
                                    &from.1,
                                    current.y,
                                    &player_data,
                                    Some(current_time),
                                ),
                                resolve_tween_val_f32(
                                    &from.2,
                                    current.z,
                                    &player_data,
                                    Some(current_time),
                                ),
                            ))
                        } else {
                            TweenValue::Scale(current)
                        };

                        let end = TweenValue::Scale(Vec3::new(
                            resolve_tween_val_f32(
                                &to.0,
                                current.x,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &to.1,
                                current.y,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &to.2,
                                current.z,
                                &player_data,
                                Some(current_time),
                            ),
                        ));
                        (start, end)
                    } else {
                        warn!("[TweenViewElement] Target has no Transform component");
                        commands.entity(chapter_entity).insert(ChapterFinished);
                        continue;
                    }
                }
                TweenTarget::Color { from, to } => {
                    if let Ok(sprite) = sprites.get(target_entity) {
                        let c = sprite.color.to_linear();
                        let current = Vec4::new(c.red, c.green, c.blue, c.alpha);

                        let start = if let Some(from) = from {
                            TweenValue::Color(Vec4::new(
                                resolve_tween_val_f32(
                                    &from.0,
                                    current.x,
                                    &player_data,
                                    Some(current_time),
                                ),
                                resolve_tween_val_f32(
                                    &from.1,
                                    current.y,
                                    &player_data,
                                    Some(current_time),
                                ),
                                resolve_tween_val_f32(
                                    &from.2,
                                    current.z,
                                    &player_data,
                                    Some(current_time),
                                ),
                                resolve_tween_val_f32(
                                    &from.3,
                                    current.w,
                                    &player_data,
                                    Some(current_time),
                                ),
                            ))
                        } else {
                            TweenValue::Color(current)
                        };

                        let end = TweenValue::Color(Vec4::new(
                            resolve_tween_val_f32(
                                &to.0,
                                current.x,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &to.1,
                                current.y,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &to.2,
                                current.z,
                                &player_data,
                                Some(current_time),
                            ),
                            resolve_tween_val_f32(
                                &to.3,
                                current.w,
                                &player_data,
                                Some(current_time),
                            ),
                        ));
                        (start, end)
                    } else {
                        warn!("[TweenViewElement] Target has no Sprite component");
                        commands.entity(chapter_entity).insert(ChapterFinished);
                        continue;
                    }
                }
                TweenTarget::Rotation { from, to } => {
                    if let Ok(transform) = transforms.get(target_entity) {
                        let (_, _, z) = transform.rotation.to_euler(EulerRot::XYZ);

                        let start = if let Some(from) = from {
                            TweenValue::Rotation(resolve_tween_val_f32(
                                from,
                                z,
                                &player_data,
                                Some(current_time),
                            ))
                        } else {
                            TweenValue::Rotation(z)
                        };

                        let end = TweenValue::Rotation(resolve_tween_val_f32(
                            to,
                            z,
                            &player_data,
                            Some(current_time),
                        ));
                        (start, end)
                    } else {
                        warn!("[TweenViewElement] Target has no Transform component");
                        commands.entity(chapter_entity).insert(ChapterFinished);
                        continue;
                    }
                }
                TweenTarget::Alpha { from, to } => {
                    if let Ok(sprite) = sprites.get(target_entity) {
                        let current_alpha = sprite.color.alpha();

                        let start = if let Some(from) = from {
                            TweenValue::Alpha(resolve_tween_val_f32(
                                from,
                                current_alpha,
                                &player_data,
                                Some(current_time),
                            ))
                        } else {
                            TweenValue::Alpha(current_alpha)
                        };

                        let end = TweenValue::Alpha(resolve_tween_val_f32(
                            to,
                            current_alpha,
                            &player_data,
                            Some(current_time),
                        ));
                        (start, end)
                    } else {
                        warn!("[TweenViewElement] Target has no Sprite component");
                        commands.entity(chapter_entity).insert(ChapterFinished);
                        continue;
                    }
                }
            };

            info!(
                "[TweenViewElement] Starting tween: {:?} -> {:?}",
                start_value, end_value
            );

            // Create the tween component
            // If not waiting for completion, spawn a separate entity for the tween
            // so it can continue running after the chapter is marked as finished
            if *wait_for_completion {
                commands.entity(chapter_entity).insert(ActiveTween {
                    target_entity,
                    start_value,
                    end_value,
                    timer: Timer::from_seconds(*duration, TimerMode::Once),
                    easing: *easing,
                    wait_for_completion: *wait_for_completion,
                });
            } else {
                // Spawn a detached tween entity that will clean itself up when done
                commands.spawn(ActiveTween {
                    target_entity,
                    start_value,
                    end_value,
                    timer: Timer::from_seconds(*duration, TimerMode::Once),
                    easing: *easing,
                    wait_for_completion: false,
                });
                // Mark chapter as finished immediately
                commands.entity(chapter_entity).insert(ChapterFinished);
            }
        }
    }
}

/// System to update active tween animations.
///
/// 更新活动补间动画的系统。
#[allow(clippy::type_complexity)]
fn process_tween_wait_chapter_system(
    mut commands: Commands,
    time: Res<Time>,
    mut tween_query: Query<(Entity, &mut ActiveTween), Without<ChapterFinished>>,
    mut transforms: Query<&mut Transform>,
    mut sprites: Query<&mut Sprite>,
    mut ui_boxes: Query<&mut UIBox>,
) {
    for (entity, mut tween) in tween_query.iter_mut() {
        tween.timer.tick(time.delta());

        let linear_t = tween.timer.elapsed_secs() / tween.timer.duration().as_secs_f32();
        let eased_t = apply_easing(linear_t.min(1.0), tween.easing);

        // Interpolate and apply the value
        let current = tween.start_value.lerp(&tween.end_value, eased_t);

        match current {
            TweenValue::BoxSize(width, height) => {
                if let Ok(mut ui_box) = ui_boxes.get_mut(tween.target_entity) {
                    ui_box.width = width;
                    ui_box.height = height;
                }
            }
            TweenValue::Position(pos) => {
                if let Ok(mut transform) = transforms.get_mut(tween.target_entity) {
                    transform.translation = pos;
                }
            }
            TweenValue::Scale(scale) => {
                if let Ok(mut transform) = transforms.get_mut(tween.target_entity) {
                    transform.scale = scale;
                }
            }
            TweenValue::Color(color) => {
                if let Ok(mut sprite) = sprites.get_mut(tween.target_entity) {
                    sprite.color = Color::linear_rgba(color.x, color.y, color.z, color.w);
                }
            }
            TweenValue::Rotation(angle) => {
                if let Ok(mut transform) = transforms.get_mut(tween.target_entity) {
                    transform.rotation = Quat::from_rotation_z(angle);
                }
            }
            TweenValue::Alpha(alpha) => {
                if let Ok(mut sprite) = sprites.get_mut(tween.target_entity) {
                    sprite.color = sprite.color.with_alpha(alpha);
                }
            }
        }

        if tween.timer.finished() {
            info!("[TweenViewElement] Tween completed");
            // If this tween is on a chapter entity (wait_for_completion: true),
            // mark the chapter as finished. Otherwise, just despawn the tween entity.
            if tween.wait_for_completion {
                commands.entity(entity).insert(ChapterFinished);
            } else {
                // This is a detached tween entity, despawn it
                commands.entity(entity).despawn();
            }
        }
    }
}

// ============================================================================
// AwaitViewInteraction Systems
// AwaitViewInteraction 系统
// ============================================================================

/// Marker component to track that this chapter is waiting for interaction.
///
/// 标记组件，用于追踪此 Chapter 正在等待交互。
#[derive(Component)]
struct AwaitingInteractionChapter {
    /// The layer ID we're waiting on.
    layer_id: String,
}

/// System to process AwaitViewInteraction chapters.
///
/// 处理 AwaitViewInteraction 章节的系统。
///
/// This system activates the specified interactive layer and marks the chapter
/// as waiting for player input. The chapter won't finish until the player
/// confirms a selection.
///
/// 此系统激活指定的交互层，并将章节标记为等待玩家输入。
/// 章节不会结束，直到玩家确认选择。
#[allow(clippy::type_complexity)]
fn process_await_view_interaction_system(
    mut commands: Commands,
    query: Query<
        (Entity, &ActiveChapter),
        (
            Without<WaitTimer>,
            Without<ChapterFinished>,
            Without<AwaitingInteractionChapter>,
        ),
    >,
    mut layer_query: Query<(Entity, &mut InteractiveLayer)>,
) {
    for (chapter_entity, active_chapter) in query.iter() {
        if let Chapter::AwaitViewInteraction {
            layer_id,
            initial_selection,
        } = &active_chapter.chapter
        {
            info!(
                "[Battle] Starting AwaitViewInteraction for layer '{}' at index {}",
                layer_id, initial_selection
            );

            // Find and activate the interactive layer
            let mut found = false;
            for (layer_entity, mut layer) in layer_query.iter_mut() {
                if layer.layer_id == *layer_id {
                    // Activate the layer
                    layer.is_active = true;
                    layer.set_selection(*initial_selection);

                    // Attach AwaitingInteraction component to the layer
                    commands.entity(layer_entity).insert(AwaitingInteraction {
                        chapter_entity: Some(chapter_entity),
                    });

                    found = true;
                    info!(
                        "[Battle] Activated InteractiveLayer '{}', waiting for player input",
                        layer_id
                    );
                    break;
                }
            }

            if !found {
                warn!(
                    "[Battle] InteractiveLayer '{}' not found! Chapter will complete immediately.",
                    layer_id
                );
                commands.entity(chapter_entity).insert(ChapterFinished);
            } else {
                // Mark this chapter as waiting
                commands
                    .entity(chapter_entity)
                    .insert(AwaitingInteractionChapter {
                        layer_id: layer_id.clone(),
                    });
            }
        }
    }
}

/// System to check for interaction completion and finish the AwaitViewInteraction chapter.
///
/// 检查交互完成情况并结束 AwaitViewInteraction 章节的系统。
///
/// Listens for SelectionConfirmedEvent and marks the corresponding chapter as finished.
///
/// 监听 SelectionConfirmedEvent 并将相应的章节标记为完成。
#[allow(clippy::type_complexity)]
fn check_await_interaction_completion_system(
    mut commands: Commands,
    mut confirm_events: MessageReader<crate::core::view::components::SelectionConfirmedEvent>,
    awaiting_query: Query<(Entity, &AwaitingInteractionChapter)>,
    mut layer_query: Query<(Entity, &mut InteractiveLayer), With<AwaitingInteraction>>,
) {
    for event in confirm_events.read() {
        info!(
            "[Battle] Received SelectionConfirmedEvent for layer '{}', index {}",
            event.layer_id, event.selected_index
        );

        // Find the chapter that is waiting for this layer
        for (chapter_entity, awaiting) in awaiting_query.iter() {
            if awaiting.layer_id == event.layer_id {
                // Store the result on the chapter entity for later use
                commands
                    .entity(chapter_entity)
                    .insert(InteractionResult::new(
                        event.selected_index,
                        event.selected_element.clone(),
                        &event.layer_id,
                    ));

                // Mark the chapter as finished
                commands.entity(chapter_entity).insert(ChapterFinished);
                commands
                    .entity(chapter_entity)
                    .remove::<AwaitingInteractionChapter>();

                info!(
                    "[Battle] AwaitViewInteraction for '{}' completed with selection {}",
                    event.layer_id, event.selected_index
                );
            }
        }

        // Deactivate the layer, reset selection, and remove AwaitingInteraction
        for (layer_entity, mut layer) in layer_query.iter_mut() {
            if layer.layer_id == event.layer_id {
                layer.is_active = false;
                layer.set_selection(0); // Reset to initial selection
                commands
                    .entity(layer_entity)
                    .remove::<AwaitingInteraction>();
                info!(
                    "[Battle] Deactivated InteractiveLayer '{}', reset selection to 0",
                    event.layer_id
                );
            }
        }
    }
}

/// System to update sprites based on InteractiveLayer selection.
///
/// 根据 InteractiveLayer 选择更新精灵的系统。
///
/// This system changes the texture of selectable elements to show which one is
/// currently selected (highlighted). When the layer is deactivated, all elements
/// revert to their unselected state.
///
/// 此系统更改可选元素的纹理以显示当前选中（高亮）的项目。
/// 当层被停用时，所有元素恢复到未选中状态。
///
/// # Convention / 约定
///
/// Button sprites follow the naming convention:
/// - `textures/battle/ui/{button_name}/false.png` - Unselected state
/// - `textures/battle/ui/{button_name}/true.png` - Selected state
///
/// 按钮精灵遵循以下命名约定：
/// - `textures/battle/ui/{按钮名}/false.png` - 未选中状态
/// - `textures/battle/ui/{按钮名}/true.png` - 选中状态
fn update_interactive_layer_sprites_system(
    asset_server: Res<AssetServer>,
    layer_query: Query<&InteractiveLayer, Changed<InteractiveLayer>>,
    mut sprite_query: Query<(&Name, &mut Sprite)>,
) {
    for layer in layer_query.iter() {
        // Update each selectable element's sprite
        // If layer is inactive, all elements show unselected state
        for (idx, element_name) in layer.selectable_elements.iter().enumerate() {
            // Only selected if layer is active AND this is the selected index
            let is_selected = layer.is_active && idx == layer.current_selection;

            // Find the entity with this name
            for (name, mut sprite) in sprite_query.iter_mut() {
                if name.as_str() == element_name {
                    // Determine the button type from the element name (e.g., "BtnFight" -> "fight")
                    let button_type = element_name
                        .strip_prefix("Btn")
                        .map(|s| s.to_lowercase())
                        .unwrap_or_else(|| element_name.to_lowercase());

                    // Build the texture path based on selection state
                    let state = if is_selected { "true" } else { "false" };
                    let texture_path = format!("textures/battle/ui/{}/{}.png", button_type, state);

                    // Load and set the new texture
                    let new_texture: Handle<Image> = asset_server.load(&texture_path);
                    sprite.image = new_texture;
                }
            }
        }
    }
}

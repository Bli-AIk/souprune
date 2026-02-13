//! # overworld.rs
//!
//! # overworld.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! The `overworld` module manages the game's overworld state, including player movement, tilemaps, characters, and UI interactions.
//!
//! `overworld` 模块管理游戏的大地图（Overworld）状态，包括玩家移动、瓦片地图、角色和用户界面交互。
//!
//! It orchestrates sub-plugins and handles camera binding to the player.
//!
//! 它负责协调子插件，并处理相机对玩家的跟随逻辑。

use crate::app_state::cleanup_entities_system;
use crate::core::camera::Followable;
use bevy::app::{App, Plugin};
use bevy::prelude::*;

use crate::core::danmaku::{DanmakuSpawnContext, DanmakuUpdate};
use bevy_fact_rule_event::{LayeredFactDatabase, LayeredRuleRegistry};

pub(crate) mod character;
pub mod chase;
mod collision;
pub(crate) mod player;
pub(crate) mod tilemap;
pub mod trigger;
pub(crate) mod view;

/// Marker component for overworld entities
///
/// 标记 Overworld 实体的组件
#[derive(Component)]
pub(crate) struct OverworldEntity();

/// Dynamic overworld sub-state identified by string name.
/// Behavior is defined in states.ron configuration file.
/// This allows mods to define arbitrary sub-states without modifying engine code.
///
/// 基于字符串名称的动态 Overworld 子状态。
/// 行为在 states.ron 配置文件中定义。
/// 这允许 mod 定义任意子状态而无需修改引擎代码。
#[derive(Debug, Clone, PartialEq, Eq, Hash, States)]
pub struct OverworldSubState(pub String);

impl Default for OverworldSubState {
    fn default() -> Self {
        // Default to "Normal" state to match configuration expectations.
        // This is the expected initial state in most game configurations.
        //
        // 默认为 "Normal" 状态以匹配配置预期。
        // 这是大多数游戏配置中的预期初始状态。
        Self("Normal".to_string())
    }
}

impl OverworldSubState {
    /// Create a new sub-state with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Check if this state matches the given name.
    pub fn is(&self, name: &str) -> bool {
        self.0 == name
    }

    /// Get the state name.
    pub fn name(&self) -> &str {
        &self.0
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct OverworldUpdate;

/// System set for FRE trigger processing (experimental).
/// Runs before DanmakuUpdate to ensure PlayPerformanceEvent is written first.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FRETriggerSet;

pub(crate) struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        use crate::app_state::AppState;

        app.configure_sets(
            Update,
            OverworldUpdate.run_if(in_state(AppState::Overworld)),
        )
        // Note: ViewUpdate run_if condition is configured in lib.rs to support both Overworld and Battle
        //
        // 注意：ViewUpdate 的运行条件在 lib.rs 中配置，以支持 Overworld 和 Battle 两个状态
        .init_state::<OverworldSubState>()
        .add_plugins((
            tilemap::TilemapPlugin,
            player::PlayerPlugin,
            character::CharacterPlugin,
            crate::core::view::CoreViewPlugin,
        ))
        .add_systems(
            OnEnter(AppState::Overworld),
            create_overworld_entities_system,
        )
        .add_systems(
            OnExit(AppState::Overworld),
            (
                cleanup_entities_system::<OverworldEntity>,
                stop_bgm_on_exit_system,
                cleanup_overworld_fre_system,
            ),
        )
        .add_systems(Update, bind_camera_target_system.in_set(OverworldUpdate))
        .add_systems(
            Update,
            collision::player_tilemap_collision_system
                .after(character::MovementSet)
                .before(crate::core::camera::CameraUpdateSet)
                .in_set(OverworldUpdate),
        )
        // Dynamic state change handler - forces player idle when entering non-movable states
        .add_systems(
            Update,
            force_player_idle_on_non_movable_state_system.in_set(OverworldUpdate),
        );
        // Note: The hardcoded view_local_facts_navigation_system has been removed.
        // Navigation is now handled by FRE rules in backpack.fre.ron.
        // The input_to_fre_event_bridge_system is kept for backward compatibility
        // with legacy string events.
        // 注意：硬编码的 view_local_facts_navigation_system 已被移除。
        // 导航现在由 backpack.fre.ron 中的 FRE 规则处理。
        // input_to_fre_event_bridge_system 保留用于与旧式字符串事件的向后兼容。

        // FRE + Danmaku integration + Chase
        app.add_plugins(bevy_fact_rule_event::FREPlugin)
            .add_plugins(chase::ChasePlugin)
            // Configure FRETriggerSet to run in OverworldUpdate
            .configure_sets(Update, FRETriggerSet.in_set(OverworldUpdate))
            // Configure DanmakuUpdate to run when in either Battle OR Overworld state
            // This allows the danmaku systems to work in both modes without conflicting in_set
            .configure_sets(
                Update,
                DanmakuUpdate
                    .run_if(in_state(AppState::Battle).or(in_state(AppState::Overworld)))
                    .after(FRETriggerSet),
            )
            .init_resource::<trigger::LoadedRuleSets>()
            .init_resource::<trigger::RuleActionDefs>()
            .init_resource::<trigger::PendingDanmakuActions>()
            .init_resource::<trigger::FocusedInteractable>()
            .add_systems(
                OnEnter(AppState::Overworld),
                (
                    trigger::setup_action_handlers_system,
                    set_overworld_danmaku_context,
                ),
            )
            .add_systems(
                Update,
                (
                    // Input-to-FRE bridge must run before FRE rules processing
                    // 输入到 FRE 桥接必须在 FRE 规则处理之前运行
                    view::input_to_fre_event_bridge_system,
                    trigger::load_fre_rules_system,
                    trigger::register_loaded_rules_system,
                    trigger::trigger_zone_detection_system,
                    // Interactable detection and input handling
                    // 可交互物体检测和输入处理
                    trigger::interactable_detection_system,
                    trigger::handle_interaction_input_system,
                    trigger::collect_danmaku_actions_system,
                    trigger::play_danmaku_from_actions_system,
                    trigger::handle_chase_state_actions_system,
                    trigger::log_fact_changes_system,
                )
                    .chain()
                    .in_set(FRETriggerSet),
            );
    }
}

fn create_overworld_entities_system(mut spawn_events: MessageWriter<player::SpawnPlayerRequest>) {
    spawn_events.write(player::SpawnPlayerRequest);
}

fn set_overworld_danmaku_context(mut spawn_context: ResMut<DanmakuSpawnContext>) {
    *spawn_context = DanmakuSpawnContext::overworld();
    info!("Danmaku: Set spawn context to Overworld");
}

#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
fn stop_bgm_on_exit_system(
    mut bgm_handle: ResMut<tilemap::CurrentBgmHandle>,
    mut current_map_bgm: ResMut<tilemap::CurrentMapBgm>,
    mut audio_instances: ResMut<Assets<bevy_kira_audio::AudioInstance>>,
) {
    if let Some(handle) = &bgm_handle.0
        && let Some(instance) = audio_instances.get_mut(handle)
    {
        instance.stop(bevy_kira_audio::AudioTween::default());
    }
    bgm_handle.0 = None;
    current_map_bgm.0 = None;
}

#[cfg(feature = "firewheel")]
fn stop_bgm_on_exit_system(
    mut commands: Commands,
    mut bgm_handle: ResMut<tilemap::CurrentBgmHandle>,
    mut current_map_bgm: ResMut<tilemap::CurrentMapBgm>,
) {
    // Despawn BGM entity to stop playback
    if let Some(entity) = bgm_handle.0 {
        commands.entity(entity).despawn();
    }
    bgm_handle.0 = None;
    current_map_bgm.0 = None;
}

fn bind_camera_target_system(
    mut camera: Query<&mut Followable, With<Camera2d>>,
    player: Query<Entity, Added<character::components::PlayerControlled>>,
) {
    for player_entity in player.iter() {
        for mut followable in camera.iter_mut() {
            followable.target = Some(player_entity);
        }
    }
}

/// System to force player idle when entering a non-movable state.
/// Reads player_movable from StateConfig.
///
/// 当进入不允许移动的状态时强制玩家空闲的系统。
/// 从 StateConfig 读取 player_movable。
fn force_player_idle_on_non_movable_state_system(
    mut commands: Commands,
    current_state: Res<State<OverworldSubState>>,
    state_config: Option<Res<crate::core::state_config::LoadedStateConfig>>,
    player_query: Query<
        Entity,
        (
            With<character::components::PlayerControlled>,
            Or<(
                With<character::components::StateWalking>,
                With<character::components::StateRunning>,
            )>,
        ),
    >,
    mut last_state: Local<String>,
) {
    // Only trigger when state changes
    if current_state.0 == *last_state {
        return;
    }
    *last_state = current_state.0.clone();

    // Check if current state allows movement
    let player_movable = state_config
        .as_ref()
        .and_then(|config| config.0.states.get(&current_state.0))
        .map(|def| def.player_movable)
        .unwrap_or(true); // Default to movable if no config

    // Force idle if movement is not allowed - remove walking/running components
    if !player_movable {
        for entity in player_query.iter() {
            commands
                .entity(entity)
                .remove::<character::components::StateWalking>()
                .remove::<character::components::StateRunning>()
                .insert(character::components::StateIdle);
        }
        info!(
            "Overworld: Forced player idle on entering non-movable state '{}'",
            current_state.0
        );
    }
}

/// System to clean up FRE state when exiting overworld.
/// Clears local layer facts and rules.
///
/// 退出 Overworld 时清理 FRE 状态的系统。
/// 清除局部层事实和规则。
fn cleanup_overworld_fre_system(
    mut layered_db: ResMut<LayeredFactDatabase>,
    mut registry: ResMut<LayeredRuleRegistry>,
    mut loaded_rule_sets: ResMut<trigger::LoadedRuleSets>,
) {
    // Clear local layer facts
    layered_db.clear_local();

    // Clear local layer rules
    registry.clear_local();

    // Reset loaded rule sets for next entry
    loaded_rule_sets.handles.clear();
    loaded_rule_sets.initialized = false;

    info!("Overworld FRE: Cleaned up local layer (facts and rules)");
}

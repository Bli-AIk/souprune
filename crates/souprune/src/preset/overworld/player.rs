//! # player.rs
//!
//! # player.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module manages overworld player movement, facing, and animation states.
//!
//! 该模块负责 Overworld 玩家移动、朝向与动画状态的管理。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It defines `PlayerPlugin`, which wires systems for direction control and idle/walk/run animations.
//!
//! 文件定义了 `PlayerPlugin`，负责连接方向控制以及空闲/行走/奔跑动画系统。

use crate::core::animation::components::SpriteAnimationClip;
use crate::core::danmaku::BulletTarget;
use crate::core::input::PlayerInputSettings;
use crate::core::sprite::params::SpriteParams;
use crate::preset::overworld::player::config::PlayerBehavior;
use bevy::app::{App, Plugin};
use bevy::log::error;
use bevy::prelude::*;

pub mod components;
pub(crate) mod config;
mod systems;
pub(crate) mod utils;

use crate::preset::overworld::{OverworldUpdate, character};
#[derive(Clone)]
pub struct SpawnPlayerRequest;

impl Message for SpawnPlayerRequest {}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        use systems::*;
        match config::PlayerBehavior::load() {
            Ok(behavior) => {
                app.insert_resource(behavior)
                    .add_message::<SpawnPlayerRequest>()
                    .add_systems(
                        schedule,
                        (
                            player_direction_control_system,
                            spawn_player_on_event,
                            player_state_transition_system,
                        )
                            .in_set(OverworldUpdate),
                    );
            }
            Err(e) => {
                bevy::log::warn!(
                    "Player behavior config not loaded: {}. Overworld player systems will be disabled.",
                    e
                );
            }
        }
    }
}

fn spawn_player_on_event(
    mut events: MessageReader<SpawnPlayerRequest>,
    mut commands: Commands,
    mut sprite_params: SpriteParams,
    player_input: Res<PlayerInputSettings>,
    asset_server: Res<AssetServer>,
    player_behavior: Res<PlayerBehavior>,
) {
    if events.read().next().is_none() {
        return;
    }

    spawn_overworld_player(
        &mut commands,
        &mut sprite_params,
        &player_input,
        &asset_server,
        &player_behavior,
    );
}

pub(super) fn force_player_idle_on_state_change_system(
    mut commands: Commands,
    players: Query<Entity, With<character::components::PlayerControlled>>,
) {
    for entity in players.iter() {
        commands
            .entity(entity)
            .remove::<character::components::StateWalking>()
            .remove::<character::components::StateRunning>()
            .insert(character::components::StateIdle);
    }
}

pub fn spawn_overworld_player(
    commands: &mut Commands,
    sprite_params: &mut SpriteParams,
    player_input: &Res<PlayerInputSettings>,
    asset_server: &Res<AssetServer>,
    player_behavior: &Res<PlayerBehavior>,
) {
    use crate::core::character_asset::{AnimationConfigAsset, CharacterAnimator};
    use crate::core::collision::Rect2DCollider;
    use crate::core::input::Action;
    use crate::preset::overworld::character::components::*;
    use leafwing_input_manager::action_state::ActionState;

    let anim_config: Handle<AnimationConfigAsset> =
        asset_server.load(&player_behavior.animation_config_path);

    let initial_clip = match SpriteAnimationClip::new(
        &mut sprite_params.create_sprite_context(),
        &player_behavior.sprite_source,
        &player_behavior.initial_clip,
        false,
        false,
        true,
        0.15,
    ) {
        Ok(clip) => clip,
        Err(e) => {
            error!(
                "Failed to load initial player animation '{}': {}. Using fallback.",
                player_behavior.initial_clip, e
            );
            SpriteAnimationClip::fallback(
                &mut sprite_params.create_sprite_context(),
                &player_behavior.initial_clip,
                0.15,
            )
        }
    };

    commands.spawn((
        Name::new("OverworldPlayer"),
        StateIdle,
        PlayerControlled,
        BulletTarget::new(),
        crate::preset::overworld::chase::ChaseHighlight,
        player_input.get_merged_map(),
        ActionState::<Action>::default(),
        Rect2DCollider::new(
            player_behavior.collider_size,
            player_behavior.collider_offset,
        ),
        CharacterBundle::new(
            player_behavior.spawn_position,
            player_behavior.initial_facing,
            player_behavior.base_speed,
            initial_clip,
            CharacterAnimator {
                config: anim_config,
            },
        ),
    ));
}

/// Pure Bevy system for player state transitions.
/// Replaces seldom_state's declarative StateMachine.
///
/// 纯 Bevy 系统实现的玩家状态转换。
/// 替换 seldom_state 的声明式 StateMachine。
pub fn player_state_transition_system(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            Option<&character::components::StateIdle>,
            Option<&character::components::StateWalking>,
            Option<&character::components::StateRunning>,
        ),
        With<character::components::PlayerControlled>,
    >,
    action_query: Query<
        &leafwing_input_manager::action_state::ActionState<crate::core::input::Action>,
        With<character::components::PlayerControlled>,
    >,
    registry: Res<crate::core::input::ActionRegistry>,
    behavior_config: Res<crate::core::input::InputBehaviorConfig>,
    sub_state: Res<State<crate::core::mode::SequenceSubState>>,
    player_behavior: Res<PlayerBehavior>,
    state_config: Option<Res<crate::core::state_config::LoadedStateConfig>>,
) {
    use crate::core::input::ActionStateExt;
    use crate::preset::overworld::character::components::*;

    // Check if current state allows player movement
    let player_movable = state_config
        .as_ref()
        .map(|c| c.is_player_movable(sub_state.name()))
        .unwrap_or(true);

    let Ok(action_state) = action_query.single() else {
        return;
    };

    // Check walking input
    let is_walking = if player_movable {
        let up_pressed = behavior_config
            .nav_up()
            .map(|name| action_state.action_pressed(&registry, name))
            .unwrap_or(false);
        let down_pressed = behavior_config
            .nav_down()
            .map(|name| action_state.action_pressed(&registry, name))
            .unwrap_or(false);
        let left_pressed = behavior_config
            .nav_left()
            .map(|name| action_state.action_pressed(&registry, name))
            .unwrap_or(false);
        let right_pressed = behavior_config
            .nav_right()
            .map(|name| action_state.action_pressed(&registry, name))
            .unwrap_or(false);

        let has_vertical = (up_pressed && !down_pressed) || (down_pressed && !up_pressed);
        let has_horizontal = (left_pressed && !right_pressed) || (right_pressed && !left_pressed);
        has_vertical || has_horizontal
    } else {
        false
    };

    // Check running input
    let is_running = if player_movable && is_walking {
        player_behavior
            .run_action
            .as_ref()
            .and_then(|name| registry.get(name))
            .map(|action| action_state.pressed(&action))
            .unwrap_or(false)
    } else {
        false
    };

    for (entity, has_idle, has_walking, has_running) in query.iter() {
        let mut entity_commands = commands.entity(entity);

        match (
            has_idle.is_some(),
            has_walking.is_some(),
            has_running.is_some(),
        ) {
            // StateIdle: transition to Walking if walking
            (true, false, false) if is_walking => {
                entity_commands.remove::<StateIdle>().insert(StateWalking);
            }
            // StateWalking: transition to Running if running, or to Idle if not walking
            (false, true, false) => {
                if !is_walking {
                    entity_commands.remove::<StateWalking>().insert(StateIdle);
                } else if is_running {
                    entity_commands
                        .remove::<StateWalking>()
                        .insert(StateRunning);
                }
            }
            // StateRunning: transition to Walking if not running, or to Idle if not walking
            (false, false, true) => {
                if !is_walking {
                    entity_commands.remove::<StateRunning>().insert(StateIdle);
                } else if !is_running {
                    entity_commands
                        .remove::<StateRunning>()
                        .insert(StateWalking);
                }
            }
            _ => {}
        }
    }
}

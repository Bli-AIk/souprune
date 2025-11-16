use crate::app_state::AppState;
use crate::core::camera::Followable;
use crate::core::input::{Action, PlayerInputSettings};
use crate::core::sprite::params::SpriteParams;
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

pub(crate) mod character;
mod player;
pub(crate) mod tilemap;
mod ui;

/// Overworld sub-states
/// Overworld 子状态
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub(crate) enum OverworldState {
    #[default]
    Normal,
    Menu,
    Cutscene,
}

pub(crate) struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<OverworldState>()
            .add_plugins((
                tilemap::TilemapPlugin,
                player::PlayerPlugin,
                character::CharacterPlugin,
                ui::UndertaleOverworldUIPlugin,
            ))
            .add_systems(
                OnEnter(AppState::Overworld),
                (
                    create_overworld_entities_system,
                    bind_camera_target_system,
                    setup_overworld_state_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                handle_overworld_state_transitions.run_if(in_state(AppState::Overworld)),
            );
    }
}

fn create_overworld_entities_system(
    mut commands: Commands,
    mut sprite_params: SpriteParams,
    player_input: Res<PlayerInputSettings>,
) {
    player::spawn_overworld_player(&mut commands, &mut sprite_params, &player_input);
}

fn bind_camera_target_system(
    mut camera: Query<&mut Followable, With<Camera2d>>,
    player: Query<Entity, With<character::components::PlayerControlled>>,
) {
    if let Ok(player_entity) = player.single() {
        for mut followable in camera.iter_mut() {
            followable.target = Some(player_entity);
        }
    }
}

/// Setup initial overworld sub-state
/// 设置初始的 overworld 子状态
fn setup_overworld_state_system(mut next_state: ResMut<NextState<OverworldState>>) {
    // Ensure we start in Normal state when entering Overworld
    next_state.set(OverworldState::Normal);
    info!("Initialized Overworld in Normal state");
}

/// Handle transitions between overworld sub-states
/// 处理 overworld 子状态之间的转换
fn handle_overworld_state_transitions(
    mut next_state: ResMut<NextState<OverworldState>>,
    current_state: Res<State<OverworldState>>,
    query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
) {
    if let Ok(action_state) = query.single()
        && action_state.just_pressed(&Action::Menu)
    {
        match current_state.get() {
            OverworldState::Normal => {
                info!("Transitioning from Normal to Menu state");
                next_state.set(OverworldState::Menu);
            }
            OverworldState::Menu => {
                info!("Transitioning from Menu to Normal state");
                next_state.set(OverworldState::Normal);
            }
            OverworldState::Cutscene => {
                info!("Menu key pressed during cutscene, ignoring");
            }
        }
    }
}

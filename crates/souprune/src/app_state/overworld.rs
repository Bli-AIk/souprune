use crate::app_state::AppState;
use crate::app_state::overworld::player::config::PlayerBehavior;
use crate::core::audio;
use crate::core::camera::Followable;
use crate::core::input::PlayerInputSettings;
use crate::core::sprite::params::SpriteParams;
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy_kira_audio::Audio;

pub(crate) mod character;
mod player;
pub(crate) mod tilemap;
pub(crate) mod ui;

/// Overworld substates
///
/// Overworld 子状态
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub(crate) enum OverworldState {
    #[default]
    Normal,
    Backpack,
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
                ui::OverworldUIPlugin,
            ))
            .add_systems(
                OnEnter(AppState::Overworld),
                (
                    create_overworld_entities_system,
                    bind_camera_target_system,
                    start_overworld_bgm,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(OverworldState::Backpack),
                player::force_player_idle_on_state_change_system,
            )
            .add_systems(
                OnEnter(OverworldState::Cutscene),
                player::force_player_idle_on_state_change_system,
            );
    }
}

fn create_overworld_entities_system(
    mut commands: Commands,
    mut sprite_params: SpriteParams,
    player_input: Res<PlayerInputSettings>,
    asset_server: Res<AssetServer>,
    player_behavior: Res<PlayerBehavior>,
) {
    player::spawn_overworld_player(
        &mut commands,
        &mut sprite_params,
        &player_input,
        &asset_server,
        &player_behavior,
    );
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

/// Start playing background music when entering the Overworld.
///
/// 进入 Overworld 时开始播放背景音乐。
fn start_overworld_bgm(audio: Res<Audio>, asset_server: Res<AssetServer>) {
    // TODO: Background music should be configurable via a resource or config file
    // TODO: 背景音乐应该通过资源或配置文件来配置
    // For now, we hardcode mus_ruins.ogg as the default BGM
    //
    // 目前，我们将 mus_ruins.ogg 硬编码为默认 BGM
    audio::play_bgm(&audio, &asset_server, "mus_ruins.ogg");
}

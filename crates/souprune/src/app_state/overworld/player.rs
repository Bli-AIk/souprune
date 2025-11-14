use crate::core::animation::components::{
    SpriteAnimationClip, SpriteAnimationCurrentFrame, SpriteAnimationTimer,
};
use crate::core::basic_components::{Direction, Facing};
use crate::core::sprite::params::SpriteParams;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::Sprite;
pub mod components;
mod systems;
pub(crate) mod utils;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::player_direction_control_system,
                systems::player_idle_anim_control_system,
                systems::player_walk_anim_control_system,
                systems::player_run_anim_control_system,
            ),
        );
    }
}

fn update_player_animation(
    sprite_params: &mut SpriteParams,
    facing: &Facing,
    sprite: &mut Sprite,
    clip: &mut SpriteAnimationClip,
    frame: &mut SpriteAnimationCurrentFrame,
    timer: &mut SpriteAnimationTimer,
    animation_prefix: &str,
) {
    let clip_name = match facing.value {
        Direction::Up => format!("{}_up", animation_prefix),
        Direction::Down => format!("{}_down", animation_prefix),
        Direction::Left => format!("{}_left", animation_prefix),
        Direction::Right => format!("{}_right", animation_prefix),
        Direction::UpLeft => format!("{}_up", animation_prefix),
        Direction::UpRight => format!("{}_up", animation_prefix),
        Direction::DownLeft => format!("{}_down", animation_prefix),
        Direction::DownRight => format!("{}_down", animation_prefix),
    };

    if clip.clip_name() != clip_name {
        *clip = change_sprite_animation(sprite_params, frame, timer, "overworld", &clip_name);
        *sprite = clip.get_current_sprite().clone();
    }
}

fn change_sprite_animation(
    sprite_params: &mut SpriteParams,
    current_frame: &mut SpriteAnimationCurrentFrame,
    timer: &mut SpriteAnimationTimer,
    module_name: &str,
    clip_name: &str,
) -> SpriteAnimationClip {
    current_frame.value = 0;

    timer.reset();

    SpriteAnimationClip::new(
        &mut sprite_params.create_sprite_context(),
        module_name,
        clip_name,
    )
}

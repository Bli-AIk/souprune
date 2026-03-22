//! # chase.rs
//!
//! Chase-state visual effects and transition orchestration for the Overworld.

use bevy::image::TextureAtlasLayout;
use bevy::prelude::*;
use bevy::sprite_render::MeshMaterial2d;
use bevy_alight_motion::sdf_material::SdfMaterial;

use crate::app_state::overworld::OverworldUpdate;
use crate::app_state::overworld::character::components::PlayerControlled;
use crate::app_state::{ModeScoped, SequenceSubState};
use crate::core::state_config::LoadedStateConfig;
use crate::core::view::PixelOutlineMaterial;
use crate::core::view::sdf_shape::ViewSdfShape;

mod state_flow;
mod visuals;

use self::state_flow::{
    chase_enabled, detect_chase_state_enter_system, detect_chase_state_exit_system,
    load_chase_config_system, update_chase_transition_system,
};
use self::visuals::{
    cleanup_chase_effects_system, spawn_chase_dark_overlay_system, spawn_heart_marker_system,
    spawn_player_outline_system, update_chase_effect_alpha_system,
    update_heart_marker_alpha_system, update_player_outline_system,
};

pub use self::state_flow::is_in_chase_state;
pub use super::chase_config::*;
pub use super::chase_damage::*;

/// Marker component for entities that should be highlighted during chase state.
#[derive(Component, Debug, Default)]
pub struct ChaseHighlight;

/// Component for chase effect entities that need alpha transition.
#[derive(Component, Debug)]
pub struct ChaseEffect {
    /// Target alpha value.
    pub target_alpha: f32,
}

impl Default for ChaseEffect {
    fn default() -> Self {
        Self { target_alpha: 0.5 }
    }
}

/// Marker for the chase dark overlay entity.
#[derive(Component)]
pub struct ChaseDarkOverlay;

/// Marker for the player outline entity, also stores current sprite size.
#[derive(Component)]
pub struct ChasePlayerOutline {
    /// Current sprite size (used to detect when mesh needs updating).
    pub current_size: Vec2,
}

/// Marker for the heart marker (judgment indicator) entity.
#[derive(Component)]
pub struct ChaseHeartMarker;

/// Root entity for organizing chase effect visualizers.
#[derive(Component)]
pub struct ChaseEffectRoot;

/// Resource to track chase state transition.
#[derive(Resource, Default)]
pub struct ChaseTransition {
    /// Whether currently in chase state.
    pub active: bool,
    /// Transition timer (0.0 to configured duration).
    pub timer: f32,
    /// Whether transitioning in or out.
    pub transitioning_in: bool,
    /// Whether cleanup has been done.
    pub cleanup_done: bool,
}

/// Resource to track if chase config has been loaded.
#[derive(Resource, Default)]
pub struct ChaseConfigLoaded(pub bool);

pub struct ChasePlugin;

impl Plugin for ChasePlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        app.init_resource::<ChaseEnabled>()
            .init_resource::<ChaseStateName>()
            .init_resource::<ChaseStateTracker>()
            .init_resource::<ChaseTransition>()
            .init_resource::<DamageUIState>()
            .init_resource::<PlayerInvincibility>()
            .init_resource::<ChaseConfigLoaded>()
            .add_message::<ChasePlayerDamageEvent>()
            .add_systems(
                schedule,
                load_chase_config_system
                    .run_if(|loaded: Res<ChaseConfigLoaded>| !loaded.0)
                    .before(super::FRETriggerSet),
            )
            .add_systems(
                schedule,
                (
                    detect_chase_state_enter_system,
                    detect_chase_state_exit_system,
                )
                    .chain()
                    .before(update_chase_transition_system)
                    .in_set(OverworldUpdate)
                    .run_if(chase_enabled),
            )
            .add_systems(
                schedule,
                (
                    update_chase_transition_system,
                    spawn_chase_dark_overlay_system,
                    spawn_player_outline_system,
                    spawn_heart_marker_system,
                    spawn_player_hitbox_system,
                    update_player_outline_system,
                    update_chase_effect_alpha_system,
                    update_heart_marker_alpha_system,
                    chase_damage_detection_system,
                    update_player_invincibility_system,
                    damage_ui_display_system,
                    cleanup_chase_effects_system,
                    cleanup_player_hitbox_system,
                )
                    .chain()
                    .in_set(OverworldUpdate)
                    .run_if(chase_enabled),
            );
    }
}

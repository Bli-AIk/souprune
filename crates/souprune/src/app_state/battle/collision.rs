//! # collision.rs
//!
//! Battle collision systems for player movement within battle box boundaries.
//!
//! Battle 碰撞系统，用于限制玩家在战斗框内移动。

use crate::app_state::battle::{BattleEntity, BattleUpdate};
use crate::core::collision::{BattleBoxBoundary, PhysicsCollider};
use crate::core::ui::components::UIBox;
use bevy::prelude::*;

/// System to constrain player position within battle box boundaries
///
/// 限制玩家位置在战斗框边界内的系统
pub(crate) fn constrain_player_to_battle_box_system(
    mut player_query: Query<
        (&mut Transform, &PhysicsCollider),
        (With<crate::core::mod_system::SoulParams>, Without<UIBox>),
    >,
    battle_box_query: Query<(&GlobalTransform, &UIBox, &Name), Without<PhysicsCollider>>,
) {
    // Find the battle box (by name "BattleBox")
    let Some((box_transform, ui_box)) = battle_box_query
        .iter()
        .find(|(_, _, name)| name.as_str() == "BattleBox")
        .map(|(t, b, _)| (t, b))
    else {
        return;
    };

    // Create boundary from UI box
    let boundary = BattleBoxBoundary::from_ui_box(
        ui_box.width(),
        ui_box.height(),
        box_transform.translation().truncate(),
    );

    // Constrain player positions
    for (mut player_transform, physics_collider) in player_query.iter_mut() {
        let current_pos = player_transform.translation.truncate();
        let constrained_pos = boundary.constrain_with_collider(current_pos, physics_collider);

        // Only update if position changed
        if (constrained_pos - current_pos).length_squared() > 0.0001 {
            info!(
                "Constraining player from {:?} to {:?}",
                current_pos, constrained_pos
            );
            player_transform.translation.x = constrained_pos.x;
            player_transform.translation.y = constrained_pos.y;
        }
    }
}

/// Plugin for battle collision systems
///
/// Battle 碰撞系统插件
pub(crate) struct BattleCollisionPlugin;

impl bevy::app::Plugin for BattleCollisionPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            Update,
            constrain_player_to_battle_box_system.in_set(BattleUpdate),
        );
    }
}

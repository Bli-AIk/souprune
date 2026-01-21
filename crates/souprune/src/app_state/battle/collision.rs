//! # collision.rs
//!
//! Battle collision systems for player movement within battle box boundaries.
//!
//! Battle 碰撞系统，用于限制玩家在战斗框内移动。

use crate::app_state::battle::{BattleMovementSet, BattleUpdate};
use crate::core::collision::{BattleBoxBoundary, PhysicsCollider};
use crate::core::mod_system::BehaviorParams;
use crate::core::ui::components::UIBox;
use bevy::prelude::*;

/// Plugin for battle collision systems
///
/// Battle 碰撞系统插件
pub(crate) struct BattleCollisionPlugin;

impl Plugin for BattleCollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            constrain_player_to_battle_box_system
                .after(BattleMovementSet)
                .in_set(BattleUpdate),
        );
    }
}

/// Marker component for the battle box boundary
///
/// 战斗框边界的标记组件
#[derive(Component)]
pub struct BattleBox;

/// Component storing battle box dimensions for AM-animated battle boxes.
/// Used when the battle box doesn't use UIBox (e.g., AM animations).
///
/// 存储 AM 动画战斗框尺寸的组件。
/// 用于不使用 UIBox 的战斗框（如 AM 动画）。
#[derive(Component, Debug, Clone)]
pub struct AmBattleBoxBounds {
    pub width: f32,
    pub height: f32,
}

/// System to constrain player position within battle box boundaries
///
/// 限制玩家位置在战斗框边界内的系统
pub(crate) fn constrain_player_to_battle_box_system(
    mut player_query: Query<
        (&mut Transform, &PhysicsCollider),
        (With<BehaviorParams>, Without<UIBox>),
    >,
    // Traditional UI-based battle box
    ui_battle_box_query: Query<
        (&GlobalTransform, &UIBox),
        (With<BattleBox>, Without<PhysicsCollider>),
    >,
    // AM-animated battle box
    am_battle_box_query: Query<
        (&GlobalTransform, &AmBattleBoxBounds),
        (With<BattleBox>, Without<UIBox>, Without<PhysicsCollider>),
    >,
) {
    // Try to find boundary from UI box first, then fall back to AM battle box
    let boundary = if let Some((box_transform, ui_box)) = ui_battle_box_query.iter().next() {
        // Create boundary from UI box
        BattleBoxBoundary::from_ui_box(
            ui_box.width(),
            ui_box.height(),
            box_transform.translation().truncate(),
        )
    } else if let Some((box_transform, am_bounds)) = am_battle_box_query.iter().next() {
        // Create boundary from AM battle box bounds
        BattleBoxBoundary::from_ui_box(
            am_bounds.width,
            am_bounds.height,
            box_transform.translation().truncate(),
        )
    } else {
        return;
    };

    // Constrain player positions
    for (mut player_transform, physics_collider) in player_query.iter_mut() {
        let current_pos = player_transform.translation.truncate();
        let constrained_pos = boundary.constrain_with_collider(current_pos, physics_collider);

        // Always apply correction if position differs (no threshold)
        // This ensures the player never visually exceeds the boundary
        //
        // 如果位置不同则总是应用修正（无阈值）
        // 这确保玩家在视觉上永远不会超出边界
        player_transform.translation.x = constrained_pos.x;
        player_transform.translation.y = constrained_pos.y;
    }
}

//! # collision.rs
//!
//! Battle collision systems for player movement within battle box boundaries.
//!
//! Battle 碰撞系统，用于限制玩家在战斗框内移动。

use crate::app_state::battle::{BattleMovementSet, BattleUpdate};
use crate::core::collision::{BattleBoxBoundary, PhysicsCollider};
use crate::core::mod_system::BehaviorParams;
use crate::core::view::components::ViewBox;
use bevy::prelude::*;

/// Plugin for battle collision systems
///
/// Battle 碰撞系统插件
pub(crate) struct BattleCollisionPlugin;

impl Plugin for BattleCollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            crate::game_schedule(app),
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
/// Used when the battle box doesn't use ViewBox (e.g., AM animations).
///
/// 存储 AM 动画战斗框尺寸的组件。
/// 用于不使用 ViewBox 的战斗框（如 AM 动画）。
#[derive(Component, Debug, Clone)]
pub struct AlightMotionBattleBoxBounds {
    pub width: f32,
    pub height: f32,
    /// Offset from entity position to the geometric center of the battle box (Bevy coords).
    /// This compensates for non-centered pivot points in AM animations.
    ///
    /// 从实体位置到战斗框几何中心的偏移（Bevy 坐标）。
    /// 用于补偿 AM 动画中非居中的锚点。
    pub center_offset: Vec2,
}

/// System to constrain player position within battle box boundaries
///
/// 限制玩家位置在战斗框边界内的系统
pub(crate) fn constrain_player_to_battle_box_system(
    mut player_query: Query<
        (&mut Transform, &PhysicsCollider),
        (With<BehaviorParams>, Without<ViewBox>),
    >,
    // Traditional UI-based battle box
    ui_battle_box_query: Query<
        (&GlobalTransform, &ViewBox),
        (With<BattleBox>, Without<PhysicsCollider>),
    >,
    // AM-animated battle box
    alight_motion_battle_box_query: Query<
        (&GlobalTransform, &AlightMotionBattleBoxBounds),
        (With<BattleBox>, Without<ViewBox>, Without<PhysicsCollider>),
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
    } else if let Some((box_transform, alight_motion_bounds)) =
        alight_motion_battle_box_query.iter().next()
    {
        // Create boundary from AM battle box bounds
        // Apply center_offset to get the actual geometric center of the battle box
        let center_pos =
            box_transform.translation().truncate() + alight_motion_bounds.center_offset;
        BattleBoxBoundary::from_ui_box(
            alight_motion_bounds.width,
            alight_motion_bounds.height,
            center_pos,
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

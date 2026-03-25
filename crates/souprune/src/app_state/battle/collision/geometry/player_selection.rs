//! # player_selection.rs
//!
//! # player_selection.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Decides which battle box should own a player after geometry changes. It compares the
//! player's collider against candidate boundaries, prefers boxes that still contain the player,
//! and falls back to the closest valid boundary when no box cleanly contains them.
//!
//! 负责在几何结构变化后决定玩家应该归属哪个战斗框。它会用玩家碰撞体去比较候选边界，
//! 优先保留仍然包含玩家的框；如果没有明确包含关系，就退回到距离最近的有效边界。

use super::*;

fn signed_distance_to_box_with_collider(
    boundary: &BattleBoxBoundary,
    player_pos: Vec2,
    collider: &PhysicsCollider,
) -> f32 {
    let collider_half_size = match collider {
        PhysicsCollider::Circle { radius } => Vec2::splat(*radius),
        PhysicsCollider::Box { half_size } => *half_size,
    };
    let effective_half_size = (boundary.half_size - collider_half_size).max(Vec2::ZERO);
    BattleBoxBoundary {
        half_size: effective_half_size,
        center: boundary.center,
    }
    .sdf_distance(player_pos)
}

/// Determine which of two boxes a player should be rebound to.
pub(in crate::app_state::battle::collision) fn select_box_id_for_player(
    player_pos: Vec2,
    collider: &PhysicsCollider,
    box_a: &BattleBoxBoundary,
    box_b: &BattleBoxBoundary,
    id_a: &str,
    id_b: &str,
) -> String {
    let dist_a = signed_distance_to_box_with_collider(box_a, player_pos, collider);
    let dist_b = signed_distance_to_box_with_collider(box_b, player_pos, collider);

    if dist_a <= 0.0 && dist_b > 0.0 {
        id_a.to_string()
    } else if dist_b <= 0.0 && dist_a > 0.0 {
        id_b.to_string()
    } else if dist_a <= dist_b {
        id_a.to_string()
    } else {
        id_b.to_string()
    }
}

pub(in crate::app_state::battle::collision) fn choose_box_index_for_player(
    current_bound: Option<&str>,
    player_pos: Vec2,
    collider: &PhysicsCollider,
    candidates: &[(String, BattleBoxBoundary)],
) -> Option<usize> {
    if candidates.is_empty() {
        return None;
    }

    if let Some(current_id) = current_bound
        && let Some((index, _)) = candidates.iter().enumerate().find(|(_, (id, boundary))| {
            id == current_id
                && signed_distance_to_box_with_collider(boundary, player_pos, collider) <= 0.0
        })
    {
        return Some(index);
    }

    if let Some((index, _)) = candidates.iter().enumerate().find(|(_, (_, boundary))| {
        signed_distance_to_box_with_collider(boundary, player_pos, collider) <= 0.0
    }) {
        return Some(index);
    }

    candidates
        .iter()
        .enumerate()
        .min_by(|(_, (_, a)), (_, (_, b))| {
            signed_distance_to_box_with_collider(a, player_pos, collider).total_cmp(
                &signed_distance_to_box_with_collider(b, player_pos, collider),
            )
        })
        .map(|(index, _)| index)
}

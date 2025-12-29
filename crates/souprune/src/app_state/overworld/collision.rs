//! # overworld/collision.rs
//!
//! ## Module Overview
//! SDF-based collision system for overworld player movement.
//! Uses the core SDF collision utilities.
//!
//! ## 模块概述
//! 用于overworld玩家移动的SDF碰撞系统。
//! 使用核心SDF碰撞工具。

use crate::app_state::overworld::character::components::PlayerControlled;
use crate::app_state::overworld::tilemap::{ObjectCollider, systems::TilemapCollider};
use crate::core::collision::{
    Rect2DCollider, SdfCollisionConfig, collect_nearby_colliders, resolve_sdf_collision,
};
use bevy::prelude::*;

type PlayerQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static Rect2DCollider),
    (With<PlayerControlled>, Without<TilemapCollider>),
>;
type TilemapCollidersQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Transform, &'static Rect2DCollider),
    (
        Or<(With<TilemapCollider>, With<ObjectCollider>)>,
        Without<PlayerControlled>,
    ),
>;

/// SDF-based collision detection and response system for overworld.
/// Uses SDF gradient for stable, consistent separation.
///
/// 用于overworld的SDF碰撞检测和响应系统。
/// 使用SDF梯度进行稳定一致的分离。
pub fn player_tilemap_collision_system(
    mut player_query: PlayerQuery,
    tilemap_colliders: TilemapCollidersQuery,
) {
    let config = SdfCollisionConfig::default();

    for (mut player_transform, player_collider) in player_query.iter_mut() {
        let player_pos = player_transform.translation.truncate() + player_collider.offset;

        // Collect nearby static colliders
        let nearby_colliders =
            collect_nearby_colliders(player_pos, config.search_radius, tilemap_colliders.iter());

        if nearby_colliders.is_empty() {
            continue;
        }

        // Resolve collision using SDF
        let separation =
            resolve_sdf_collision(player_pos, player_collider, &nearby_colliders, &config);

        if separation.length_squared() > 0.0001 {
            player_transform.translation.x += separation.x;
            player_transform.translation.y += separation.y;
        }
    }
}

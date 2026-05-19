//! # top_down/collision.rs
//!
//! ## Module Overview
//! SDF-based collision system for top_down player movement.
//! Uses the core SDF collision utilities.
//!
//! ## 模块概述
//! 用于top_down玩家移动的SDF碰撞系统。
//! 使用核心SDF碰撞工具。

use crate::core::collision::{
    Rect2DCollider, SdfCollisionConfig, collect_nearby_colliders, resolve_sdf_collision,
};
use crate::core::top_down::character::components::PlayerControlled;
use crate::core::top_down::tilemap::systems::TilemapCollider;
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
    (With<TilemapCollider>, Without<PlayerControlled>),
>;

/// SDF-based collision detection and response system for top_down.
/// Uses SDF gradient for stable, consistent separation.
///
/// 用于top_down的SDF碰撞检测和响应系统。
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

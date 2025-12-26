//! # collision/systems.rs
//!
//! ## Module Overview  
//! SDF-based collision system with gradient-based separation.
//! Uses SDF union operations for both detection AND separation for consistency.
//!
//! ## 模块概述
//! 基于SDF的碰撞系统，使用梯度进行分离。
//! 在检测和分离阶段都使用SDF并集操作以保证一致性。

use crate::app_state::overworld::character::components::PlayerControlled;
use crate::app_state::overworld::tilemap::{ObjectCollider, systems::TilemapCollider};
use crate::core::collision::components::Rect2DCollider;
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

/// True SDF-based collision detection and response system
/// Uses SDF gradient for stable, consistent separation
///
/// 真正基于SDF的碰撞检测和响应系统
/// 使用SDF梯度进行稳定一致的分离
pub fn player_tilemap_collision_system(
    mut player_query: PlayerQuery,
    tilemap_colliders: TilemapCollidersQuery,
) {
    const MAX_ITERATIONS: u8 = 4;

    for (mut player_transform, player_collider) in player_query.iter_mut() {
        let initial_player_pos = player_transform.translation.truncate() + player_collider.offset;

        // Collect all nearby tiles for SDF merge
        //
        // 收集所有附近的瓦片以进行SDF合并
        let nearby_tiles = collect_nearby_tiles(&initial_player_pos, tilemap_colliders);

        if nearby_tiles.is_empty() {
            continue;
        }

        for _ in 0..MAX_ITERATIONS {
            let player_pos = player_transform.translation.truncate() + player_collider.offset;

            let (distance, min_idx_opt) =
                merged_sdf_distance_and_index(player_pos, player_collider, &nearby_tiles);

            if distance < 0.0 {
                if let Some(min_idx) = min_idx_opt {
                    let separation = calculate_analytic_separation(
                        player_pos,
                        player_collider,
                        &nearby_tiles,
                        min_idx,
                        distance,
                    );
                    player_transform.translation.x += separation.x;
                    player_transform.translation.y += separation.y;
                } else {
                    // Should not happen if distance is negative
                    break;
                }
            } else {
                // No collision, resolution is complete for this player
                break;
            }
        }
    }
}

/// Collect tiles within collision range, sorted by distance for deterministic results
/// 收集碰撞范围内的瓦片，按距离排序确保结果确定性
fn collect_nearby_tiles(
    player_pos: &Vec2,
    tilemap_colliders: TilemapCollidersQuery,
) -> Vec<(Vec2, Vec2)> {
    // (position, half_size)
    let mut nearby_tiles = Vec::new();
    let search_radius = 40.0;

    for (tile_transform, tile_collider) in tilemap_colliders.iter() {
        let tile_pos = tile_transform.translation.truncate() + tile_collider.offset;
        let distance_sq = (tile_pos - *player_pos).length_squared();

        if distance_sq < search_radius * search_radius {
            nearby_tiles.push((tile_pos, tile_collider.size * 0.5));
        }
    }

    // Sort by distance for deterministic behavior
    nearby_tiles.sort_by(|a, b| {
        let dist_a = (a.0 - *player_pos).length_squared();
        let dist_b = (b.0 - *player_pos).length_squared();
        dist_a
            .partial_cmp(&dist_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    nearby_tiles
}

/// SDF collision detection using merged tiles (returns min distance and index of closest tile)
fn merged_sdf_distance_and_index(
    player_pos: Vec2,
    player_collider: &Rect2DCollider,
    tiles: &[(Vec2, Vec2)],
) -> (f32, Option<usize>) {
    let player_half_size = player_collider.size * 0.5;
    let mut min_distance = f32::INFINITY;
    let mut min_idx: Option<usize> = None;

    for (i, &(tile_pos, tile_half_size)) in tiles.iter().enumerate() {
        let relative_pos = player_pos - tile_pos;
        let expanded_half_size = tile_half_size + player_half_size;
        let distance = sdf_box(relative_pos, expanded_half_size);
        if distance < min_distance {
            min_distance = distance;
            min_idx = Some(i);
        }
    }

    (min_distance, min_idx)
}

/// Calculate separation analytically using closest point on the expanded box
///
/// 使用扩展盒子上的最近点进行解析分离计算
fn calculate_analytic_separation(
    player_pos: Vec2,
    player_collider: &Rect2DCollider,
    tiles: &[(Vec2, Vec2)],
    min_idx: usize,
    distance: f32,
) -> Vec2 {
    let player_half_size = player_collider.size * 0.5;

    let (tile_pos, tile_half_size) = tiles[min_idx];
    let rel = player_pos - tile_pos;
    let box_half = tile_half_size + player_half_size;

    // Closest point on box surface (clamp to box)
    let clamped = Vec2::new(
        rel.x.clamp(-box_half.x, box_half.x),
        rel.y.clamp(-box_half.y, box_half.y),
    );

    // If point is outside -> normal = (rel - clamped).normalized()
    let delta = rel - clamped;
    if delta.length_squared() > f32::EPSILON {
        let normal = delta.normalize();
        // push by penetration depth (distance is signed), add small margin
        const SEPARATION_MARGIN: f32 = 0.01;
        return -normal * (distance - SEPARATION_MARGIN);
    }

    // Point is inside the expanded box: pick axis of smallest penetration (stable)
    let pen_x = box_half.x - rel.x.abs();
    let pen_y = box_half.y - rel.y.abs();

    if pen_x < pen_y {
        let sign = if rel.x >= 0.0 { 1.0 } else { -1.0 };
        Vec2::new(sign * (pen_x + 0.01), 0.0)
    } else {
        let sign = if rel.y >= 0.0 { 1.0 } else { -1.0 };
        Vec2::new(0.0, sign * (pen_y + 0.01))
    }
}

/// SDF function for a box (rectangle)
/// Implementation of the box SDF from Inigo Quilez
///
/// 盒子（矩形）的SDF函数
/// 基于Inigo Quilez的盒子SDF实现
pub fn sdf_box(point: Vec2, half_size: Vec2) -> f32 {
    let d = point.abs() - half_size;
    let outside_distance = d.max(Vec2::ZERO).length();
    let inside_distance = d.x.max(d.y).min(0.0);

    outside_distance + inside_distance
}

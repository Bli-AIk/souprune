//! # collision/systems.rs
//!
//! ## Module Overview  
//! SDF-based collision system with gradient-based separation.
//! Uses SDF union operations for both detection AND separation for consistency.
//!
//! ## 模块概述
//! 基于SDF的碰撞系统，使用梯度进行分离。
//! 在检测和分离阶段都使用SDF并集操作以保证一致性。

use crate::core::collision::components::Rect2DCollider;
use bevy::prelude::*;

/// Marker component for static collision tiles/objects
///
/// 静态碰撞瓦片/对象的标记组件
#[derive(Component)]
pub struct StaticCollider;

/// Marker component for dynamic collision entities (e.g., player)
///
/// 动态碰撞实体的标记组件（如玩家）
#[derive(Component)]
pub struct DynamicCollider;

/// SDF collision response configuration
///
/// SDF碰撞响应配置
pub struct SdfCollisionConfig {
    pub max_iterations: u8,
    pub search_radius: f32,
}

impl Default for SdfCollisionConfig {
    fn default() -> Self {
        Self {
            max_iterations: 4,
            search_radius: 40.0,
        }
    }
}

/// Resolve SDF-based collision between a dynamic entity and static colliders.
/// Returns the separation vector to apply to the entity.
///
/// 解决动态实体和静态碰撞体之间的SDF碰撞。
/// 返回需要应用到实体的分离向量。
pub fn resolve_sdf_collision(
    entity_pos: Vec2,
    entity_collider: &Rect2DCollider,
    static_colliders: &[(Vec2, Vec2)], // (position, half_size)
    config: &SdfCollisionConfig,
) -> Vec2 {
    let mut total_separation = Vec2::ZERO;
    let mut current_pos = entity_pos;

    for _ in 0..config.max_iterations {
        let (distance, min_idx_opt) =
            merged_sdf_distance_and_index(current_pos, entity_collider, static_colliders);

        if distance < 0.0 {
            if let Some(min_idx) = min_idx_opt {
                let separation = calculate_analytic_separation(
                    current_pos,
                    entity_collider,
                    static_colliders,
                    min_idx,
                    distance,
                );
                total_separation += separation;
                current_pos += separation;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    total_separation
}

/// Collect colliders within range, sorted by distance for deterministic results.
/// Returns (position, half_size) tuples.
///
/// 收集范围内的碰撞体，按距离排序确保结果确定性。
/// 返回(位置, 半尺寸)元组。
pub fn collect_nearby_colliders<'a>(
    center_pos: Vec2,
    search_radius: f32,
    colliders: impl Iterator<Item = (&'a Transform, &'a Rect2DCollider)>,
) -> Vec<(Vec2, Vec2)> {
    let mut nearby = Vec::new();
    let search_radius_sq = search_radius * search_radius;

    for (transform, collider) in colliders {
        let pos = transform.translation.truncate() + collider.offset;
        let distance_sq = (pos - center_pos).length_squared();

        if distance_sq < search_radius_sq {
            nearby.push((pos, collider.size * 0.5));
        }
    }

    // Sort by distance for deterministic behavior
    nearby.sort_by(|a, b| {
        let dist_a = (a.0 - center_pos).length_squared();
        let dist_b = (b.0 - center_pos).length_squared();
        dist_a
            .partial_cmp(&dist_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    nearby
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

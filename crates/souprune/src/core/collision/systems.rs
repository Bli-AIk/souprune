//! # collision/systems.rs
//!
//! ## Module Overview  
//! This module contains non-debug collision systems.
//! Debug-specific collision visualization is handled in extra/debug/collider_debug.rs
//!
//! ## 模块概述
//! 该模块包含非调试碰撞系统。
//! 调试专用的碰撞可视化在 extra/debug/collider_debug.rs 中处理

use crate::app_state::overworld::character::components::PlayerControlled;
use crate::app_state::overworld::tilemap::TilemapCollider;
use crate::core::collision::components::Rect2DCollider;
use bevy::prelude::*;

/// AABB collision detection and response system for player vs tilemap colliders
///
/// 玩家与瓦片地图碰撞体的 AABB 碰撞检测和响应系统
pub fn player_tilemap_collision_system(
    mut player_query: Query<
        (&mut Transform, &Rect2DCollider),
        (With<PlayerControlled>, Without<TilemapCollider>),
    >,
    tilemap_colliders: Query<
        (&Transform, &Rect2DCollider),
        (With<TilemapCollider>, Without<PlayerControlled>),
    >,
) {
    for (mut player_transform, player_collider) in player_query.iter_mut() {
        let player_aabb = get_aabb(&player_transform, player_collider);

        for (tile_transform, tile_collider) in tilemap_colliders.iter() {
            let tile_aabb = get_aabb(tile_transform, tile_collider);

            if aabb_intersects(&player_aabb, &tile_aabb) {
                // 计算重叠量并完全分离
                let overlap = calculate_overlap(&player_aabb, &tile_aabb);

                // 选择最小重叠方向进行完全分离
                if overlap.x.abs() < overlap.y.abs() {
                    // 水平分离
                    player_transform.translation.x += overlap.x;
                } else {
                    // 垂直分离
                    player_transform.translation.y += overlap.y;
                }

                // 只处理第一个碰撞以避免多重推动
                break;
            }
        }
    }
}

/// AABB (Axis-Aligned Bounding Box) structure
/// AABB（轴对齐包围盒）结构
#[derive(Debug, Clone, Copy)]
struct AABB {
    min: Vec2,
    max: Vec2,
}

/// Get AABB from transform and collider
/// 从变换和碰撞体获取 AABB
fn get_aabb(transform: &Transform, collider: &Rect2DCollider) -> AABB {
    let center = transform.translation.truncate() + collider.offset;
    let half_size = collider.size * 0.5;

    AABB {
        min: center - half_size,
        max: center + half_size,
    }
}

/// Check if two AABBs intersect
/// 检查两个 AABB 是否相交
fn aabb_intersects(a: &AABB, b: &AABB) -> bool {
    a.min.x < b.max.x && a.max.x > b.min.x && a.min.y < b.max.y && a.max.y > b.min.y
}

/// Calculate overlap vector to separate two intersecting AABBs
/// 计算重叠向量以分离两个相交的 AABB
fn calculate_overlap(player: &AABB, tile: &AABB) -> Vec2 {
    let left_overlap = player.max.x - tile.min.x;
    let right_overlap = tile.max.x - player.min.x;
    let top_overlap = player.max.y - tile.min.y;
    let bottom_overlap = tile.max.y - player.min.y;

    let x_overlap = if left_overlap < right_overlap {
        -left_overlap // 推向左
    } else {
        right_overlap // 推向右
    };

    let y_overlap = if bottom_overlap < top_overlap {
        bottom_overlap // 推向下
    } else {
        -top_overlap // 推向上
    };

    Vec2::new(x_overlap, y_overlap)
}

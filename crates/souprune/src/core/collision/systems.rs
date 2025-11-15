//! # collision/systems.rs
//!
//! ## Module Overview  
//! This module contains non-debug collision systems using SDF-based collision detection.
//! Debug-specific collision visualization is handled in extra/debug/collider_debug.rs
//!
//! ## 模块概述
//! 该模块包含基于SDF碰撞检测的非调试碰撞系统。
//! 调试专用的碰撞可视化在 extra/debug/collider_debug.rs 中处理

use crate::app_state::overworld::character::components::PlayerControlled;
use crate::app_state::overworld::tilemap::TilemapCollider;
use crate::core::collision::components::Rect2DCollider;
use bevy::prelude::*;

/// SDF-based collision detection and response system for player vs tilemap colliders
/// Provides pixel-perfect collision detection without tunneling
///
/// 基于SDF的玩家与瓦片地图碰撞检测和响应系统
/// 提供像素完美的碰撞检测，无穿透问题
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
        let player_pos = player_transform.translation.truncate() + player_collider.offset;

        // 检查玩家的SDF点是否在任何瓦片内部
        for (tile_transform, tile_collider) in tilemap_colliders.iter() {
            let tile_pos = tile_transform.translation.truncate() + tile_collider.offset;

            // 使用SDF检测碰撞
            let distance =
                sdf_collision_detection(player_pos, player_collider, tile_pos, tile_collider);

            if distance < 0.0 {
                // 玩家在瓦片内部，需要推出
                let separation =
                    calculate_sdf_separation(player_pos, player_collider, tile_pos, tile_collider);

                player_transform.translation.x += separation.x;
                player_transform.translation.y += separation.y;

                // 只处理第一个碰撞以避免多重推动
                break;
            }
        }
    }
}

/// SDF collision detection between two rectangular colliders
/// Returns the minimum distance between the player and tile (negative if overlapping)
///
/// 两个矩形碰撞体之间的SDF碰撞检测
/// 返回玩家与瓦片之间的最小距离（重叠时为负值）
fn sdf_collision_detection(
    player_pos: Vec2,
    player_collider: &Rect2DCollider,
    tile_pos: Vec2,
    tile_collider: &Rect2DCollider,
) -> f32 {
    // 将玩家位置转换为相对于瓦片中心的坐标
    let relative_pos = player_pos - tile_pos;

    // 计算玩家包围盒到瓦片的SDF距离
    // 这考虑了玩家自己的尺寸
    let expanded_tile_size = tile_collider.size + player_collider.size;

    sdf_box(relative_pos, expanded_tile_size * 0.5)
}

/// Calculate separation vector using SDF gradient
/// 使用SDF梯度计算分离向量
fn calculate_sdf_separation(
    player_pos: Vec2,
    player_collider: &Rect2DCollider,
    tile_pos: Vec2,
    tile_collider: &Rect2DCollider,
) -> Vec2 {
    let relative_pos = player_pos - tile_pos;
    let expanded_tile_size = tile_collider.size + player_collider.size;
    let half_size = expanded_tile_size * 0.5;

    // 计算SDF梯度（法向量方向）
    let gradient = sdf_box_gradient(relative_pos, half_size);

    // 计算当前距离
    let distance = sdf_box(relative_pos, half_size);

    // 分离距离 = -distance + 小的安全边距
    let separation_distance = -distance + 0.1; // 0.1像素的安全边距

    gradient * separation_distance
}

/// SDF function for a box (rectangle)
/// Implementation of the box SDF from Inigo Quilez
///
/// 盒子（矩形）的SDF函数
/// 基于Inigo Quilez的盒子SDF实现
fn sdf_box(point: Vec2, half_size: Vec2) -> f32 {
    let d = point.abs() - half_size;
    let outside_distance = d.max(Vec2::ZERO).length();
    let inside_distance = d.x.max(d.y).min(0.0);

    outside_distance + inside_distance
}

/// Calculate the gradient (normal direction) of the box SDF
/// 计算盒子SDF的梯度（法向量方向）
fn sdf_box_gradient(point: Vec2, half_size: Vec2) -> Vec2 {
    let epsilon = 0.001;

    let dx = sdf_box(point + Vec2::new(epsilon, 0.0), half_size)
        - sdf_box(point - Vec2::new(epsilon, 0.0), half_size);
    let dy = sdf_box(point + Vec2::new(0.0, epsilon), half_size)
        - sdf_box(point - Vec2::new(0.0, epsilon), half_size);

    Vec2::new(dx, dy).normalize_or_zero()
}

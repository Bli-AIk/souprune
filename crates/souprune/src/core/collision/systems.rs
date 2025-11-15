//! # collision/systems.rs
//!
//! ## Module Overview  
//! This module contains non-debug collision systems using SDF-based collision detection.
//! Features SDF merging for seamless tilemap collision without gaps.
//! Debug-specific collision visualization is handled in extra/debug/collider_debug.rs
//!
//! ## 模块概述
//! 该模块包含基于SDF碰撞检测的非调试碰撞系统。
//! 具有SDF合并功能，实现无缝隙的瓦片地图碰撞。
//! 调试专用的碰撞可视化在 extra/debug/collider_debug.rs 中处理

use crate::app_state::overworld::character::components::PlayerControlled;
use crate::app_state::overworld::tilemap::TilemapCollider;
use crate::core::collision::components::Rect2DCollider;
use bevy::prelude::*;

/// SDF-based collision detection and response system for player vs tilemap colliders
/// Uses SDF union operations to create seamless collision areas
///
/// 基于SDF的玩家与瓦片地图碰撞检测和响应系统
/// 使用SDF并集操作创建无缝碰撞区域
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

        // 收集附近的所有瓦片进行SDF合并
        let nearby_tiles = collect_nearby_tiles(&player_pos, &tilemap_colliders);

        if !nearby_tiles.is_empty() {
            // 计算合并后的SDF距离
            let distance =
                merged_sdf_collision_detection(player_pos, player_collider, &nearby_tiles);

            if distance < 0.0 {
                // 玩家在合并的SDF内部，需要推出
                let separation =
                    calculate_merged_sdf_separation(player_pos, player_collider, &nearby_tiles);

                player_transform.translation.x += separation.x;
                player_transform.translation.y += separation.y;
            }
        }
    }
}

/// Collect tiles within collision range for SDF merging
/// 收集碰撞范围内的瓦片进行SDF合并
fn collect_nearby_tiles(
    player_pos: &Vec2,
    tilemap_colliders: &Query<
        (&Transform, &Rect2DCollider),
        (With<TilemapCollider>, Without<PlayerControlled>),
    >,
) -> Vec<(Vec2, Vec2)> {
    // (position, half_size)
    let mut nearby_tiles = Vec::new();
    let search_radius = 50.0; // 搜索半径，可以根据需要调整

    for (tile_transform, tile_collider) in tilemap_colliders.iter() {
        let tile_pos = tile_transform.translation.truncate() + tile_collider.offset;

        // 只收集附近的瓦片以提高性能
        if (tile_pos - *player_pos).length() < search_radius {
            nearby_tiles.push((tile_pos, tile_collider.size * 0.5));
        }
    }

    nearby_tiles
}

/// SDF collision detection using merged tiles
/// 使用合并瓦片的SDF碰撞检测
fn merged_sdf_collision_detection(
    player_pos: Vec2,
    player_collider: &Rect2DCollider,
    tiles: &[(Vec2, Vec2)], // (position, half_size)
) -> f32 {
    let player_half_size = player_collider.size * 0.5;

    // 对所有瓦片SDF进行并集操作（取最小值）
    let mut min_distance = f32::INFINITY;

    for &(tile_pos, tile_half_size) in tiles.iter() {
        let relative_pos = player_pos - tile_pos;
        // 扩展瓦片尺寸以包含玩家半径
        let expanded_half_size = tile_half_size + player_half_size;

        let distance = sdf_box(relative_pos, expanded_half_size);
        min_distance = min_distance.min(distance);
    }

    min_distance
}

/// Calculate separation using merged SDF gradient
/// 使用合并SDF梯度计算分离
fn calculate_merged_sdf_separation(
    player_pos: Vec2,
    player_collider: &Rect2DCollider,
    tiles: &[(Vec2, Vec2)],
) -> Vec2 {
    let current_distance = merged_sdf_collision_detection(player_pos, player_collider, tiles);

    if current_distance >= 0.0 {
        return Vec2::ZERO;
    }

    // 计算合并SDF的梯度
    let gradient = merged_sdf_gradient(player_pos, player_collider, tiles);

    // 分离距离 = -distance + 安全边距
    let separation_distance = -current_distance + 0.2; // 稍微增加安全边距

    gradient * separation_distance
}

/// Calculate gradient of merged SDF
/// 计算合并SDF的梯度
fn merged_sdf_gradient(
    player_pos: Vec2,
    player_collider: &Rect2DCollider,
    tiles: &[(Vec2, Vec2)],
) -> Vec2 {
    let epsilon = 0.01;

    let center_dist = merged_sdf_collision_detection(player_pos, player_collider, tiles);
    let x_dist = merged_sdf_collision_detection(
        player_pos + Vec2::new(epsilon, 0.0),
        player_collider,
        tiles,
    );
    let y_dist = merged_sdf_collision_detection(
        player_pos + Vec2::new(0.0, epsilon),
        player_collider,
        tiles,
    );

    let gradient = Vec2::new(
        (x_dist - center_dist) / epsilon,
        (y_dist - center_dist) / epsilon,
    );

    gradient.normalize_or_zero()
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

/// Alternative implementation: Create a single merged collision polygon
/// 替代实现：创建单个合并的碰撞多边形
#[allow(dead_code)]
pub fn create_merged_collision_polygon(
    tiles: &[(Vec2, Vec2)], // (center, half_size)
) -> Vec<Vec2> {
    // 这里可以实现更复杂的多边形合并算法
    // 例如使用Clipper库或简单的矩形并集

    // 简单实现：找到包围所有瓦片的最小边界框
    if tiles.is_empty() {
        return Vec::new();
    }

    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for &(center, half_size) in tiles.iter() {
        min_x = min_x.min(center.x - half_size.x);
        max_x = max_x.max(center.x + half_size.x);
        min_y = min_y.min(center.y - half_size.y);
        max_y = max_y.max(center.y + half_size.y);
    }

    // 返回合并后的矩形顶点
    vec![
        Vec2::new(min_x, min_y),
        Vec2::new(max_x, min_y),
        Vec2::new(max_x, max_y),
        Vec2::new(min_x, max_y),
    ]
}

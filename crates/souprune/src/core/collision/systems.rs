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
/// SDF-based collision detection and response system for player vs tilemap colliders
/// Uses SDF union operations to create seamless collision areas
///
/// 基于SDF的玩家与瓦片地图碰撞检测和响应系统
/// 使用SDF并集操作创建无缝碰撞区域
pub fn player_tilemap_collision_system(
    mut player_query: PlayerQuery,
    tilemap_colliders: TilemapCollidersQuery,
) {
    for (mut player_transform, player_collider) in player_query.iter_mut() {
        let player_pos = player_transform.translation.truncate() + player_collider.offset;

        // Collect all nearby tiles for SDF merge
        let nearby_tiles = collect_nearby_tiles(&player_pos, tilemap_colliders);

        if !nearby_tiles.is_empty() {
            let distance =
                merged_sdf_collision_detection(player_pos, player_collider, &nearby_tiles);

            if distance < 0.0 {
                let separation =
                    calculate_merged_sdf_separation(player_pos, player_collider, &nearby_tiles);

                player_transform.translation.x += separation.x;
                player_transform.translation.y += separation.y;

                info!("{}", separation);
            }
        }
    }
}

/// Collect tiles within collision range for SDF merging
/// 收集碰撞范围内的瓦片进行SDF合并
fn collect_nearby_tiles(
    player_pos: &Vec2,
    tilemap_colliders: TilemapCollidersQuery,
) -> Vec<(Vec2, Vec2)> {
    // (position, half_size)
    let mut nearby_tiles = Vec::new();
    let search_radius = 25.0;

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

/// Calculate separation handling both edge and corner collisions
/// 计算分离向量，处理边缘和角落碰撞
fn calculate_merged_sdf_separation(
    player_pos: Vec2,
    player_collider: &Rect2DCollider,
    tiles: &[(Vec2, Vec2)],
) -> Vec2 {
    let player_half_size = player_collider.size * 0.5;

    // 收集所有相交的瓦片
    let mut overlapping_tiles = Vec::new();

    for &(tile_pos, tile_half_size) in tiles.iter() {
        let relative_pos = player_pos - tile_pos;
        let expanded_half_size = tile_half_size + player_half_size;

        // 检查是否与此瓦片相交
        if relative_pos.abs().x < expanded_half_size.x
            && relative_pos.abs().y < expanded_half_size.y
        {
            let x_penetration = expanded_half_size.x - relative_pos.abs().x;
            let y_penetration = expanded_half_size.y - relative_pos.abs().y;

            overlapping_tiles.push((
                tile_pos,
                tile_half_size,
                relative_pos,
                x_penetration,
                y_penetration,
            ));
        }
    }

    if overlapping_tiles.is_empty() {
        return Vec2::ZERO;
    }

    // 优先处理 1 个或 2 个相交瓦片的常见情形，避免直线瓦片被误判为角落碰撞
    let slop_axis = 0.1; // 单轴最小分离冗余
    let slop_diag = 0.05; // 对角最小分离冗余

    if overlapping_tiles.len() == 1 {
        // 单瓦片碰撞：使用最小分离轴
        let (_, _, relative_pos, x_penetration, y_penetration) = overlapping_tiles[0];
        return if x_penetration < y_penetration {
            Vec2::new(
                if relative_pos.x > 0.0 {
                    x_penetration + slop_axis
                } else {
                    -(x_penetration + slop_axis)
                },
                0.0,
            )
        } else {
            Vec2::new(
                0.0,
                if relative_pos.y > 0.0 {
                    y_penetration + slop_axis
                } else {
                    -(y_penetration + slop_axis)
                },
            )
        };
    }

    if overlapping_tiles.len() == 2 {
        // 两瓦片：判定是否为“两个对角线的瓦片”（分别在 x/y 轴主导阻挡）
        let (_, _, rel_a, x_pen_a, y_pen_a) = overlapping_tiles[0];
        let (_, _, rel_b, x_pen_b, y_pen_b) = overlapping_tiles[1];

        let a_x_dominant = x_pen_a < y_pen_a;
        let b_x_dominant = x_pen_b < y_pen_b;

        return if a_x_dominant == b_x_dominant {
            // 直线瓦片（同一主导轴）：按该轴的最小分离处理
            if a_x_dominant {
                // X 轴主导：选择更小的 x 穿透
                let (pen, sign) = if x_pen_a <= x_pen_b {
                    (x_pen_a, if rel_a.x > 0.0 { 1.0 } else { -1.0 })
                } else {
                    (x_pen_b, if rel_b.x > 0.0 { 1.0 } else { -1.0 })
                };
                Vec2::new(sign * (pen + slop_axis), 0.0)
            } else {
                // Y 轴主导：选择更小的 y 穿透
                let (pen, sign) = if y_pen_a <= y_pen_b {
                    (y_pen_a, if rel_a.y > 0.0 { 1.0 } else { -1.0 })
                } else {
                    (y_pen_b, if rel_b.y > 0.0 { 1.0 } else { -1.0 })
                };
                Vec2::new(0.0, sign * (pen + slop_axis))
            }
        } else {
            // 对角线瓦片（不同主导轴）：合成对角分离，x 来自 x 主导瓦片，y 来自 y 主导瓦片
            let sign_pen = |pos: f32, pen: f32| {
                if pos > 0.0 {
                    pen + slop_diag
                } else {
                    -(pen + slop_diag)
                }
            };

            let (dx, dy) = if a_x_dominant {
                (sign_pen(rel_a.x, x_pen_a), sign_pen(rel_b.y, y_pen_b))
            } else {
                (sign_pen(rel_b.x, x_pen_b), sign_pen(rel_a.y, y_pen_a))
            };
            Vec2::new(dx, dy)
        };
    }

    // 对于更多瓦片的复杂角落/凹形/凸形组合，可以考虑基于 SDF 的梯度方向或约束求解器
    // 多瓦片（>=3）：综合考虑所有重叠瓦片，按主导轴分别求平均分离
    // 保留原有逻辑并做轻微稳健性调整
    let mut x_separation = 0.0;
    let mut y_separation = 0.0;
    let mut x_count = 0;
    let mut y_count = 0;

    for (_, _, relative_pos, x_pen, y_pen) in &overlapping_tiles {
        // 如果这个瓦片主要在 x 轴上阻挡（或 y 相对更大）
        if x_pen < &0.5 || y_pen > &(x_pen * 1.5) {
            x_separation += if relative_pos.x > 0.0 {
                *x_pen
            } else {
                -*x_pen
            };
            x_count += 1;
        }

        // 如果这个瓦片主要在 y 轴上阻挡（或 x 相对更大）
        if y_pen < &0.5 || x_pen > &(y_pen * 1.5) {
            y_separation += if relative_pos.y > 0.0 {
                *y_pen
            } else {
                -*y_pen
            };
            y_count += 1;
        }
    }

    // 计算平均分离，但保证至少有一定的分离量
    let final_x = if x_count > 0 {
        x_separation / x_count as f32 + slop_diag
    } else {
        0.0
    };
    let final_y = if y_count > 0 {
        y_separation / y_count as f32 + slop_diag
    } else {
        0.0
    };

    Vec2::new(final_x, final_y)
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

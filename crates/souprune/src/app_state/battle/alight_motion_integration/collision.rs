//! # collision.rs
//!
//! # collision.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Turns imported Alight Motion layers into battle collision data. It identifies bullet
//! and battle-box layers, derives their runtime bounds from animated layer specs, and keeps those
//! bounds synchronized as the animation scale changes over time.
//!
//! 负责把导入的 Alight Motion 图层转换成战斗碰撞数据。它会识别子弹层和战斗框层，
//! 从动画化的图层规格里推导运行时边界，并在缩放随时间变化时持续同步这些边界。

use super::{AlightMotionBattleBoxMarker, AlightMotionBattleConfig, AlightMotionBulletMarker};
use bevy::prelude::*;
use bevy_alight_motion::prelude::{
    AmAnimated, AmLayerSpec, AmPendingLayers, AmPlayback, interpolate_vec2,
};

use crate::core::alight_motion_runtime::AlightMotionPerformanceState;
use crate::core::battle_box::{
    AlightMotionBattleBoxBounds, BattleBox, BattleBoxId, BattleBoxState, BattleBoxVisualStyle,
};
use crate::core::collision::TriggerCollider;
use crate::core::danmaku::{
    Bullet, BulletDamage, BulletHitBehavior, BulletLastHitTime, BulletMotionState,
};

/// System to add collision components to marked AM entities.
pub(super) fn add_am_collision_system(
    mut commands: Commands,
    am_config: Res<AlightMotionBattleConfig>,
    am_state: Res<AlightMotionPerformanceState>,
    bullet_marker_query: Query<Entity, (With<AlightMotionBulletMarker>, Without<Bullet>)>,
    battle_box_marker_query: Query<Entity, (With<AlightMotionBattleBoxMarker>, Without<BattleBox>)>,
    layer_spec_query: Query<&AmLayerSpec>,
    animated_query: Query<&AmAnimated>,
    parent_query: Query<&ChildOf>,
) {
    for entity in bullet_marker_query.iter() {
        let (width, height) = if let Ok(spec) = layer_spec_query.get(entity) {
            if let Some((w, h)) = get_layer_size(spec) {
                info!(
                    "[AM Battle] Entity {:?} layer spec size: {}x{} (spec={:?})",
                    entity, w, h, spec
                );
                (w, h)
            } else {
                info!(
                    "[AM Battle] SKIPPING entity {:?} - not a visual element (spec={:?})",
                    entity, spec
                );
                continue;
            }
        } else {
            info!("[AM Battle] SKIPPING entity {:?} - no AmLayerSpec", entity);
            continue;
        };

        let total_scale =
            compute_total_scale(entity, &animated_query, &parent_query, am_state.final_scale);
        let half_size = Vec2::new(width * total_scale.x / 2.0, height * total_scale.y / 2.0);

        commands.entity(entity).insert((
            Bullet,
            TriggerCollider::Box { half_size },
            BulletDamage(am_config.bullet_damage),
            BulletHitBehavior {
                despawn_on_hit: false,
                damage_on_player_moving: false,
                damage_on_player_stationary: false,
                invincibility_duration: 0.0,
            },
            BulletLastHitTime::default(),
            BulletMotionState::new(Vec2::ZERO),
        ));

        info!(
            "[AM Battle] ADDED COLLISION to entity {:?} (half_size={:?}, size=({:.1}x{:.1}), total_scale={:?}, damage={})",
            entity, half_size, width, height, total_scale, am_config.bullet_damage
        );
    }

    for entity in battle_box_marker_query.iter() {
        let is_visual = layer_spec_query
            .get(entity)
            .ok()
            .is_some_and(is_visual_element);
        if !is_visual {
            continue;
        }

        let total_scale =
            compute_total_scale(entity, &animated_query, &parent_query, am_state.final_scale);

        let (width, height) = if let Ok(spec) = layer_spec_query.get(entity) {
            if let Some((w, h)) = get_layer_size(spec) {
                (w.abs() * total_scale.x, h.abs() * total_scale.y)
            } else {
                am_config.default_battle_box_size
            }
        } else {
            am_config.default_battle_box_size
        };

        let center_offset = if let Ok(animated) = animated_query.get(entity) {
            -animated.anchor_offset * total_scale
        } else {
            Vec2::ZERO
        };

        commands.entity(entity).insert((
            BattleBox,
            BattleBoxId("main".to_string()),
            BattleBoxState::default(),
            BattleBoxVisualStyle::default(),
            AlightMotionBattleBoxBounds {
                width,
                height,
                center_offset,
            },
        ));

        info!(
            "[AM Battle] Added BattleBox to entity {:?} (size={}x{}, total_scale={:?}, center_offset={:?})",
            entity, width, height, total_scale, center_offset
        );
    }
}

/// System to dynamically update battle box bounds based on current animation time.
pub(super) fn update_am_battle_box_bounds_system(
    playback: Option<Res<AmPlayback>>,
    am_state: Res<AlightMotionPerformanceState>,
    mut battle_box_query: Query<(
        Entity,
        &AmAnimated,
        &AmLayerSpec,
        &mut AlightMotionBattleBoxBounds,
    )>,
    parent_query: Query<&ChildOf>,
    animated_query: Query<&AmAnimated>,
) {
    let Some(playback) = playback else {
        return;
    };
    if !am_state.is_playing {
        return;
    }

    let current_time_ms = playback.current_time_ms;

    for (entity, animated, layer_spec, mut bounds) in battle_box_query.iter_mut() {
        let (base_width, base_height) = match layer_spec {
            AmLayerSpec::SdfShape { width, height, .. } => (width.abs(), height.abs()),
            AmLayerSpec::Image { width, height, .. } => (width.abs(), height.abs()),
            _ => continue,
        };

        let total_scale = compute_total_scale_at_time(
            entity,
            &animated_query,
            &parent_query,
            am_state.final_scale,
            current_time_ms,
        );

        let local_time = animated.calc_local_time(current_time_ms);
        let local_scale = get_animated_scale_at_time(&animated.scale, local_time);
        let new_width = base_width * total_scale.x * local_scale.x;
        let new_height = base_height * total_scale.y * local_scale.y;
        let full_scale = total_scale * local_scale;
        let new_center_offset = -animated.anchor_offset * full_scale;

        if (bounds.width - new_width).abs() > 0.1
            || (bounds.height - new_height).abs() > 0.1
            || (bounds.center_offset - new_center_offset).length() > 0.1
        {
            bounds.width = new_width;
            bounds.height = new_height;
            bounds.center_offset = new_center_offset;
        }
    }
}

/// System to synchronize inv_fit_scale with the scale applied by souprune.
pub(super) fn sync_am_fit_scale_system(
    am_state: Res<AlightMotionPerformanceState>,
    mut pending_layers_query: Query<&mut AmPendingLayers>,
) {
    if !am_state.is_playing {
        return;
    }

    for mut pending_layers in pending_layers_query.iter_mut() {
        let expected_inv_fit_scale = 1.0 / am_state.final_scale;
        if (pending_layers.inv_fit_scale - expected_inv_fit_scale).abs() > 0.0001 {
            info!(
                "[AM Battle] Updating inv_fit_scale from {} to {} (final_scale={})",
                pending_layers.inv_fit_scale, expected_inv_fit_scale, am_state.final_scale
            );
            pending_layers.inv_fit_scale = expected_inv_fit_scale;
        }
    }
}

fn is_visual_element(spec: &AmLayerSpec) -> bool {
    matches!(
        spec,
        AmLayerSpec::SpriteShape { .. }
            | AmLayerSpec::SdfShape { .. }
            | AmLayerSpec::Image { .. }
            | AmLayerSpec::Text { .. }
    )
}

fn get_layer_size(spec: &AmLayerSpec) -> Option<(f32, f32)> {
    match spec {
        AmLayerSpec::SpriteShape { width, height, .. } => Some((*width, *height)),
        AmLayerSpec::SdfShape { width, height, .. } => Some((*width, *height)),
        AmLayerSpec::Image { width, height, .. } => Some((*width, *height)),
        AmLayerSpec::Text { .. }
        | AmLayerSpec::Null
        | AmLayerSpec::EmbedScene
        | AmLayerSpec::Camera { .. } => None,
    }
}

fn get_animated_scale(animated: &AmAnimated) -> Vec2 {
    if let Some(val) = &animated.scale.value {
        return Vec2::new(val[0].abs(), val[1].abs());
    }
    if let Some(kf) = animated.scale.keyframes.first() {
        let parts: Vec<&str> = kf.value.split(',').collect();
        if parts.len() == 2
            && let (Ok(x), Ok(y)) = (
                parts[0].trim().parse::<f32>(),
                parts[1].trim().parse::<f32>(),
            )
        {
            return Vec2::new(x.abs(), y.abs());
        }
    }
    Vec2::ONE
}

fn compute_total_scale(
    entity: Entity,
    animated_query: &Query<&AmAnimated>,
    parent_query: &Query<&ChildOf>,
    final_scale: f32,
) -> Vec2 {
    let mut total_scale = Vec2::splat(final_scale);
    let mut current = entity;

    loop {
        if let Ok(animated) = animated_query.get(current) {
            total_scale *= get_animated_scale(animated);
        }

        if let Ok(child_of) = parent_query.get(current) {
            current = child_of.0;
        } else {
            break;
        }
    }

    total_scale
}

fn get_animated_scale_at_time(
    scale_prop: &bevy_alight_motion::prelude::AmAnimatedVec2,
    local_time_ms: f32,
) -> Vec2 {
    if let Some([x, y]) = interpolate_vec2(scale_prop, local_time_ms) {
        Vec2::new(x.abs(), y.abs())
    } else {
        Vec2::ONE
    }
}

fn compute_total_scale_at_time(
    entity: Entity,
    animated_query: &Query<&AmAnimated>,
    parent_query: &Query<&ChildOf>,
    final_scale: f32,
    current_time_ms: f32,
) -> Vec2 {
    let mut total_scale = Vec2::splat(final_scale);
    let mut current = entity;

    if let Ok(child_of) = parent_query.get(current) {
        current = child_of.0;
    } else {
        return total_scale;
    }

    loop {
        if let Ok(animated) = animated_query.get(current) {
            let local_time = animated.calc_local_time(current_time_ms);
            total_scale *= get_animated_scale_at_time(&animated.scale, local_time);
        }

        if let Ok(child_of) = parent_query.get(current) {
            current = child_of.0;
        } else {
            break;
        }
    }

    total_scale
}

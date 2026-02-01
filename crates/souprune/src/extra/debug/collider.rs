//! # collider.rs
//!
//! # collider.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Implements debug visualization for 2D colliders using Bevy's native Gizmos system.
//! Press F3 to toggle collider visualization (Green for Player, Light Green for others, Cyan for triggers).
//!
//! 使用 Bevy 原生 Gizmos 系统实现 2D 碰撞体的调试可视化。
//! 按 F3 切换碰撞体可视化（玩家为绿色，其他为浅绿色，触发器为青色）。

#[cfg(feature = "debug")]
pub mod debug_collider {
    use crate::app_state::overworld::character::components::PlayerControlled;
    use crate::app_state::overworld::tilemap::systems::TilemapCollider;
    use crate::app_state::overworld::tilemap::*;
    use crate::app_state::overworld::trigger::TriggerZone;
    use crate::core::collision::{HitboxOffset, PhysicsCollider, Rect2DCollider, TriggerCollider};
    use bevy::color::palettes::css;
    use bevy::math::Isometry2d;
    use bevy::prelude::*;
    use bevy_alight_motion::sdf_material::SdfMaterial;

    /// Custom GizmoConfigGroup for collider debug visualization.
    ///
    /// 用于碰撞体调试可视化的自定义 GizmoConfigGroup。
    #[derive(Default, Reflect, GizmoConfigGroup)]
    pub struct ColliderGizmos;

    /// Set up the collider debug systems.
    ///
    /// 设置碰撞体调试系统。
    pub fn setup_collider_debug(app: &mut App) {
        app.init_gizmo_group::<ColliderGizmos>().add_systems(
            Update,
            (
                toggle_collider_visibility_system,
                draw_rect_collider_gizmos_system,
                draw_trigger_zone_gizmos_system,
                draw_battle_collider_gizmos_system,
                draw_am_mask_gizmos_system,
            ),
        );

        // Start with gizmos disabled
        if let Some(mut store) = app.world_mut().get_resource_mut::<GizmoConfigStore>() {
            let (config, _) = store.config_mut::<ColliderGizmos>();
            config.enabled = false;
        }
    }

    /// Toggle collider visibility with the F3 key.
    ///
    /// F3 键切换碰撞体可见性。
    fn toggle_collider_visibility_system(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut config_store: ResMut<GizmoConfigStore>,
    ) {
        if keyboard.just_pressed(KeyCode::F3) {
            let (config, _) = config_store.config_mut::<ColliderGizmos>();
            config.enabled = !config.enabled;
            info!(
                "Collider visualization: {}",
                if config.enabled { "ON" } else { "OFF" }
            );
        }
    }

    /// Draw rectangular colliders (Player, Tilemap, Object) using Gizmos.
    ///
    /// 使用 Gizmos 绘制矩形碰撞体（玩家、瓦片地图、对象）。
    #[allow(clippy::type_complexity)]
    fn draw_rect_collider_gizmos_system(
        mut gizmos: Gizmos<ColliderGizmos>,
        player_colliders: Query<
            (&GlobalTransform, &Rect2DCollider),
            (With<PlayerControlled>, Without<TriggerZone>),
        >,
        tilemap_colliders: Query<
            (&GlobalTransform, &Rect2DCollider),
            (
                With<TilemapCollider>,
                Without<PlayerControlled>,
                Without<TriggerZone>,
            ),
        >,
        object_colliders: Query<
            (&GlobalTransform, &Rect2DCollider),
            (
                With<ObjectCollider>,
                Without<PlayerControlled>,
                Without<TriggerZone>,
            ),
        >,
    ) {
        // Player colliders (bright green)
        for (transform, collider) in player_colliders.iter() {
            let pos = transform.translation().truncate() + collider.offset;
            gizmos.rect_2d(Isometry2d::from_translation(pos), collider.size, css::LIME);
        }

        // Tilemap colliders (light green)
        for (transform, collider) in tilemap_colliders.iter() {
            let pos = transform.translation().truncate() + collider.offset;
            gizmos.rect_2d(
                Isometry2d::from_translation(pos),
                collider.size,
                css::LIGHT_GREEN,
            );
        }

        // Object colliders (light green)
        for (transform, collider) in object_colliders.iter() {
            let pos = transform.translation().truncate() + collider.offset;
            gizmos.rect_2d(
                Isometry2d::from_translation(pos),
                collider.size,
                css::LIGHT_GREEN,
            );
        }
    }

    /// Draw TriggerZone entities using Gizmos (cyan).
    ///
    /// 使用 Gizmos 绘制 TriggerZone 实体（青色）。
    fn draw_trigger_zone_gizmos_system(
        mut gizmos: Gizmos<ColliderGizmos>,
        trigger_zones: Query<(&GlobalTransform, &Rect2DCollider, &TriggerZone)>,
    ) {
        for (transform, collider, trigger_zone) in trigger_zones.iter() {
            let pos = transform.translation().truncate() + collider.offset;
            let color = if trigger_zone.player_inside {
                css::LIGHT_CYAN // Brighter when active
            } else {
                css::DARK_CYAN
            };
            gizmos.rect_2d(Isometry2d::from_translation(pos), collider.size, color);
        }
    }

    /// Draw Battle colliders (PhysicsCollider, TriggerCollider, BattleBox) using Gizmos.
    ///
    /// 使用 Gizmos 绘制 Battle 碰撞体。
    #[allow(clippy::type_complexity)]
    fn draw_battle_collider_gizmos_system(
        mut gizmos: Gizmos<ColliderGizmos>,
        physics_colliders: Query<(&GlobalTransform, &PhysicsCollider)>,
        trigger_colliders: Query<(
            &GlobalTransform,
            &TriggerCollider,
            Option<&HitboxOffset>,
            Option<&crate::app_state::overworld::chase::ChasePlayerHitbox>,
        )>,
        battle_boxes: Query<
            (&GlobalTransform, &crate::core::view::components::ViewBox),
            With<crate::app_state::battle::collision::BattleBox>,
        >,
        am_battle_boxes: Query<
            (
                &GlobalTransform,
                &crate::app_state::battle::collision::AmBattleBoxBounds,
            ),
            (
                With<crate::app_state::battle::collision::BattleBox>,
                Without<crate::core::view::components::ViewBox>,
            ),
        >,
    ) {
        // Physics colliders (green)
        for (transform, physics_collider) in physics_colliders.iter() {
            let pos = transform.translation().truncate();
            match physics_collider {
                PhysicsCollider::Circle { radius } => {
                    gizmos.circle_2d(Isometry2d::from_translation(pos), *radius, css::LIME);
                }
                PhysicsCollider::Box { half_size } => {
                    gizmos.rect_2d(
                        Isometry2d::from_translation(pos),
                        *half_size * 2.0,
                        css::LIME,
                    );
                }
            }
        }

        // Trigger colliders (green for battle, red for chase hitbox)
        for (transform, trigger_collider, hitbox_offset, chase_hitbox) in trigger_colliders.iter() {
            let offset = hitbox_offset.map(|o| o.0).unwrap_or(Vec2::ZERO);
            let pos = transform.translation().truncate() + offset;
            let color = if chase_hitbox.is_some() {
                css::RED
            } else {
                css::LIME
            };

            match trigger_collider {
                TriggerCollider::Circle { radius } => {
                    gizmos.circle_2d(Isometry2d::from_translation(pos), *radius, color);
                }
                TriggerCollider::Box { half_size } => {
                    gizmos.rect_2d(Isometry2d::from_translation(pos), *half_size * 2.0, color);
                }
            }
        }

        // Battle boxes (green)
        for (transform, view_box) in battle_boxes.iter() {
            let pos = transform.translation().truncate();
            let size = Vec2::new(view_box.width(), view_box.height());
            gizmos.rect_2d(Isometry2d::from_translation(pos), size, css::LIME);
        }

        // AM Battle boxes (dark cyan)
        for (transform, am_bounds) in am_battle_boxes.iter() {
            let pos = transform.translation().truncate() + am_bounds.center_offset;
            let size = Vec2::new(am_bounds.width, am_bounds.height);
            gizmos.rect_2d(Isometry2d::from_translation(pos), size, css::DARK_CYAN);
        }
    }

    /// Draw AM masks using Gizmos (red).
    /// Reads mask_params from SdfMaterial to get mask bounds.
    ///
    /// 使用 Gizmos 绘制 AM 遮罩（红色）。
    fn draw_am_mask_gizmos_system(
        mut gizmos: Gizmos<ColliderGizmos>,
        sdf_query: Query<&bevy::sprite_render::MeshMaterial2d<SdfMaterial>>,
        sdf_materials: Res<Assets<SdfMaterial>>,
    ) {
        // Find SDF shapes with active masks and draw their bounds
        for material_handle in sdf_query.iter() {
            if let Some(material) = sdf_materials.get(&material_handle.0) {
                if material.uniform_data.mask_type > 0.5 {
                    let params = material.uniform_data.mask_params;
                    let center = Vec2::new(params.x, params.y);
                    let size = Vec2::new(params.z * 2.0, params.w * 2.0);
                    gizmos.rect_2d(Isometry2d::from_translation(center), size, css::RED);
                    // Only draw the first mask found
                    break;
                }
            }
        }
    }
}

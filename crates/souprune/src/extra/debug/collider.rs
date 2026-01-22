//! # collider.rs
//!
//! # collider.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Implements debug visualization for 2D colliders (Rect2DCollider), rendering them as colored outlines (Green for Player, Dark/Light Green for others) when F3 is pressed.
//!
//! 实现 2D 碰撞体 (Rect2DCollider) 的调试可视化，按下 F3 时将其渲染为彩色轮廓（玩家为绿色，其他为深/浅绿色）。

#[cfg(feature = "debug")]
pub mod debug_collider {

    /// Debug resource controlling collider visibility.
    ///
    /// 控制碰撞体可见性的调试资源。
    #[derive(Resource, Default)]
    pub struct ColliderDebugSettings {
        pub show_colliders: bool,
    }

    use crate::app_state::overworld::character::components::PlayerControlled;
    use crate::app_state::overworld::tilemap::systems::TilemapCollider;
    use crate::app_state::overworld::tilemap::*;
    use crate::app_state::overworld::trigger::TriggerZone;
    use crate::core::collision::{PhysicsCollider, Rect2DCollider, TriggerCollider};
    use bevy::prelude::*;
    use bevy_smud::prelude::*;

    /// Marker component for collision visualizer entities.
    ///
    /// 碰撞体可视化实体的标记组件。
    #[derive(Component)]
    pub struct ColliderVisualizer {
        pub parent: Entity,
    }

    /// Marker to prevent duplicate Battle collider visualizers
    ///
    /// 防止重复创建 Battle 碰撞体可视化器的标记
    #[derive(Component)]
    pub struct BattleColliderVisualized;

    /// Marker component for AM mask visualizer entities.
    ///
    /// AM 遮罩可视化实体的标记组件。
    #[derive(Component)]
    pub struct AmMaskVisualizer {
        /// The mask layer id this visualizer is tracking
        pub mask_layer_id: u64,
    }

    /// Root entity for organizing all debug visualizers.
    ///
    /// 用于组织所有调试可视化器的根实体。
    #[derive(Component)]
    pub struct DebugVisualizerRoot;

    /// Set up the collider debug systems.
    ///
    /// 设置碰撞体调试系统。
    pub fn setup_collider_debug(app: &mut App) {
        app.init_resource::<ColliderDebugSettings>()
            .add_systems(Startup, setup_debug_visualizer_root_system)
            .add_systems(
                Update,
                (
                    toggle_collider_visibility_system,
                    render_rect_colliders_system,
                    update_collider_visualizer_positions_system,
                    render_battle_colliders_system,
                    update_battle_collider_visualizer_positions_system,
                ),
            );

        app.add_systems(Update, render_trigger_zones_system);

        // AM mask debug visualization
        app.add_systems(
            Update,
            (
                render_am_masks_system,
                update_am_mask_visualizer_positions_system,
            ),
        );
    }

    /// Set up the root entity for debug visualizers.
    ///
    /// 设置调试可视化器的根实体。
    fn setup_debug_visualizer_root_system(mut commands: Commands) {
        commands.spawn((
            DebugVisualizerRoot,
            Name::new("DebugVisualizers"),
            Transform::default(),
            Visibility::default(),
        ));
    }

    /// Toggle collider visibility with the F3 key (debug only).
    ///
    /// F3 键切换碰撞体可见性的系统（仅调试模式）。
    fn toggle_collider_visibility_system(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut settings: ResMut<ColliderDebugSettings>,
    ) {
        if keyboard.just_pressed(KeyCode::F3) {
            settings.show_colliders = !settings.show_colliders;
            info!(
                "Collider visualization: {}",
                if settings.show_colliders { "ON" } else { "OFF" }
            );
        }
    }

    /// System to render rectangular colliders using bevy_smud SDF (debug only).
    ///
    /// 使用bevy_smud SDF渲染矩形碰撞体的系统（仅调试模式）。
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    fn render_rect_colliders_system(
        mut commands: Commands,
        mut shaders: ResMut<Assets<Shader>>,
        settings: Res<ColliderDebugSettings>,
        debug_root: Query<Entity, With<DebugVisualizerRoot>>,
        player_colliders: Query<
            (Entity, &Transform, &Rect2DCollider),
            (
                With<PlayerControlled>,
                Without<SmudShape>,
                Without<ColliderVisualizer>,
            ),
        >,
        tilemap_colliders: Query<
            (Entity, &Transform, &Rect2DCollider),
            (
                With<TilemapCollider>,
                Without<PlayerControlled>,
                Without<SmudShape>,
                Without<ColliderVisualizer>,
            ),
        >,
        object_colliders: Query<
            (Entity, &Transform, &Rect2DCollider),
            (
                With<ObjectCollider>,
                Without<PlayerControlled>,
                Without<SmudShape>,
                Without<ColliderVisualizer>,
            ),
        >,
        // Queries for Battle colliders to prevent cleanup
        battle_physics_colliders: Query<Entity, With<PhysicsCollider>>,
        battle_trigger_colliders: Query<Entity, With<TriggerCollider>>,
        battle_boxes: Query<Entity, With<crate::app_state::battle::collision::BattleBox>>,
        fre_trigger_zones: Query<Entity, With<TriggerZone>>,
        existing_visualizers: Query<(Entity, &ColliderVisualizer)>,
    ) {
        let Ok(debug_root_entity) = debug_root.single() else {
            return;
        };

        // If colliders are hidden in the settings, remove every visualizer and exit early.
        //
        // 若调试设置隐藏了碰撞体，则移除所有可视化器并提前返回。
        if !settings.show_colliders {
            for (visualizer_entity, _) in existing_visualizers.iter() {
                commands.entity(visualizer_entity).despawn();
            }
            return;
        }

        // Remove visualizers for entities that no longer have colliders.
        //
        // 对于已失去碰撞体的实体，移除其对应的可视化器。
        for (visualizer_entity, visualizer) in existing_visualizers.iter() {
            let mut parent_exists = player_colliders.get(visualizer.parent).is_ok()
                || tilemap_colliders.get(visualizer.parent).is_ok()
                || object_colliders.get(visualizer.parent).is_ok()
                || battle_physics_colliders.get(visualizer.parent).is_ok()
                || battle_trigger_colliders.get(visualizer.parent).is_ok()
                || battle_boxes.get(visualizer.parent).is_ok();

            parent_exists = parent_exists || fre_trigger_zones.get(visualizer.parent).is_ok();

            if !parent_exists {
                commands.entity(visualizer_entity).despawn();
            }
        }

        // Helper closure for spawning collider visualizers.
        //
        // 用于生成碰撞体可视化器的辅助闭包。
        let mut create_visualizer = |entity: Entity,
                                     transform: &Transform,
                                     collider: &Rect2DCollider,
                                     color: Color,
                                     name: String| {
            // Skip entities that already own a visualizer.
            //
            // 若实体已拥有可视化器则跳过。
            let has_visualizer = existing_visualizers
                .iter()
                .any(|(_, vis)| vis.parent == entity);
            if has_visualizer {
                return;
            }

            // Build a thin-border SDF via a distance field outline.
            //
            // 用距离场轮廓构建细边框 SDF。
            let border_sdf = shaders.add_sdf_expr(format!(
                "abs(smud::sd_box(p, vec2<f32>({}, {}))) - {}",
                collider.size.x / 2.0,
                collider.size.y / 2.0,
                0.125
            ));

            // Calculate the frame size from the collider dimensions.
            //
            // 根据碰撞体尺寸计算框架大小。
            let frame_size = (collider.size.x.max(collider.size.y) / 2.0) + 2.0;

            // Combine the transform and offset to get the final position.
            //
            // 将变换与偏移相加以得到最终位置。
            let final_position = transform.translation + collider.offset.extend(0.1);

            // Spawn the visual representation as a child of the debug root.
            //
            // 将可视化实体生成在调试根节点下。
            commands.entity(debug_root_entity).with_children(|parent| {
                parent.spawn((
                    ColliderVisualizer { parent: entity },
                    SmudShape {
                        color,
                        sdf: border_sdf,
                        frame: Frame::Quad(frame_size),
                        fill: SIMPLE_FILL_HANDLE,
                        ..default()
                    },
                    Transform::from_translation(final_position),
                    Name::new(name),
                ));
            });
        };

        // Create visualizers for player colliders (green).
        //
        // 为玩家碰撞体创建绿色可视化器。
        for (entity, transform, collider) in player_colliders.iter() {
            create_visualizer(
                entity,
                transform,
                collider,
                Color::hsl(120.0, 1.0, 0.5),
                "Player Collider".to_string(),
            );
        }

        // Create visualizers for tilemap colliders (dark green).
        //
        // 为瓦片地图碰撞体创建深绿色可视化器。
        for (entity, transform, collider) in tilemap_colliders.iter() {
            create_visualizer(
                entity,
                transform,
                collider,
                Color::hsl(120.0, 0.75, 0.75),
                "Tilemap Collider".to_string(),
            );
        }

        // Create visualizers for object colliders (light green).
        //
        // 为对象碰撞体创建浅绿色可视化器。
        for (entity, transform, collider) in object_colliders.iter() {
            create_visualizer(
                entity,
                transform,
                collider,
                Color::hsl(120.0, 0.75, 0.75),
                "Object Collider".to_string(),
            );
        }
    }

    /// System to render FRE TriggerZone entities (cyan color).
    ///
    /// 渲染 FRE TriggerZone 实体的系统（青色）。
    #[allow(clippy::type_complexity)]
    fn render_trigger_zones_system(
        mut commands: Commands,
        mut shaders: ResMut<Assets<Shader>>,
        settings: Res<ColliderDebugSettings>,
        debug_root: Query<Entity, With<DebugVisualizerRoot>>,
        trigger_zones: Query<
            (Entity, &Transform, &Rect2DCollider, &TriggerZone),
            (Without<SmudShape>, Without<ColliderVisualizer>),
        >,
        existing_visualizers: Query<(Entity, &ColliderVisualizer)>,
    ) {
        let Ok(debug_root_entity) = debug_root.single() else {
            return;
        };

        if !settings.show_colliders {
            return;
        }

        for (entity, transform, collider, trigger_zone) in trigger_zones.iter() {
            // Skip entities that already own a visualizer.
            let has_visualizer = existing_visualizers
                .iter()
                .any(|(_, vis)| vis.parent == entity);
            if has_visualizer {
                continue;
            }

            // Build a thin-border SDF (cyan color for trigger zones)
            let border_sdf = shaders.add_sdf_expr(format!(
                "abs(smud::sd_box(p, vec2<f32>({}, {}))) - {}",
                collider.size.x / 2.0,
                collider.size.y / 2.0,
                0.125
            ));

            let frame_size = (collider.size.x.max(collider.size.y) / 2.0) + 2.0;
            let final_position = transform.translation + collider.offset.extend(0.1);

            // Use cyan color for trigger zones to distinguish from colliders
            let color = if trigger_zone.player_inside {
                Color::hsl(180.0, 1.0, 0.7) // Brighter cyan when active
            } else {
                Color::hsl(180.0, 1.0, 0.5) // Normal cyan
            };

            commands.entity(debug_root_entity).with_children(|parent| {
                parent.spawn((
                    ColliderVisualizer { parent: entity },
                    SmudShape {
                        color,
                        sdf: border_sdf,
                        frame: Frame::Quad(frame_size),
                        fill: SIMPLE_FILL_HANDLE,
                        ..default()
                    },
                    Transform::from_translation(final_position),
                    Name::new(format!("TriggerZone_{}", trigger_zone.id)),
                ));
            });
        }
    }

    /// Update visualizer positions when parent transforms change.
    ///
    /// 当父变换改变时更新可视化器位置的系统。
    #[allow(clippy::type_complexity)]
    fn update_collider_visualizer_positions_system(
        mut visualizers: Query<(&mut Transform, &ColliderVisualizer), Without<Rect2DCollider>>,
        colliders: Query<
            (&Transform, &Rect2DCollider),
            (With<Rect2DCollider>, Without<ColliderVisualizer>),
        >,
    ) {
        for (mut vis_transform, visualizer) in visualizers.iter_mut() {
            if let Ok((parent_transform, collider)) = colliders.get(visualizer.parent) {
                let final_position = parent_transform.translation + collider.offset.extend(0.1);
                vis_transform.translation = final_position;
            }
        }
    }

    /// System to render Battle colliders (PhysicsCollider and TriggerCollider)
    ///
    /// Battle 碰撞体（物理碰撞体和触发器）的渲染系统
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::type_complexity)]
    fn render_battle_colliders_system(
        mut commands: Commands,
        mut shaders: ResMut<Assets<Shader>>,
        settings: Res<ColliderDebugSettings>,
        debug_root: Query<Entity, With<DebugVisualizerRoot>>,
        physics_colliders: Query<
            (Entity, &Transform, &crate::core::collision::PhysicsCollider),
            (
                Without<SmudShape>,
                Without<ColliderVisualizer>,
                Without<BattleColliderVisualized>,
            ),
        >,
        trigger_colliders: Query<
            (
                Entity,
                &GlobalTransform,
                &crate::core::collision::TriggerCollider,
                Option<&crate::core::collision::HitboxOffset>,
            ),
            (
                Without<SmudShape>,
                Without<ColliderVisualizer>,
                Without<BattleColliderVisualized>,
            ),
        >,
        chase_hitboxes: Query<Entity, With<crate::app_state::overworld::chase::ChasePlayerHitbox>>,
        battle_boxes: Query<
            (
                Entity,
                &GlobalTransform,
                &crate::core::ui::components::UIBox,
            ),
            (
                With<crate::app_state::battle::collision::BattleBox>,
                Without<SmudShape>,
                Without<ColliderVisualizer>,
                Without<BattleColliderVisualized>,
                Without<crate::core::collision::PhysicsCollider>,
            ),
        >,
        // AM animated battle boxes (no UIBox, use AmBattleBoxBounds instead)
        am_battle_boxes: Query<
            (
                Entity,
                &GlobalTransform,
                &crate::app_state::battle::collision::AmBattleBoxBounds,
            ),
            (
                With<crate::app_state::battle::collision::BattleBox>,
                Without<crate::core::ui::components::UIBox>,
                Without<SmudShape>,
                Without<ColliderVisualizer>,
                Without<BattleColliderVisualized>,
                Without<crate::core::collision::PhysicsCollider>,
            ),
        >,
        existing_visualizers: Query<(Entity, &ColliderVisualizer)>,
    ) {
        use crate::core::collision::{PhysicsCollider, TriggerCollider};

        let Ok(debug_root_entity) = debug_root.single() else {
            return;
        };

        if !settings.show_colliders {
            return;
        }

        // Visualize physics colliders (green)
        for (entity, transform, physics_collider) in physics_colliders.iter() {
            // Check if visualizer already exists
            let has_visualizer = existing_visualizers
                .iter()
                .any(|(_, vis)| vis.parent == entity);

            if has_visualizer {
                continue;
            }

            let (sdf, frame_size, name) = match physics_collider {
                PhysicsCollider::Circle { radius } => {
                    let sdf = shaders
                        .add_sdf_expr(format!("abs(smud::sd_circle(p, {})) - 0.125", radius));
                    (sdf, radius + 2.0, "Physics Collider (Circle)")
                }
                PhysicsCollider::Box { half_size } => {
                    let sdf = shaders.add_sdf_expr(format!(
                        "abs(smud::sd_box(p, vec2<f32>({}, {}))) - 0.125",
                        half_size.x, half_size.y
                    ));
                    let max_dim = half_size.x.max(half_size.y) + 2.0;
                    (sdf, max_dim, "Physics Collider (Box)")
                }
            };

            commands.entity(debug_root_entity).with_children(|parent| {
                parent.spawn((
                    SmudShape {
                        color: Color::hsl(120.0, 1.0, 0.5),
                        sdf,
                        frame: Frame::Quad(frame_size),
                        fill: SIMPLE_FILL_HANDLE,
                        ..default()
                    },
                    Transform::from_translation(transform.translation + Vec3::new(0.0, 0.0, 20.0)),
                    ColliderVisualizer { parent: entity },
                    Name::new(name.to_string()),
                ));
            });
        }

        // Visualize trigger colliders (green for battle, red for chase hitbox)
        // 可视化触发器碰撞体（战斗为绿色，追逐战判定框为红色）
        for (entity, transform, trigger_collider, hitbox_offset) in trigger_colliders.iter() {
            let has_visualizer = existing_visualizers
                .iter()
                .any(|(_, vis)| vis.parent == entity);

            if has_visualizer {
                continue;
            }

            // Determine if this is a chase hitbox (red) or battle collider (green)
            let is_chase_hitbox = chase_hitboxes.get(entity).is_ok();

            let color = if is_chase_hitbox {
                Color::hsl(0.0, 1.0, 0.5) // Red for chase hitbox
            } else {
                Color::hsl(120.0, 1.0, 0.5) // Green for battle
            };

            let (sdf, frame_size, name) = match trigger_collider {
                TriggerCollider::Circle { radius } => {
                    let sdf = shaders
                        .add_sdf_expr(format!("abs(smud::sd_circle(p, {})) - 0.125", radius));
                    (
                        sdf,
                        radius + 2.0,
                        if is_chase_hitbox {
                            "Chase Hitbox (Circle)"
                        } else {
                            "Trigger Collider (Circle)"
                        },
                    )
                }
                TriggerCollider::Box { half_size } => {
                    let sdf = shaders.add_sdf_expr(format!(
                        "abs(smud::sd_box(p, vec2<f32>({}, {}))) - 0.125",
                        half_size.x, half_size.y
                    ));
                    let max_dim = half_size.x.max(half_size.y) + 2.0;
                    (
                        sdf,
                        max_dim,
                        if is_chase_hitbox {
                            "Chase Hitbox (Box)"
                        } else {
                            "Trigger Collider (Box)"
                        },
                    )
                }
            };

            // Apply hitbox offset if present
            let offset = hitbox_offset.map(|o| o.0).unwrap_or(Vec2::ZERO);
            let position = transform.translation() + offset.extend(20.0);

            commands.entity(debug_root_entity).with_children(|parent| {
                parent.spawn((
                    SmudShape {
                        color,
                        sdf,
                        frame: Frame::Quad(frame_size),
                        fill: SIMPLE_FILL_HANDLE,
                        ..default()
                    },
                    Transform::from_translation(position),
                    ColliderVisualizer { parent: entity },
                    Name::new(name.to_string()),
                ));
            });
        }

        // Visualize Battle Box (White)
        for (entity, global_transform, ui_box) in battle_boxes.iter() {
            let has_visualizer = existing_visualizers
                .iter()
                .any(|(_, vis)| vis.parent == entity);

            if has_visualizer {
                continue;
            }

            let half_width = ui_box.width() / 2.0;
            let half_height = ui_box.height() / 2.0;

            let sdf = shaders.add_sdf_expr(format!(
                "abs(smud::sd_box(p, vec2<f32>({}, {}))) - 0.125",
                half_width, half_height
            ));

            let frame_size = half_width.max(half_height) + 2.0;

            commands.entity(debug_root_entity).with_children(|parent| {
                parent.spawn((
                    SmudShape {
                        color: Color::hsl(120.0, 1.0, 0.5),
                        sdf,
                        frame: Frame::Quad(frame_size),
                        fill: SIMPLE_FILL_HANDLE,
                        ..default()
                    },
                    Transform::from_translation(
                        global_transform.translation() + Vec3::new(0.0, 0.0, 50.0),
                    ),
                    ColliderVisualizer { parent: entity },
                    Name::new("Battle Box Debug"),
                ));
            });
        }

        // Visualize AM Battle Box (Cyan to distinguish from UI battle box)
        for (entity, global_transform, am_bounds) in am_battle_boxes.iter() {
            let has_visualizer = existing_visualizers
                .iter()
                .any(|(_, vis)| vis.parent == entity);

            if has_visualizer {
                continue;
            }

            let half_width = am_bounds.width / 2.0;
            let half_height = am_bounds.height / 2.0;

            let sdf = shaders.add_sdf_expr(format!(
                "abs(smud::sd_box(p, vec2<f32>({}, {}))) - 0.125",
                half_width, half_height
            ));

            let frame_size = half_width.max(half_height) + 2.0;

            // Apply center_offset to get the actual geometric center
            let center_pos = global_transform.translation()
                + Vec3::new(am_bounds.center_offset.x, am_bounds.center_offset.y, 50.0);

            commands.entity(debug_root_entity).with_children(|parent| {
                parent.spawn((
                    SmudShape {
                        color: Color::hsl(180.0, 1.0, 0.5), // Cyan for AM battle box
                        sdf,
                        frame: Frame::Quad(frame_size),
                        fill: SIMPLE_FILL_HANDLE,
                        ..default()
                    },
                    Transform::from_translation(center_pos),
                    ColliderVisualizer { parent: entity },
                    Name::new("AM Battle Box Debug"),
                ));
            });
        }
    }

    /// Update Battle collider visualizer positions
    ///
    /// 更新 Battle 碰撞体可视化器位置
    #[allow(clippy::type_complexity)]
    fn update_battle_collider_visualizer_positions_system(
        mut visualizers: Query<
            (&mut Transform, &ColliderVisualizer),
            (
                Without<crate::core::collision::PhysicsCollider>,
                Without<crate::core::collision::TriggerCollider>,
            ),
        >,
        physics_colliders: Query<
            &Transform,
            (
                With<crate::core::collision::PhysicsCollider>,
                Without<ColliderVisualizer>,
            ),
        >,
        trigger_colliders: Query<
            (
                &GlobalTransform,
                Option<&crate::core::collision::HitboxOffset>,
            ),
            (
                With<crate::core::collision::TriggerCollider>,
                Without<ColliderVisualizer>,
            ),
        >,
        battle_boxes: Query<
            &GlobalTransform,
            (
                With<crate::core::ui::components::UIBox>,
                With<crate::app_state::battle::collision::BattleBox>,
                Without<ColliderVisualizer>,
            ),
        >,
        am_battle_boxes: Query<
            (
                &GlobalTransform,
                &crate::app_state::battle::collision::AmBattleBoxBounds,
            ),
            (
                With<crate::app_state::battle::collision::BattleBox>,
                Without<crate::core::ui::components::UIBox>,
                Without<ColliderVisualizer>,
            ),
        >,
    ) {
        for (mut vis_transform, visualizer) in visualizers.iter_mut() {
            if let Ok(parent_transform) = physics_colliders.get(visualizer.parent) {
                vis_transform.translation.x = parent_transform.translation.x;
                vis_transform.translation.y = parent_transform.translation.y;
            } else if let Ok((parent_global, hitbox_offset)) =
                trigger_colliders.get(visualizer.parent)
            {
                let offset = hitbox_offset.map(|o| o.0).unwrap_or(Vec2::ZERO);
                vis_transform.translation.x = parent_global.translation().x + offset.x;
                vis_transform.translation.y = parent_global.translation().y + offset.y;
            } else if let Ok(parent_global) = battle_boxes.get(visualizer.parent) {
                vis_transform.translation.x = parent_global.translation().x;
                vis_transform.translation.y = parent_global.translation().y;
            } else if let Ok((parent_global, am_bounds)) = am_battle_boxes.get(visualizer.parent) {
                // Apply center_offset to get the actual geometric center
                vis_transform.translation.x =
                    parent_global.translation().x + am_bounds.center_offset.x;
                vis_transform.translation.y =
                    parent_global.translation().y + am_bounds.center_offset.y;
            }
        }
    }

    /// System to render AM masks as semi-transparent red rectangles (debug only).
    /// Reads actual mask_params from SdfMaterial to ensure exact match.
    ///
    /// 以红色半透明矩形渲染 AM 遮罩的系统（仅调试模式）。
    /// 从 SdfMaterial 读取实际的 mask_params 以确保完全匹配。
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::type_complexity)]
    fn render_am_masks_system(
        mut commands: Commands,
        mut shaders: ResMut<Assets<Shader>>,
        settings: Res<ColliderDebugSettings>,
        debug_root: Query<Entity, With<DebugVisualizerRoot>>,
        // Query SDF shapes that have masks to get actual mask_params
        sdf_query: Query<(
            Entity,
            &MeshMaterial2d<bevy_alight_motion::sdf_material::SdfMaterial>,
        )>,
        sdf_materials: Res<Assets<bevy_alight_motion::sdf_material::SdfMaterial>>,
        existing_visualizers: Query<(Entity, &AmMaskVisualizer)>,
    ) {
        let Ok(debug_root_entity) = debug_root.single() else {
            return;
        };

        // Remove visualizers when debug mode is off
        if !settings.show_colliders {
            for (visualizer_entity, _) in existing_visualizers.iter() {
                commands.entity(visualizer_entity).despawn();
            }
            return;
        }

        // Find the first SDF shape with an active mask and read its mask_params
        let mut found_mask_params: Option<(f32, f32, f32, f32)> = None;
        let mut total_sdf_shapes = 0;
        let mut masked_shapes = 0;
        for (_, material_handle) in sdf_query.iter() {
            total_sdf_shapes += 1;
            if let Some(material) = sdf_materials.get(&material_handle.0) {
                let mask_type = material.uniform_data.mask_type;
                if mask_type > 0.5 {
                    masked_shapes += 1;
                    // This shape has an active mask
                    let params = material.uniform_data.mask_params;
                    if found_mask_params.is_none() {
                        found_mask_params = Some((params.x, params.y, params.z, params.w));
                    }
                }
            }
        }

        // Check if we already have a visualizer
        let has_existing_visualizer = !existing_visualizers.is_empty();

        // Only create if we don't have one yet
        if let Some((center_x, center_y, half_width, half_height)) = found_mask_params {
            if !has_existing_visualizer {
                // Create an outline SDF (same style as other collider visualizers)
                let outline_sdf = shaders.add_sdf_expr(format!(
                    "abs(smud::sd_box(p, vec2<f32>({}, {}))) - 0.5",
                    half_width, half_height
                ));

                let frame_size = half_width.max(half_height) + 5.0;

                bevy::log::info!(
                    "[MaskDebug] Creating visualizer: center=({:.1},{:.1}), half_size=({:.1},{:.1}), frame={:.1}",
                    center_x, center_y, half_width, half_height, frame_size
                );
                bevy::log::info!(
                    "[MaskDebug] Stats: total_sdf_shapes={}, masked_shapes={}",
                    total_sdf_shapes, masked_shapes
                );

                commands.entity(debug_root_entity).with_children(|parent| {
                    parent.spawn((
                        AmMaskVisualizer { mask_layer_id: 0 },
                        SmudShape {
                            color: Color::hsl(0.0, 1.0, 0.5), // Solid red (same as chase hitbox)
                            sdf: outline_sdf,
                            frame: Frame::Quad(frame_size),
                            fill: SIMPLE_FILL_HANDLE,
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(center_x, center_y, 100.0)),
                        Name::new("AM Mask Debug (from shader params)"),
                    ));
                });
            }
        } else {
            // No mask found, remove visualizers
            for (visualizer_entity, _) in existing_visualizers.iter() {
                commands.entity(visualizer_entity).despawn();
            }
        }
    }

    /// Update AM mask visualizer positions based on actual shader mask_params.
    ///
    /// 根据实际 shader mask_params 更新 AM 遮罩可视化器位置。
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::type_complexity)]
    fn update_am_mask_visualizer_positions_system(
        mut commands: Commands,
        mut shaders: ResMut<Assets<Shader>>,
        // Query SDF shapes that have masks to get actual mask_params
        sdf_query: Query<&MeshMaterial2d<bevy_alight_motion::sdf_material::SdfMaterial>>,
        sdf_materials: Res<Assets<bevy_alight_motion::sdf_material::SdfMaterial>>,
        mut visualizers: Query<(Entity, &mut Transform, &mut SmudShape, &AmMaskVisualizer)>,
    ) {
        // Find the first SDF shape with an active mask and read its mask_params
        let mut found_mask_params: Option<(f32, f32, f32, f32)> = None;
        for material_handle in sdf_query.iter() {
            if let Some(material) = sdf_materials.get(&material_handle.0) {
                let mask_type = material.uniform_data.mask_type;
                if mask_type > 0.5 {
                    let params = material.uniform_data.mask_params;
                    found_mask_params = Some((params.x, params.y, params.z, params.w));
                    break;
                }
            }
        }

        if let Some((center_x, center_y, half_width, half_height)) = found_mask_params {
            for (entity, mut vis_transform, mut smud_shape, _) in visualizers.iter_mut() {
                // Update position
                vis_transform.translation.x = center_x;
                vis_transform.translation.y = center_y;

                // Update SDF dimensions by recreating the shader
                let new_sdf = shaders.add_sdf_expr(format!(
                    "smud::sd_box(p, vec2<f32>({}, {}))",
                    half_width, half_height
                ));
                smud_shape.sdf = new_sdf;
                smud_shape.frame = Frame::Quad(half_width.max(half_height) + 2.0);

                let _ = (entity, &mut commands); // suppress unused warning
            }
        }
    }
}

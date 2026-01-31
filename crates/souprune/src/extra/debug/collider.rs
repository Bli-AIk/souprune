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
    use crate::core::view::sdf_shape::ViewSdfShape;
    use bevy::prelude::*;
    use bevy::sprite_render::MeshMaterial2d;
    use bevy_alight_motion::sdf_material::SdfMaterial;

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

    /// Helper function to create debug collider visualization using SdfMaterial.
    /// Creates a stroke-only rectangle to show collider bounds.
    ///
    /// 使用 SdfMaterial 创建调试碰撞体可视化的辅助函数。
    /// 创建一个仅边框的矩形来显示碰撞体边界。
    fn spawn_debug_visualizer(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        sdf_materials: &mut ResMut<Assets<SdfMaterial>>,
        debug_root: Entity,
        parent: Entity,
        half_width: f32,
        half_height: f32,
        color: Color,
        position: Vec3,
        name: String,
    ) {
        let shape = ViewSdfShape {
            color: Color::NONE,
            half_width,
            half_height,
        };

        let material = SdfMaterial::new(
            bevy_alight_motion::sdf_material::SdfShapeType::BoxMiter,
            half_width,
            half_height,
            Color::NONE,
            0.5,
            color,
        );

        let frame_size = (half_width.max(half_height) * 2.0) + 4.0;
        let mesh = meshes.add(Rectangle::new(frame_size, frame_size));
        let mat_handle = sdf_materials.add(material);

        commands.entity(debug_root).with_children(|parent_builder| {
            parent_builder.spawn((
                ColliderVisualizer { parent },
                shape,
                Mesh2d(mesh),
                MeshMaterial2d(mat_handle),
                Transform::from_translation(position),
                Name::new(name),
            ));
        });
    }

    /// Helper function to create a circle debug visualizer.
    ///
    /// 创建圆形调试可视化器的辅助函数。
    fn spawn_circle_debug_visualizer(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        sdf_materials: &mut ResMut<Assets<SdfMaterial>>,
        debug_root: Entity,
        parent: Entity,
        radius: f32,
        color: Color,
        position: Vec3,
        name: String,
    ) {
        let material = SdfMaterial::new(
            bevy_alight_motion::sdf_material::SdfShapeType::Circle,
            radius,
            radius,
            Color::NONE,
            0.5,
            color,
        );

        let frame_size = radius * 2.0 + 4.0;
        let mesh = meshes.add(Rectangle::new(frame_size, frame_size));
        let mat_handle = sdf_materials.add(material);

        commands.entity(debug_root).with_children(|parent_builder| {
            parent_builder.spawn((
                ColliderVisualizer { parent },
                Mesh2d(mesh),
                MeshMaterial2d(mat_handle),
                Transform::from_translation(position),
                Name::new(name),
            ));
        });
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

    /// System to render rectangular colliders using SdfMaterial (debug only).
    ///
    /// 使用 SdfMaterial 渲染矩形碰撞体的系统（仅调试模式）。
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    fn render_rect_colliders_system(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut sdf_materials: ResMut<Assets<SdfMaterial>>,
        settings: Res<ColliderDebugSettings>,
        debug_root: Query<Entity, With<DebugVisualizerRoot>>,
        player_colliders: Query<
            (Entity, &Transform, &Rect2DCollider),
            (
                With<PlayerControlled>,
                Without<ViewSdfShape>,
                Without<ColliderVisualizer>,
            ),
        >,
        tilemap_colliders: Query<
            (Entity, &Transform, &Rect2DCollider),
            (
                With<TilemapCollider>,
                Without<PlayerControlled>,
                Without<ViewSdfShape>,
                Without<ColliderVisualizer>,
            ),
        >,
        object_colliders: Query<
            (Entity, &Transform, &Rect2DCollider),
            (
                With<ObjectCollider>,
                Without<PlayerControlled>,
                Without<ViewSdfShape>,
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

            // Create a stroke-only shape (no fill, just outline)
            // Use transparent fill with colored stroke
            // 创建仅边框的形状（无填充，仅轮廓）
            // 使用透明填充和彩色边框
            let shape = ViewSdfShape {
                color: Color::NONE, // Transparent fill
                half_width: collider.size.x / 2.0,
                half_height: collider.size.y / 2.0,
            };

            // Create material with stroke
            let material = SdfMaterial::new(
                bevy_alight_motion::sdf_material::SdfShapeType::BoxMiter,
                collider.size.x / 2.0,
                collider.size.y / 2.0,
                Color::NONE, // Transparent fill
                0.5,         // Stroke width
                color,       // Stroke color
            );

            // Calculate the frame size from the collider dimensions.
            //
            // 根据碰撞体尺寸计算框架大小。
            let frame_size = collider.size.x.max(collider.size.y) + 4.0;
            let mesh = meshes.add(Rectangle::new(frame_size, frame_size));
            let mat_handle = sdf_materials.add(material);

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
                    shape,
                    Mesh2d(mesh),
                    MeshMaterial2d(mat_handle),
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
        mut meshes: ResMut<Assets<Mesh>>,
        mut sdf_materials: ResMut<Assets<SdfMaterial>>,
        settings: Res<ColliderDebugSettings>,
        debug_root: Query<Entity, With<DebugVisualizerRoot>>,
        trigger_zones: Query<
            (Entity, &Transform, &Rect2DCollider, &TriggerZone),
            (Without<ViewSdfShape>, Without<ColliderVisualizer>),
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

            let final_position = transform.translation + collider.offset.extend(0.1);

            // Use cyan color for trigger zones to distinguish from colliders
            let color = if trigger_zone.player_inside {
                Color::hsl(180.0, 1.0, 0.7) // Brighter cyan when active
            } else {
                Color::hsl(180.0, 1.0, 0.5) // Normal cyan
            };

            spawn_debug_visualizer(
                &mut commands,
                &mut meshes,
                &mut sdf_materials,
                debug_root_entity,
                entity,
                collider.size.x / 2.0,
                collider.size.y / 2.0,
                color,
                final_position,
                format!("TriggerZone_{}", trigger_zone.id),
            );
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
        mut meshes: ResMut<Assets<Mesh>>,
        mut sdf_materials: ResMut<Assets<SdfMaterial>>,
        settings: Res<ColliderDebugSettings>,
        debug_root: Query<Entity, With<DebugVisualizerRoot>>,
        physics_colliders: Query<
            (Entity, &Transform, &crate::core::collision::PhysicsCollider),
            (
                Without<ViewSdfShape>,
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
                Without<ViewSdfShape>,
                Without<ColliderVisualizer>,
                Without<BattleColliderVisualized>,
            ),
        >,
        chase_hitboxes: Query<Entity, With<crate::app_state::overworld::chase::ChasePlayerHitbox>>,
        battle_boxes: Query<
            (
                Entity,
                &GlobalTransform,
                &crate::core::view::components::ViewBox,
            ),
            (
                With<crate::app_state::battle::collision::BattleBox>,
                Without<ViewSdfShape>,
                Without<ColliderVisualizer>,
                Without<BattleColliderVisualized>,
                Without<crate::core::collision::PhysicsCollider>,
            ),
        >,
        // AM animated battle boxes (no ViewBox, use AmBattleBoxBounds instead)
        am_battle_boxes: Query<
            (
                Entity,
                &GlobalTransform,
                &crate::app_state::battle::collision::AmBattleBoxBounds,
            ),
            (
                With<crate::app_state::battle::collision::BattleBox>,
                Without<crate::core::view::components::ViewBox>,
                Without<ViewSdfShape>,
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
            let has_visualizer = existing_visualizers
                .iter()
                .any(|(_, vis)| vis.parent == entity);

            if has_visualizer {
                continue;
            }

            let position = transform.translation + Vec3::new(0.0, 0.0, 20.0);
            let color = Color::hsl(120.0, 1.0, 0.5);

            match physics_collider {
                PhysicsCollider::Circle { radius } => {
                    spawn_circle_debug_visualizer(
                        &mut commands,
                        &mut meshes,
                        &mut sdf_materials,
                        debug_root_entity,
                        entity,
                        *radius,
                        color,
                        position,
                        "Physics Collider (Circle)".to_string(),
                    );
                }
                PhysicsCollider::Box { half_size } => {
                    spawn_debug_visualizer(
                        &mut commands,
                        &mut meshes,
                        &mut sdf_materials,
                        debug_root_entity,
                        entity,
                        half_size.x,
                        half_size.y,
                        color,
                        position,
                        "Physics Collider (Box)".to_string(),
                    );
                }
            };
        }

        // Visualize trigger colliders (green for battle, red for chase hitbox)
        for (entity, transform, trigger_collider, hitbox_offset) in trigger_colliders.iter() {
            let has_visualizer = existing_visualizers
                .iter()
                .any(|(_, vis)| vis.parent == entity);

            if has_visualizer {
                continue;
            }

            let is_chase_hitbox = chase_hitboxes.get(entity).is_ok();
            let color = if is_chase_hitbox {
                Color::hsl(0.0, 1.0, 0.5) // Red for chase hitbox
            } else {
                Color::hsl(120.0, 1.0, 0.5) // Green for battle
            };

            let offset = hitbox_offset.map(|o| o.0).unwrap_or(Vec2::ZERO);
            let position = transform.translation() + offset.extend(20.0);

            match trigger_collider {
                TriggerCollider::Circle { radius } => {
                    let name = if is_chase_hitbox {
                        "Chase Hitbox (Circle)"
                    } else {
                        "Trigger Collider (Circle)"
                    };
                    spawn_circle_debug_visualizer(
                        &mut commands,
                        &mut meshes,
                        &mut sdf_materials,
                        debug_root_entity,
                        entity,
                        *radius,
                        color,
                        position,
                        name.to_string(),
                    );
                }
                TriggerCollider::Box { half_size } => {
                    let name = if is_chase_hitbox {
                        "Chase Hitbox (Box)"
                    } else {
                        "Trigger Collider (Box)"
                    };
                    spawn_debug_visualizer(
                        &mut commands,
                        &mut meshes,
                        &mut sdf_materials,
                        debug_root_entity,
                        entity,
                        half_size.x,
                        half_size.y,
                        color,
                        position,
                        name.to_string(),
                    );
                }
            };
        }

        // Visualize Battle Box (Green)
        for (entity, global_transform, ui_box) in battle_boxes.iter() {
            let has_visualizer = existing_visualizers
                .iter()
                .any(|(_, vis)| vis.parent == entity);

            if has_visualizer {
                continue;
            }

            let half_width = ui_box.width() / 2.0;
            let half_height = ui_box.height() / 2.0;
            let position = global_transform.translation() + Vec3::new(0.0, 0.0, 50.0);

            spawn_debug_visualizer(
                &mut commands,
                &mut meshes,
                &mut sdf_materials,
                debug_root_entity,
                entity,
                half_width,
                half_height,
                Color::hsl(120.0, 1.0, 0.5),
                position,
                "Battle Box Debug".to_string(),
            );
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
            let center_pos = global_transform.translation()
                + Vec3::new(am_bounds.center_offset.x, am_bounds.center_offset.y, 50.0);

            spawn_debug_visualizer(
                &mut commands,
                &mut meshes,
                &mut sdf_materials,
                debug_root_entity,
                entity,
                half_width,
                half_height,
                Color::hsl(180.0, 1.0, 0.5), // Cyan
                center_pos,
                "AM Battle Box Debug".to_string(),
            );
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
                With<crate::core::view::components::ViewBox>,
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
                Without<crate::core::view::components::ViewBox>,
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
        mut meshes: ResMut<Assets<Mesh>>,
        mut sdf_materials: ResMut<Assets<SdfMaterial>>,
        settings: Res<ColliderDebugSettings>,
        debug_root: Query<Entity, With<DebugVisualizerRoot>>,
        // Query SDF shapes that have masks to get actual mask_params
        sdf_query: Query<(
            Entity,
            &MeshMaterial2d<bevy_alight_motion::sdf_material::SdfMaterial>,
        )>,
        existing_visualizers: Query<(Entity, &AmMaskVisualizer)>,
    ) {
        // Debug log every frame when F3 is on
        if settings.show_colliders {
            bevy::log::debug!("[MaskDebug] System running, show_colliders=true");
        }

        let Ok(debug_root_entity) = debug_root.single() else {
            bevy::log::warn!("[MaskDebug] No DebugVisualizerRoot found!");
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
        // Collect mask params first, then use sdf_materials for adding new material
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
                bevy::log::info!(
                    "[MaskDebug] Creating visualizer: center=({:.1},{:.1}), half_size=({:.1},{:.1})",
                    center_x,
                    center_y,
                    half_width,
                    half_height
                );
                bevy::log::info!(
                    "[MaskDebug] Stats: total_sdf_shapes={}, masked_shapes={}",
                    total_sdf_shapes,
                    masked_shapes
                );

                // Create outline using SdfMaterial with stroke
                let material = SdfMaterial::new(
                    bevy_alight_motion::sdf_material::SdfShapeType::BoxMiter,
                    half_width,
                    half_height,
                    Color::NONE,
                    0.5,
                    Color::hsl(0.0, 1.0, 0.5), // Red
                );

                let frame_size = (half_width.max(half_height) * 2.0) + 10.0;
                let mesh = meshes.add(Rectangle::new(frame_size, frame_size));
                let mat_handle = sdf_materials.add(material);

                commands.entity(debug_root_entity).with_children(|parent| {
                    parent.spawn((
                        AmMaskVisualizer { mask_layer_id: 0 },
                        Mesh2d(mesh),
                        MeshMaterial2d(mat_handle),
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
        // Query SDF shapes that have masks to get actual mask_params
        sdf_query: Query<&MeshMaterial2d<bevy_alight_motion::sdf_material::SdfMaterial>>,
        sdf_materials: Res<Assets<bevy_alight_motion::sdf_material::SdfMaterial>>,
        mut visualizers: Query<(&mut Transform, &AmMaskVisualizer)>,
    ) {
        // Find the first SDF shape with an active mask and read its mask_params
        let mut found_mask_params: Option<(f32, f32)> = None;
        for material_handle in sdf_query.iter() {
            if let Some(material) = sdf_materials.get(&material_handle.0) {
                let mask_type = material.uniform_data.mask_type;
                if mask_type > 0.5 {
                    let params = material.uniform_data.mask_params;
                    found_mask_params = Some((params.x, params.y)); // center only
                    break;
                }
            }
        }

        // Only update position, don't recreate shader every frame
        if let Some((center_x, center_y)) = found_mask_params {
            for (mut vis_transform, _) in visualizers.iter_mut() {
                vis_transform.translation.x = center_x;
                vis_transform.translation.y = center_y;
            }
        }
    }
}

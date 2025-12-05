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
    use crate::core::collision::Rect2DCollider;
    use bevy::prelude::*;
    use bevy_smud::prelude::*;

    /// Marker component for collision visualizer entities.
    ///
    /// 碰撞体可视化实体的标记组件。
    #[derive(Component)]
    pub struct ColliderVisualizer {
        pub parent: Entity,
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
            .add_systems(Startup, setup_debug_visualizer_root)
            .add_systems(
                Update,
                (
                    toggle_collider_visibility_system,
                    render_rect_colliders_system,
                    update_collider_visualizer_positions_system,
                ),
            );
    }

    /// Set up the root entity for debug visualizers.
    ///
    /// 设置调试可视化器的根实体。
    fn setup_debug_visualizer_root(mut commands: Commands) {
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
            let parent_exists = player_colliders.get(visualizer.parent).is_ok()
                || tilemap_colliders.get(visualizer.parent).is_ok()
                || object_colliders.get(visualizer.parent).is_ok();
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

    /// Update visualizer positions when parent transforms change.
    ///
    /// 当父变换改变时更新可视化器位置的系统。
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
}

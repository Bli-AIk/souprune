#[cfg(feature = "debug")]
pub mod debug_collider {

    /// Debug resource to control collider visibility
    /// 控制碰撞体可见性的调试资源
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

    /// Marker component for collision visualizer entities
    /// 碰撞体可视化实体的标记组件
    #[derive(Component)]
    pub struct ColliderVisualizer {
        pub parent: Entity,
    }

    /// Root entity for organizing all debug visualizers
    /// 用于组织所有调试可视化器的根实体
    #[derive(Component)]
    pub struct DebugVisualizerRoot;

    /// Setup collider debug systems
    /// 设置碰撞体调试系统
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

    /// Setup the root entity for debug visualizers
    /// 设置调试可视化器的根实体
    fn setup_debug_visualizer_root(mut commands: Commands) {
        commands.spawn((
            DebugVisualizerRoot,
            Name::new("Debug Visualizers"),
            Transform::default(),
            Visibility::default(),
        ));
    }

    /// System to toggle collider visibility with F3 key (debug only)
    /// F3键切换碰撞体可见性的系统（仅调试模式）
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

        // If debug settings don't show colliders, remove all visualizers and return
        if !settings.show_colliders {
            for (visualizer_entity, _) in existing_visualizers.iter() {
                commands.entity(visualizer_entity).despawn();
            }
            return;
        }

        // Remove existing visualizers for entities that no longer have colliders
        for (visualizer_entity, visualizer) in existing_visualizers.iter() {
            let parent_exists = player_colliders.get(visualizer.parent).is_ok()
                || tilemap_colliders.get(visualizer.parent).is_ok()
                || object_colliders.get(visualizer.parent).is_ok();
            if !parent_exists {
                commands.entity(visualizer_entity).despawn();
            }
        }

        // Helper closure to create collider visualizer
        let mut create_visualizer = |entity: Entity,
                                     transform: &Transform,
                                     collider: &Rect2DCollider,
                                     color: Color,
                                     name: String| {
            // Check if this entity already has a visualizer
            let has_visualizer = existing_visualizers
                .iter()
                .any(|(_, vis)| vis.parent == entity);
            if has_visualizer {
                return;
            }

            // Create a thin border SDF using distance field outline
            let border_sdf = shaders.add_sdf_expr(format!(
                "abs(smud::sd_box(p, vec2<f32>({}, {}))) - {}",
                collider.size.x / 2.0,
                collider.size.y / 2.0,
                0.125
            ));

            // Calculate frame size based on collider size
            let frame_size = (collider.size.x.max(collider.size.y) / 2.0) + 2.0;

            // Calculate final position including offset
            let final_position = transform.translation + collider.offset.extend(0.1);

            // Spawn the visual representation as a child of debug root
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

        // Create visualizers for player colliders (green)
        for (entity, transform, collider) in player_colliders.iter() {
            create_visualizer(
                entity,
                transform,
                collider,
                Color::hsl(120.0, 1.0, 0.5),
                "Player Collider".to_string(),
            );
        }

        // Create visualizers for tilemap colliders (dark_green)
        for (entity, transform, collider) in tilemap_colliders.iter() {
            create_visualizer(
                entity,
                transform,
                collider,
                Color::hsl(120.0, 0.75, 0.75),
                "Tilemap Collider".to_string(),
            );
        }

        // Create visualizers for object colliders (light_green)
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

    /// System to update visualizer positions when parent transforms change
    /// 当父变换改变时更新可视化器位置的系统
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

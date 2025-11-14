#[cfg(feature = "debug")]
pub mod debug_collider {

    /// Debug resource to control collider visibility
    /// 控制碰撞体可见性的调试资源
    #[derive(Resource, Default)]
    pub struct ColliderDebugSettings {
        pub show_colliders: bool,
    }

    use crate::app_state::overworld::character::components::PlayerControlled;
    use crate::core::collision::Rect2DCollider;
    use bevy::prelude::*;
    use bevy_smud::prelude::*;

    /// Marker component for collision visualizer entities
    /// 碰撞体可视化实体的标记组件
    #[derive(Component)]
    pub struct ColliderVisualizer {
        pub parent: Entity,
    }

    /// Setup collider debug systems
    /// 设置碰撞体调试系统
    pub fn setup_collider_debug(app: &mut App) {
        app.init_resource::<ColliderDebugSettings>().add_systems(
            Update,
            (
                toggle_collider_visibility_system,
                render_player_rect_colliders_system,
                update_player_visualizer_positions_system,
            ),
        );
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

    /// System to render rectangular colliders using bevy_smud SDF (player only, debug only).
    ///
    /// 使用bevy_smud SDF渲染矩形碰撞体的系统（仅玩家，仅调试模式）。
    fn render_player_rect_colliders_system(
        mut commands: Commands,
        mut shaders: ResMut<Assets<Shader>>,
        settings: Res<ColliderDebugSettings>,
        player_query: Query<
            (Entity, &Transform, &Rect2DCollider),
            (
                With<PlayerControlled>,
                Without<SmudShape>,
                Without<ColliderVisualizer>,
            ),
        >,
        existing_visualizers: Query<(Entity, &ColliderVisualizer)>,
    ) {
        // If debug settings don't show colliders, remove all visualizers and return
        if !settings.show_colliders {
            for (visualizer_entity, _) in existing_visualizers.iter() {
                commands.entity(visualizer_entity).despawn();
            }
            return;
        }

        // Remove existing visualizers for entities that no longer have colliders
        for (visualizer_entity, visualizer) in existing_visualizers.iter() {
            if player_query.get(visualizer.parent).is_err() {
                commands.entity(visualizer_entity).despawn();
            }
        }

        // Create visualizers for new player colliders
        for (entity, transform, collider) in player_query.iter() {
            // Check if this entity already has a visualizer
            let has_visualizer = existing_visualizers
                .iter()
                .any(|(_, vis)| vis.parent == entity);
            if has_visualizer {
                continue;
            }

            // Create a thin border SDF using distance field outline
            let border_sdf = shaders.add_sdf_expr(format!(
                "abs(smud::sd_box(p, vec2<f32>({}, {}))) - {}",
                collider.size.x / 2.0,
                collider.size.y / 2.0,
                0.25
            ));

            // Calculate frame size based on collider size
            let frame_size = (collider.size.x.max(collider.size.y) / 2.0) + 2.0;

            // Calculate final position including offset
            let final_position = transform.translation + collider.offset.extend(0.1);

            // Spawn the visual representation as a separate entity
            commands.spawn((
                ColliderVisualizer { parent: entity },
                SmudShape {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    sdf: border_sdf,
                    frame: Frame::Quad(frame_size),
                    fill: SIMPLE_FILL_HANDLE,
                    ..default()
                },
                Transform::from_translation(final_position),
            ));
        }
    }

    /// System to update visualizer positions when parent transforms change
    /// 当父变换改变时更新可视化器位置的系统
    fn update_player_visualizer_positions_system(
        mut visualizers: Query<(&mut Transform, &ColliderVisualizer), Without<Rect2DCollider>>,
        players: Query<
            (&Transform, &Rect2DCollider),
            (
                With<PlayerControlled>,
                With<Rect2DCollider>,
                Without<ColliderVisualizer>,
            ),
        >,
    ) {
        for (mut vis_transform, visualizer) in visualizers.iter_mut() {
            if let Ok((parent_transform, collider)) = players.get(visualizer.parent) {
                let final_position = parent_transform.translation + collider.offset.extend(0.1);
                vis_transform.translation = final_position;
            }
        }
    }
}

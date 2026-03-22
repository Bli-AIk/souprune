//! # visuals.rs
//!
//! # visuals.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This file renders the visual side of the overworld chase effect. It spawns and updates the dark
//! overlay, player outline, and heart marker that sell the chase state, while keeping those helper
//! entities synchronized with the player sprite and transition alpha.
//!
//! 这个文件负责大地图追逐效果的视觉部分。它会生成并更新黑色遮罩、玩家描边和心形标记，
//! 用来强化 chase 状态的表现，同时让这些辅助实体持续跟随玩家精灵和过渡透明度。

use super::*;

/// Spawn the dark overlay when entering chase state.
pub(super) fn spawn_chase_dark_overlay_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sdf_materials: ResMut<Assets<SdfMaterial>>,
    transition: Res<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
    chase_root: Query<Entity, With<ChaseEffectRoot>>,
    existing_overlay: Query<Entity, With<ChaseDarkOverlay>>,
) {
    if !transition.transitioning_in || !existing_overlay.is_empty() {
        return;
    }

    let Ok(root_entity) = chase_root.single() else {
        return;
    };

    let overlay_size = chase_config.dark_overlay.overlay_size;
    let shape = ViewSdfShape::new(overlay_size, overlay_size, Color::srgba(0.0, 0.0, 0.0, 0.0));
    let mesh = meshes.add(shape.create_mesh());
    let material = sdf_materials.add(shape.to_material());

    commands.entity(root_entity).with_children(|parent| {
        parent.spawn((
            ChaseDarkOverlay,
            ChaseEffect {
                target_alpha: chase_config.dark_overlay.target_alpha,
            },
            shape,
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, chase_config.dark_overlay.z_offset),
            Name::new("ChaseDarkOverlay"),
        ));
    });

    info!("Chase: Spawned dark overlay");
}

/// Spawn player outline mesh using PixelOutlineMaterial.
pub(super) fn spawn_player_outline_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PixelOutlineMaterial>>,
    images: Res<Assets<Image>>,
    atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    transition: Res<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
    chase_root: Query<Entity, With<ChaseEffectRoot>>,
    player_query: Query<(Entity, &Sprite, &Transform), With<PlayerControlled>>,
    existing_outline: Query<Entity, With<ChasePlayerOutline>>,
) {
    if !transition.transitioning_in || !existing_outline.is_empty() {
        return;
    }

    let Ok(root_entity) = chase_root.single() else {
        return;
    };

    let Ok((_player_entity, sprite, player_transform)) = player_query.single() else {
        return;
    };

    let texture = sprite.image.clone();
    let Some(image) = images.get(&texture) else {
        return;
    };
    let tex_size = image.size().as_vec2();

    let (uv_rect, sprite_size) = if let Some(ref atlas) = sprite.texture_atlas {
        if let Some(layout) = atlas_layouts.get(&atlas.layout) {
            let rect = layout.textures[atlas.index];
            let uv = Vec4::new(
                rect.min.x as f32 / tex_size.x,
                rect.min.y as f32 / tex_size.y,
                rect.max.x as f32 / tex_size.x,
                rect.max.y as f32 / tex_size.y,
            );
            let size = Vec2::new(rect.width() as f32, rect.height() as f32);
            (uv, size)
        } else {
            (Vec4::new(0.0, 0.0, 1.0, 1.0), tex_size)
        }
    } else if let Some(rect) = sprite.rect {
        let uv = Vec4::new(
            rect.min.x / tex_size.x,
            rect.min.y / tex_size.y,
            rect.max.x / tex_size.x,
            rect.max.y / tex_size.y,
        );
        let size = Vec2::new(rect.width(), rect.height());
        (uv, size)
    } else {
        (Vec4::new(0.0, 0.0, 1.0, 1.0), tex_size)
    };

    let flip = Vec4::new(
        if sprite.flip_x { 1.0 } else { 0.0 },
        if sprite.flip_y { 1.0 } else { 0.0 },
        0.0,
        0.0,
    );

    let outline_size = sprite_size + Vec2::splat(chase_config.outline.padding);
    let mesh = meshes.add(Rectangle::new(outline_size.x, outline_size.y));

    let material = materials.add(PixelOutlineMaterial {
        params: LinearRgba::new(1.0, 0.0, 0.0, 0.0),
        uv_rect,
        flip,
        texture,
    });

    commands.entity(root_entity).with_children(|parent| {
        parent.spawn((
            ChasePlayerOutline {
                current_size: sprite_size,
            },
            ChaseEffect { target_alpha: 1.0 },
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_translation(
                player_transform.translation + Vec3::new(0.0, 0.0, chase_config.outline.z_offset),
            ),
            Name::new("ChasePlayerOutline"),
        ));
    });

    info!("Chase: Spawned player outline");
}

/// Spawn heart marker (judgment indicator) as child of player entity.
pub(super) fn spawn_heart_marker_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    transition: Res<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
    player_query: Query<Entity, With<PlayerControlled>>,
    existing_markers: Query<Entity, With<ChaseHeartMarker>>,
) {
    if !transition.transitioning_in || !existing_markers.is_empty() {
        return;
    }

    let Ok(player_entity) = player_query.single() else {
        return;
    };

    let config = &chase_config.heart_marker;
    if config.texture_path.is_empty() {
        warn!("Chase: No heart marker texture path configured");
        return;
    }

    let offset = config.offset.to_vec2();
    let texture: Handle<Image> = asset_server.load(&config.texture_path);

    commands.entity(player_entity).with_children(|parent| {
        parent.spawn((
            ChaseHeartMarker,
            Sprite {
                image: texture,
                color: Color::srgba(config.color.r, config.color.g, config.color.b, 0.0),
                ..default()
            },
            Transform::from_xyz(offset.x, offset.y, config.z_offset)
                .with_scale(Vec3::splat(config.scale)),
            Name::new("ChaseHeartMarker"),
        ));
    });

    info!("Chase: Spawned heart marker");
}

/// Update player outline position and texture to follow player sprite.
pub(super) fn update_player_outline_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut outline_query: Query<
        (
            Entity,
            &mut Transform,
            &mut ChasePlayerOutline,
            &MeshMaterial2d<PixelOutlineMaterial>,
        ),
        Without<PlayerControlled>,
    >,
    player_query: Query<(&Transform, &Sprite), With<PlayerControlled>>,
    images: Res<Assets<Image>>,
    atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    mut materials: ResMut<Assets<PixelOutlineMaterial>>,
    transition: Res<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
) {
    if !transition.active {
        return;
    }

    let Ok((player_transform, sprite)) = player_query.single() else {
        return;
    };

    for (entity, mut outline_transform, mut outline_marker, material_handle) in
        outline_query.iter_mut()
    {
        outline_transform.translation.x = player_transform.translation.x;
        outline_transform.translation.y = player_transform.translation.y;

        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };
        material.texture = sprite.image.clone();
        material.flip = Vec4::new(
            if sprite.flip_x { 1.0 } else { 0.0 },
            if sprite.flip_y { 1.0 } else { 0.0 },
            0.0,
            0.0,
        );

        let Some(image) = images.get(&sprite.image) else {
            continue;
        };
        let tex_size = image.size().as_vec2();
        let (new_uv_rect, sprite_size) = if let Some(ref atlas) = sprite.texture_atlas {
            if let Some(layout) = atlas_layouts.get(&atlas.layout) {
                let rect = layout.textures[atlas.index];
                let uv = Vec4::new(
                    rect.min.x as f32 / tex_size.x,
                    rect.min.y as f32 / tex_size.y,
                    rect.max.x as f32 / tex_size.x,
                    rect.max.y as f32 / tex_size.y,
                );
                let size = Vec2::new(rect.width() as f32, rect.height() as f32);
                (uv, size)
            } else {
                (Vec4::new(0.0, 0.0, 1.0, 1.0), tex_size)
            }
        } else if let Some(rect) = sprite.rect {
            let uv = Vec4::new(
                rect.min.x / tex_size.x,
                rect.min.y / tex_size.y,
                rect.max.x / tex_size.x,
                rect.max.y / tex_size.y,
            );
            let size = Vec2::new(rect.width(), rect.height());
            (uv, size)
        } else {
            (Vec4::new(0.0, 0.0, 1.0, 1.0), tex_size)
        };

        material.uv_rect = new_uv_rect;

        if (outline_marker.current_size - sprite_size).length() > 0.01 {
            outline_marker.current_size = sprite_size;
            let outline_size = sprite_size + Vec2::splat(chase_config.outline.padding);
            let new_mesh = meshes.add(Rectangle::new(outline_size.x, outline_size.y));
            commands.entity(entity).insert(Mesh2d(new_mesh));
        }
    }
}

/// Update alpha values for all chase effects based on transition progress.
pub(super) fn update_chase_effect_alpha_system(
    transition: Res<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
    sdf_effects: Query<(&ViewSdfShape, &MeshMaterial2d<SdfMaterial>, &ChaseEffect)>,
    mut sdf_materials: ResMut<Assets<SdfMaterial>>,
    mut outline_query: Query<&MeshMaterial2d<PixelOutlineMaterial>, With<ChasePlayerOutline>>,
    mut materials: ResMut<Assets<PixelOutlineMaterial>>,
) {
    let duration = chase_config.transition_duration();
    let progress = if duration > 0.0 {
        transition.timer / duration
    } else {
        1.0
    };

    for (shape, mat_handle, effect) in sdf_effects.iter() {
        let target_alpha = effect.target_alpha * progress;
        if let Some(material) = sdf_materials.get_mut(&mat_handle.0) {
            let mut new_shape = shape.clone();
            let mut color = shape.color.to_srgba();
            color.alpha = target_alpha;
            new_shape.color = color.into();
            *material = new_shape.to_material();
        }
    }

    for material_handle in outline_query.iter_mut() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.params.alpha = progress;
        }
    }
}

/// Update heart marker alpha based on transition progress.
pub(super) fn update_heart_marker_alpha_system(
    transition: Res<ChaseTransition>,
    chase_config: Res<ChaseConfig>,
    player_invincibility: Res<PlayerInvincibility>,
    mut heart_markers: Query<&mut Sprite, With<ChaseHeartMarker>>,
) {
    if player_invincibility.is_invincible() || !transition.active {
        return;
    }

    let duration = chase_config.transition_duration();
    let progress = if duration > 0.0 {
        transition.timer / duration
    } else {
        1.0
    };

    let config = &chase_config.heart_marker;
    for mut sprite in heart_markers.iter_mut() {
        sprite.color = Color::srgba(
            config.color.r,
            config.color.g,
            config.color.b,
            config.color.a * progress,
        );
    }
}

/// Cleanup chase effects when transition out is complete.
pub(super) fn cleanup_chase_effects_system(
    mut commands: Commands,
    mut transition: ResMut<ChaseTransition>,
    chase_root: Query<Entity, With<ChaseEffectRoot>>,
    overlays: Query<Entity, With<ChaseDarkOverlay>>,
    outlines: Query<Entity, With<ChasePlayerOutline>>,
    heart_markers: Query<Entity, With<ChaseHeartMarker>>,
) {
    if transition.active || transition.transitioning_in || transition.timer > 0.0 {
        return;
    }

    if transition.cleanup_done {
        return;
    }

    for entity in overlays.iter() {
        commands.entity(entity).despawn();
    }
    for entity in outlines.iter() {
        commands.entity(entity).despawn();
    }
    for entity in heart_markers.iter() {
        commands.entity(entity).despawn();
    }

    if let Ok(root) = chase_root.single() {
        commands.entity(root).despawn();
    }

    transition.cleanup_done = true;
    info!("Chase: Cleaned up all effects");
}

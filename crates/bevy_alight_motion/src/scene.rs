//! Scene building and coordinate transformation.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::animation::AmAnimated;
use crate::loader::AmProject;
use crate::schema::{AmAnimatedFloat, AmAnimatedVec2, AmAnimatedVec3, AmLayer, AmScene, AmShape};

/// Component bundle for an AM project root.
#[derive(Bundle)]
pub struct AmProjectBundle {
    /// Transform for coordinate system conversion.
    pub transform: Transform,
    /// Global transform.
    pub global_transform: GlobalTransform,
    /// Visibility.
    pub visibility: Visibility,
    /// Inherited visibility.
    pub inherited_visibility: InheritedVisibility,
    /// View visibility.
    pub view_visibility: ViewVisibility,
    /// Marker component.
    pub marker: AmProjectRoot,
}

/// Marker component for the project root entity.
#[derive(Component, Debug, Clone)]
pub struct AmProjectRoot {
    /// Project handle.
    pub handle: Handle<AmProject>,
    /// Whether the scene has been spawned.
    pub spawned: bool,
}

/// Component marking an AM layer entity.
#[derive(Component, Debug, Clone)]
pub struct AmLayerMarker {
    /// Layer ID.
    pub id: u64,
    /// Layer label.
    pub label: String,
}

/// Configuration for scene building.
#[derive(Debug, Clone)]
pub struct AmSceneConfig {
    /// Canvas width.
    pub canvas_width: f32,
    /// Canvas height.
    pub canvas_height: f32,
    /// Whether to flip Y axis (AM uses top-left origin).
    pub flip_y: bool,
    /// Z-spacing between layers.
    pub z_spacing: f32,
}

impl Default for AmSceneConfig {
    fn default() -> Self {
        Self {
            canvas_width: 1280.0,
            canvas_height: 960.0,
            flip_y: true,
            z_spacing: 0.001,
        }
    }
}

/// Convert AM coordinates to Bevy coordinates.
///
/// AM: Origin at top-left, Y increases downward.
/// Bevy: Origin at center, Y increases upward.
pub fn am_to_bevy_coords(x: f32, y: f32, config: &AmSceneConfig) -> (f32, f32) {
    let bx = x - config.canvas_width / 2.0;
    let by = if config.flip_y {
        config.canvas_height / 2.0 - y
    } else {
        y - config.canvas_height / 2.0
    };
    (bx, by)
}

/// Spawn entities from an AM scene.
pub fn spawn_scene(
    commands: &mut Commands,
    scene: &AmScene,
    images: &HashMap<String, Handle<Image>>,
    parent: Entity,
    config: &AmSceneConfig,
) -> HashMap<u64, Entity> {
    let mut entity_map: HashMap<u64, Entity> = HashMap::new();
    let mut deferred_parents: Vec<(Entity, u64)> = Vec::new();

    // First pass: create all entities
    for (idx, layer) in scene.layers.iter().enumerate() {
        let z = idx as f32 * config.z_spacing;

        match layer {
            AmLayer::Shape(shape) => {
                let entity = spawn_shape(commands, shape, images, config, z);
                entity_map.insert(shape.id, entity);

                if shape.parent != 0 {
                    deferred_parents.push((entity, shape.parent));
                } else {
                    commands.entity(parent).add_child(entity);
                }
            }
            AmLayer::Nullobj(null) => {
                let entity = spawn_null(commands, null, config, z);
                entity_map.insert(null.id, entity);

                if null.parent != 0 {
                    deferred_parents.push((entity, null.parent));
                } else {
                    commands.entity(parent).add_child(entity);
                }
            }
            AmLayer::EmbedScene(embed) => {
                let entity = spawn_embed_scene(commands, embed, images, config, z);
                entity_map.insert(embed.id, entity);

                if embed.parent != 0 {
                    deferred_parents.push((entity, embed.parent));
                } else {
                    commands.entity(parent).add_child(entity);
                }
            }
        }
    }

    // Second pass: establish parent relationships
    for (child_entity, parent_id) in deferred_parents {
        if let Some(&parent_entity) = entity_map.get(&parent_id) {
            commands.entity(parent_entity).add_child(child_entity);
        } else {
            // Parent not found, attach to root
            commands.entity(parent).add_child(child_entity);
        }
    }

    entity_map
}

/// Spawn a shape layer.
fn spawn_shape(
    commands: &mut Commands,
    shape: &AmShape,
    images: &HashMap<String, Handle<Image>>,
    config: &AmSceneConfig,
    z: f32,
) -> Entity {
    // Get initial transform values
    let (tx, ty) = get_initial_location(&shape.transform.location, config);
    let rotation = get_initial_rotation(&shape.transform.rotation);
    let (sx, sy) = get_initial_scale(&shape.transform.scale);
    let opacity = get_initial_opacity(&shape.transform.opacity);

    // Get size from properties
    let (width, height) = get_shape_size(&shape.properties);

    let transform = Transform {
        translation: Vec3::new(tx, ty, z),
        rotation: Quat::from_rotation_z(rotation.to_radians()),
        scale: Vec3::new(sx, sy, 1.0),
    };

    let mut entity = commands.spawn((
        AmLayerMarker {
            id: shape.id,
            label: shape.label.clone(),
        },
        AmAnimated {
            layer_id: shape.id,
            start_time: shape.start_time,
            end_time: shape.end_time,
            location: shape.transform.location.clone(),
            rotation: shape.transform.rotation.clone(),
            scale: shape.transform.scale.clone(),
            opacity: shape.transform.opacity.clone(),
        },
        transform,
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));

    // Add sprite if it's a media fill
    if shape.fill_type == "media" && !shape.fill_image.is_empty() {
        if let Some(handle) = images.get(&shape.fill_image) {
            entity.insert(Sprite {
                image: handle.clone(),
                color: Color::srgba(1.0, 1.0, 1.0, opacity),
                custom_size: Some(Vec2::new(width, height)),
                ..default()
            });
        }
    } else if shape.fill_type == "color" {
        // Color fill - create a colored sprite
        let color = if let Some(fill_color) = &shape.fill_color {
            crate::schema::parse_color(&fill_color.value)
                .map(|c| Color::srgba(c[0], c[1], c[2], c[3] * opacity))
                .unwrap_or(Color::srgba(1.0, 1.0, 1.0, opacity))
        } else {
            Color::srgba(1.0, 1.0, 1.0, opacity)
        };

        entity.insert(Sprite {
            color,
            custom_size: Some(Vec2::new(width, height)),
            ..default()
        });
    }

    entity.id()
}

/// Spawn a null object.
fn spawn_null(
    commands: &mut Commands,
    null: &crate::schema::AmNullObj,
    config: &AmSceneConfig,
    z: f32,
) -> Entity {
    let (tx, ty) = get_initial_location(&null.transform.location, config);
    let rotation = get_initial_rotation(&null.transform.rotation);
    let (sx, sy) = get_initial_scale(&null.transform.scale);

    let transform = Transform {
        translation: Vec3::new(tx, ty, z),
        rotation: Quat::from_rotation_z(rotation.to_radians()),
        scale: Vec3::new(sx, sy, 1.0),
    };

    commands
        .spawn((
            AmLayerMarker {
                id: null.id,
                label: null.label.clone(),
            },
            AmAnimated {
                layer_id: null.id,
                start_time: null.start_time,
                end_time: null.end_time,
                location: null.transform.location.clone(),
                rotation: null.transform.rotation.clone(),
                scale: null.transform.scale.clone(),
                opacity: null.transform.opacity.clone(),
            },
            transform,
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id()
}

/// Spawn an embedded scene.
fn spawn_embed_scene(
    commands: &mut Commands,
    embed: &crate::schema::AmEmbedScene,
    images: &HashMap<String, Handle<Image>>,
    config: &AmSceneConfig,
    z: f32,
) -> Entity {
    let (tx, ty) = get_initial_location(&embed.transform.location, config);
    let rotation = get_initial_rotation(&embed.transform.rotation);
    let (sx, sy) = get_initial_scale(&embed.transform.scale);

    let transform = Transform {
        translation: Vec3::new(tx, ty, z),
        rotation: Quat::from_rotation_z(rotation.to_radians()),
        scale: Vec3::new(sx, sy, 1.0),
    };

    let entity = commands
        .spawn((
            AmLayerMarker {
                id: embed.id,
                label: embed.label.clone(),
            },
            AmAnimated {
                layer_id: embed.id,
                start_time: embed.start_time,
                end_time: embed.end_time,
                location: embed.transform.location.clone(),
                rotation: embed.transform.rotation.clone(),
                scale: embed.transform.scale.clone(),
                opacity: embed.transform.opacity.clone(),
            },
            transform,
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id();

    // Recursively spawn nested scene
    let nested_config = AmSceneConfig {
        canvas_width: embed.scene.width as f32,
        canvas_height: embed.scene.height as f32,
        ..config.clone()
    };

    spawn_scene(commands, &embed.scene, images, entity, &nested_config);

    entity
}

/// Get initial location from animated property.
fn get_initial_location(prop: &AmAnimatedVec3, config: &AmSceneConfig) -> (f32, f32) {
    let (x, y) = if let Some(val) = &prop.value {
        (val[0], val[1])
    } else if let Some(kf) = prop.keyframes.first() {
        crate::schema::parse_vec3(&kf.value)
            .map(|v| (v[0], v[1]))
            .unwrap_or((0.0, 0.0))
    } else {
        (config.canvas_width / 2.0, config.canvas_height / 2.0)
    };

    am_to_bevy_coords(x, y, config)
}

/// Get initial rotation from animated property.
fn get_initial_rotation(prop: &AmAnimatedFloat) -> f32 {
    if let Some(val) = prop.value {
        -val // Negate for Bevy's coordinate system
    } else if let Some(kf) = prop.keyframes.first() {
        -kf.value.parse().unwrap_or(0.0)
    } else {
        0.0
    }
}

/// Get initial scale from animated property.
fn get_initial_scale(prop: &AmAnimatedVec2) -> (f32, f32) {
    if let Some(val) = &prop.value {
        (val[0], val[1])
    } else if let Some(kf) = prop.keyframes.first() {
        crate::schema::parse_vec2(&kf.value)
            .unwrap_or([1.0, 1.0])
            .into()
    } else {
        (1.0, 1.0)
    }
}

/// Get initial opacity from animated property.
fn get_initial_opacity(prop: &AmAnimatedFloat) -> f32 {
    if let Some(val) = prop.value {
        val
    } else if let Some(kf) = prop.keyframes.first() {
        kf.value.parse().unwrap_or(1.0)
    } else {
        1.0
    }
}

/// Get shape size from properties.
fn get_shape_size(properties: &[crate::schema::AmProperty]) -> (f32, f32) {
    for prop in properties {
        if prop.name == "size"
            && prop.prop_type == "vec2"
            && let Ok(size) = crate::schema::parse_vec2(&prop.value)
        {
            return (size[0], size[1]);
        }
    }
    (100.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_am_to_bevy_coords() {
        let config = AmSceneConfig {
            canvas_width: 1280.0,
            canvas_height: 960.0,
            flip_y: true,
            z_spacing: 0.001,
        };

        // Center of AM canvas should be at Bevy origin
        let (x, y) = am_to_bevy_coords(640.0, 480.0, &config);
        assert!((x - 0.0).abs() < 0.01, "Center X should be 0, got {}", x);
        assert!((y - 0.0).abs() < 0.01, "Center Y should be 0, got {}", y);

        // Top-left of AM canvas
        let (x, y) = am_to_bevy_coords(0.0, 0.0, &config);
        assert!(
            (x - (-640.0)).abs() < 0.01,
            "Top-left X should be -640, got {}",
            x
        );
        assert!(
            (y - 480.0).abs() < 0.01,
            "Top-left Y should be 480, got {}",
            y
        );

        // Bottom-right of AM canvas
        let (x, y) = am_to_bevy_coords(1280.0, 960.0, &config);
        assert!(
            (x - 640.0).abs() < 0.01,
            "Bottom-right X should be 640, got {}",
            x
        );
        assert!(
            (y - (-480.0)).abs() < 0.01,
            "Bottom-right Y should be -480, got {}",
            y
        );
    }

    #[test]
    fn test_get_shape_size() {
        let props = vec![crate::schema::AmProperty {
            name: "size".to_string(),
            prop_type: "vec2".to_string(),
            value: "200.0,300.0".to_string(),
            keyframes: vec![],
        }];

        let (w, h) = get_shape_size(&props);
        assert!((w - 200.0).abs() < 0.01);
        assert!((h - 300.0).abs() < 0.01);
    }
}

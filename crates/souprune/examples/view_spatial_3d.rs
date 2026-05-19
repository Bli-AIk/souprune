//! 3D manual-acceptance harness for loading a spatial View plane.
//!
//! 加载空间 View 平面的 3D 手工验收示例。
//!
//! ```bash
//! cargo run -p souprune --example view_spatial_3d
//! ```

use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use std::path::PathBuf;

use bevy::asset::UnapprovedPathMode;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::ecs::message::MessageReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use souprune::core::camera::MainGameCamera;
use souprune::core::view::SpawnViewRequest;
use souprune::core::view::layout::ViewLayoutRect;
use souprune::core::view::spatial::{ViewSpatialHit, ViewSpatialRoot};

const VIEW_PATH: &str = "view/spatial_plane.view.ron";
const ORBIT_DRAG_SENSITIVITY: f32 = 0.006;
const ORBIT_MAX_PITCH: f32 = FRAC_PI_2 - 0.08;
const ORBIT_DEFAULT_RADIUS: f32 = 6.2;
const ORBIT_MIN_RADIUS: f32 = 0.5;

fn main() {
    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/assets");

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(bevy::image::ImagePlugin::default_nearest())
            .set(bevy::asset::AssetPlugin {
                file_path: asset_root.to_string_lossy().into_owned(),
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "SoupRune Spatial View".into(),
                    resolution: WindowResolution::new(960, 540),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
    );

    app.add_plugins((
        souprune::get_file_importer_plugins(),
        souprune::get_third_plugins(),
        souprune::core::CorePlugin,
    ));
    #[cfg(feature = "debug")]
    app.add_plugins(bevy_brp_extras::BrpExtrasPlugin);

    app.insert_resource(souprune::config::load_config());
    souprune::init_game_state(&mut app);
    souprune::insert_input_resources(&mut app);
    app.init_resource::<SpatialOrbitCameraState>();
    app.insert_resource(ClearColor(Color::srgb(0.015, 0.018, 0.024)));
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        orbit_spatial_camera_system.before(souprune::core::view::ViewUpdate),
    );
    app.add_systems(
        Update,
        (
            sync_spatial_acceptance_meshes,
            draw_spatial_acceptance_gizmos,
        )
            .chain()
            .after(souprune::core::view::ViewUpdate),
    );
    app.run();
}

fn setup(
    mut commands: Commands,
    mut next_state: ResMut<NextState<souprune::app_state::AppState>>,
    mut spawn_writer: MessageWriter<SpawnViewRequest>,
) {
    commands.spawn((
        Name::new("View Spatial Camera"),
        Camera3d::default(),
        Tonemapping::None,
        Transform::from_xyz(0.0, 0.2, 6.2).looking_at(Vec3::ZERO, Vec3::Y),
        MainGameCamera,
    ));
    commands.spawn((
        Name::new("SpatialAnchor"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));

    next_state.set(souprune::app_state::AppState::Running);
    spawn_writer.write(SpawnViewRequest {
        path: VIEW_PATH.to_string(),
        mode_scope: None,
        pre_spawn_events: Vec::new(),
        bindings: None,
    });
}

#[derive(Resource, Debug, Clone, Copy)]
struct SpatialOrbitCameraState {
    initialized: bool,
    yaw: f32,
    pitch: f32,
    radius: f32,
}

impl Default for SpatialOrbitCameraState {
    fn default() -> Self {
        Self {
            initialized: false,
            yaw: 0.0,
            pitch: 0.0,
            radius: ORBIT_DEFAULT_RADIUS,
        }
    }
}

impl SpatialOrbitCameraState {
    fn initialize_from_camera(&mut self, camera_position: Vec3, target: Vec3) {
        let offset = camera_position - target;
        let radius = offset.length().max(ORBIT_MIN_RADIUS);
        let horizontal = Vec2::new(offset.x, offset.z).length();

        self.initialized = true;
        self.yaw = offset.x.atan2(offset.z);
        self.pitch = offset
            .y
            .atan2(horizontal)
            .clamp(-ORBIT_MAX_PITCH, ORBIT_MAX_PITCH);
        self.radius = radius;
    }
}

fn orbit_spatial_camera_system(
    mut state: ResMut<SpatialOrbitCameraState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut motion_events: MessageReader<MouseMotion>,
    mut camera_query: Query<(&mut Transform, &Camera), (With<Camera3d>, With<MainGameCamera>)>,
    spatial_roots: Query<&GlobalTransform, With<ViewSpatialRoot>>,
) {
    let Some(target) = spatial_roots
        .iter()
        .next()
        .map(GlobalTransform::translation)
    else {
        motion_events.clear();
        return;
    };
    let Some((mut camera_transform, _)) =
        camera_query.iter_mut().find(|(_, camera)| camera.is_active)
    else {
        motion_events.clear();
        return;
    };

    if !state.initialized {
        state.initialize_from_camera(camera_transform.translation, target);
    }

    if mouse_button.pressed(MouseButton::Right) {
        let total_delta: Vec2 = motion_events.read().map(|event| event.delta).sum();
        if total_delta.length_squared() > 0.01 {
            state.yaw -= total_delta.x * ORBIT_DRAG_SENSITIVITY;
            state.pitch = (state.pitch - total_delta.y * ORBIT_DRAG_SENSITIVITY)
                .clamp(-ORBIT_MAX_PITCH, ORBIT_MAX_PITCH);
        }
    } else {
        motion_events.clear();
    }

    *camera_transform = orbit_camera_transform(target, state.yaw, state.pitch, state.radius);
}

#[derive(Component)]
struct SpatialAcceptanceMesh {
    source: Entity,
    width: f32,
    height: f32,
}

fn sync_spatial_acceptance_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut previews_by_source: Local<HashMap<Entity, Entity>>,
    spatial_roots: Query<&ViewSpatialRoot>,
    layout_rects: Query<(Entity, &GlobalTransform, &ViewLayoutRect, Option<&Name>)>,
    mut previews: Query<(
        &mut SpatialAcceptanceMesh,
        &mut Transform,
        &mut Mesh3d,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let pixels_per_unit = spatial_roots
        .iter()
        .next()
        .map(|root| root.plane.pixels_per_unit)
        .map(valid_pixels_per_unit)
        .unwrap_or(100.0);

    let mut stale_sources = Vec::new();
    for (&source, &preview_entity) in previews_by_source.iter() {
        if layout_rects.get(source).is_err() || previews.get(preview_entity).is_err() {
            stale_sources.push(source);
        }
    }
    for source in stale_sources {
        if let Some(preview_entity) = previews_by_source.remove(&source) {
            commands.entity(preview_entity).despawn();
        }
    }

    for (source, global_transform, rect, name) in &layout_rects {
        let width = rect.width / pixels_per_unit;
        let height = rect.height / pixels_per_unit;
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            continue;
        }

        let transform = preview_transform(global_transform, preview_depth(name));
        let Some(preview_entity) = previews_by_source.get(&source).copied() else {
            let preview_entity = commands
                .spawn((
                    Name::new(preview_name(name)),
                    Mesh3d(meshes.add(Rectangle::new(width, height))),
                    MeshMaterial3d(materials.add(preview_material(rect_color(name)))),
                    transform,
                    SpatialAcceptanceMesh {
                        source,
                        width,
                        height,
                    },
                ))
                .id();
            previews_by_source.insert(source, preview_entity);
            continue;
        };

        let Ok((mut preview, mut preview_transform, mut mesh, mut material)) =
            previews.get_mut(preview_entity)
        else {
            previews_by_source.remove(&source);
            continue;
        };

        debug_assert_eq!(preview.source, source);
        *preview_transform = transform;
        if (preview.width - width).abs() > f32::EPSILON
            || (preview.height - height).abs() > f32::EPSILON
        {
            preview.width = width;
            preview.height = height;
            *mesh = Mesh3d(meshes.add(Rectangle::new(width, height)));
            *material = MeshMaterial3d(materials.add(preview_material(rect_color(name))));
        }
    }
}

fn draw_spatial_acceptance_gizmos(
    mut gizmos: Gizmos,
    spatial_roots: Query<(&GlobalTransform, &ViewSpatialRoot)>,
    layout_rects: Query<(&GlobalTransform, &ViewLayoutRect, Option<&Name>)>,
    hits: Query<&ViewSpatialHit>,
) {
    let Some((root_transform, spatial_root)) = spatial_roots.iter().next() else {
        return;
    };
    draw_reference_plane(&mut gizmos, root_transform, spatial_root);

    let pixels_per_unit = valid_pixels_per_unit(spatial_root.plane.pixels_per_unit);
    for (transform, rect, name) in &layout_rects {
        draw_layout_rect(
            &mut gizmos,
            transform,
            rect.width / pixels_per_unit,
            rect.height / pixels_per_unit,
            rect_color(name),
        );
    }
    for hit in &hits {
        draw_hit_marker(&mut gizmos, hit.world_position);
    }
}

fn draw_reference_plane(
    gizmos: &mut Gizmos,
    transform: &GlobalTransform,
    spatial_root: &ViewSpatialRoot,
) {
    let half_width = spatial_root.plane.plane_size.0 * 0.5;
    let half_height = spatial_root.plane.plane_size.1 * 0.5;
    let z = -0.04;
    let top_left = Vec3::new(-half_width, half_height, z);
    let top_right = Vec3::new(half_width, half_height, z);
    let bottom_right = Vec3::new(half_width, -half_height, z);
    let bottom_left = Vec3::new(-half_width, -half_height, z);

    draw_polyline(
        gizmos,
        transform,
        &[top_left, top_right, bottom_right, bottom_left],
        Color::srgba(0.42, 0.56, 0.72, 0.8),
    );
    gizmos.line(
        transform.transform_point(Vec3::new(-half_width, 0.0, z)),
        transform.transform_point(Vec3::new(half_width, 0.0, z)),
        Color::srgb(0.95, 0.22, 0.28),
    );
    gizmos.line(
        transform.transform_point(Vec3::new(0.0, -half_height, z)),
        transform.transform_point(Vec3::new(0.0, half_height, z)),
        Color::srgb(0.24, 0.82, 0.42),
    );
}

fn draw_layout_rect(
    gizmos: &mut Gizmos,
    transform: &GlobalTransform,
    width: f32,
    height: f32,
    color: Color,
) {
    draw_polyline(
        gizmos,
        transform,
        &[
            Vec3::ZERO,
            Vec3::new(width, 0.0, 0.0),
            Vec3::new(width, -height, 0.0),
            Vec3::new(0.0, -height, 0.0),
        ],
        color,
    );
}

fn draw_hit_marker(gizmos: &mut Gizmos, position: Vec3) {
    let extent = 0.06;
    gizmos.line(
        position + Vec3::new(-extent, 0.0, 0.0),
        position + Vec3::new(extent, 0.0, 0.0),
        Color::srgb(1.0, 1.0, 1.0),
    );
    gizmos.line(
        position + Vec3::new(0.0, -extent, 0.0),
        position + Vec3::new(0.0, extent, 0.0),
        Color::srgb(1.0, 1.0, 1.0),
    );
}

fn draw_polyline(
    gizmos: &mut Gizmos,
    transform: &GlobalTransform,
    points: &[Vec3; 4],
    color: Color,
) {
    for index in 0..points.len() {
        let start = transform.transform_point(points[index]);
        let end = transform.transform_point(points[(index + 1) % points.len()]);
        gizmos.line(start, end, color);
    }
}

fn rect_color(name: Option<&Name>) -> Color {
    let Some(name) = name else {
        return Color::srgb(0.86, 0.9, 0.96);
    };
    match name.as_str() {
        "SpatialPanel" => Color::srgb(0.95, 0.68, 0.22),
        "SpatialRow" => Color::srgb(0.22, 0.7, 0.98),
        "SpatialAbsoluteMarker" => Color::srgb(0.98, 0.18, 0.34),
        _ => Color::srgb(0.88, 0.92, 1.0),
    }
}

fn preview_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        unlit: true,
        cull_mode: None,
        ..default()
    }
}

fn preview_transform(global_transform: &GlobalTransform, depth: f32) -> Transform {
    let mut transform = global_transform.compute_transform();
    transform.scale = Vec3::ONE;
    transform.translation += transform.rotation * Vec3::new(0.0, 0.0, depth);
    transform
}

fn preview_depth(name: Option<&Name>) -> f32 {
    let Some(name) = name else {
        return 0.02;
    };
    match name.as_str() {
        "SpatialPanel" => -0.04,
        "SpatialRow" => 0.02,
        "SpatialRowItemA" => 0.05,
        "SpatialRowItemB" => 0.06,
        "SpatialAbsoluteMarker" => 0.07,
        _ => 0.03,
    }
}

fn preview_name(name: Option<&Name>) -> String {
    let Some(name) = name else {
        return "SpatialAcceptanceMesh".to_string();
    };
    format!("SpatialAcceptanceMesh:{}", name.as_str())
}

fn orbit_camera_transform(target: Vec3, yaw: f32, pitch: f32, radius: f32) -> Transform {
    let offset = orbit_camera_offset(yaw, pitch, radius);
    let mut transform = Transform::from_translation(target + offset);
    transform.look_at(target, Vec3::Y);
    transform
}

fn orbit_camera_offset(yaw: f32, pitch: f32, radius: f32) -> Vec3 {
    let clamped_pitch = pitch.clamp(-ORBIT_MAX_PITCH, ORBIT_MAX_PITCH);
    let clamped_radius = radius.max(ORBIT_MIN_RADIUS);
    let horizontal = clamped_pitch.cos() * clamped_radius;

    Vec3::new(
        yaw.sin() * horizontal,
        clamped_pitch.sin() * clamped_radius,
        yaw.cos() * horizontal,
    )
}

fn valid_pixels_per_unit(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orbit_camera_offset_uses_yaw_pitch_and_radius() {
        let offset = orbit_camera_offset(0.0, 0.0, 6.0);

        assert!((offset.x - 0.0).abs() < 0.0001);
        assert!((offset.y - 0.0).abs() < 0.0001);
        assert!((offset.z - 6.0).abs() < 0.0001);
    }
}

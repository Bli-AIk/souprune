//! 3D manual-acceptance harness for loading a spatial View plane.
//!
//! 加载空间 View 平面的 3D 手工验收示例。
//!
//! ```bash
//! cargo run -p souprune --example view_spatial_3d
//! ```

use std::path::PathBuf;

use bevy::asset::UnapprovedPathMode;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use souprune::core::camera::MainGameCamera;
use souprune::core::view::layout::ViewLayoutRect;
use souprune::core::view::spatial::{ViewSpatialHit, ViewSpatialRoot};
use souprune::core::view::SpawnViewRequest;

const VIEW_PATH: &str = "view/spatial_plane.view.ron";

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

    souprune::init_game_state(&mut app);
    app.insert_resource(ClearColor(Color::srgb(0.015, 0.018, 0.024)));
    app.add_systems(Startup, setup);
    app.add_systems(Update, draw_spatial_acceptance_gizmos);
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

fn valid_pixels_per_unit(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

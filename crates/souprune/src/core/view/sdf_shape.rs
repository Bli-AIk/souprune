//! # sdf_shape.rs
//!
//! # sdf_shape.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module provides SDF shape rendering using bevy_alight_motion's SdfMaterial.
//!
//! 本模块使用 bevy_alight_motion 的 SdfMaterial 提供 SDF 形状渲染。

use bevy::prelude::*;
use bevy_alight_motion::sdf_material::{SdfMaterial, SdfShapeType};

/// Component for SDF shape rendering in View system.
///
/// 用于 View 系统的 SDF 形状渲染组件。
#[derive(Component, Debug, Clone)]
pub struct ViewSdfShape {
    /// Fill color
    pub color: Color,
    /// Half width of the box
    pub half_width: f32,
    /// Half height of the box
    pub half_height: f32,
}

impl Default for ViewSdfShape {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            half_width: 50.0,
            half_height: 50.0,
        }
    }
}

impl ViewSdfShape {
    /// Create a new ViewSdfShape with specified dimensions and color.
    pub fn new(width: f32, height: f32, color: Color) -> Self {
        Self {
            color,
            half_width: width / 2.0,
            half_height: height / 2.0,
        }
    }

    /// Calculate the frame_half value used by the shader.
    /// This must match the mesh half size for correct rendering.
    fn frame_half(&self) -> f32 {
        // Add some margin for anti-aliasing and stroke rendering
        self.half_width.max(self.half_height) + 10.0
    }

    /// Create the SdfMaterial for this shape.
    pub fn to_material(&self) -> SdfMaterial {
        let frame_half = self.frame_half();
        SdfMaterial::new_with_frame_half(
            SdfShapeType::BoxMiter, // Sharp corners for UI boxes
            self.half_width,
            self.half_height,
            self.color,
            0.0,         // No stroke
            Color::NONE, // No stroke color
            frame_half,
        )
    }

    /// Calculate the frame size (mesh size) needed to render this shape.
    /// This must equal frame_half * 2.0 for correct UV mapping.
    pub fn frame_size(&self) -> f32 {
        self.frame_half() * 2.0
    }

    /// Create a quad mesh for this shape.
    pub fn create_mesh(&self) -> Mesh {
        let frame = self.frame_size();
        Rectangle::new(frame, frame).into()
    }
}

/// Component to mark an entity as an SDF box shape.
///
/// 标记实体为 SDF 矩形形状的组件。
#[derive(Component, Debug, Clone)]
pub struct SdfBoxShape {
    /// Half width of the box
    pub half_width: f32,
    /// Half height of the box
    pub half_height: f32,
    /// Fill color
    pub fill_color: Color,
    /// Stroke width (0 for no stroke)
    pub stroke_width: f32,
    /// Stroke color
    pub stroke_color: Color,
    /// Shape type (round, miter, or bevel corners)
    pub shape_type: SdfShapeType,
}

impl Default for SdfBoxShape {
    fn default() -> Self {
        Self {
            half_width: 50.0,
            half_height: 50.0,
            fill_color: Color::WHITE,
            stroke_width: 0.0,
            stroke_color: Color::BLACK,
            shape_type: SdfShapeType::BoxRound,
        }
    }
}

impl SdfBoxShape {
    /// Create a new SDF box shape.
    pub fn new(width: f32, height: f32, fill_color: Color) -> Self {
        Self {
            half_width: width / 2.0,
            half_height: height / 2.0,
            fill_color,
            ..Default::default()
        }
    }

    /// Add a stroke to the shape.
    pub fn with_stroke(mut self, width: f32, color: Color) -> Self {
        self.stroke_width = width;
        self.stroke_color = color;
        self
    }

    /// Set corner style to miter (sharp corners).
    pub fn with_miter_corners(mut self) -> Self {
        self.shape_type = SdfShapeType::BoxMiter;
        self
    }

    /// Set corner style to bevel (cut corners).
    pub fn with_bevel_corners(mut self) -> Self {
        self.shape_type = SdfShapeType::BoxBevel;
        self
    }

    /// Create an SdfMaterial from this shape definition.
    pub fn to_material(&self) -> SdfMaterial {
        SdfMaterial::new(
            self.shape_type,
            self.half_width,
            self.half_height,
            self.fill_color,
            self.stroke_width,
            self.stroke_color,
        )
    }

    /// Create a quad mesh for this shape.
    pub fn create_mesh(&self) -> Mesh {
        // The mesh size should be large enough to contain the shape plus stroke
        let frame_half =
            self.half_width.max(self.half_height) * 2.0 + self.stroke_width * 2.0 + 10.0;
        Rectangle::new(frame_half * 2.0, frame_half * 2.0).into()
    }
}

/// Bundle for spawning an SDF box shape entity.
#[derive(Bundle)]
pub struct SdfBoxBundle {
    pub shape: SdfBoxShape,
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<SdfMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl SdfBoxBundle {
    /// Create a new SDF box bundle.
    pub fn new(
        shape: SdfBoxShape,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<SdfMaterial>,
        transform: Transform,
    ) -> Self {
        let mesh = meshes.add(shape.create_mesh());
        let material = materials.add(shape.to_material());

        Self {
            shape,
            mesh: Mesh2d(mesh),
            material: MeshMaterial2d(material),
            transform,
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}

/// Helper function to spawn an SDF box shape.
pub fn spawn_sdf_box(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<SdfMaterial>,
    width: f32,
    height: f32,
    fill_color: Color,
    position: Vec3,
) -> Entity {
    let shape = SdfBoxShape::new(width, height, fill_color);
    let bundle = SdfBoxBundle::new(
        shape,
        meshes,
        materials,
        Transform::from_translation(position),
    );
    commands.spawn(bundle).id()
}

/// Helper function to spawn an SDF box with border.
#[expect(clippy::too_many_arguments)] // reason: Bevy system with many parameters
pub fn spawn_sdf_box_with_border(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<SdfMaterial>,
    width: f32,
    height: f32,
    fill_color: Color,
    border_width: f32,
    border_color: Color,
    position: Vec3,
) -> Entity {
    let shape = SdfBoxShape::new(width, height, fill_color).with_stroke(border_width, border_color);
    let bundle = SdfBoxBundle::new(
        shape,
        meshes,
        materials,
        Transform::from_translation(position),
    );
    commands.spawn(bundle).id()
}

/// System to update SDF shapes when their SdfBoxShape component changes.
pub fn update_sdf_box_shapes(
    query: Query<(&SdfBoxShape, &MeshMaterial2d<SdfMaterial>), Changed<SdfBoxShape>>,
    mut materials: ResMut<Assets<SdfMaterial>>,
) {
    for (shape, material_handle) in query.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            *material = shape.to_material();
        }
    }
}

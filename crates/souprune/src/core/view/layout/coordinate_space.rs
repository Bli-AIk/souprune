//! # coordinate_space.rs
//!
//! # coordinate_space.rs 文件
//!
//! Defines View coordinate-space conversion rules and applies them after RON
//! deserialization.
//!
//! 定义 View 坐标空间转换规则，并在 RON 反序列化后应用这些规则。

use super::serde_types::{SerializableTransform, SerializableVec2, SerializableVec3};
use super::view_schema::{SpriteDef, ViewLayoutAsset, ViewNodeDef};
use crate::core::sequencer::chapter_schema::Value;
use serde::{Deserialize, Serialize};

/// Coordinate system preset for View layouts.
/// Defines how coordinates in RON files are transformed to Bevy's y-up world space.
///
/// View 布局的坐标系预设。
/// 定义 RON 文件中的坐标如何转换为 Bevy 的 y-up 世界坐标。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Default, PartialEq, Eq)]
pub enum CoordinateSystem {
    /// Bevy standard: y-up, pivot(0,0)=bottom-left. No transformation.
    ///
    /// Bevy 标准坐标系：y-up, pivot(0,0)=左下角。不做任何转换。
    #[default]
    Standard,

    /// Screen-space / y-down coordinate system.
    /// Used by GMS, LÖVE, HTML Canvas, Pygame, and most 2D game engines.
    /// Transformation: translation.y negated, pivot.y flipped to 1.0 - y.
    ///
    /// 屏幕坐标系（y-down）。
    /// 适用于 GMS、LÖVE、HTML Canvas、Pygame 等 y-down 引擎的坐标。
    /// 转换规则：translation.y 取反，pivot.y 翻转为 1.0 - y。
    YDown,
}

/// Full coordinate-space description for imported View layouts.
///
/// 导入型 View 布局的完整坐标空间描述。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CoordinateSpaceDef {
    /// Normalized source-canvas position of coordinate `(0, 0)`.
    /// `(0, 0)` is top-left, `(0.5, 0.5)` is center, `(1, 0)` is top-right.
    ///
    /// 源画布中坐标 `(0, 0)` 的归一化位置。
    /// `(0, 0)` 为左上，`(0.5, 0.5)` 为中心，`(1, 0)` 为右上。
    pub axis_origin: SerializableVec2,

    /// Source coordinate Y-axis direction.
    ///
    /// 源坐标 Y 轴方向。
    pub y_axis: YAxisDirectionDef,

    /// Source positive rotation direction.
    ///
    /// 源坐标正旋转方向。
    pub rotation: RotationDirectionDef,

    /// Source canvas size in source units.
    ///
    /// 源画布尺寸，以源坐标单位表示。
    pub extent: CoordinateExtentDef,
}

/// Source coordinate Y-axis direction.
///
/// 源坐标 Y 轴方向。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Default, PartialEq, Eq)]
pub enum YAxisDirectionDef {
    /// Positive Y points up.
    ///
    /// Y 正方向朝上。
    #[default]
    Up,
    /// Positive Y points down.
    ///
    /// Y 正方向朝下。
    Down,
}

/// Source positive rotation direction.
///
/// 源坐标正旋转方向。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Default, PartialEq, Eq)]
pub enum RotationDirectionDef {
    /// Positive angles rotate counter-clockwise.
    ///
    /// 正角度逆时针旋转。
    #[default]
    CounterClockwise,
    /// Positive angles rotate clockwise.
    ///
    /// 正角度顺时针旋转。
    Clockwise,
}

/// Source canvas extent.
///
/// 源画布尺寸。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum CoordinateExtentDef {
    /// Explicit `(width, height)` in source units.
    ///
    /// 以源坐标单位显式给出的 `(宽, 高)`。
    Explicit((f32, f32)),
}

impl CoordinateExtentDef {
    fn size(self) -> (f32, f32) {
        match self {
            Self::Explicit(size) => size,
        }
    }
}

impl ViewLayoutAsset {
    /// Apply coordinate system transformation to all nodes.
    /// Called once after deserialization; converts all coordinates to Standard (y-up).
    ///
    /// 对所有节点应用坐标系变换。
    /// 反序列化后调用一次，将所有坐标转换为 Standard（y-up）。
    pub fn apply_coordinate_system(&mut self) {
        if let Some(space) = self.coordinate_space.clone() {
            for root in &mut self.roots {
                apply_coordinate_space_to_node(root, &space, true);
            }
            self.coordinate_system = CoordinateSystem::Standard;
            return;
        }

        match self.coordinate_system {
            CoordinateSystem::Standard => {}
            CoordinateSystem::YDown => {
                for root in &mut self.roots {
                    flip_node_y(root);
                }
                self.coordinate_system = CoordinateSystem::Standard;
            }
        }
    }
}

fn apply_coordinate_space_to_node(
    node: &mut ViewNodeDef,
    space: &CoordinateSpaceDef,
    is_root: bool,
) {
    let has_node_transform = node.transform.is_some();
    if let Some(transform) = &mut node.transform {
        convert_transform(transform, space, is_root);
    }

    let content_is_root = is_root && !has_node_transform;
    if let Some(sprite) = &mut node.sprite {
        convert_sprite(sprite, space, content_is_root);
    }
    if let Some(state_sprite) = &mut node.state_sprite
        && let Some(transform) = &mut state_sprite.transform
    {
        convert_transform(transform, space, content_is_root);
    }
    for text in &mut node.texts {
        convert_transform(&mut text.transform, space, content_is_root);
    }
    if let Some(view_box) = &mut node.view_box {
        convert_translation(&mut view_box.offset, space, content_is_root);
    }
    for child in &mut node.children {
        apply_coordinate_space_to_node(child, space, false);
    }
}

fn convert_sprite(sprite: &mut SpriteDef, space: &CoordinateSpaceDef, is_root: bool) {
    if let Some(transform) = &mut sprite.transform {
        convert_transform(transform, space, is_root);
    }
    if space.y_axis == YAxisDirectionDef::Down
        && let Some(pivot) = &mut sprite.pivot
    {
        flip_pivot_y(pivot);
    }
}

fn convert_transform(
    transform: &mut SerializableTransform,
    space: &CoordinateSpaceDef,
    is_root: bool,
) {
    if let Some(translation) = &mut transform.translation {
        convert_translation(translation, space, is_root);
    }
    if space.rotation == RotationDirectionDef::Clockwise
        && let Some(rotation) = &mut transform.rotation
    {
        *rotation = negate_value(rotation);
    }
}

fn convert_translation(
    translation: &mut SerializableVec3,
    space: &CoordinateSpaceDef,
    is_root: bool,
) {
    translation.0 = convert_x_value(&translation.0, space, is_root);
    translation.1 = convert_y_value(&translation.1, space, is_root);
}

fn convert_x_value(value: &Value<f32>, space: &CoordinateSpaceDef, is_root: bool) -> Value<f32> {
    if !is_root {
        return value.clone();
    }
    add_values(value, &root_x_offset(space))
}

fn convert_y_value(value: &Value<f32>, space: &CoordinateSpaceDef, is_root: bool) -> Value<f32> {
    let signed = match space.y_axis {
        YAxisDirectionDef::Up => value.clone(),
        YAxisDirectionDef::Down => negate_value(value),
    };

    if is_root {
        add_values(&signed, &root_y_offset(space))
    } else {
        signed
    }
}

fn root_x_offset(space: &CoordinateSpaceDef) -> Value<f32> {
    let (width, _) = space.extent.size();
    match &space.axis_origin.0 {
        Value::Static(origin_x) => Value::Static((origin_x - 0.5) * width),
        Value::Expr(expr) => Value::Expr(format!("(({}) - 0.5) * {}", expr, width)),
    }
}

fn root_y_offset(space: &CoordinateSpaceDef) -> Value<f32> {
    let (_, height) = space.extent.size();
    match &space.axis_origin.1 {
        Value::Static(origin_y) => Value::Static((0.5 - origin_y) * height),
        Value::Expr(expr) => Value::Expr(format!("(0.5 - ({})) * {}", expr, height)),
    }
}

fn flip_node_y(node: &mut ViewNodeDef) {
    if let Some(transform) = &mut node.transform {
        flip_transform_y(transform);
    }
    if let Some(sprite) = &mut node.sprite {
        flip_sprite_y(sprite);
    }
    if let Some(state_sprite) = &mut node.state_sprite
        && let Some(transform) = &mut state_sprite.transform
    {
        flip_transform_y(transform);
    }
    for text in &mut node.texts {
        flip_transform_y(&mut text.transform);
    }
    if let Some(view_box) = &mut node.view_box {
        flip_value_y(&mut view_box.offset.1);
    }
    for child in &mut node.children {
        flip_node_y(child);
    }
}

fn flip_sprite_y(sprite: &mut SpriteDef) {
    if let Some(transform) = &mut sprite.transform {
        flip_transform_y(transform);
    }
    if let Some(pivot) = &mut sprite.pivot {
        flip_pivot_y(pivot);
    }
}

fn flip_transform_y(transform: &mut SerializableTransform) {
    if let Some(translation) = &mut transform.translation {
        flip_value_y(&mut translation.1);
    }
}

fn flip_pivot_y(pivot: &mut SerializableVec2) {
    pivot.1 = match &pivot.1 {
        Value::Static(v) => Value::Static(1.0 - v),
        Value::Expr(s) => Value::Expr(format!("1.0 - ({})", s)),
    };
}

fn flip_value_y(value: &mut Value<f32>) {
    *value = negate_value(value);
}

fn add_values(left: &Value<f32>, right: &Value<f32>) -> Value<f32> {
    match (left, right) {
        (Value::Static(left), Value::Static(right)) => Value::Static(left + right),
        (Value::Static(left), value) if *left == 0.0 => value.clone(),
        (value, Value::Static(right)) if *right == 0.0 => value.clone(),
        (left, right) => Value::Expr(format!("({}) + ({})", value_expr(left), value_expr(right))),
    }
}

fn negate_value(value: &Value<f32>) -> Value<f32> {
    match value {
        Value::Static(value) => Value::Static(-value),
        Value::Expr(expr) => Value::Expr(format!("-({})", expr)),
    }
}

fn value_expr(value: &Value<f32>) -> String {
    match value {
        Value::Static(value) => value.to_string(),
        Value::Expr(expr) => expr.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::view::layout::ViewLayoutAsset;

    fn static_value(value: &Value<f32>) -> f32 {
        match value {
            Value::Static(value) => *value,
            Value::Expr(expr) => panic!("expected static value, got expression {expr}"),
        }
    }

    #[test]
    fn coordinate_space_converts_root_and_child_transforms() {
        let mut asset: ViewLayoutAsset = ron::from_str(
            r#"
            (
                coordinate_space: Some((
                    axis_origin: (0.0, 0.0),
                    y_axis: Down,
                    rotation: CounterClockwise,
                    extent: Explicit((640.0, 480.0)),
                )),
                roots: [
                    (
                        name: "obj_maddummy",
                        transform: Some((translation: Some((270.0, 80.0, 0.0)))),
                        children: [
                            (
                                name: "obj_maddum_drawer",
                                transform: Some((translation: Some((50.0, 10.0, 0.0)))),
                                children: [
                                    (
                                        name: "Base",
                                        sprite: Some((
                                            visual: "base.png",
                                            transform: Some((
                                                translation: Some((5.0, 75.0, 0.0)),
                                                rotation: Some(-3.0),
                                            )),
                                            pivot: Some((0.476, 0.0)),
                                        )),
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            )
            "#,
        )
        .expect("layout should parse");

        asset.apply_coordinate_system();

        let root = asset.roots.first().expect("root node");
        let root_translation = root
            .transform
            .as_ref()
            .and_then(|transform| transform.translation.as_ref())
            .expect("root translation");
        assert_eq!(static_value(&root_translation.0), -50.0);
        assert_eq!(static_value(&root_translation.1), 160.0);

        let drawer = root.children.first().expect("drawer node");
        let drawer_translation = drawer
            .transform
            .as_ref()
            .and_then(|transform| transform.translation.as_ref())
            .expect("drawer translation");
        assert_eq!(static_value(&drawer_translation.0), 50.0);
        assert_eq!(static_value(&drawer_translation.1), -10.0);

        let base = drawer.children.first().expect("base node");
        let sprite = base.sprite.as_ref().expect("base sprite");
        let base_translation = sprite
            .transform
            .as_ref()
            .and_then(|transform| transform.translation.as_ref())
            .expect("base translation");
        assert_eq!(static_value(&base_translation.0), 5.0);
        assert_eq!(static_value(&base_translation.1), -75.0);
        let pivot = sprite.pivot.as_ref().expect("base pivot");
        assert_eq!(static_value(&pivot.1), 1.0);
    }

    #[test]
    fn legacy_y_down_flips_node_transform_without_origin_offset() {
        let mut asset: ViewLayoutAsset = ron::from_str(
            r#"
            (
                coordinate_system: YDown,
                roots: [
                    (
                        name: "Container",
                        transform: Some((translation: Some((10.0, 20.0, 0.0)))),
                    ),
                ],
            )
            "#,
        )
        .expect("layout should parse");

        asset.apply_coordinate_system();

        let translation = asset.roots[0]
            .transform
            .as_ref()
            .and_then(|transform| transform.translation.as_ref())
            .expect("node translation");
        assert_eq!(static_value(&translation.0), 10.0);
        assert_eq!(static_value(&translation.1), -20.0);
    }
}

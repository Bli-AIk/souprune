//! Pure Taffy layout solving for view layout assets.
//!
//! 视图布局资源的纯 Taffy 布局求解。

use super::{
    SerializableAlignItems, SerializableAlignSelf, SerializableDisplay, SerializableJustifyContent,
    SerializablePositionType, SerializableRect, SerializableVal, StyleDef, StyleGap,
    UiFlexDirection, ViewLayoutAsset, ViewLayoutSlot, ViewLayoutSlots, ViewNodeDef,
    layout_child_path, layout_root_path,
};
use bevy::prelude::Vec2;
use std::error::Error;
use std::fmt;
use taffy::prelude::{
    AlignItems as TaffyAlignItems, AvailableSpace, Dimension, Display as TaffyDisplay,
    FlexDirection as TaffyFlexDirection, JustifyContent as TaffyJustifyContent, LengthPercentage,
    LengthPercentageAuto, NodeId, Position as TaffyPosition, Rect, Size, Style, TaffyTree,
};

/// Error returned when pure view layout solving fails.
///
/// 纯视图布局求解失败时返回的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewLayoutError {
    message: String,
}

impl ViewLayoutError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ViewLayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for ViewLayoutError {}

/// Compute stable layout slots for a view layout asset without spawning entities.
///
/// 在不生成实体的情况下，为视图布局资源计算稳定布局槽位。
pub fn compute_taffy_layout(
    asset: &ViewLayoutAsset,
    viewport_size: Vec2,
) -> Result<ViewLayoutSlots, ViewLayoutError> {
    let mut tree = TaffyTree::new();
    let mut nodes = Vec::new();
    for (root_idx, root) in asset.roots.iter().enumerate() {
        build_node(
            &mut tree,
            root,
            layout_root_path(root_idx, root),
            viewport_size,
            false,
            &mut nodes,
        )?;
    }

    let root_ids = nodes
        .iter()
        .filter(|(_, path, _)| !path.contains('/'))
        .map(|(node_id, _, _)| *node_id)
        .collect::<Vec<_>>();

    let available = Size {
        width: AvailableSpace::Definite(viewport_size.x),
        height: AvailableSpace::Definite(viewport_size.y),
    };
    for root_id in root_ids {
        tree.compute_layout(root_id, available)
            .map_err(|error| ViewLayoutError::new(format!("failed to compute layout: {error}")))?;
    }

    let mut slots = ViewLayoutSlots::new();
    for (node_id, path, name) in nodes {
        let layout = tree
            .layout(node_id)
            .map_err(|error| ViewLayoutError::new(format!("failed to read layout: {error}")))?;
        slots.push(ViewLayoutSlot {
            path,
            name,
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        });
    }

    Ok(slots)
}

fn build_node(
    tree: &mut TaffyTree,
    node: &ViewNodeDef,
    path: String,
    viewport_size: Vec2,
    ancestor_hidden: bool,
    nodes: &mut Vec<(NodeId, String, String)>,
) -> Result<NodeId, ViewLayoutError> {
    let hidden = ancestor_hidden || matches!(node.style.display, Some(SerializableDisplay::None));
    let children = node
        .children
        .iter()
        .enumerate()
        .map(|(child_idx, child)| {
            build_node(
                tree,
                child,
                layout_child_path(&path, child_idx, child),
                viewport_size,
                hidden,
                nodes,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let style = to_taffy_style(&node.style, viewport_size);
    let node_id = if children.is_empty() {
        tree.new_leaf(style)
    } else {
        tree.new_with_children(style, &children)
    }
    .map_err(|error| ViewLayoutError::new(format!("failed to create layout node: {error}")))?;
    if !hidden {
        nodes.push((node_id, path, node.name.clone()));
    }
    Ok(node_id)
}

fn to_taffy_style(style: &StyleDef, viewport_size: Vec2) -> Style {
    Style {
        display: style.display.map_or(TaffyDisplay::Flex, to_taffy_display),
        position: style
            .position_type
            .map_or(TaffyPosition::Relative, to_taffy_position),
        inset: Rect {
            left: style.left.map_or(LengthPercentageAuto::auto(), |value| {
                to_length_percentage_auto(value, viewport_size)
            }),
            right: style.right.map_or(LengthPercentageAuto::auto(), |value| {
                to_length_percentage_auto(value, viewport_size)
            }),
            top: style.top.map_or(LengthPercentageAuto::auto(), |value| {
                to_length_percentage_auto(value, viewport_size)
            }),
            bottom: style.bottom.map_or(LengthPercentageAuto::auto(), |value| {
                to_length_percentage_auto(value, viewport_size)
            }),
        },
        size: Size {
            width: style.width.map_or(Dimension::auto(), |value| {
                to_dimension(value, viewport_size)
            }),
            height: style.height.map_or(Dimension::auto(), |value| {
                to_dimension(value, viewport_size)
            }),
        },
        margin: style.margin.map_or_else(auto_rect_zero, |rect| {
            to_length_percentage_auto_rect(rect, viewport_size)
        }),
        padding: style.padding.map_or_else(length_rect_zero, |rect| {
            to_length_percentage_rect(rect, viewport_size)
        }),
        gap: style.gap.map_or_else(length_size_zero, |gap| {
            to_length_percentage_gap(gap, viewport_size)
        }),
        align_items: style.align_items.map(to_taffy_align_items),
        align_self: style.align_self.and_then(to_taffy_align_self),
        justify_content: style.justify_content.map(to_taffy_justify_content),
        flex_direction: style
            .flex_direction
            .map_or(TaffyFlexDirection::Row, to_taffy_flex_direction),
        ..Default::default()
    }
}

fn to_length_percentage_auto_rect(
    rect: SerializableRect,
    viewport_size: Vec2,
) -> Rect<LengthPercentageAuto> {
    Rect {
        left: to_length_percentage_auto(rect.left, viewport_size),
        right: to_length_percentage_auto(rect.right, viewport_size),
        top: to_length_percentage_auto(rect.top, viewport_size),
        bottom: to_length_percentage_auto(rect.bottom, viewport_size),
    }
}

fn auto_rect_zero() -> Rect<LengthPercentageAuto> {
    Rect {
        left: LengthPercentageAuto::length(0.0),
        right: LengthPercentageAuto::length(0.0),
        top: LengthPercentageAuto::length(0.0),
        bottom: LengthPercentageAuto::length(0.0),
    }
}

fn length_rect_zero() -> Rect<LengthPercentage> {
    Rect {
        left: LengthPercentage::length(0.0),
        right: LengthPercentage::length(0.0),
        top: LengthPercentage::length(0.0),
        bottom: LengthPercentage::length(0.0),
    }
}

fn length_size_zero() -> Size<LengthPercentage> {
    Size {
        width: LengthPercentage::length(0.0),
        height: LengthPercentage::length(0.0),
    }
}

fn to_length_percentage_rect(
    rect: SerializableRect,
    viewport_size: Vec2,
) -> Rect<LengthPercentage> {
    Rect {
        left: to_length_percentage(rect.left, viewport_size),
        right: to_length_percentage(rect.right, viewport_size),
        top: to_length_percentage(rect.top, viewport_size),
        bottom: to_length_percentage(rect.bottom, viewport_size),
    }
}

fn to_length_percentage_gap(gap: StyleGap, viewport_size: Vec2) -> Size<LengthPercentage> {
    Size {
        width: to_length_percentage(gap.column, viewport_size),
        height: to_length_percentage(gap.row, viewport_size),
    }
}

fn to_dimension(value: SerializableVal, viewport_size: Vec2) -> Dimension {
    match value {
        SerializableVal::Auto => Dimension::auto(),
        SerializableVal::Px(value) => Dimension::length(value),
        SerializableVal::Percent(value) => Dimension::percent(value / 100.0),
        SerializableVal::Vw(value) => Dimension::length(viewport_size.x * value / 100.0),
        SerializableVal::Vh(value) => Dimension::length(viewport_size.y * value / 100.0),
    }
}

fn to_length_percentage_auto(value: SerializableVal, viewport_size: Vec2) -> LengthPercentageAuto {
    match value {
        SerializableVal::Auto => LengthPercentageAuto::auto(),
        SerializableVal::Px(value) => LengthPercentageAuto::length(value),
        SerializableVal::Percent(value) => LengthPercentageAuto::percent(value / 100.0),
        SerializableVal::Vw(value) => LengthPercentageAuto::length(viewport_size.x * value / 100.0),
        SerializableVal::Vh(value) => LengthPercentageAuto::length(viewport_size.y * value / 100.0),
    }
}

fn to_length_percentage(value: SerializableVal, viewport_size: Vec2) -> LengthPercentage {
    match value {
        SerializableVal::Auto => LengthPercentage::length(0.0),
        SerializableVal::Px(value) => LengthPercentage::length(value),
        SerializableVal::Percent(value) => LengthPercentage::percent(value / 100.0),
        SerializableVal::Vw(value) => LengthPercentage::length(viewport_size.x * value / 100.0),
        SerializableVal::Vh(value) => LengthPercentage::length(viewport_size.y * value / 100.0),
    }
}

fn to_taffy_display(display: SerializableDisplay) -> TaffyDisplay {
    match display {
        SerializableDisplay::Flex => TaffyDisplay::Flex,
        SerializableDisplay::None => TaffyDisplay::None,
    }
}

fn to_taffy_position(position: SerializablePositionType) -> TaffyPosition {
    match position {
        SerializablePositionType::Relative => TaffyPosition::Relative,
        SerializablePositionType::Absolute => TaffyPosition::Absolute,
    }
}

fn to_taffy_flex_direction(direction: UiFlexDirection) -> TaffyFlexDirection {
    match direction {
        UiFlexDirection::Row => TaffyFlexDirection::Row,
        UiFlexDirection::Column => TaffyFlexDirection::Column,
        UiFlexDirection::RowReverse => TaffyFlexDirection::RowReverse,
        UiFlexDirection::ColumnReverse => TaffyFlexDirection::ColumnReverse,
    }
}

fn to_taffy_justify_content(justify_content: SerializableJustifyContent) -> TaffyJustifyContent {
    match justify_content {
        SerializableJustifyContent::Start => TaffyJustifyContent::Start,
        SerializableJustifyContent::End => TaffyJustifyContent::End,
        SerializableJustifyContent::Center => TaffyJustifyContent::Center,
        SerializableJustifyContent::SpaceBetween => TaffyJustifyContent::SpaceBetween,
        SerializableJustifyContent::SpaceAround => TaffyJustifyContent::SpaceAround,
        SerializableJustifyContent::SpaceEvenly => TaffyJustifyContent::SpaceEvenly,
    }
}

fn to_taffy_align_items(align_items: SerializableAlignItems) -> TaffyAlignItems {
    match align_items {
        SerializableAlignItems::Start => TaffyAlignItems::Start,
        SerializableAlignItems::End => TaffyAlignItems::End,
        SerializableAlignItems::Center => TaffyAlignItems::Center,
        SerializableAlignItems::Baseline => TaffyAlignItems::Baseline,
        SerializableAlignItems::Stretch => TaffyAlignItems::Stretch,
    }
}

fn to_taffy_align_self(align_self: SerializableAlignSelf) -> Option<TaffyAlignItems> {
    match align_self {
        SerializableAlignSelf::Auto => None,
        SerializableAlignSelf::Start => Some(TaffyAlignItems::Start),
        SerializableAlignSelf::End => Some(TaffyAlignItems::End),
        SerializableAlignSelf::Center => Some(TaffyAlignItems::Center),
        SerializableAlignSelf::Baseline => Some(TaffyAlignItems::Baseline),
        SerializableAlignSelf::Stretch => Some(TaffyAlignItems::Stretch),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::view::layout::{
        SerializableDisplay, SerializablePositionType, SerializableTransform, SerializableVal,
        StyleDef, StyleGap, UiFlexDirection, ViewNodeDef,
    };

    fn asset(root: ViewNodeDef) -> ViewLayoutAsset {
        ViewLayoutAsset {
            roots: vec![root],
            requires: Vec::new(),
            facts: None,
            world_space: false,
            coordinate_system: Default::default(),
            coordinate_space: None,
        }
    }

    fn node(name: &str, style: StyleDef, children: Vec<ViewNodeDef>) -> ViewNodeDef {
        ViewNodeDef {
            name: name.to_string(),
            tags: Vec::new(),
            style,
            transform: None,
            visible_when: None,
            background_color: None,
            border_color: None,
            image: None,
            sprite: None,
            state_sprite: None,
            texts: Vec::new(),
            view_box: None,
            children,
            repeat: None,
        }
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.01,
            "expected {actual} to be close to {expected}"
        );
    }

    #[test]
    fn row_flex_centers_children_with_gap() {
        let child_style = StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            ..Default::default()
        };
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                flex_direction: Some(UiFlexDirection::Row),
                justify_content: Some(
                    crate::core::view::layout::SerializableJustifyContent::Center,
                ),
                gap: Some(StyleGap {
                    row: SerializableVal::Px(0.0),
                    column: SerializableVal::Px(20.0),
                }),
                ..Default::default()
            },
            vec![
                node("first", child_style.clone(), Vec::new()),
                node("second", child_style, Vec::new()),
            ],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        assert_close(slots.get("0:root/0:first").expect("first slot").x, 210.0);
        assert_close(slots.get("0:root/1:second").expect("second slot").x, 330.0);
    }

    #[test]
    fn absolute_child_uses_parent_inset_and_does_not_participate_in_sibling_flex() {
        let sized_child = StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            ..Default::default()
        };
        let absolute_child = StyleDef {
            width: Some(SerializableVal::Px(50.0)),
            height: Some(SerializableVal::Px(30.0)),
            left: Some(SerializableVal::Px(25.0)),
            top: Some(SerializableVal::Px(35.0)),
            position_type: Some(SerializablePositionType::Absolute),
            ..Default::default()
        };
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                flex_direction: Some(UiFlexDirection::Row),
                gap: Some(StyleGap {
                    row: SerializableVal::Px(0.0),
                    column: SerializableVal::Px(20.0),
                }),
                ..Default::default()
            },
            vec![
                node("first", sized_child.clone(), Vec::new()),
                node("absolute", absolute_child, Vec::new()),
                node("second", sized_child, Vec::new()),
            ],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        let absolute = slots.get("0:root/1:absolute").expect("absolute slot");
        assert_close(absolute.x, 25.0);
        assert_close(absolute.y, 35.0);
        assert_close(slots.get("0:root/0:first").expect("first slot").x, 0.0);
        assert_close(slots.get("0:root/2:second").expect("second slot").x, 120.0);
    }

    #[test]
    fn display_none_node_is_absent_from_slots_and_flex_flow() {
        let visible_child = StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            ..Default::default()
        };
        let hidden_child = StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            display: Some(SerializableDisplay::None),
            ..Default::default()
        };
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                flex_direction: Some(UiFlexDirection::Row),
                gap: Some(StyleGap {
                    row: SerializableVal::Px(0.0),
                    column: SerializableVal::Px(20.0),
                }),
                ..Default::default()
            },
            vec![
                node("first", visible_child.clone(), Vec::new()),
                node("hidden", hidden_child, Vec::new()),
                node("second", visible_child, Vec::new()),
            ],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        assert!(slots.get("0:root/1:hidden").is_none());
        assert_close(slots.get("0:root/0:first").expect("first slot").x, 0.0);
        assert_close(slots.get("0:root/2:second").expect("second slot").x, 120.0);
    }

    #[test]
    fn explicit_transform_is_not_applied_to_solver_output() {
        let mut child = node(
            "child",
            StyleDef {
                width: Some(SerializableVal::Px(100.0)),
                height: Some(SerializableVal::Px(40.0)),
                ..Default::default()
            },
            Vec::new(),
        );
        child.transform = Some(SerializableTransform {
            translation: Some((
                crate::core::sequencer::chapter_schema::Value::Static(1000.0),
                crate::core::sequencer::chapter_schema::Value::Static(2000.0),
                crate::core::sequencer::chapter_schema::Value::Static(0.0),
            )),
            rotation: None,
            scale: None,
        });
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                ..Default::default()
            },
            vec![child],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        let child = slots.get("0:root/0:child").expect("child slot");
        assert_close(child.x, 0.0);
        assert_close(child.y, 0.0);
    }

    #[test]
    fn sibling_index_keeps_duplicate_names_distinct() {
        let child_style = StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            ..Default::default()
        };
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                flex_direction: Some(UiFlexDirection::Row),
                gap: Some(StyleGap {
                    row: SerializableVal::Px(0.0),
                    column: SerializableVal::Px(20.0),
                }),
                ..Default::default()
            },
            vec![
                node("dup", child_style.clone(), Vec::new()),
                node("dup", child_style, Vec::new()),
            ],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        assert_close(slots.get("0:root/0:dup").expect("first duplicate").x, 0.0);
        assert_close(
            slots.get("0:root/1:dup").expect("second duplicate").x,
            120.0,
        );
    }

    #[test]
    fn manual_acceptance_view_asset_parses_and_solves() {
        let mut asset: ViewLayoutAsset = ron::from_str(include_str!(
            "../../../../examples/assets/view/taffy_minimal.view.ron"
        ))
        .expect("manual acceptance view should parse");
        asset.apply_coordinate_system();

        let slots = compute_taffy_layout(&asset, Vec2::new(640.0, 480.0)).unwrap();

        let centered = slots
            .get("0:TaffyMinimalRoot/0:CenteredElement")
            .expect("centered element slot");
        assert_close(centered.x, 240.0);
        assert_close(centered.width, 160.0);
        assert!(
            slots
                .get("0:TaffyMinimalRoot/3:HiddenDisplayNoneProbe")
                .is_none()
        );
    }
}

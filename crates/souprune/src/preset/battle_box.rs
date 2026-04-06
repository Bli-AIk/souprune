//! Shared battle box runtime types.
//!
//! 被 sequencer、view 和 battle 系统共同使用的战斗框基础类型。

use crate::core::view::components::ViewBox;
use bevy::ecs::message::Message;
use bevy::prelude::*;
use bevy_tween::interpolation::EaseKind;
use serde::{Deserialize, Serialize};

/// Marker component for a battle box boundary.
#[derive(Component)]
pub struct BattleBox;

/// Unique identifier for a battle box.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BattleBoxId(pub String);

/// Binds a player to a specific battle box by ID.
#[derive(Component, Debug, Clone)]
pub struct BoundToBattleBox(pub String);

/// Runtime state of a battle box.
#[derive(Component, Debug, Clone)]
pub struct BattleBoxState {
    pub active: bool,
    pub collision_enabled: bool,
}

impl Default for BattleBoxState {
    fn default() -> Self {
        Self {
            active: true,
            collision_enabled: true,
        }
    }
}

/// Runtime visual style for battle box SDF rendering.
#[derive(Component, Debug, Clone)]
pub struct BattleBoxVisualStyle {
    pub border_width: f32,
    pub fill_shader: Option<String>,
    pub structure_file: Option<String>,
    pub fill_color: Color,
}

impl BattleBoxVisualStyle {
    pub fn from_view_box(view_box: &ViewBox) -> Self {
        Self {
            border_width: view_box.border_width,
            fill_shader: view_box.fill_shader.clone(),
            structure_file: view_box.structure_file.clone(),
            fill_color: view_box.fill_color,
        }
    }

    pub(crate) fn to_view_box(&self, width: f32, height: f32) -> ViewBox {
        ViewBox::new_full(
            width,
            height,
            self.border_width,
            Vec::new(),
            self.fill_shader.clone(),
            self.structure_file.clone(),
            self.fill_color,
        )
    }

    /// Convert a visible gap into the inner boundary gap used by split geometry.
    pub(crate) fn boundary_gap_for_visible_gap(&self, visible_gap: f32) -> f32 {
        if self.structure_file.is_some() && self.border_width > 0.0 {
            visible_gap + self.border_width * 2.0
        } else {
            visible_gap
        }
    }
}

impl Default for BattleBoxVisualStyle {
    fn default() -> Self {
        Self {
            border_width: 5.0,
            fill_shader: None,
            structure_file: None,
            fill_color: Color::BLACK,
        }
    }
}

/// Component storing battle box dimensions for AM-animated battle boxes.
#[derive(Component, Debug, Clone)]
pub struct AlightMotionBattleBoxBounds {
    pub width: f32,
    pub height: f32,
    pub center_offset: Vec2,
}

/// Axis along which to split a battle box.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq)]
pub enum SplitAxis {
    Vertical,
    #[default]
    Horizontal,
}

/// Policy for how gap affects split box dimensions.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq)]
pub enum GapPolicy {
    #[default]
    Expands,
    Includes,
}

/// Event to trigger a battle box split.
#[derive(Message)]
pub struct SplitBattleBox {
    pub source_box: String,
    pub result_boxes: (String, String),
    pub split_axis: SplitAxis,
    pub split_position: f32,
    pub gap: f32,
    pub gap_policy: GapPolicy,
    pub duration: f32,
    pub easing: EaseKind,
}

/// Event to trigger merging two battle boxes back into one.
#[derive(Message)]
pub struct MergeBattleBoxes {
    pub source_boxes: (String, String),
    pub result_box: String,
    pub gap_policy: GapPolicy,
    pub duration: f32,
    pub easing: EaseKind,
}

//! Shared battle runtime types.
//!
//! 被 sequencer、view 和 battle 系统共同使用的战斗框基础类型。

use bevy::ecs::message::Message;
use bevy::prelude::*;
use bevy_tween::interpolation::EaseKind;

use crate::core::collision::ConstraintHandle;
use crate::core::view::components::ViewBox;

/// Marker component for a battle box boundary.
///
/// 战斗框边界的标记组件。
#[derive(Component)]
pub struct BattleBox;

/// Unique identifier for a battle box.
///
/// 战斗框的唯一标识。
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BattleBoxId(pub String);

/// Binds a player to a specific battle box by ID.
///
/// 通过 ID 将玩家绑定到指定战斗框。
#[derive(Component, Debug, Clone)]
pub struct BoundToBattleBox {
    /// Current battle-box id selected by gameplay semantics.
    ///
    /// 当前玩法语义选择的战斗框 ID。
    pub box_id: String,
    /// Host-owned movement constraint handle for this binding.
    ///
    /// 此绑定对应的宿主移动约束句柄。
    pub constraint: Option<ConstraintHandle>,
}

impl BoundToBattleBox {
    /// Create a binding to a battle box id.
    ///
    /// 创建指向某个战斗框 ID 的绑定。
    pub fn new(box_id: impl Into<String>) -> Self {
        Self {
            box_id: box_id.into(),
            constraint: None,
        }
    }

    /// Change the selected battle-box id and return the invalidated constraint.
    ///
    /// 修改选中的战斗框 ID，并返回失效的旧约束。
    pub fn replace_box_id(&mut self, box_id: impl Into<String>) -> Option<ConstraintHandle> {
        self.box_id = box_id.into();
        self.constraint
    }

    /// Clear the current host-owned movement constraint.
    ///
    /// 清除当前宿主拥有的移动约束。
    pub fn clear_constraint(&mut self) -> Option<ConstraintHandle> {
        self.constraint.take()
    }
}

/// Runtime state of a battle box.
///
/// 战斗框的运行时状态。
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
///
/// 战斗框 SDF 渲染使用的运行时视觉样式。
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
    ///
    /// 将可见间隙转换为分割几何使用的内部边界间隙。
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
///
/// 存储 Alight Motion 动画战斗框尺寸的组件。
#[derive(Component, Debug, Clone)]
pub struct AlightMotionBattleBoxBounds {
    pub width: f32,
    pub height: f32,
    pub center_offset: Vec2,
}

/// Axis along which a battle box split runs.
///
/// 战斗框分割使用的轴向。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SplitAxis {
    /// Split into left/right boxes.
    ///
    /// 分割为左右两个框。
    Vertical,
    /// Split into top/bottom boxes.
    ///
    /// 分割为上下两个框。
    #[default]
    Horizontal,
}

/// Policy for how the visible gap affects split dimensions.
///
/// 可见间隙如何影响分割后尺寸的策略。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GapPolicy {
    /// Expand the total occupied boundary to include the gap.
    ///
    /// 扩大整体占用边界以容纳间隙。
    #[default]
    Expands,
    /// Keep the total boundary fixed and include the gap within it.
    ///
    /// 保持整体边界不变，并在其中包含间隙。
    Includes,
}

/// Event to trigger a battle box split.
///
/// 触发战斗框分割的事件。
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
///
/// 触发两个战斗框合并为一个战斗框的事件。
#[derive(Message)]
pub struct MergeBattleBoxes {
    pub source_boxes: (String, String),
    pub result_box: String,
    pub gap_policy: GapPolicy,
    pub duration: f32,
    pub easing: EaseKind,
}

/// System that detects newly spawned ViewBox entities with a "BattleBox" tag
/// and adds game-specific battle box components.
///
/// 检测带有 "BattleBox" 标签的新 ViewBox 实体，并添加战斗框运行时组件。
pub(crate) fn apply_battle_box_tag_system(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &crate::core::view::components::ViewNodeTags,
            &ViewBox,
        ),
        Added<crate::core::view::components::ViewNodeTags>,
    >,
) {
    for (entity, tags, view_box) in query.iter() {
        if tags.0.contains(&"BattleBox".to_string()) {
            let style = BattleBoxVisualStyle::from_view_box(view_box);
            commands.entity(entity).insert((
                BattleBox,
                BattleBoxId("main".to_string()),
                BattleBoxState::default(),
                style,
            ));
            info!("[UI Box] Added BattleBox marker via tag handler");
        }
    }
}

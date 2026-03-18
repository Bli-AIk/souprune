//! # collision.rs
//!
//! Battle collision systems for player movement within battle box boundaries.
//! Supports multiple simultaneous battle boxes with ID-based player binding.
//!
//! Battle 碰撞系统，用于限制玩家在战斗框内移动。
//! 支持多个同时存在的战斗框，通过 ID 绑定玩家。

use crate::app_state::battle::{BattleMovementSet, BattleUpdate};
use crate::core::collision::{BattleBoxBoundary, PhysicsCollider};
use crate::core::mod_system::BehaviorParams;
use crate::core::view::components::ViewBox;
use crate::core::view::sdf_view_shape::spawn_view_box_sdf_children;
use bevy::ecs::message::{Message, MessageReader};
use bevy::prelude::*;
use bevy_alight_motion::sdf_material::SdfMaterial;
use serde::{Deserialize, Serialize};

/// Plugin for battle collision systems
///
/// Battle 碰撞系统插件
pub(crate) struct BattleCollisionPlugin;

impl Plugin for BattleCollisionPlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        app.add_message::<SplitBattleBox>()
            .add_message::<MergeBattleBoxes>()
            .add_systems(
                schedule,
                (
                    handle_split_battle_box_system,
                    handle_merge_battle_boxes_system,
                    constrain_player_to_battle_box_system,
                )
                    .chain()
                    .after(BattleMovementSet)
                    .in_set(BattleUpdate),
            );
    }
}

// ─── Components ─────────────────────────────────────────────────────

/// Marker component for the battle box boundary
///
/// 战斗框边界的标记组件
#[derive(Component)]
pub struct BattleBox;

/// Unique identifier for a battle box.
/// Used to bind players to specific boxes and for split/merge operations.
///
/// 战斗框的唯一标识符。
/// 用于将玩家绑定到特定的框以及分裂/合并操作。
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BattleBoxId(pub String);

/// Binds a player to a specific battle box by ID.
///
/// 通过 ID 将玩家绑定到特定的战斗框。
#[derive(Component, Debug, Clone)]
pub struct BoundToBattleBox(pub String);

/// Runtime state of a battle box.
///
/// 战斗框的运行时状态。
#[derive(Component, Debug, Clone)]
pub struct BattleBoxState {
    /// Whether the box is active (participates in collision and rendering)
    pub active: bool,
    /// Whether collision constraint is enabled (can render but not constrain)
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
/// 战斗框 SDF 渲染的运行时视觉样式。
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

    fn to_view_box(&self, width: f32, height: f32) -> ViewBox {
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
/// Used when the battle box doesn't use ViewBox (e.g., AM animations).
///
/// 存储 AM 动画战斗框尺寸的组件。
/// 用于不使用 ViewBox 的战斗框（如 AM 动画）。
#[derive(Component, Debug, Clone)]
pub struct AlightMotionBattleBoxBounds {
    pub width: f32,
    pub height: f32,
    /// Offset from entity position to the geometric center of the battle box (Bevy coords).
    /// This compensates for non-centered pivot points in AM animations.
    ///
    /// 从实体位置到战斗框几何中心的偏移（Bevy 坐标）。
    /// 用于补偿 AM 动画中非居中的锚点。
    pub center_offset: Vec2,
}

// ─── Split / Merge Events ───────────────────────────────────────────

/// Axis along which to split a battle box.
///
/// 分裂战斗框所沿的轴。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SplitAxis {
    /// Vertical cut → produces left and right boxes
    Vertical,
    /// Horizontal cut → produces top and bottom boxes
    Horizontal,
}

/// Event to trigger a battle box split.
///
/// 触发战斗框分裂的事件。
#[derive(Message)]
pub struct SplitBattleBox {
    /// ID of the box being split
    pub source_box: String,
    /// IDs for the two resulting boxes
    pub result_boxes: (String, String),
    /// Axis along which to split
    pub split_axis: SplitAxis,
    /// Split position relative to box center (0.0 = exact center)
    pub split_position: f32,
    /// Gap between the two new boxes (pixels)
    pub gap: f32,
    /// Animation duration in seconds (0 = instant)
    pub duration: f32,
}

/// Event to trigger merging two battle boxes back into one.
///
/// 触发将两个战斗框合并回一个的事件。
#[derive(Message)]
pub struct MergeBattleBoxes {
    /// IDs of the two boxes to merge
    pub source_boxes: (String, String),
    /// ID of the resulting merged box
    pub result_box: String,
    /// Animation duration in seconds (0 = instant)
    pub duration: f32,
}

// ─── Split / Merge Algorithms ───────────────────────────────────────

/// Split a rectangular boundary along an axis.
fn split_rect_box(
    original: &BattleBoxBoundary,
    axis: &SplitAxis,
    split_pos: f32,
    gap: f32,
) -> (BattleBoxBoundary, BattleBoxBoundary) {
    match axis {
        SplitAxis::Vertical => {
            let left_width = original.half_size.x + split_pos;
            let right_width = original.half_size.x - split_pos;
            let half_gap = gap / 2.0;

            let left = BattleBoxBoundary {
                half_size: Vec2::new(left_width / 2.0, original.half_size.y),
                center: Vec2::new(
                    original.center.x - original.half_size.x + left_width / 2.0 - half_gap,
                    original.center.y,
                ),
            };
            let right = BattleBoxBoundary {
                half_size: Vec2::new(right_width / 2.0, original.half_size.y),
                center: Vec2::new(
                    original.center.x + original.half_size.x - right_width / 2.0 + half_gap,
                    original.center.y,
                ),
            };
            (left, right)
        }
        SplitAxis::Horizontal => {
            let top_height = original.half_size.y + split_pos;
            let bottom_height = original.half_size.y - split_pos;
            let half_gap = gap / 2.0;

            let top = BattleBoxBoundary {
                half_size: Vec2::new(original.half_size.x, top_height / 2.0),
                center: Vec2::new(
                    original.center.x,
                    original.center.y + original.half_size.y - top_height / 2.0 + half_gap,
                ),
            };
            let bottom = BattleBoxBoundary {
                half_size: Vec2::new(original.half_size.x, bottom_height / 2.0),
                center: Vec2::new(
                    original.center.x,
                    original.center.y - original.half_size.y + bottom_height / 2.0 - half_gap,
                ),
            };
            (top, bottom)
        }
    }
}

/// Determine which of two boxes a player should be rebound to based on distance.
fn nearest_box_id(
    player_pos: Vec2,
    box_a: &BattleBoxBoundary,
    box_b: &BattleBoxBoundary,
    id_a: &str,
    id_b: &str,
) -> String {
    let dist_a = (player_pos - box_a.center).length();
    let dist_b = (player_pos - box_b.center).length();
    if dist_a <= dist_b {
        id_a.to_string()
    } else {
        id_b.to_string()
    }
}

// ─── Helper: resolve boundary from box entity ───────────────────────

/// Resolve the `BattleBoxBoundary` from a battle box entity.
/// Returns `None` if the box is inactive or collision is disabled.
fn resolve_boundary(
    transform: &GlobalTransform,
    ui_box: Option<&ViewBox>,
    am_bounds: Option<&AlightMotionBattleBoxBounds>,
    state: &BattleBoxState,
) -> Option<BattleBoxBoundary> {
    if !state.active || !state.collision_enabled {
        return None;
    }
    if let Some(vb) = ui_box {
        Some(BattleBoxBoundary::from_ui_box(
            vb.width(),
            vb.height(),
            transform.translation().truncate(),
        ))
    } else if let Some(am) = am_bounds {
        let center = transform.translation().truncate() + am.center_offset;
        Some(BattleBoxBoundary::from_ui_box(am.width, am.height, center))
    } else {
        None
    }
}

fn resolve_visual_style(
    ui_box: Option<&ViewBox>,
    style: Option<&BattleBoxVisualStyle>,
) -> BattleBoxVisualStyle {
    style
        .cloned()
        .or_else(|| ui_box.map(BattleBoxVisualStyle::from_view_box))
        .unwrap_or_default()
}

// ─── Systems ────────────────────────────────────────────────────────

/// System to constrain player position within their bound battle box.
///
/// 限制玩家位置在其绑定的战斗框边界内。
pub(crate) fn constrain_player_to_battle_box_system(
    mut player_query: Query<
        (&mut Transform, &PhysicsCollider, &BoundToBattleBox),
        (With<BehaviorParams>, Without<ViewBox>),
    >,
    ui_boxes: Query<
        (&GlobalTransform, &ViewBox, &BattleBoxId, &BattleBoxState),
        (With<BattleBox>, Without<PhysicsCollider>),
    >,
    am_boxes: Query<
        (
            &GlobalTransform,
            &AlightMotionBattleBoxBounds,
            &BattleBoxId,
            &BattleBoxState,
        ),
        (With<BattleBox>, Without<ViewBox>, Without<PhysicsCollider>),
    >,
) {
    for (mut player_tf, collider, bound) in player_query.iter_mut() {
        let target_id = &bound.0;

        let boundary = if let Some((tf, vb, _, state)) =
            ui_boxes.iter().find(|(_, _, id, _)| id.0 == *target_id)
        {
            resolve_boundary(tf, Some(vb), None, state)
        } else if let Some((tf, am, _, state)) =
            am_boxes.iter().find(|(_, _, id, _)| id.0 == *target_id)
        {
            resolve_boundary(tf, None, Some(am), state)
        } else {
            None
        };

        let Some(boundary) = boundary else { continue };

        let current_pos = player_tf.translation.truncate();
        let constrained = boundary.constrain_with_collider(current_pos, collider);
        player_tf.translation.x = constrained.x;
        player_tf.translation.y = constrained.y;
    }
}

/// Handle `SplitBattleBox` events: deactivate source, spawn two new boxes.
fn handle_split_battle_box_system(
    mut commands: Commands,
    mut events: MessageReader<SplitBattleBox>,
    mut player_query: Query<(&Transform, &mut BoundToBattleBox), With<BehaviorParams>>,
    mut ui_boxes: Query<
        (
            &GlobalTransform,
            &ViewBox,
            &BattleBoxId,
            &mut BattleBoxState,
            Option<&BattleBoxVisualStyle>,
            Option<&mut Visibility>,
        ),
        (With<BattleBox>, Without<PhysicsCollider>),
    >,
    mut am_boxes: Query<
        (
            &GlobalTransform,
            &AlightMotionBattleBoxBounds,
            &BattleBoxId,
            &mut BattleBoxState,
            Option<&BattleBoxVisualStyle>,
            Option<&mut Visibility>,
        ),
        (With<BattleBox>, Without<ViewBox>, Without<PhysicsCollider>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sdf_materials: ResMut<Assets<SdfMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    for ev in events.read() {
        // Find and deactivate source box, get its boundary and visual style.
        let source_data = 'find: {
            if let Some((tf, vb, _, mut state, style, visibility)) = ui_boxes
                .iter_mut()
                .find(|(_, _, id, _, _, _)| id.0 == ev.source_box)
            {
                let b = resolve_boundary(tf, Some(vb), None, &state);
                let style = resolve_visual_style(Some(vb), style);
                state.active = false;
                if let Some(mut visibility) = visibility {
                    *visibility = Visibility::Hidden;
                }
                break 'find b.map(|boundary| (boundary, style));
            }
            if let Some((tf, am, _, mut state, style, visibility)) = am_boxes
                .iter_mut()
                .find(|(_, _, id, _, _, _)| id.0 == ev.source_box)
            {
                let b = resolve_boundary(tf, None, Some(am), &state);
                let style = resolve_visual_style(None, style);
                state.active = false;
                if let Some(mut visibility) = visibility {
                    *visibility = Visibility::Hidden;
                }
                break 'find b.map(|boundary| (boundary, style));
            }
            warn!("SplitBattleBox: source box '{}' not found", ev.source_box);
            None
        };

        let Some((original, style)) = source_data else {
            continue;
        };

        let (box_a, box_b) = split_rect_box(&original, &ev.split_axis, ev.split_position, ev.gap);

        let (id_a, id_b) = (&ev.result_boxes.0, &ev.result_boxes.1);

        // Spawn two new battle box entities with their own SDF visuals.
        spawn_standalone_box(
            &mut commands,
            &mut meshes,
            &mut sdf_materials,
            &mut color_materials,
            id_a,
            &box_a,
            &style,
        );
        spawn_standalone_box(
            &mut commands,
            &mut meshes,
            &mut sdf_materials,
            &mut color_materials,
            id_b,
            &box_b,
            &style,
        );

        // Rebind players that were bound to the source box
        for (player_tf, mut bound) in player_query.iter_mut() {
            if bound.0 == ev.source_box {
                let pos = player_tf.translation.truncate();
                bound.0 = nearest_box_id(pos, &box_a, &box_b, id_a, id_b);
            }
        }

        info!(
            "Split '{}' → '{}' + '{}' (axis={:?}, pos={}, gap={}, duration={})",
            ev.source_box, id_a, id_b, ev.split_axis, ev.split_position, ev.gap, ev.duration
        );
    }
}

/// Handle `MergeBattleBoxes` events: deactivate two sources, spawn merged box.
fn handle_merge_battle_boxes_system(
    mut commands: Commands,
    mut events: MessageReader<MergeBattleBoxes>,
    mut player_query: Query<&mut BoundToBattleBox, With<BehaviorParams>>,
    mut ui_boxes: Query<
        (
            &GlobalTransform,
            &ViewBox,
            &BattleBoxId,
            &mut BattleBoxState,
            Option<&BattleBoxVisualStyle>,
            Option<&mut Visibility>,
        ),
        (With<BattleBox>, Without<PhysicsCollider>),
    >,
    mut am_boxes: Query<
        (
            &GlobalTransform,
            &AlightMotionBattleBoxBounds,
            &BattleBoxId,
            &mut BattleBoxState,
            Option<&BattleBoxVisualStyle>,
            Option<&mut Visibility>,
        ),
        (With<BattleBox>, Without<ViewBox>, Without<PhysicsCollider>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sdf_materials: ResMut<Assets<SdfMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    for ev in events.read() {
        let ids = [&ev.source_boxes.0, &ev.source_boxes.1];
        let mut boundaries: Vec<BattleBoxBoundary> = Vec::with_capacity(2);
        let mut visual_style: Option<BattleBoxVisualStyle> = None;

        // Deactivate sources and collect boundaries
        for target_id in &ids {
            let found = 'find: {
                if let Some((tf, vb, _, mut state, style, visibility)) = ui_boxes
                    .iter_mut()
                    .find(|(_, _, id, _, _, _)| &id.0 == *target_id)
                {
                    let b = resolve_boundary(tf, Some(vb), None, &state);
                    visual_style.get_or_insert_with(|| resolve_visual_style(Some(vb), style));
                    state.active = false;
                    if let Some(mut visibility) = visibility {
                        *visibility = Visibility::Hidden;
                    }
                    break 'find b;
                }
                if let Some((tf, am, _, mut state, style, visibility)) = am_boxes
                    .iter_mut()
                    .find(|(_, _, id, _, _, _)| &id.0 == *target_id)
                {
                    let b = resolve_boundary(tf, None, Some(am), &state);
                    visual_style.get_or_insert_with(|| resolve_visual_style(None, style));
                    state.active = false;
                    if let Some(mut visibility) = visibility {
                        *visibility = Visibility::Hidden;
                    }
                    break 'find b;
                }
                warn!("MergeBattleBoxes: source box '{}' not found", target_id);
                None
            };
            if let Some(b) = found {
                boundaries.push(b);
            }
        }

        if boundaries.len() < 2 {
            warn!(
                "MergeBattleBoxes: need 2 valid source boxes, found {}",
                boundaries.len()
            );
            continue;
        }

        // Compute merged AABB from two boundaries
        let merged = merge_boundaries(&boundaries[0], &boundaries[1]);
        let default_visual_style = BattleBoxVisualStyle::default();
        spawn_standalone_box(
            &mut commands,
            &mut meshes,
            &mut sdf_materials,
            &mut color_materials,
            &ev.result_box,
            &merged,
            visual_style.as_ref().unwrap_or(&default_visual_style),
        );

        // Rebind all players from either source to the merged box
        for mut bound in player_query.iter_mut() {
            if bound.0 == ev.source_boxes.0 || bound.0 == ev.source_boxes.1 {
                bound.0 = ev.result_box.clone();
            }
        }

        info!(
            "Merged '{}' + '{}' → '{}' (duration={})",
            ev.source_boxes.0, ev.source_boxes.1, ev.result_box, ev.duration
        );
    }
}

// ─── Standalone Box Helpers ─────────────────────────────────────────

/// Spawn a standalone battle box entity with its own SDF visual.
fn spawn_standalone_box(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    sdf_materials: &mut ResMut<Assets<SdfMaterial>>,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
    id: &str,
    boundary: &BattleBoxBoundary,
    visual_style: &BattleBoxVisualStyle,
) {
    let view_box = visual_style.to_view_box(boundary.half_size.x * 2.0, boundary.half_size.y * 2.0);
    let entity = commands
        .spawn((
            BattleBox,
            BattleBoxId(id.to_string()),
            BattleBoxState::default(),
            visual_style.clone(),
            AlightMotionBattleBoxBounds {
                width: boundary.half_size.x * 2.0,
                height: boundary.half_size.y * 2.0,
                center_offset: Vec2::ZERO,
            },
            Transform::from_translation(boundary.center.extend(0.0)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Name::new(format!("BattleBox:{id}")),
        ))
        .id();

    spawn_view_box_sdf_children(
        commands,
        entity,
        &view_box,
        meshes,
        sdf_materials,
        color_materials,
    );
}

/// Merge two boundaries into one AABB that encloses both.
fn merge_boundaries(a: &BattleBoxBoundary, b: &BattleBoxBoundary) -> BattleBoxBoundary {
    let min = Vec2::new(
        (a.center.x - a.half_size.x).min(b.center.x - b.half_size.x),
        (a.center.y - a.half_size.y).min(b.center.y - b.half_size.y),
    );
    let max = Vec2::new(
        (a.center.x + a.half_size.x).max(b.center.x + b.half_size.x),
        (a.center.y + a.half_size.y).max(b.center.y + b.half_size.y),
    );
    let center = (min + max) / 2.0;
    let half_size = (max - min) / 2.0;
    BattleBoxBoundary { half_size, center }
}

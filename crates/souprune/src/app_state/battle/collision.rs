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

type UiBattleBoxReadQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GlobalTransform,
        &'static ViewBox,
        &'static BattleBoxId,
        &'static BattleBoxState,
        Option<&'static BattleBoxVisualStyle>,
    ),
    (With<BattleBox>, Without<PhysicsCollider>),
>;

type AmBattleBoxReadQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GlobalTransform,
        &'static AlightMotionBattleBoxBounds,
        &'static BattleBoxId,
        &'static BattleBoxState,
        Option<&'static BattleBoxVisualStyle>,
    ),
    (With<BattleBox>, Without<ViewBox>, Without<PhysicsCollider>),
>;

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
                    animate_battle_box_split_system,
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

/// Tracks an ongoing split animation for a pair of battle boxes.
/// 跟踪一对战斗框正在进行的分裂动画。
#[derive(Component)]
pub struct BattleBoxSplitAnimation {
    /// Starting boundary for box A (at animation start).
    /// box A 的起始边界（动画开始时）。
    pub start_boundary_a: BattleBoxBoundary,
    /// Starting boundary for box B (at animation start).
    /// box B 的起始边界（动画开始时）。
    pub start_boundary_b: BattleBoxBoundary,
    /// Target boundary for box A (at animation end).
    /// box A 的目标边界（动画结束时）。
    pub target_boundary_a: BattleBoxBoundary,
    /// Target boundary for box B (at animation end).
    /// box B 的目标边界（动画结束时）。
    pub target_boundary_b: BattleBoxBoundary,
    /// Entity ID for box A.
    /// box A 的实体 ID。
    pub box_entity_a: Entity,
    /// Entity ID for box B.
    /// box B 的实体 ID。
    pub box_entity_b: Entity,
    /// Visual style to apply during animation.
    /// 动画期间应用的视觉样式。
    pub visual_style: BattleBoxVisualStyle,
    /// Animation progress (0.0 to 1.0).
    /// 动画进度（0.0 到 1.0）。
    pub progress: f32,
    /// Total animation duration in seconds.
    /// 总动画时长（秒）。
    pub duration: f32,
}

// ─── Split / Merge Events ───────────────────────────────────────────

/// Axis along which to split a battle box.
///
/// 分裂战斗框所沿的轴。
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq)]
pub enum SplitAxis {
    /// Vertical cut → produces left and right boxes
    Vertical,
    /// Horizontal cut → produces top and bottom boxes
    #[default]
    Horizontal,
}

/// Policy for how gap affects split box dimensions.
/// 间隙如何影响分裂后 box 尺寸的策略。
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq)]
pub enum GapPolicy {
    /// Gap expands outward: two box widths sum to original width, total span increases.
    /// 间隙向外扩展：两个 box 宽度之和等于原宽度，总跨度增加。
    #[default]
    Expands,
    /// Gap includes in width: two box widths + gap = original width, total span preserved.
    /// 间隙计入宽度：两个 box 宽度 + gap = 原宽度，总跨度保持不变。
    Includes,
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
    /// Policy for how gap affects dimensions
    pub gap_policy: GapPolicy,
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

#[derive(Debug, Clone, Copy)]
enum BattleBoxSourceKind {
    Ui,
    Am,
}

impl BattleBoxSourceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ui => "ui",
            Self::Am => "am",
        }
    }
}

#[derive(Debug, Clone)]
struct BattleBoxCandidate {
    entity: Entity,
    id: String,
    kind: BattleBoxSourceKind,
    active: bool,
    collision_enabled: bool,
    boundary: Option<BattleBoxBoundary>,
    visual_style: BattleBoxVisualStyle,
}

impl BattleBoxCandidate {
    fn is_live(&self) -> bool {
        self.active && self.collision_enabled && self.boundary.is_some()
    }

    fn summary(&self) -> String {
        format!(
            "{}:{}@{:?}(active={}, collision={}, boundary={})",
            self.kind.as_str(),
            self.id,
            self.entity,
            self.active,
            self.collision_enabled,
            self.boundary.is_some()
        )
    }
}

// ─── Split / Merge Algorithms ───────────────────────────────────────

/// Split a rectangular boundary along an axis.
fn split_rect_box(
    original: &BattleBoxBoundary,
    axis: &SplitAxis,
    split_pos: f32,
    gap: f32,
    gap_policy: GapPolicy,
) -> (BattleBoxBoundary, BattleBoxBoundary) {
    match axis {
        SplitAxis::Vertical => {
            let left_width = original.half_size.x + split_pos;
            let right_width = original.half_size.x - split_pos;
            let half_gap = gap / 2.0;

            match gap_policy {
                GapPolicy::Expands => {
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
                GapPolicy::Includes => {
                    let total_width = original.half_size.x * 2.0;
                    let scale = (total_width - gap) / total_width;
                    let left_width_scaled = left_width * scale;
                    let right_width_scaled = right_width * scale;

                    let left = BattleBoxBoundary {
                        half_size: Vec2::new(left_width_scaled / 2.0, original.half_size.y),
                        center: Vec2::new(
                            original.center.x - half_gap - left_width_scaled / 2.0,
                            original.center.y,
                        ),
                    };
                    let right = BattleBoxBoundary {
                        half_size: Vec2::new(right_width_scaled / 2.0, original.half_size.y),
                        center: Vec2::new(
                            original.center.x + half_gap + right_width_scaled / 2.0,
                            original.center.y,
                        ),
                    };
                    (left, right)
                }
            }
        }
        SplitAxis::Horizontal => {
            let top_height = original.half_size.y + split_pos;
            let bottom_height = original.half_size.y - split_pos;
            let half_gap = gap / 2.0;

            match gap_policy {
                GapPolicy::Expands => {
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
                            original.center.y - original.half_size.y + bottom_height / 2.0
                                - half_gap,
                        ),
                    };
                    (top, bottom)
                }
                GapPolicy::Includes => {
                    let total_height = original.half_size.y * 2.0;
                    let scale = (total_height - gap) / total_height;
                    let top_height_scaled = top_height * scale;
                    let bottom_height_scaled = bottom_height * scale;

                    let top = BattleBoxBoundary {
                        half_size: Vec2::new(original.half_size.x, top_height_scaled / 2.0),
                        center: Vec2::new(
                            original.center.x,
                            original.center.y + half_gap + top_height_scaled / 2.0,
                        ),
                    };
                    let bottom = BattleBoxBoundary {
                        half_size: Vec2::new(original.half_size.x, bottom_height_scaled / 2.0),
                        center: Vec2::new(
                            original.center.x,
                            original.center.y - half_gap - bottom_height_scaled / 2.0,
                        ),
                    };
                    (top, bottom)
                }
            }
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

fn collect_battle_box_candidates(
    target_id: &str,
    ui_boxes: &UiBattleBoxReadQuery,
    am_boxes: &AmBattleBoxReadQuery,
) -> Vec<BattleBoxCandidate> {
    let mut candidates = Vec::new();

    for (entity, tf, vb, box_id, state, style) in ui_boxes.iter() {
        if box_id.0 != target_id {
            continue;
        }

        candidates.push(BattleBoxCandidate {
            entity,
            id: box_id.0.clone(),
            kind: BattleBoxSourceKind::Ui,
            active: state.active,
            collision_enabled: state.collision_enabled,
            boundary: resolve_boundary(tf, Some(vb), None, state),
            visual_style: resolve_visual_style(Some(vb), style),
        });
    }

    for (entity, tf, am, box_id, state, style) in am_boxes.iter() {
        if box_id.0 != target_id {
            continue;
        }

        candidates.push(BattleBoxCandidate {
            entity,
            id: box_id.0.clone(),
            kind: BattleBoxSourceKind::Am,
            active: state.active,
            collision_enabled: state.collision_enabled,
            boundary: resolve_boundary(tf, None, Some(am), state),
            visual_style: resolve_visual_style(None, style),
        });
    }

    candidates
}

fn describe_battle_box_candidates(candidates: &[BattleBoxCandidate]) -> String {
    candidates
        .iter()
        .map(BattleBoxCandidate::summary)
        .collect::<Vec<_>>()
        .join(", ")
}

fn resolve_live_battle_box(
    op_name: &str,
    target_id: &str,
    ui_boxes: &UiBattleBoxReadQuery,
    am_boxes: &AmBattleBoxReadQuery,
) -> Option<BattleBoxCandidate> {
    let candidates = collect_battle_box_candidates(target_id, ui_boxes, am_boxes);
    if candidates.is_empty() {
        warn!("{op_name}: source box '{target_id}' not found");
        return None;
    }

    let live = candidates
        .iter()
        .filter(|candidate| candidate.is_live())
        .cloned()
        .collect::<Vec<_>>();

    match live.len() {
        1 => live.into_iter().next(),
        0 => {
            warn!(
                "{op_name}: source box '{target_id}' exists but has no usable live match: {}",
                describe_battle_box_candidates(&candidates)
            );
            None
        }
        count => {
            warn!(
                "{op_name}: source box '{target_id}' is ambiguous; found {count} live matches: {}",
                describe_battle_box_candidates(&live)
            );
            None
        }
    }
}

fn retire_battle_box(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .insert(Visibility::Hidden)
        .remove::<(BattleBox, BattleBoxId, BattleBoxState, BattleBoxVisualStyle)>();
}

fn retire_existing_battle_boxes_with_id(
    commands: &mut Commands,
    op_name: &str,
    target_id: &str,
    keep_entities: &[Entity],
    ui_boxes: &UiBattleBoxReadQuery,
    am_boxes: &AmBattleBoxReadQuery,
) {
    for candidate in collect_battle_box_candidates(target_id, ui_boxes, am_boxes) {
        if keep_entities.contains(&candidate.entity) {
            continue;
        }

        warn!(
            "{op_name}: retiring pre-existing box for result id '{target_id}': {}",
            candidate.summary()
        );
        retire_battle_box(commands, candidate.entity);
    }
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
    ui_boxes: UiBattleBoxReadQuery,
    am_boxes: AmBattleBoxReadQuery,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sdf_materials: ResMut<Assets<SdfMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    for ev in events.read() {
        let Some(source_box) =
            resolve_live_battle_box("SplitBattleBox", &ev.source_box, &ui_boxes, &am_boxes)
        else {
            continue;
        };
        let original = source_box
            .boundary
            .clone()
            .expect("live battle box candidate must have a boundary");
        let style = source_box.visual_style.clone();

        retire_existing_battle_boxes_with_id(
            &mut commands,
            "SplitBattleBox",
            &ev.result_boxes.0,
            &[],
            &ui_boxes,
            &am_boxes,
        );
        retire_existing_battle_boxes_with_id(
            &mut commands,
            "SplitBattleBox",
            &ev.result_boxes.1,
            &[],
            &ui_boxes,
            &am_boxes,
        );
        retire_battle_box(&mut commands, source_box.entity);

        let (box_a, box_b) = split_rect_box(
            &original,
            &ev.split_axis,
            ev.split_position,
            ev.gap,
            ev.gap_policy,
        );

        let (id_a, id_b) = (&ev.result_boxes.0, &ev.result_boxes.1);

        if ev.duration > 0.0 {
            // Animated split: spawn boxes at original center, then animate outward
            let start_boundary = BattleBoxBoundary {
                half_size: Vec2::new(original.half_size.x / 2.0, original.half_size.y / 2.0),
                center: original.center,
            };

            let entity_a =
                spawn_standalone_box_entity(&mut commands, id_a, &start_boundary, &style);
            let entity_b =
                spawn_standalone_box_entity(&mut commands, id_b, &start_boundary, &style);

            spawn_view_box_sdf_children(
                &mut commands,
                entity_a,
                &style.to_view_box(
                    start_boundary.half_size.x * 2.0,
                    start_boundary.half_size.y * 2.0,
                ),
                &mut meshes,
                &mut sdf_materials,
                &mut color_materials,
            );
            spawn_view_box_sdf_children(
                &mut commands,
                entity_b,
                &style.to_view_box(
                    start_boundary.half_size.x * 2.0,
                    start_boundary.half_size.y * 2.0,
                ),
                &mut meshes,
                &mut sdf_materials,
                &mut color_materials,
            );

            commands.spawn(BattleBoxSplitAnimation {
                start_boundary_a: start_boundary.clone(),
                start_boundary_b: start_boundary,
                target_boundary_a: box_a.clone(),
                target_boundary_b: box_b.clone(),
                box_entity_a: entity_a,
                box_entity_b: entity_b,
                visual_style: style,
                progress: 0.0,
                duration: ev.duration,
            });
        } else {
            // Instant split: spawn boxes directly at target positions
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
        }

        // Rebind players that were bound to the source box
        for (player_tf, mut bound) in player_query.iter_mut() {
            if bound.0 == ev.source_box {
                let pos = player_tf.translation.truncate();
                bound.0 = nearest_box_id(pos, &box_a, &box_b, id_a, id_b);
            }
        }

        info!(
            "Split '{}' → '{}' + '{}' (axis={:?}, pos={}, gap={}, gap_policy={:?}, duration={})",
            ev.source_box,
            id_a,
            id_b,
            ev.split_axis,
            ev.split_position,
            ev.gap,
            ev.gap_policy,
            ev.duration
        );
    }
}

/// Handle `MergeBattleBoxes` events: deactivate two sources, spawn merged box.
fn handle_merge_battle_boxes_system(
    mut commands: Commands,
    mut events: MessageReader<MergeBattleBoxes>,
    mut player_query: Query<&mut BoundToBattleBox, With<BehaviorParams>>,
    ui_boxes: UiBattleBoxReadQuery,
    am_boxes: AmBattleBoxReadQuery,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sdf_materials: ResMut<Assets<SdfMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    for ev in events.read() {
        if ev.source_boxes.0 == ev.source_boxes.1 {
            warn!(
                "MergeBattleBoxes: duplicate source id '{}' is not supported",
                ev.source_boxes.0
            );
            continue;
        }

        let ids = [&ev.source_boxes.0, &ev.source_boxes.1];
        let mut boundaries: Vec<BattleBoxBoundary> = Vec::with_capacity(2);
        let mut visual_style: Option<BattleBoxVisualStyle> = None;
        let mut source_entities: Vec<Entity> = Vec::with_capacity(2);

        // Resolve sources and collect boundaries
        for target_id in &ids {
            let Some(source_box) =
                resolve_live_battle_box("MergeBattleBoxes", target_id, &ui_boxes, &am_boxes)
            else {
                continue;
            };
            boundaries.push(
                source_box
                    .boundary
                    .clone()
                    .expect("live battle box candidate must have a boundary"),
            );
            visual_style.get_or_insert(source_box.visual_style.clone());
            source_entities.push(source_box.entity);
        }

        if boundaries.len() < 2 {
            warn!(
                "MergeBattleBoxes: need 2 valid source boxes, found {}",
                boundaries.len()
            );
            continue;
        }

        for entity in &source_entities {
            retire_battle_box(&mut commands, *entity);
        }
        retire_existing_battle_boxes_with_id(
            &mut commands,
            "MergeBattleBoxes",
            &ev.result_box,
            &source_entities,
            &ui_boxes,
            &am_boxes,
        );

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

/// Spawn a standalone battle box entity without SDF children.
/// Used for animation where visual is added separately.
fn spawn_standalone_box_entity(
    commands: &mut Commands,
    id: &str,
    boundary: &BattleBoxBoundary,
    visual_style: &BattleBoxVisualStyle,
) -> Entity {
    commands
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
        .id()
}

// ─── Animation Systems ────────────────────────────────────────────────

/// System to animate battle box split animations.
/// 战斗框分裂动画系统。
fn animate_battle_box_split_system(
    mut commands: Commands,
    time: Res<Time>,
    mut animations: Query<(Entity, &mut BattleBoxSplitAnimation)>,
    mut box_query: Query<(&mut Transform, &mut AlightMotionBattleBoxBounds)>,
    child_query: Query<&Children>,
    mut shape_query: Query<&mut crate::core::view::sdf_shape::ViewSdfShape>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_query: Query<&mut Mesh2d>,
) {
    for (anim_entity, mut anim) in animations.iter_mut() {
        anim.progress += time.delta_secs() / anim.duration;

        if anim.progress >= 1.0 {
            // Animation complete - set final positions
            if let Ok((mut transform, mut bounds)) = box_query.get_mut(anim.box_entity_a) {
                transform.translation = anim.target_boundary_a.center.extend(0.0);
                bounds.width = anim.target_boundary_a.half_size.x * 2.0;
                bounds.height = anim.target_boundary_a.half_size.y * 2.0;
            }
            if let Ok((mut transform, mut bounds)) = box_query.get_mut(anim.box_entity_b) {
                transform.translation = anim.target_boundary_b.center.extend(0.0);
                bounds.width = anim.target_boundary_b.half_size.x * 2.0;
                bounds.height = anim.target_boundary_b.half_size.y * 2.0;
            }
            commands.entity(anim_entity).despawn();
            continue;
        }

        // Easing: smooth step (ease in-out)
        let t = anim.progress * anim.progress * (3.0 - 2.0 * anim.progress);

        // Interpolate boundaries
        let current_a = interpolate_boundary(&anim.start_boundary_a, &anim.target_boundary_a, t);
        let current_b = interpolate_boundary(&anim.start_boundary_b, &anim.target_boundary_b, t);

        // Update box A
        if let Ok((mut transform, mut bounds)) = box_query.get_mut(anim.box_entity_a) {
            transform.translation = current_a.center.extend(0.0);
            bounds.width = current_a.half_size.x * 2.0;
            bounds.height = current_a.half_size.y * 2.0;

            update_sdf_visual(
                &anim.box_entity_a,
                &current_a,
                &anim.visual_style,
                &child_query,
                &mut shape_query,
                &mut meshes,
                &mut mesh_query,
            );
        }

        // Update box B
        if let Ok((mut transform, mut bounds)) = box_query.get_mut(anim.box_entity_b) {
            transform.translation = current_b.center.extend(0.0);
            bounds.width = current_b.half_size.x * 2.0;
            bounds.height = current_b.half_size.y * 2.0;

            update_sdf_visual(
                &anim.box_entity_b,
                &current_b,
                &anim.visual_style,
                &child_query,
                &mut shape_query,
                &mut meshes,
                &mut mesh_query,
            );
        }
    }
}

/// Linear interpolation between two boundaries.
fn interpolate_boundary(
    start: &BattleBoxBoundary,
    end: &BattleBoxBoundary,
    t: f32,
) -> BattleBoxBoundary {
    BattleBoxBoundary {
        half_size: start.half_size.lerp(end.half_size, t),
        center: start.center.lerp(end.center, t),
    }
}

/// Update SDF visual for a battle box entity during animation.
fn update_sdf_visual(
    box_entity: &Entity,
    boundary: &BattleBoxBoundary,
    visual_style: &BattleBoxVisualStyle,
    child_query: &Query<&Children>,
    shape_query: &mut Query<&mut crate::core::view::sdf_shape::ViewSdfShape>,
    meshes: &mut ResMut<Assets<Mesh>>,
    mesh_query: &mut Query<&mut Mesh2d>,
) {
    let Ok(children) = child_query.get(*box_entity) else {
        return;
    };
    for child in children.iter() {
        if let Ok(mut shape) = shape_query.get_mut(child) {
            shape.half_width = boundary.half_size.x;
            shape.half_height = boundary.half_size.y;
            shape.color = visual_style.fill_color;
            let new_mesh = shape.create_mesh();
            if let Ok(mut mesh_handle) = mesh_query.get_mut(child) {
                mesh_handle.0 = meshes.add(new_mesh);
            }
        }
    }
}

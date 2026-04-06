//! # collision.rs
//!
//! Battle collision systems for player movement within battle box boundaries.
//! Supports multiple simultaneous battle boxes with ID-based player binding.
//!
//! Battle 碰撞系统，用于限制玩家在战斗框内移动。
//! 支持多个同时存在的战斗框，通过 ID 绑定玩家。

use crate::preset::battle_box::{
    AlightMotionBattleBoxBounds, BattleBox, BattleBoxId, BattleBoxState, BattleBoxVisualStyle,
    BoundToBattleBox, GapPolicy, MergeBattleBoxes, SplitAxis, SplitBattleBox,
};
use crate::preset::battle_runtime::{BattleMovementSet, BattleUpdate};
use crate::core::collision::{BattleBoxBoundary, PhysicsCollider};
use crate::core::mod_system::BehaviorParams;
use crate::core::view::components::ViewBox;
use crate::core::view::sdf_view_shape::spawn_view_box_sdf_children;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy_alight_motion::sdf_material::SdfMaterial;
use bevy_tween::interpolation::EaseKind;

mod animation;
mod geometry;

use self::animation::{
    animate_battle_box_merge_system, animate_battle_box_split_system, finalize_merged_battle_box,
    spawn_standalone_box, spawn_standalone_box_entity,
};
use self::geometry::{
    choose_box_index_for_player, merge_boundaries, plan_merge_animation, resolve_boundary,
    resolve_live_battle_box, retire_battle_box, retire_existing_battle_boxes_with_id,
    select_box_id_for_player, split_rect_box,
};

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
                    animate_battle_box_merge_system,
                    constrain_player_to_battle_box_system,
                )
                    .chain()
                    .after(BattleMovementSet)
                    .in_set(BattleUpdate),
            );
    }
}

/// Tracks an ongoing split animation for a pair of battle boxes.
/// 跟踪一对战斗框正在进行的分裂动画。
#[derive(Component)]
pub struct BattleBoxSplitAnimation {
    /// Original unsplit source boundary.
    /// 原始未分裂的源边界。
    pub original_boundary: BattleBoxBoundary,
    /// Axis used to split the source box.
    /// 源框分裂所用的轴。
    pub split_axis: SplitAxis,
    /// Split line offset from center.
    /// 相对中心的分裂线偏移。
    pub split_position: f32,
    /// Final target visible gap for the animation.
    /// 动画的目标可见间隙。
    pub target_visible_gap: f32,
    /// Policy that determines how gap affects box size.
    /// 决定 gap 如何影响 box 尺寸的策略。
    pub gap_policy: GapPolicy,
    /// Easing function used to animate the visible gap.
    /// 用于驱动可见间隙动画的缓动函数。
    pub easing: EaseKind,
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

/// Tracks an ongoing merge animation for a pair of battle boxes.
/// 跟踪一对战斗框正在进行的合并动画。
#[derive(Component)]
pub struct BattleBoxMergeAnimation {
    /// Source box IDs that should be rebound on completion.
    /// 动画完成后需要重绑定的源框 ID。
    pub source_boxes: (String, String),
    /// ID of the final merged box.
    /// 最终合并结果框的 ID。
    pub result_box: String,
    /// Entity ID for box A.
    /// box A 的实体 ID。
    pub box_entity_a: Entity,
    /// Entity ID for box B.
    /// box B 的实体 ID。
    pub box_entity_b: Entity,
    /// Starting boundary for box A.
    /// box A 的起始边界。
    pub start_boundary_a: BattleBoxBoundary,
    /// Starting boundary for box B.
    /// box B 的起始边界。
    pub start_boundary_b: BattleBoxBoundary,
    /// Target boundary for box A at the end of the merge.
    /// merge 结束时 box A 的目标边界。
    pub target_boundary_a: BattleBoxBoundary,
    /// Target boundary for box B at the end of the merge.
    /// merge 结束时 box B 的目标边界。
    pub target_boundary_b: BattleBoxBoundary,
    /// Final merged boundary to spawn on completion.
    /// 动画完成后生成的最终合并边界。
    pub merged_boundary: BattleBoxBoundary,
    /// Visual style to apply during animation.
    /// 动画期间应用的视觉样式。
    pub visual_style: BattleBoxVisualStyle,
    /// Easing function used by the merge tween.
    /// merge tween 使用的缓动函数。
    pub easing: EaseKind,
    /// Animation progress (0.0 to 1.0).
    /// 动画进度（0.0 到 1.0）。
    pub progress: f32,
    /// Total animation duration in seconds.
    /// 总动画时长（秒）。
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

#[derive(Debug, Clone)]
struct MergeAnimationPlan {
    axis: SplitAxis,
    ordered_indices: (usize, usize),
    target_boundary_a: BattleBoxBoundary,
    target_boundary_b: BattleBoxBoundary,
    merged_boundary: BattleBoxBoundary,
}

// ─── Systems ────────────────────────────────────────────────────────

/// System to constrain player position within their bound battle box.
///
/// 限制玩家位置在其绑定的战斗框边界内。
pub(crate) fn constrain_player_to_battle_box_system(
    mut player_query: Query<
        (&mut Transform, &PhysicsCollider, &mut BoundToBattleBox),
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
    let mut live_boxes: Vec<(String, BattleBoxBoundary)> = Vec::new();
    for (tf, vb, id, state) in ui_boxes.iter() {
        if let Some(boundary) = resolve_boundary(tf, Some(vb), None, state) {
            live_boxes.push((id.0.clone(), boundary));
        }
    }
    for (tf, am, id, state) in am_boxes.iter() {
        if let Some(boundary) = resolve_boundary(tf, None, Some(am), state) {
            live_boxes.push((id.0.clone(), boundary));
        }
    }

    for (mut player_tf, collider, mut bound) in player_query.iter_mut() {
        let current_pos = player_tf.translation.truncate();
        let Some(selected_index) =
            choose_box_index_for_player(Some(&bound.0), current_pos, collider, &live_boxes)
        else {
            continue;
        };
        let (selected_id, boundary) = &live_boxes[selected_index];
        if bound.0 != *selected_id {
            debug!(
                "Rebinding moving player from battle box '{}' to '{}'",
                bound.0, selected_id
            );
            bound.0 = selected_id.clone();
        }

        let constrained = boundary.constrain_with_collider(current_pos, collider);
        player_tf.translation.x = constrained.x;
        player_tf.translation.y = constrained.y;
    }
}

/// Handle `SplitBattleBox` events: deactivate source, spawn two new boxes.
fn handle_split_battle_box_system(
    mut commands: Commands,
    mut events: MessageReader<SplitBattleBox>,
    mut player_query: Query<
        (&Transform, &PhysicsCollider, &mut BoundToBattleBox),
        With<BehaviorParams>,
    >,
    ui_boxes: UiBattleBoxReadQuery,
    am_boxes: AmBattleBoxReadQuery,
    children_query: Query<&Children>,
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
            &children_query,
            &ui_boxes,
            &am_boxes,
        );
        retire_existing_battle_boxes_with_id(
            &mut commands,
            "SplitBattleBox",
            &ev.result_boxes.1,
            &[],
            &children_query,
            &ui_boxes,
            &am_boxes,
        );
        retire_battle_box(&mut commands, source_box.entity, &children_query);

        let target_boundary_gap = style.boundary_gap_for_visible_gap(ev.gap);
        let (box_a, box_b) = split_rect_box(
            &original,
            &ev.split_axis,
            ev.split_position,
            target_boundary_gap,
            ev.gap_policy,
        );

        let (id_a, id_b) = (&ev.result_boxes.0, &ev.result_boxes.1);

        if ev.duration > 0.0 {
            // Animated split only animates the gap.
            // Start from the same geometry as `gap = 0`, then open the gap over time.
            let (start_boundary_a, start_boundary_b) = split_rect_box(
                &original,
                &ev.split_axis,
                ev.split_position,
                style.boundary_gap_for_visible_gap(0.0),
                ev.gap_policy,
            );

            let entity_a =
                spawn_standalone_box_entity(&mut commands, id_a, &start_boundary_a, &style);
            let entity_b =
                spawn_standalone_box_entity(&mut commands, id_b, &start_boundary_b, &style);

            spawn_view_box_sdf_children(
                &mut commands,
                entity_a,
                &style.to_view_box(
                    start_boundary_a.half_size.x * 2.0,
                    start_boundary_a.half_size.y * 2.0,
                ),
                &mut meshes,
                &mut sdf_materials,
                &mut color_materials,
            );
            spawn_view_box_sdf_children(
                &mut commands,
                entity_b,
                &style.to_view_box(
                    start_boundary_b.half_size.x * 2.0,
                    start_boundary_b.half_size.y * 2.0,
                ),
                &mut meshes,
                &mut sdf_materials,
                &mut color_materials,
            );

            commands.spawn(BattleBoxSplitAnimation {
                original_boundary: original.clone(),
                split_axis: ev.split_axis,
                split_position: ev.split_position,
                target_visible_gap: ev.gap,
                gap_policy: ev.gap_policy,
                easing: ev.easing,
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
        for (player_tf, collider, mut bound) in player_query.iter_mut() {
            if bound.0 == ev.source_box {
                let pos = player_tf.translation.truncate();
                bound.0 = select_box_id_for_player(pos, collider, &box_a, &box_b, id_a, id_b);
            }
        }

        info!(
            "Split '{}' → '{}' + '{}' (axis={:?}, pos={}, gap={}, gap_policy={:?}, duration={}, easing={:?})",
            ev.source_box,
            id_a,
            id_b,
            ev.split_axis,
            ev.split_position,
            ev.gap,
            ev.gap_policy,
            ev.duration,
            ev.easing
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
    children_query: Query<&Children>,
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
        let mut resolved_sources: Vec<BattleBoxCandidate> = Vec::with_capacity(2);
        let mut visual_style: Option<BattleBoxVisualStyle> = None;
        let mut source_entities: Vec<Entity> = Vec::with_capacity(2);

        // Resolve sources and collect boundaries
        for target_id in &ids {
            let Some(source_box) =
                resolve_live_battle_box("MergeBattleBoxes", target_id, &ui_boxes, &am_boxes)
            else {
                continue;
            };
            visual_style.get_or_insert(source_box.visual_style.clone());
            source_entities.push(source_box.entity);
            resolved_sources.push(source_box);
        }

        if resolved_sources.len() < 2 {
            warn!(
                "MergeBattleBoxes: need 2 valid source boxes, found {}",
                resolved_sources.len()
            );
            continue;
        }

        retire_existing_battle_boxes_with_id(
            &mut commands,
            "MergeBattleBoxes",
            &ev.result_box,
            &source_entities,
            &children_query,
            &ui_boxes,
            &am_boxes,
        );

        let boundaries = [
            resolved_sources[0]
                .boundary
                .clone()
                .expect("live battle box candidate must have a boundary"),
            resolved_sources[1]
                .boundary
                .clone()
                .expect("live battle box candidate must have a boundary"),
        ];
        let merge_plan = plan_merge_animation(&boundaries, ev.gap_policy);
        let default_visual_style = BattleBoxVisualStyle::default();
        let style = visual_style
            .as_ref()
            .unwrap_or(&default_visual_style)
            .clone();

        if ev.duration > 0.0 {
            let Some(plan) = merge_plan.as_ref() else {
                warn!(
                    "MergeBattleBoxes: failed to infer merge geometry for '{}' + '{}'; falling back to instant merge",
                    ev.source_boxes.0, ev.source_boxes.1
                );
                let merged = merge_boundaries(&boundaries[0], &boundaries[1]);
                finalize_merged_battle_box(
                    &mut commands,
                    &source_entities,
                    &children_query,
                    &mut meshes,
                    &mut sdf_materials,
                    &mut color_materials,
                    &mut player_query,
                    &ev.source_boxes,
                    &ev.result_box,
                    &merged,
                    &style,
                );
                continue;
            };

            let source_a = &resolved_sources[plan.ordered_indices.0];
            let source_b = &resolved_sources[plan.ordered_indices.1];
            commands.spawn(BattleBoxMergeAnimation {
                source_boxes: ev.source_boxes.clone(),
                result_box: ev.result_box.clone(),
                box_entity_a: source_a.entity,
                box_entity_b: source_b.entity,
                start_boundary_a: source_a
                    .boundary
                    .clone()
                    .expect("live battle box candidate must have a boundary"),
                start_boundary_b: source_b
                    .boundary
                    .clone()
                    .expect("live battle box candidate must have a boundary"),
                target_boundary_a: plan.target_boundary_a.clone(),
                target_boundary_b: plan.target_boundary_b.clone(),
                merged_boundary: plan.merged_boundary.clone(),
                visual_style: style.clone(),
                easing: ev.easing,
                progress: 0.0,
                duration: ev.duration,
            });
        } else {
            let merged = merge_plan
                .as_ref()
                .map(|plan| plan.merged_boundary.clone())
                .unwrap_or_else(|| merge_boundaries(&boundaries[0], &boundaries[1]));

            finalize_merged_battle_box(
                &mut commands,
                &source_entities,
                &children_query,
                &mut meshes,
                &mut sdf_materials,
                &mut color_materials,
                &mut player_query,
                &ev.source_boxes,
                &ev.result_box,
                &merged,
                &style,
            );
        }

        info!(
            "Merged '{}' + '{}' → '{}' (axis={:?}, gap_policy={:?}, duration={}, easing={:?})",
            ev.source_boxes.0,
            ev.source_boxes.1,
            ev.result_box,
            merge_plan
                .as_ref()
                .map(|plan| plan.axis)
                .unwrap_or(SplitAxis::Vertical),
            ev.gap_policy,
            ev.duration,
            ev.easing
        );
    }
}

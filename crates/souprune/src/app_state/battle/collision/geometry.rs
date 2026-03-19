use super::*;

/// Split a rectangular boundary along an axis.
pub(super) fn split_rect_box(
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
            let min_x = original.center.x - original.half_size.x;
            let max_x = original.center.x + original.half_size.x;

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
                        center: Vec2::new(min_x + left_width_scaled / 2.0, original.center.y),
                    };
                    let right = BattleBoxBoundary {
                        half_size: Vec2::new(right_width_scaled / 2.0, original.half_size.y),
                        center: Vec2::new(max_x - right_width_scaled / 2.0, original.center.y),
                    };
                    (left, right)
                }
            }
        }
        SplitAxis::Horizontal => {
            // Positive split_pos moves the split line upward, making the top box smaller
            // and the bottom box larger.
            let top_height = original.half_size.y - split_pos;
            let bottom_height = original.half_size.y + split_pos;
            let half_gap = gap / 2.0;
            let min_y = original.center.y - original.half_size.y;
            let max_y = original.center.y + original.half_size.y;

            match gap_policy {
                GapPolicy::Expands => {
                    let top = BattleBoxBoundary {
                        half_size: Vec2::new(original.half_size.x, top_height / 2.0),
                        center: Vec2::new(original.center.x, max_y + half_gap - top_height / 2.0),
                    };
                    let bottom = BattleBoxBoundary {
                        half_size: Vec2::new(original.half_size.x, bottom_height / 2.0),
                        center: Vec2::new(
                            original.center.x,
                            min_y - half_gap + bottom_height / 2.0,
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
                        center: Vec2::new(original.center.x, max_y - top_height_scaled / 2.0),
                    };
                    let bottom = BattleBoxBoundary {
                        half_size: Vec2::new(original.half_size.x, bottom_height_scaled / 2.0),
                        center: Vec2::new(original.center.x, min_y + bottom_height_scaled / 2.0),
                    };
                    (top, bottom)
                }
            }
        }
    }
}

fn infer_merge_axis(a: &BattleBoxBoundary, b: &BattleBoxBoundary) -> Option<SplitAxis> {
    const EPSILON: f32 = 0.001;

    let overlap_x = (a.half_size.x + b.half_size.x) - (a.center.x - b.center.x).abs();
    let overlap_y = (a.half_size.y + b.half_size.y) - (a.center.y - b.center.y).abs();

    if overlap_x + EPSILON < overlap_y {
        return Some(SplitAxis::Vertical);
    }
    if overlap_y + EPSILON < overlap_x {
        return Some(SplitAxis::Horizontal);
    }

    let delta_x = (a.center.x - b.center.x).abs();
    let delta_y = (a.center.y - b.center.y).abs();

    if delta_y + EPSILON < delta_x {
        Some(SplitAxis::Vertical)
    } else if delta_x + EPSILON < delta_y {
        Some(SplitAxis::Horizontal)
    } else {
        None
    }
}

pub(super) fn plan_merge_animation(
    boundaries: &[BattleBoxBoundary; 2],
    gap_policy: GapPolicy,
) -> Option<MergeAnimationPlan> {
    let axis = infer_merge_axis(&boundaries[0], &boundaries[1])?;

    match axis {
        SplitAxis::Vertical => {
            let ordered_indices = if boundaries[0].center.x <= boundaries[1].center.x {
                (0, 1)
            } else {
                (1, 0)
            };
            let left = &boundaries[ordered_indices.0];
            let right = &boundaries[ordered_indices.1];

            let left_min = left.center.x - left.half_size.x;
            let left_max = left.center.x + left.half_size.x;
            let right_min = right.center.x - right.half_size.x;
            let right_max = right.center.x + right.half_size.x;

            let (target_left, target_right) = match gap_policy {
                GapPolicy::Expands => {
                    let split_x = (left_max + right_min) * 0.5;
                    (
                        BattleBoxBoundary {
                            half_size: left.half_size,
                            center: Vec2::new(split_x - left.half_size.x, left.center.y),
                        },
                        BattleBoxBoundary {
                            half_size: right.half_size,
                            center: Vec2::new(split_x + right.half_size.x, right.center.y),
                        },
                    )
                }
                GapPolicy::Includes => {
                    let outer_span = right_max - left_min;
                    let combined_width = left.half_size.x * 2.0 + right.half_size.x * 2.0;
                    if outer_span <= 0.0 || combined_width <= 0.0 {
                        return None;
                    }

                    let scale = combined_width / outer_span;
                    if scale <= f32::EPSILON {
                        return None;
                    }

                    let target_left_width = (left.half_size.x * 2.0) / scale;
                    let target_right_width = (right.half_size.x * 2.0) / scale;
                    (
                        BattleBoxBoundary {
                            half_size: Vec2::new(target_left_width * 0.5, left.half_size.y),
                            center: Vec2::new(left_min + target_left_width * 0.5, left.center.y),
                        },
                        BattleBoxBoundary {
                            half_size: Vec2::new(target_right_width * 0.5, right.half_size.y),
                            center: Vec2::new(right_max - target_right_width * 0.5, right.center.y),
                        },
                    )
                }
            };

            let merged_boundary = merge_boundaries(&target_left, &target_right);
            Some(MergeAnimationPlan {
                axis,
                ordered_indices,
                target_boundary_a: target_left,
                target_boundary_b: target_right,
                merged_boundary,
            })
        }
        SplitAxis::Horizontal => {
            let ordered_indices = if boundaries[0].center.y >= boundaries[1].center.y {
                (0, 1)
            } else {
                (1, 0)
            };
            let top = &boundaries[ordered_indices.0];
            let bottom = &boundaries[ordered_indices.1];

            let top_max = top.center.y + top.half_size.y;
            let top_min = top.center.y - top.half_size.y;
            let bottom_max = bottom.center.y + bottom.half_size.y;
            let bottom_min = bottom.center.y - bottom.half_size.y;

            let (target_top, target_bottom) = match gap_policy {
                GapPolicy::Expands => {
                    let split_y = (top_min + bottom_max) * 0.5;
                    (
                        BattleBoxBoundary {
                            half_size: top.half_size,
                            center: Vec2::new(top.center.x, split_y + top.half_size.y),
                        },
                        BattleBoxBoundary {
                            half_size: bottom.half_size,
                            center: Vec2::new(bottom.center.x, split_y - bottom.half_size.y),
                        },
                    )
                }
                GapPolicy::Includes => {
                    let outer_span = top_max - bottom_min;
                    let combined_height = top.half_size.y * 2.0 + bottom.half_size.y * 2.0;
                    if outer_span <= 0.0 || combined_height <= 0.0 {
                        return None;
                    }

                    let scale = combined_height / outer_span;
                    if scale <= f32::EPSILON {
                        return None;
                    }

                    let target_top_height = (top.half_size.y * 2.0) / scale;
                    let target_bottom_height = (bottom.half_size.y * 2.0) / scale;
                    (
                        BattleBoxBoundary {
                            half_size: Vec2::new(top.half_size.x, target_top_height * 0.5),
                            center: Vec2::new(top.center.x, top_max - target_top_height * 0.5),
                        },
                        BattleBoxBoundary {
                            half_size: Vec2::new(bottom.half_size.x, target_bottom_height * 0.5),
                            center: Vec2::new(
                                bottom.center.x,
                                bottom_min + target_bottom_height * 0.5,
                            ),
                        },
                    )
                }
            };

            let merged_boundary = merge_boundaries(&target_top, &target_bottom);
            Some(MergeAnimationPlan {
                axis,
                ordered_indices,
                target_boundary_a: target_top,
                target_boundary_b: target_bottom,
                merged_boundary,
            })
        }
    }
}

fn signed_distance_to_box_with_collider(
    boundary: &BattleBoxBoundary,
    player_pos: Vec2,
    collider: &PhysicsCollider,
) -> f32 {
    let collider_half_size = match collider {
        PhysicsCollider::Circle { radius } => Vec2::splat(*radius),
        PhysicsCollider::Box { half_size } => *half_size,
    };
    let effective_half_size = (boundary.half_size - collider_half_size).max(Vec2::ZERO);
    BattleBoxBoundary {
        half_size: effective_half_size,
        center: boundary.center,
    }
    .sdf_distance(player_pos)
}

/// Determine which of two boxes a player should be rebound to.
pub(super) fn select_box_id_for_player(
    player_pos: Vec2,
    collider: &PhysicsCollider,
    box_a: &BattleBoxBoundary,
    box_b: &BattleBoxBoundary,
    id_a: &str,
    id_b: &str,
) -> String {
    let dist_a = signed_distance_to_box_with_collider(box_a, player_pos, collider);
    let dist_b = signed_distance_to_box_with_collider(box_b, player_pos, collider);

    if dist_a <= 0.0 && dist_b > 0.0 {
        id_a.to_string()
    } else if dist_b <= 0.0 && dist_a > 0.0 {
        id_b.to_string()
    } else if dist_a <= dist_b {
        id_a.to_string()
    } else {
        id_b.to_string()
    }
}

pub(super) fn choose_box_index_for_player(
    current_bound: Option<&str>,
    player_pos: Vec2,
    collider: &PhysicsCollider,
    candidates: &[(String, BattleBoxBoundary)],
) -> Option<usize> {
    if candidates.is_empty() {
        return None;
    }

    if let Some(current_id) = current_bound
        && let Some((index, _)) = candidates.iter().enumerate().find(|(_, (id, boundary))| {
            id == current_id
                && signed_distance_to_box_with_collider(boundary, player_pos, collider) <= 0.0
        })
    {
        return Some(index);
    }

    if let Some((index, _)) = candidates.iter().enumerate().find(|(_, (_, boundary))| {
        signed_distance_to_box_with_collider(boundary, player_pos, collider) <= 0.0
    }) {
        return Some(index);
    }

    candidates
        .iter()
        .enumerate()
        .min_by(|(_, (_, a)), (_, (_, b))| {
            signed_distance_to_box_with_collider(a, player_pos, collider).total_cmp(
                &signed_distance_to_box_with_collider(b, player_pos, collider),
            )
        })
        .map(|(index, _)| index)
}

/// Resolve the `BattleBoxBoundary` from a battle box entity.
/// Returns `None` if the box is inactive or collision is disabled.
pub(super) fn resolve_boundary(
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

pub(super) fn resolve_live_battle_box(
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

fn hide_entity_and_descendants(
    commands: &mut Commands,
    entity: Entity,
    children_query: &Query<&Children>,
) {
    commands.entity(entity).insert(Visibility::Hidden);

    let Ok(children) = children_query.get(entity) else {
        return;
    };

    for child in children.iter() {
        hide_entity_and_descendants(commands, child, children_query);
    }
}

pub(super) fn retire_battle_box(
    commands: &mut Commands,
    entity: Entity,
    children_query: &Query<&Children>,
) {
    hide_entity_and_descendants(commands, entity, children_query);
    commands
        .entity(entity)
        .remove::<(BattleBox, BattleBoxId, BattleBoxState, BattleBoxVisualStyle)>();
}

pub(super) fn retire_existing_battle_boxes_with_id(
    commands: &mut Commands,
    op_name: &str,
    target_id: &str,
    keep_entities: &[Entity],
    children_query: &Query<&Children>,
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
        retire_battle_box(commands, candidate.entity, children_query);
    }
}

/// Merge two boundaries into one AABB that encloses both.
pub(super) fn merge_boundaries(a: &BattleBoxBoundary, b: &BattleBoxBoundary) -> BattleBoxBoundary {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 0.001,
            "expected {a} ~= {b}, diff={}",
            (a - b).abs()
        );
    }

    fn boundary(width: f32, height: f32) -> BattleBoxBoundary {
        BattleBoxBoundary {
            center: Vec2::ZERO,
            half_size: Vec2::new(width / 2.0, height / 2.0),
        }
    }

    #[test]
    fn horizontal_split_positive_position_makes_top_smaller() {
        let original = boundary(300.0, 200.0);
        let (top, bottom) = split_rect_box(
            &original,
            &SplitAxis::Horizontal,
            20.0,
            0.0,
            GapPolicy::Expands,
        );

        approx_eq(top.half_size.y * 2.0, 80.0);
        approx_eq(bottom.half_size.y * 2.0, 120.0);
        assert!(top.center.y > bottom.center.y);
    }

    #[test]
    fn horizontal_includes_preserves_outer_edges_and_gap() {
        let original = boundary(300.0, 200.0);
        let gap = 20.0;
        let (top, bottom) = split_rect_box(
            &original,
            &SplitAxis::Horizontal,
            20.0,
            gap,
            GapPolicy::Includes,
        );

        approx_eq(top.center.y + top.half_size.y, 100.0);
        approx_eq(bottom.center.y - bottom.half_size.y, -100.0);
        approx_eq(
            (top.center.y - top.half_size.y) - (bottom.center.y + bottom.half_size.y),
            gap,
        );
    }

    #[test]
    fn vertical_includes_preserves_outer_edges_and_gap() {
        let original = boundary(300.0, 200.0);
        let gap = 20.0;
        let (left, right) = split_rect_box(
            &original,
            &SplitAxis::Vertical,
            20.0,
            gap,
            GapPolicy::Includes,
        );

        approx_eq(left.center.x - left.half_size.x, -150.0);
        approx_eq(right.center.x + right.half_size.x, 150.0);
        approx_eq(
            (right.center.x - right.half_size.x) - (left.center.x + left.half_size.x),
            gap,
        );
    }

    #[test]
    fn structured_vertical_split_gap_matches_requested_visible_gap() {
        let original = boundary(300.0, 200.0);
        let style = BattleBoxVisualStyle {
            border_width: 5.0,
            fill_shader: None,
            structure_file: Some("shared/view_structures/view_box.sdf.ron".to_string()),
            fill_color: Color::BLACK,
        };
        let requested_visible_gap = 20.0;
        let (left, right) = split_rect_box(
            &original,
            &SplitAxis::Vertical,
            0.0,
            style.boundary_gap_for_visible_gap(requested_visible_gap),
            GapPolicy::Expands,
        );

        let visible_gap = (right.center.x - right.half_size.x - style.border_width)
            - (left.center.x + left.half_size.x + style.border_width);
        approx_eq(visible_gap, requested_visible_gap);
    }

    #[test]
    fn structured_zero_visible_gap_keeps_outer_edges_touching() {
        let original = boundary(300.0, 200.0);
        let style = BattleBoxVisualStyle {
            border_width: 5.0,
            fill_shader: None,
            structure_file: Some("shared/view_structures/view_box.sdf.ron".to_string()),
            fill_color: Color::BLACK,
        };
        let (top, bottom) = split_rect_box(
            &original,
            &SplitAxis::Horizontal,
            0.0,
            style.boundary_gap_for_visible_gap(0.0),
            GapPolicy::Includes,
        );

        let visible_gap = (top.center.y - top.half_size.y - style.border_width)
            - (bottom.center.y + bottom.half_size.y + style.border_width);
        approx_eq(visible_gap, 0.0);
    }

    #[test]
    fn vertical_expands_merge_recovers_original_boundary() {
        let original = boundary(300.0, 200.0);
        let (left, right) = split_rect_box(
            &original,
            &SplitAxis::Vertical,
            20.0,
            30.0,
            GapPolicy::Expands,
        );
        let expected = split_rect_box(
            &original,
            &SplitAxis::Vertical,
            20.0,
            0.0,
            GapPolicy::Expands,
        );

        let plan =
            plan_merge_animation(&[left.clone(), right.clone()], GapPolicy::Expands).unwrap();

        assert_eq!(plan.axis, SplitAxis::Vertical);
        approx_eq(plan.merged_boundary.center.x, original.center.x);
        approx_eq(plan.merged_boundary.center.y, original.center.y);
        approx_eq(plan.merged_boundary.half_size.x, original.half_size.x);
        approx_eq(plan.merged_boundary.half_size.y, original.half_size.y);
        approx_eq(plan.target_boundary_a.center.x, expected.0.center.x);
        approx_eq(plan.target_boundary_b.center.x, expected.1.center.x);
        approx_eq(plan.target_boundary_a.half_size.x, expected.0.half_size.x);
        approx_eq(plan.target_boundary_b.half_size.x, expected.1.half_size.x);
    }

    #[test]
    fn horizontal_includes_merge_recovers_original_boundary() {
        let original = boundary(300.0, 200.0);
        let (top, bottom) = split_rect_box(
            &original,
            &SplitAxis::Horizontal,
            20.0,
            20.0,
            GapPolicy::Includes,
        );
        let expected = split_rect_box(
            &original,
            &SplitAxis::Horizontal,
            20.0,
            0.0,
            GapPolicy::Includes,
        );

        let plan =
            plan_merge_animation(&[top.clone(), bottom.clone()], GapPolicy::Includes).unwrap();

        assert_eq!(plan.axis, SplitAxis::Horizontal);
        approx_eq(plan.merged_boundary.center.x, original.center.x);
        approx_eq(plan.merged_boundary.center.y, original.center.y);
        approx_eq(plan.merged_boundary.half_size.x, original.half_size.x);
        approx_eq(plan.merged_boundary.half_size.y, original.half_size.y);
        approx_eq(plan.target_boundary_a.center.y, expected.0.center.y);
        approx_eq(plan.target_boundary_b.center.y, expected.1.center.y);
        approx_eq(plan.target_boundary_a.half_size.y, expected.0.half_size.y);
        approx_eq(plan.target_boundary_b.half_size.y, expected.1.half_size.y);
    }

    #[test]
    fn player_rebind_prefers_box_that_currently_contains_player() {
        let original = boundary(300.0, 200.0);
        let (left, right) = split_rect_box(
            &original,
            &SplitAxis::Vertical,
            100.0,
            0.0,
            GapPolicy::Expands,
        );
        let player_pos = Vec2::new(70.0, 0.0);
        let collider = PhysicsCollider::Circle { radius: 8.0 };

        assert_eq!(
            select_box_id_for_player(player_pos, &collider, &left, &right, "left", "right"),
            "left"
        );
    }

    #[test]
    fn dynamic_box_choice_switches_when_current_binding_no_longer_contains_player() {
        let original = boundary(300.0, 200.0);
        let (left, right) = split_rect_box(
            &original,
            &SplitAxis::Vertical,
            100.0,
            0.0,
            GapPolicy::Expands,
        );
        let player_pos = Vec2::new(70.0, 0.0);
        let collider = PhysicsCollider::Circle { radius: 8.0 };
        let candidates = vec![
            ("left".to_string(), left.clone()),
            ("right".to_string(), right.clone()),
        ];

        assert_eq!(
            choose_box_index_for_player(Some("right"), player_pos, &collider, &candidates),
            Some(0)
        );
    }
}

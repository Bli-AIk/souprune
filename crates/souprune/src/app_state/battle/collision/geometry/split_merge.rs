use super::*;

/// Split a rectangular boundary along an axis.
pub(in crate::app_state::battle::collision) fn split_rect_box(
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

pub(in crate::app_state::battle::collision) fn plan_merge_animation(
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

/// Merge two boundaries into one AABB that encloses both.
pub(in crate::app_state::battle::collision) fn merge_boundaries(
    a: &BattleBoxBoundary,
    b: &BattleBoxBoundary,
) -> BattleBoxBoundary {
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

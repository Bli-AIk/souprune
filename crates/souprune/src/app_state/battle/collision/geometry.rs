use super::*;

mod candidate_resolution;
mod player_selection;
mod split_merge;

pub(super) use candidate_resolution::{
    resolve_boundary, resolve_live_battle_box, retire_battle_box,
    retire_existing_battle_boxes_with_id,
};
pub(super) use player_selection::{choose_box_index_for_player, select_box_id_for_player};
pub(super) use split_merge::{merge_boundaries, plan_merge_animation, split_rect_box};

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

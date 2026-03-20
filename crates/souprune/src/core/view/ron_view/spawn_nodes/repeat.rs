use super::super::parsing::PlayerDataView;
use super::super::parsing::evaluate_float_expr_with_repeat;
use bevy::prelude::*;

pub(super) fn resolve_repeat_item(
    player_data: &PlayerDataView<'_>,
    source: &str,
    index: usize,
) -> Option<String> {
    if let Some(list) = player_data.get_fact_string_list(source) {
        list.into_iter().nth(index)
    } else if let Some(list) = player_data.get_fact_int_list(source) {
        list.into_iter().nth(index).map(|value| value.to_string())
    } else {
        None
    }
}

pub(super) fn build_transform(
    transform_def: &crate::core::view::layout::serde_types::SerializableTransform,
    player_data: &PlayerDataView<'_>,
    repeat_ctx: Option<&super::super::parsing::RepeatContext>,
) -> Transform {
    let mut transform = Transform::default();
    if let Some(translation) = &transform_def.translation {
        transform.translation = Vec3::new(
            evaluate_float_expr_with_repeat(&translation.0, player_data, None, repeat_ctx),
            evaluate_float_expr_with_repeat(&translation.1, player_data, None, repeat_ctx),
            evaluate_float_expr_with_repeat(&translation.2, player_data, None, repeat_ctx),
        );
    }
    if let Some(scale) = &transform_def.scale {
        transform.scale = Vec3::new(
            evaluate_float_expr_with_repeat(&scale.0, player_data, None, repeat_ctx),
            evaluate_float_expr_with_repeat(&scale.1, player_data, None, repeat_ctx),
            evaluate_float_expr_with_repeat(&scale.2, player_data, None, repeat_ctx),
        );
    }
    if let Some(rotation) = transform_def.rotation {
        transform.rotation = Quat::from_rotation_z(rotation.to_radians());
    }
    transform
}

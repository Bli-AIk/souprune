//! # repeat.rs
//!
//! # repeat.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Contains the low-level helpers that make repeated view nodes work. It resolves the
//! current repeat item from fact arrays and evaluates transforms with repeat-specific variables so
//! each spawned copy gets its own concrete position and scale.
//!
//! 放的是让 repeat 节点真正可用的底层辅助逻辑。它会从事实数组里取出当前项，并用
//! repeat 专属变量去求值变换，让每一个复制出来的节点都有各自明确的位置和缩放。

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
    if let Some(rotation) = &transform_def.rotation {
        let rot_degrees = evaluate_float_expr_with_repeat(rotation, player_data, None, repeat_ctx);
        transform.rotation = Quat::from_rotation_z(rot_degrees.to_radians());
    }
    transform
}

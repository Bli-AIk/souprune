//! # postprocess.rs
//!
//! # postprocess.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Applies the runtime-only decorations that come after a view node is spawned. It
//! wires up `VisibleWhen`, records dynamic sprite definitions, and marks entities that need
//! time-dependent transform updates.
//!
//! 负责节点生成之后才需要补上的运行时装饰。它会挂接 `VisibleWhen`，记录动态精灵
//! 定义，并给那些需要随时间更新变换的实体打上标记。

use super::super::parsing::{
    PlayerDataView, evaluate_visible_when, preprocess_sprite_def_for_repeat,
    vec3_tuple_depends_on_time,
};
use bevy::prelude::*;

pub(super) fn apply_visible_when(
    commands: &mut Commands,
    entity_id: Entity,
    visible_when_expr: &str,
    node_name: &str,
    player_data: &PlayerDataView<'_>,
    repeat_ctx: Option<&super::super::parsing::RepeatContext>,
) {
    let expr = visible_when_expr.trim();
    if expr.is_empty() {
        return;
    }

    let processed_expr = substitute_repeat_vars(expr, repeat_ctx);

    info!(
        "Adding VisibleWhen to entity {:?} ({}): '{}' (original: '{}')",
        entity_id, node_name, processed_expr, expr
    );
    commands
        .entity(entity_id)
        .insert(crate::core::view::components::VisibleWhen {
            expression: processed_expr.clone(),
        });

    let is_visible = evaluate_visible_when(&processed_expr, player_data);
    if !is_visible {
        commands.entity(entity_id).insert(Visibility::Hidden);
    }
}

fn substitute_repeat_vars(
    expr: &str,
    repeat_ctx: Option<&super::super::parsing::RepeatContext>,
) -> String {
    let Some(ctx) = repeat_ctx else {
        return expr.to_string();
    };
    let mut result = expr.to_string();
    result = result.replace("@i", &ctx.index.to_string());
    result = result.replace("@index", &ctx.index.to_string());
    for (var_name, var_value) in &ctx.variables {
        result = result.replace(&format!("@{}", var_name), var_value);
    }
    result
}

pub(super) fn apply_dynamic_element(
    commands: &mut Commands,
    entity_id: Entity,
    node_def: &crate::core::view::layout::view_schema::ViewNodeDef,
    repeat_ctx: Option<&super::super::parsing::RepeatContext>,
) {
    let sprite_def = node_def
        .sprite
        .as_ref()
        .expect("sprite must exist when apply_dynamic_element is called");
    let (has_dynamic, has_time_dependency) = check_sprite_dynamics(sprite_def, &node_def.name);

    if !has_dynamic {
        info!("No dynamic properties found for {}", node_def.name);
        return;
    }

    info!(
        "Adding DynamicViewElement to entity {:?} ({}) [time_dependent={}]",
        entity_id, node_def.name, has_time_dependency
    );

    let processed_sprite_def = if let Some(ctx) = repeat_ctx {
        preprocess_sprite_def_for_repeat(sprite_def, ctx)
    } else {
        sprite_def.clone()
    };

    commands
        .entity(entity_id)
        .insert(crate::core::view::components::DynamicViewElement {
            sprite_def: Some(processed_sprite_def),
            text_def: None,
            view_box_def: None,
        });

    if has_time_dependency {
        commands
            .entity(entity_id)
            .insert(crate::core::view::components::TimeDependentTransform);
    }
}

fn check_sprite_dynamics(
    sprite_def: &crate::core::view::layout::view_schema::SpriteDef,
    node_name: &str,
) -> (bool, bool) {
    let mut has_dynamic = false;
    let mut has_time_dependency = false;

    if let Some(transform) = &sprite_def.transform {
        if let Some(translation) = &transform.translation {
            let tx = translation.0.is_dynamic();
            let ty = translation.1.is_dynamic();
            let tz = translation.2.is_dynamic();
            info!(
                "Checking dynamics for {}: x={}, y={}, z={}",
                node_name, tx, ty, tz
            );
            if tx || ty || tz {
                has_dynamic = true;
            }
            if vec3_tuple_depends_on_time(translation) {
                has_time_dependency = true;
            }
        }
        if let Some(scale) = &transform.scale {
            if scale.0.is_dynamic() || scale.1.is_dynamic() || scale.2.is_dynamic() {
                has_dynamic = true;
            }
            if vec3_tuple_depends_on_time(scale) {
                has_time_dependency = true;
            }
        }
    }
    (has_dynamic, has_time_dependency)
}

//! # resolve.rs
//!
//! # 属性解析模块
//!
//! Functions for resolving property bindings to concrete values.
//!
//! 将属性绑定解析为具体值的函数。

use super::tree::{
    DesiredLagAnimation, DesiredMaterial, DesiredMaterialAnimations, DesiredSprite, DesiredText,
    MaterialParamDef,
};
use crate::core::view::layout::serde_types::serializable_color_to_color;
use crate::core::view::layout::view_schema::MaterialParamValue;
use crate::core::view::layout::{SpriteDef, TextDef};
use crate::core::view::ron_view::parsing::{
    PlayerDataView, RepeatContext, evaluate_float_expr, evaluate_float_expr_with_repeat,
    evaluate_visible_when,
};
use bevy::prelude::*;
use bevy::sprite::Anchor;

/// Resolve the transform for a node definition.
/// 解析节点定义的变换。
pub fn resolve_transform(
    player_data: &PlayerDataView,
    sprite_def: Option<&SpriteDef>,
    repeat_ctx: Option<&RepeatContext>,
) -> Transform {
    let Some(sprite) = sprite_def else {
        return Transform::IDENTITY;
    };

    let Some(ref tr) = sprite.transform else {
        return Transform::IDENTITY;
    };

    // Resolve translation
    let translation = if let Some(ref trans) = tr.translation {
        if let Some(ctx) = repeat_ctx {
            Vec3::new(
                evaluate_float_expr_with_repeat(&trans.0, player_data, None, Some(ctx)),
                evaluate_float_expr_with_repeat(&trans.1, player_data, None, Some(ctx)),
                evaluate_float_expr_with_repeat(&trans.2, player_data, None, Some(ctx)),
            )
        } else {
            Vec3::new(
                evaluate_float_expr(&trans.0, player_data, None),
                evaluate_float_expr(&trans.1, player_data, None),
                evaluate_float_expr(&trans.2, player_data, None),
            )
        }
    } else {
        Vec3::ZERO
    };

    // Resolve scale
    let scale = if let Some(ref sc) = tr.scale {
        if let Some(ctx) = repeat_ctx {
            Vec3::new(
                evaluate_float_expr_with_repeat(&sc.0, player_data, None, Some(ctx)),
                evaluate_float_expr_with_repeat(&sc.1, player_data, None, Some(ctx)),
                evaluate_float_expr_with_repeat(&sc.2, player_data, None, Some(ctx)),
            )
        } else {
            Vec3::new(
                evaluate_float_expr(&sc.0, player_data, None),
                evaluate_float_expr(&sc.1, player_data, None),
                evaluate_float_expr(&sc.2, player_data, None),
            )
        }
    } else {
        Vec3::ONE
    };

    // Resolve rotation (single value for Z-axis rotation)
    let rotation = if let Some(rot_z) = tr.rotation {
        Quat::from_rotation_z(rot_z.to_radians())
    } else {
        Quat::IDENTITY
    };

    // Apply pivot offset if present
    // 如果存在 pivot 则应用偏移
    let final_translation = if let Some(pivot) = &sprite.pivot {
        let (pivot_x, pivot_y) =
            crate::core::view::layout::serde_types::vec2_tuple_to_static(pivot);
        let shift_x = (0.5 - pivot_x) * scale.x;
        let shift_y = (0.5 - pivot_y) * scale.y;
        let shift = rotation * Vec3::new(shift_x, shift_y, 0.0);
        translation + shift
    } else {
        translation
    };

    Transform {
        translation: final_translation,
        rotation,
        scale,
    }
}

/// Resolve visibility from visible_when expression.
/// 从 visible_when 表达式解析可见性。
pub fn resolve_visibility(
    player_data: &PlayerDataView,
    visible_when: Option<&str>,
    repeat_ctx: Option<&RepeatContext>,
) -> Visibility {
    let Some(expr) = visible_when else {
        return Visibility::Inherited;
    };

    // Replace @i with concrete index if in repeat context
    let processed_expr = if let Some(ctx) = repeat_ctx {
        expr.replace("@i", &ctx.index.to_string())
            .replace("@index", &ctx.index.to_string())
    } else {
        expr.to_string()
    };

    let is_visible = evaluate_visible_when(&processed_expr, player_data);

    if is_visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

/// Resolve sprite definition to desired state.
/// 将精灵定义解析为期望状态。
pub fn resolve_sprite(sprite_def: Option<&SpriteDef>) -> Option<DesiredSprite> {
    let sprite = sprite_def?;

    // Get color with default white
    let color = if let Some(ref c) = sprite.color {
        serializable_color_to_color(c)
    } else {
        Color::WHITE
    };

    Some(DesiredSprite {
        visual: format!("{:?}", sprite.visual),
        color,
        anchor: Anchor::CENTER, // Default anchor, could be extended
        flip_x: sprite.flip_x,
        flip_y: sprite.flip_y,
    })
}

/// Resolve text definitions to desired state.
/// 将文本定义解析为期望状态。
pub fn resolve_texts(
    player_data: &PlayerDataView,
    texts: &[TextDef],
    repeat_ctx: Option<&RepeatContext>,
) -> Vec<DesiredText> {
    texts
        .iter()
        .map(|text_def| resolve_single_text(player_data, text_def, repeat_ctx))
        .collect()
}

/// Resolve a single text definition.
/// 解析单个文本定义。
pub fn resolve_single_text(
    _player_data: &PlayerDataView,
    text_def: &TextDef,
    repeat_ctx: Option<&RepeatContext>,
) -> DesiredText {
    // Get content with default empty string
    let raw_content = text_def.content.clone().unwrap_or_default();

    // For repeat elements, handle @i replacement
    let content = if let Some(ctx) = repeat_ctx {
        raw_content
            .replace("@i", &ctx.index.to_string())
            .replace("@index", &ctx.index.to_string())
    } else {
        raw_content
    };

    // Convert ViewFontDef to string representation
    let font_name = format!("{:?}", text_def.font);

    DesiredText {
        content,
        font: font_name,
        font_size: 16.0, // TextDef doesn't have font_size, using default
        color: serializable_color_to_color(&text_def.color),
        anchor: Anchor::CENTER, // Default, could be extended
    }
}

/// Resolve material definition from sprite def.
/// 从精灵定义解析材质定义。
pub fn resolve_material(sprite_def: Option<&SpriteDef>) -> Option<DesiredMaterial> {
    let sprite = sprite_def?;
    let material_def = sprite.material.as_ref()?;

    // Convert params
    let params = material_def
        .params
        .iter()
        .map(|(k, v)| {
            let def = match v {
                MaterialParamValue::Static(val) => MaterialParamDef::Static(*val),
                MaterialParamValue::Expr(expr) => MaterialParamDef::Expr(expr.clone()),
            };
            (k.clone(), def)
        })
        .collect();

    // Convert animations if present
    let animations = material_def.animations.as_ref().map(|anim_def| {
        let lag = anim_def.lag.as_ref().map(|lag_def| DesiredLagAnimation {
            source: lag_def.source.clone(),
            target: lag_def.target.clone(),
            delay: lag_def.delay,
            duration: lag_def.duration,
            easing: lag_def.easing.clone(),
        });

        DesiredMaterialAnimations { lag }
    });

    Some(DesiredMaterial {
        shader: material_def.shader.clone(),
        params,
        animations,
    })
}

/// Process visible_when expression for repeat context.
/// 为重复上下文处理 visible_when 表达式。
pub fn process_visible_when_for_repeat(
    visible_when: Option<&str>,
    repeat_ctx: Option<&RepeatContext>,
) -> Option<String> {
    let expr = visible_when?;

    let processed = if let Some(ctx) = repeat_ctx {
        expr.replace("@i", &ctx.index.to_string())
            .replace("@index", &ctx.index.to_string())
    } else {
        expr.to_string()
    };

    Some(processed)
}

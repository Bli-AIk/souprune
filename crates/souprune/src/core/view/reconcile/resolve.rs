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
use crate::core::view::layout::{SerializableTransform, SpriteDef, TextDef, ViewNodeDef};
use crate::core::view::layout::{TextAlignDef, TextAnchorDef};
use crate::core::view::ron_view::parsing::{
    PlayerDataView, RepeatContext, evaluate_float_expr, evaluate_float_expr_with_repeat,
    evaluate_visible_when,
};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_bitmap_text::{TextAlign, TextAnchor};

fn resolve_text_anchor(anchor: Option<TextAnchorDef>) -> TextAnchor {
    match anchor.unwrap_or_default() {
        TextAnchorDef::TopLeft => TextAnchor::TOP_LEFT,
        TextAnchorDef::TopCenter => TextAnchor::TOP_CENTER,
        TextAnchorDef::TopRight => TextAnchor::TOP_RIGHT,
        TextAnchorDef::CenterLeft => TextAnchor::CENTER_LEFT,
        TextAnchorDef::Center => TextAnchor::CENTER,
        TextAnchorDef::CenterRight => TextAnchor::CENTER_RIGHT,
        TextAnchorDef::BottomLeft => TextAnchor::BOTTOM_LEFT,
        TextAnchorDef::BottomCenter => TextAnchor::BOTTOM_CENTER,
        TextAnchorDef::BottomRight => TextAnchor::BOTTOM_RIGHT,
    }
}

fn resolve_text_align(align: Option<TextAlignDef>) -> TextAlign {
    align.unwrap_or_default().into()
}

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

    resolve_transform_def(player_data, tr, repeat_ctx, Some(sprite))
}

/// Resolve the transform for a view node.
/// Node transforms take precedence over sprite transforms.
///
/// 解析视图节点的变换。
/// 节点级变换优先于精灵变换。
pub fn resolve_node_transform(
    player_data: &PlayerDataView,
    node_def: &ViewNodeDef,
    repeat_ctx: Option<&RepeatContext>,
) -> Transform {
    if let Some(transform) = &node_def.transform {
        return resolve_transform_def(player_data, transform, repeat_ctx, None);
    }

    resolve_transform(player_data, node_def.sprite.as_ref(), repeat_ctx)
}

fn resolve_transform_def(
    player_data: &PlayerDataView,
    tr: &SerializableTransform,
    repeat_ctx: Option<&RepeatContext>,
    sprite: Option<&SpriteDef>,
) -> Transform {
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
    let rotation = if let Some(ref rot_z) = tr.rotation {
        let degrees = if let Some(ctx) = repeat_ctx {
            evaluate_float_expr_with_repeat(rot_z, player_data, None, Some(ctx))
        } else {
            evaluate_float_expr(rot_z, player_data, None)
        };
        Quat::from_rotation_z(degrees.to_radians())
    } else {
        Quat::IDENTITY
    };

    // Material sprites do not have Bevy's Sprite anchor component, so their
    // pivot remains a transform offset. Regular sprites carry pivot through
    // Anchor and must keep their authored translation unchanged.
    //
    // material 精灵没有 Bevy 的 Sprite anchor 组件，因此 pivot 仍然表现为
    // transform 偏移。普通精灵通过 Anchor 携带 pivot，translation 必须保持源数据。
    let final_translation = if let Some(pivot) = sprite.and_then(|sprite| {
        if sprite.material.is_some() {
            sprite.pivot.as_ref()
        } else {
            None
        }
    }) {
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

/// Resolve the transform for a ViewBox node from its offset.
/// ViewBox 节点的变换来自 offset 字段（仅平移，无缩放/旋转）。
/// Evaluates dynamic expressions against the current facts.
pub fn resolve_viewbox_transform(
    view_box: &crate::core::view::layout::ViewBoxLogicDef,
    player_data: &PlayerDataView,
) -> Transform {
    let offset = Vec3::new(
        evaluate_float_expr(&view_box.offset.0, player_data, None),
        evaluate_float_expr(&view_box.offset.1, player_data, None),
        evaluate_float_expr(&view_box.offset.2, player_data, None),
    );
    Transform::from_translation(offset)
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

    let processed_expr = if let Some(ctx) = repeat_ctx {
        resolve_repeat_variables(expr, ctx)
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

    let content = if let Some(ctx) = repeat_ctx {
        resolve_repeat_variables(&raw_content, ctx)
    } else {
        raw_content
    };

    // Font identifier is now a plain string
    let font_name = text_def.font.clone();

    DesiredText {
        content,
        font: font_name,
        font_size: 16.0, // TextDef doesn't have font_size, using default
        color: serializable_color_to_color(&text_def.color),
        align: resolve_text_align(text_def.align),
        anchor: resolve_text_anchor(text_def.anchor),
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
        resolve_repeat_variables(expr, ctx)
    } else {
        expr.to_string()
    };

    Some(processed)
}

fn resolve_repeat_variables(input: &str, repeat_ctx: &RepeatContext) -> String {
    let mut result = replace_repeat_variable(input, "i", &repeat_ctx.index.to_string());
    result = replace_repeat_variable(&result, "index", &repeat_ctx.index.to_string());

    let mut variables = repeat_ctx.variables.iter().collect::<Vec<_>>();
    variables.sort_by_key(|(name, _)| std::cmp::Reverse(name.len()));
    for (name, value) in variables {
        result = replace_repeat_variable(&result, name, value);
    }

    result
}

fn replace_repeat_variable(input: &str, name: &str, value: &str) -> String {
    let token = format!("@{name}");
    let mut output = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(pos) = rest.find(&token) {
        let after_token = &rest[pos + token.len()..];
        output.push_str(&rest[..pos]);
        if after_token
            .chars()
            .next()
            .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_')
        {
            output.push_str(value);
        } else {
            output.push_str(&token);
        }
        rest = after_token;
    }

    output.push_str(rest);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::view::layout::ViewNodeDef;
    use bevy_fact_rule_event::LayeredFactDatabase;

    fn player_data(db: &LayeredFactDatabase) -> PlayerDataView<'_> {
        PlayerDataView::new(db)
    }

    #[test]
    fn regular_sprite_pivot_does_not_offset_desired_transform() {
        let node: ViewNodeDef = ron::from_str(
            r#"
            (
                name: "Cursor",
                sprite: Some((
                    visual: "common/view/heartsmall",
                    transform: Some((
                        translation: Some((-23.0, 22.5, 6.0)),
                    )),
                    pivot: Some((0.0, 1.0)),
                )),
            )
            "#,
        )
        .expect("node should parse");
        let db = LayeredFactDatabase::default();

        let transform = resolve_node_transform(&player_data(&db), &node, None);

        assert_eq!(transform.translation, Vec3::new(-23.0, 22.5, 6.0));
    }

    #[test]
    fn material_sprite_pivot_offsets_desired_transform() {
        let node: ViewNodeDef = ron::from_str(
            r#"
            (
                name: "HealthBar",
                sprite: Some((
                    visual: "procedural://white_pixel",
                    transform: Some((
                        translation: Some((10.0, 20.0, 0.0)),
                        scale: Some((100.0, 16.0, 1.0)),
                    )),
                    pivot: Some((0.0, 0.5)),
                    material: Some((
                        shader: "assets/shaders/hp_bar.wgsl",
                    )),
                )),
            )
            "#,
        )
        .expect("node should parse");
        let db = LayeredFactDatabase::default();

        let transform = resolve_node_transform(&player_data(&db), &node, None);

        assert_eq!(transform.translation, Vec3::new(60.0, 20.0, 0.0));
    }
}

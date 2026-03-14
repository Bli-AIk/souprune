use super::super::components::*;
use super::super::layout::*;
use super::super::sdf_view_shape::parse_text_preserving_whitespace;
use super::parsing::{
    PlayerDataView, evaluate_condition, evaluate_float_expr, evaluate_visible_when,
    resolve_text_content, vec3_tuple_depends_on_time,
};
use super::resources::RonDrivenView;
use crate::core::sprite::params::SpriteParams;
use bevy::prelude::*;
use crate::core::game_action::GameFreAsset;

/// Helper function to build ViewTextConfig from TextDef.
///
/// 从 TextDef 构建 ViewTextConfig 的辅助函数。
pub fn build_text_config(
    text_def: &TextDef,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &PlayerDataView<'_>,
    item_registry: &crate::core::item::ItemRegistry,
) -> ViewTextConfig {
    let raw_content = text_def.content.as_deref().unwrap_or("");
    info!(
        "[build_text_config] Building text config for '{}' with raw_content: '{}'",
        text_def.id, raw_content
    );

    let mut content = resolve_text_content(raw_content, mortar_strings, player_data, item_registry);

    info!(
        "[build_text_config] Resolved content for '{}': '{}'",
        text_def.id, content
    );

    let color = if let Some(conditional_style) = &text_def.conditional_style {
        let condition_met = evaluate_condition(&conditional_style.condition, player_data);
        if condition_met {
            let (r, g, b, a) = color_tuple_to_static(&conditional_style.color);
            let conditional_color = Srgba::new(r, g, b, a);
            content = format!(
                "{{#{:02x}{:02x}{:02x}:{}}}",
                (conditional_color.red * 255.0) as u8,
                (conditional_color.green * 255.0) as u8,
                (conditional_color.blue * 255.0) as u8,
                content
            );
            conditional_color
        } else {
            let (r, g, b, a) = color_tuple_to_static(&text_def.color);
            Srgba::new(r, g, b, a)
        }
    } else {
        let (r, g, b, a) = color_tuple_to_static(&text_def.color);
        Srgba::new(r, g, b, a)
    };

    ViewTextConfig {
        name: Name::new(text_def.id.clone()),
        content,
        template: Some(raw_content.to_string()),
        font: text_def.font.clone().into(),
        world_scale: {
            let (x, y) = vec2_tuple_to_static(&text_def.world_scale);
            Vec2::new(x, y)
        },
        color,
        transform: {
            let translation = if let Some(trans) = &text_def.transform.translation {
                Vec3::new(
                    evaluate_float_expr(&trans.0, player_data, None),
                    evaluate_float_expr(&trans.1, player_data, None),
                    evaluate_float_expr(&trans.2, player_data, None),
                )
            } else {
                Vec3::ZERO
            };
            let mut t = Transform::from_translation(translation);
            if let Some(scale) = &text_def.transform.scale {
                t.scale = Vec3::new(
                    evaluate_float_expr(&scale.0, player_data, None),
                    evaluate_float_expr(&scale.1, player_data, None),
                    evaluate_float_expr(&scale.2, player_data, None),
                );
            }
            if let Some(rot) = text_def.transform.rotation {
                t.rotation = Quat::from_rotation_z(rot.to_radians());
            }
            t
        },
        line_height: text_def.line_height.unwrap_or(1.0),
        visible_when: text_def.visible_when.clone(),
        ..Default::default()
    }
}

pub(crate) fn spawn_container_texts(
    parent: &mut ChildSpawnerCommands,
    texts: &[TextDef],
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &PlayerDataView<'_>,
    item_registry: &crate::core::item::ItemRegistry,
) {
    use bevy_rich_text3d::Text3dStyling;

    for text_def in texts {
        let text_config = build_text_config(text_def, mortar_strings, player_data, item_registry);

        info!(
            "[UI Container] Spawning text '{}' for container",
            text_config.name
        );

        let text3d = parse_text_preserving_whitespace(&text_config.content);

        let text_world_transform = text_config.transform;

        let mut cmd = parent.spawn((
            text_config.name.clone(),
            text3d,
            Text3dStyling {
                font: text_config.font.font_name().into(),
                size: text_config.font.default_size(),
                world_scale: Some(text_config.world_scale),
                color: text_config.color,
                align: text_config.align,
                anchor: text_config.anchor,
                line_height: text_config.line_height,
                ..Default::default()
            },
            Mesh2d::default(),
            // Use NeedsTextMaterial marker instead of default handle to avoid purple box
            // 使用 NeedsTextMaterial 标记而不是默认句柄以避免紫色方块
            super::super::text::NeedsTextMaterial,
            text_world_transform,
            Visibility::Hidden,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            super::super::text::NeedsGlyphRefresh,
            RonDrivenView,
        ));

        if let Some(template) = &text_config.template {
            cmd.insert(ViewTextTemplate(template.clone()));
        }

        // Add VisibleWhen component if text has visible_when expression
        // 如果文本有 visible_when 表达式则添加 VisibleWhen 组件
        if let Some(expr) = text_def
            .visible_when
            .as_deref()
            .map(str::trim)
            .filter(|e| !e.is_empty())
        {
            let is_visible = evaluate_visible_when(expr, player_data);

            let depth_value = player_data.get_fact_int("depth");
            info!(
                "Adding VisibleWhen to text '{}': '{}' -> {} (depth={:?}, has_local_facts={})",
                text_config.name,
                expr,
                is_visible,
                depth_value,
                player_data.local_facts().is_some()
            );

            cmd.insert(VisibleWhen {
                expression: expr.to_string(),
            });
            let visibility = if is_visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            cmd.insert(visibility);
        }

        // Add DynamicViewElement if transform has dynamic expressions
        let has_dynamic = text_def
            .transform
            .translation
            .as_ref()
            .is_some_and(is_dynamic_vec3)
            || text_def
                .transform
                .scale
                .as_ref()
                .is_some_and(is_dynamic_vec3);

        // Check for time dependency in text transform
        // 检查文本变换中的时间依赖
        let has_time_dependency = text_def
            .transform
            .translation
            .as_ref()
            .is_some_and(vec3_tuple_depends_on_time)
            || text_def
                .transform
                .scale
                .as_ref()
                .is_some_and(vec3_tuple_depends_on_time);

        if has_dynamic {
            cmd.insert(super::super::components::DynamicViewElement {
                sprite_def: None,
                text_def: Some(text_def.clone()),
            });

            // Add TimeDependentTransform marker if expression uses @time
            // 如果表达式使用 @time 则添加 TimeDependentTransform 标记
            if has_time_dependency {
                cmd.insert(super::super::components::TimeDependentTransform);
            }
        }
    }
}

pub(super) fn spawn_ui_sprite(
    parent: &mut EntityCommands,
    asset_server: &AssetServer,
    sprite_def: &SpriteDef,
    _sprite_params: &mut SpriteParams,
    node_name: &str,
    _animation_assets: &Assets<crate::core::character_asset::AnimationConfigAsset>,
    player_data: &PlayerDataView<'_>,
) {
    use crate::config::load_config;
    use crate::core::visual::{ResolvedVisual, get_asset_path, resolve_visual_path};

    let mut transform = Transform::default();
    if let Some(t_def) = &sprite_def.transform {
        if let Some(trans) = &t_def.translation {
            transform.translation = Vec3::new(
                evaluate_float_expr(&trans.0, player_data, None),
                evaluate_float_expr(&trans.1, player_data, None),
                evaluate_float_expr(&trans.2, player_data, None),
            );
        }
        if let Some(scale) = &t_def.scale {
            transform.scale = Vec3::new(
                evaluate_float_expr(&scale.0, player_data, None),
                evaluate_float_expr(&scale.1, player_data, None),
                evaluate_float_expr(&scale.2, player_data, None),
            );
        }
        if let Some(rot) = t_def.rotation {
            transform.rotation = Quat::from_rotation_z(rot.to_radians());
        }
    }

    let config = load_config();
    let visual_path = sprite_def.visual.path().to_owned();

    // Handle special protocol paths (e.g., "procedural://white_pixel")
    if visual_path.contains("://") {
        // Direct load for special protocols
        let texture_handle = asset_server.load(&visual_path);
        spawn_static_sprite(parent, sprite_def, texture_handle, transform, node_name);
        return;
    }

    // Use Visual's automatic type detection
    if let Some(resolved) = resolve_visual_path(&visual_path, &config.project.mod_name) {
        let asset_path = get_asset_path(&resolved, &config.project.mod_name);

        match resolved {
            ResolvedVisual::CharacterAnimation(_) => {
                // Character animation (.character.ron)
                let config_handle = asset_server
                    .load::<crate::core::character_asset::AnimationConfigAsset>(&asset_path);

                parent.with_children(|p| {
                    p.spawn((
                        crate::core::character_asset::CharacterAnimator {
                            config: config_handle,
                        },
                        ViewAnimationState {
                            state_name: sprite_def
                                .initial_state
                                .clone()
                                .unwrap_or("Idle".to_string()),
                        },
                        transform,
                        Visibility::default(),
                        Name::new(format!("{}_sprite", node_name)),
                    ));
                });
            }
            ResolvedVisual::Sprite(_) | ResolvedVisual::FrameAnimation(_) => {
                // Static sprite or frame animation (treat as static for now)
                let texture_handle = asset_server.load(&asset_path);
                spawn_static_sprite(parent, sprite_def, texture_handle, transform, node_name);
            }
        }
    } else {
        // Fallback: try direct load (for backwards compatibility with full paths)
        let texture_handle = asset_server.load(&visual_path);
        spawn_static_sprite(parent, sprite_def, texture_handle, transform, node_name);
    }
}

/// Helper function to spawn a static sprite with all properties.
fn spawn_static_sprite(
    parent: &mut EntityCommands,
    sprite_def: &SpriteDef,
    texture_handle: Handle<Image>,
    transform: Transform,
    node_name: &str,
) {
    let anchor_component = if let Some(pivot) = &sprite_def.pivot {
        let (x, y) = vec2_tuple_to_static(pivot);
        bevy::sprite::Anchor(Vec2::new(x - 0.5, y - 0.5))
    } else {
        bevy::sprite::Anchor(Vec2::ZERO)
    };

    parent.with_children(|p| {
        p.spawn((
            Sprite {
                image: texture_handle,
                flip_x: sprite_def.flip_x,
                flip_y: sprite_def.flip_y,
                color: sprite_def
                    .color
                    .as_ref()
                    .map(|c| {
                        let (r, g, b, a) = color_tuple_to_static(c);
                        Color::srgba(r, g, b, a)
                    })
                    .unwrap_or(Color::WHITE),
                ..Default::default()
            },
            anchor_component,
            transform,
            Visibility::default(),
            Name::new(format!("{}_sprite", node_name)),
        ));
    });
}

/// Helper function to spawn a standalone static sprite (not nested under a parent).
pub(super) fn spawn_standalone_static_sprite(
    parent: &mut ChildSpawnerCommands,
    sprite_def: &SpriteDef,
    view_element: &Option<ViewElement>,
    texture_handle: Handle<Image>,
    transform: Transform,
    node_name: &str,
    spawned_entity_id: &mut Option<Entity>,
    debug_path: &str,
) {
    let anchor_component = if let Some(pivot) = &sprite_def.pivot {
        let (pivot_x, pivot_y) = vec2_tuple_to_static(pivot);
        bevy::sprite::Anchor(Vec2::new(pivot_x - 0.5, pivot_y - 0.5))
    } else {
        bevy::sprite::Anchor(Vec2::ZERO)
    };

    let mut entity_cmd = parent.spawn((
        Sprite {
            image: texture_handle,
            flip_x: sprite_def.flip_x,
            flip_y: sprite_def.flip_y,
            color: sprite_def
                .color
                .as_ref()
                .map(|c| {
                    let (r, g, b, a) = color_tuple_to_static(c);
                    Color::srgba(r, g, b, a)
                })
                .unwrap_or(Color::WHITE),
            ..Default::default()
        },
        anchor_component,
        transform,
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Name::new(node_name.to_string()),
        RonDrivenView,
    ));
    if let Some(ve) = view_element {
        entity_cmd.insert(ve.clone());
    }

    let entity_id = entity_cmd.id();
    *spawned_entity_id = Some(entity_id);

    info!(
        "[UI Sprite] Spawned static sprite '{}' (Entity {:?}) with image: {:?}",
        node_name, entity_id, debug_path
    );
}

/// Resolve simple localization references in a string.
/// Format: {{path:KEY}} -> looks up "path:KEY" in mortar_strings
///
/// 解析字符串中的简单本地化引用。
/// 格式：{{path:KEY}} -> 在 mortar_strings 中查找 "path:KEY"
pub(super) fn resolve_simple_localization(
    s: &str,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
) -> String {
    // Check if the entire string is a localization reference
    if s.starts_with("{{") && s.ends_with("}}") && s.len() > 4 {
        let key = &s[2..s.len() - 2];
        if let Some(value) = mortar_strings.get(key) {
            return value.to_string();
        }
    }
    // Return original string if not a localization reference or not found
    s.to_string()
}

/// Load facts from a FreAsset into the ViewRoot's local_facts.
///
/// 将 FreAsset 中的事实加载到 ViewRoot 的 local_facts 中。
pub fn load_fre_into_view_root(
    view_root: &mut crate::core::view::components::ViewRoot,
    fre_asset: &GameFreAsset,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    enum_registry: &bevy_fact_rule_event::EnumRegistry,
) {
    use bevy_fact_rule_event::FactValue;

    for (key, fact_value) in fre_asset.resolve_facts(enum_registry) {
        match fact_value {
            FactValue::Int(i) => view_root.local_facts.set(key.clone(), i),
            FactValue::Float(f) => view_root.local_facts.set(key.clone(), f),
            FactValue::Bool(b) => view_root.local_facts.set(key.clone(), b),
            FactValue::String(s) => {
                let resolved = resolve_simple_localization(&s, mortar_strings);
                view_root.local_facts.set(key.clone(), resolved)
            }
            FactValue::StringList(list) => {
                let resolved_list: Vec<String> = list
                    .iter()
                    .map(|s| resolve_simple_localization(s, mortar_strings))
                    .collect();
                view_root.local_facts.set(key.clone(), resolved_list)
            }
            FactValue::IntList(list) => view_root.local_facts.set(key.clone(), list),
            FactValue::FloatList(list) => view_root.local_facts.set(key.clone(), list),
            FactValue::BoolList(list) => view_root.local_facts.set(key.clone(), list),
        }
    }
}

/// Load a procedural image handle for special protocols.
/// Returns a Handle<Image> for the requested procedural resource.
///
/// 加载程序化图像句柄用于特殊协议。
/// 返回请求的程序化资源的 Handle<Image>。
pub fn load_procedural_image_handle(
    visual_path: &str,
    asset_server: &AssetServer,
) -> Handle<Image> {
    // Currently we just pass it to asset_server which has custom handlers
    // for procedural:// protocol
    // We need to convert to owned string to avoid lifetime issues
    asset_server.load(visual_path.to_string())
}

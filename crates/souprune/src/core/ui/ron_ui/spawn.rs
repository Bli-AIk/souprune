use super::super::components::*;
use super::super::layout::*;
use super::super::smud_shape::parse_text_preserving_whitespace;
use super::parsing::{evaluate_condition, evaluate_float_expr, resolve_text_content};
use super::resources::{RonDrivenUI, UIGenerated, UILayoutHandle, UILayoutWatcher};
use crate::core::sprite::params::SpriteParams;
use bevy::prelude::*;

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn spawn_ron_ui_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_layout_handle: Option<Res<UILayoutHandle>>,
    ui_layouts: Res<Assets<UILayoutAsset>>,
    animation_assets: Res<Assets<crate::core::character_asset::AnimationConfigAsset>>,
    overworld_ui_query: Query<(Entity, &RonUI), (Without<UIGenerated>, Without<UIBox>)>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut sprite_params: SpriteParams,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    player_data: Res<crate::core::data::PlayerData>,
    item_registry: Res<crate::core::item::ItemRegistry>,
    mut watcher: Option<ResMut<UILayoutWatcher>>,
) {
    let Some(ui_layout_handle) = ui_layout_handle else {
        return;
    };

    let Some(ui_layout) = ui_layouts.get(&ui_layout_handle.handle) else {
        return;
    };

    let mut spawned_any = false;
    for (ui_entity, _ron_ui) in overworld_ui_query.iter() {
        info!("Spawning UI from RON layout");

        let camera_transform = match camera_query.single() {
            Ok(transform) => transform,
            Err(_) => {
                warn!("No Camera2d found for UI spawning!");
                return;
            }
        };

        spawn_ron_ui_for_entity(
            &mut commands,
            &asset_server,
            ui_entity,
            ui_layout,
            camera_transform,
            &mut sprite_params,
            &animation_assets,
            &mortar_strings,
            &player_data,
            &item_registry,
        );
        commands.entity(ui_entity).insert(UIGenerated);
        spawned_any = true;
    }

    if spawned_any && let Some(ref mut w) = watcher {
        w.pending_reload = false;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_ron_ui_for_entity(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_entity: Entity,
    ui_layout: &UILayoutAsset,
    camera_transform: &Transform,
    sprite_params: &mut SpriteParams,
    animation_assets: &Assets<crate::core::character_asset::AnimationConfigAsset>,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &crate::core::data::PlayerData,
    item_registry: &crate::core::item::ItemRegistry,
) {
    for root in &ui_layout.roots {
        spawn_ui_node(
            commands,
            asset_server,
            ui_entity,
            root,
            camera_transform,
            sprite_params,
            animation_assets,
            mortar_strings,
            player_data,
            item_registry,
            true, // Top-level nodes
        );
    }
}

/// Helper function to build UITextConfig from TextDef.
///
/// 从 TextDef 构建 UITextConfig 的辅助函数。
pub fn build_text_config(
    text_def: &TextDef,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &crate::core::data::PlayerData,
    item_registry: &crate::core::item::ItemRegistry,
) -> UITextConfig {
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
            let conditional_color = Srgba::new(
                conditional_style.color.r,
                conditional_style.color.g,
                conditional_style.color.b,
                conditional_style.color.a,
            );
            content = format!(
                "{{#{:02x}{:02x}{:02x}:{}}}",
                (conditional_color.red * 255.0) as u8,
                (conditional_color.green * 255.0) as u8,
                (conditional_color.blue * 255.0) as u8,
                content
            );
            conditional_color
        } else {
            Srgba::new(
                text_def.color.r,
                text_def.color.g,
                text_def.color.b,
                text_def.color.a,
            )
        }
    } else {
        Srgba::new(
            text_def.color.r,
            text_def.color.g,
            text_def.color.b,
            text_def.color.a,
        )
    };

    UITextConfig {
        name: Name::new(text_def.id.clone()),
        content,
        template: Some(raw_content.to_string()),
        font: text_def.font.clone().into(),
        world_scale: text_def.world_scale.clone().into(),
        color,
        transform: {
            let translation = Vec3::new(
                evaluate_float_expr(&text_def.transform.translation.x, player_data, None),
                evaluate_float_expr(&text_def.transform.translation.y, player_data, None),
                evaluate_float_expr(&text_def.transform.translation.z, player_data, None),
            );
            let mut t = Transform::from_translation(translation);
            if let Some(scale) = &text_def.transform.scale {
                t.scale = Vec3::new(
                    evaluate_float_expr(&scale.x, player_data, None),
                    evaluate_float_expr(&scale.y, player_data, None),
                    evaluate_float_expr(&scale.z, player_data, None),
                );
            }
            if let Some(rot) = text_def.transform.rotation {
                t.rotation = Quat::from_rotation_z(rot.to_radians());
            }
            t
        },
        line_height: text_def.line_height.unwrap_or(1.0),
        ..Default::default()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_ui_node(
    commands: &mut Commands,
    asset_server: &AssetServer,
    parent_entity: Entity,
    node_def: &UINodeDef,
    camera_transform: &Transform,
    sprite_params: &mut SpriteParams,
    animation_assets: &Assets<crate::core::character_asset::AnimationConfigAsset>,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &crate::core::data::PlayerData,
    item_registry: &crate::core::item::ItemRegistry,
    is_top_level: bool,
) {
    // Determine if this node has a UIBox (ui_shape_logic)
    let has_ui_box = node_def.ui_shape_logic.is_some();
    // Determine if this is a standalone sprite node (sprite without UIBox)
    let is_standalone_sprite = !has_ui_box && node_def.sprite.is_some();
    // Determine if this is a pure container (no UIBox, no standalone sprite, but may have texts/children)
    let is_pure_container = !has_ui_box
        && !is_standalone_sprite
        && (!node_def.texts.is_empty() || !node_def.children.is_empty());

    // Variable to track the spawned entity ID for recursive child processing
    // 用于追踪生成的实体 ID 以进行递归子节点处理的变量
    let mut spawned_entity_id: Option<Entity> = None;

    commands.entity(parent_entity).with_children(|parent| {
        // =====================================================================
        // Case 1: Standalone Sprite Node (no UIBox, has sprite)
        // 情况 1: 独立精灵节点（无 UIBox，有 sprite）
        // =====================================================================
        if is_standalone_sprite {
            let sprite_def = node_def.sprite.as_ref().unwrap();
            let mut transform = Transform::default();
            if let Some(t_def) = &sprite_def.transform {
                transform.translation = Vec3::new(
                    evaluate_float_expr(&t_def.translation.x, player_data, None),
                    evaluate_float_expr(&t_def.translation.y, player_data, None),
                    evaluate_float_expr(&t_def.translation.z, player_data, None),
                );
                if let Some(scale) = &t_def.scale {
                    transform.scale = Vec3::new(
                        evaluate_float_expr(&scale.x, player_data, None),
                        evaluate_float_expr(&scale.y, player_data, None),
                        evaluate_float_expr(&scale.z, player_data, None),
                    );
                }
                if let Some(rot) = t_def.rotation {
                    transform.rotation = Quat::from_rotation_z(rot.to_radians());
                }
            }


            info!(
                "[UI Sprite] Spawning standalone sprite '{}' at position: {:?}, scale: {:?}",
                node_def.name, transform.translation, transform.scale
            );

            if sprite_def.is_animation {
                let config_handle = asset_server
                    .load::<crate::core::character_asset::AnimationConfigAsset>(&sprite_def.path);

                parent.spawn((
                    crate::core::character_asset::CharacterAnimator {
                        config: config_handle,
                    },
                    UIAnimationState {
                        state_name: sprite_def
                            .initial_state
                            .clone()
                            .unwrap_or("Idle".to_string()),
                    },
                    transform,
                    Visibility::default(),
                    Name::new(node_def.name.clone()),
                    RonDrivenUI,
                ));
                info!("[UI Sprite] Spawned animated sprite '{}'", node_def.name);
            } else {
                // Check if using procedural texture or custom shader
                let use_custom_material = sprite_def.custom_shader.is_some();

                if use_custom_material {
                    // Use Material2d with custom shader (HP bar)
                    // Requires procedural textures resource
                    // 使用自定义着色器的 Material2d（HP 条）
                    // 需要程序生成纹理资源

                    // This will be handled by a separate system after ProceduralTextures is available
                    // For now, mark with a special component to be processed later
                    // 这将由单独的系统在 ProceduralTextures 可用后处理
                    // 现在用特殊组件标记以便稍后处理
                    let mut final_transform = Transform::from_translation(transform.translation)
                        .with_scale(transform.scale)
                        .with_rotation(transform.rotation);

                    if let Some(pivot) = &sprite_def.pivot {
                        let shift_x = (0.5 - pivot.x) * transform.scale.x;
                        let shift_y = (0.5 - pivot.y) * transform.scale.y;
                        let shift = transform.rotation * Vec3::new(shift_x, shift_y, 0.0);
                        final_transform.translation += shift;
                    }

                    let entity_id = parent
                        .spawn((
                            final_transform,
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                            Name::new(node_def.name.clone()),
                            RonDrivenUI,
                            HPBarSprite {
                                shader_params: sprite_def
                                    .shader_params
                                    .as_ref()
                                    .map(|c| c.to_static_color())
                                    .unwrap_or(Color::WHITE),
                            },
                        ))
                        .id();

                    // Store entity ID to add DynamicUIElement later outside closure
                    spawned_entity_id = Some(entity_id);

                    info!(
                        "[UI Sprite] Spawned HP bar sprite '{}' (Entity {:?}) - will apply material in setup system",
                        node_def.name, entity_id
                    );
                } else {
                    // Standard sprite
                    let texture_handle = if sprite_def.path.starts_with("procedural://") {
                        // This will be replaced by setup system
                        // For now use default handle
                        Handle::default()
                    } else {
                        asset_server.load(&sprite_def.path)
                    };

                    let anchor_component = if let Some(pivot) = &sprite_def.pivot {
                        bevy::sprite::Anchor(Vec2::new(pivot.x - 0.5, pivot.y - 0.5))
                    } else {
                        bevy::sprite::Anchor(Vec2::ZERO)
                    };

                    let entity_id = parent
                        .spawn((
                            Sprite {
                                image: texture_handle.clone(),
                                flip_x: sprite_def.flip_x,
                                flip_y: sprite_def.flip_y,
                                color: sprite_def
                                    .color
                                    .clone()
                                    .map(Color::from)
                                    .unwrap_or(Color::WHITE),
                                ..Default::default()
                            },
                            anchor_component,
                            transform,
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                            Name::new(node_def.name.clone()),
                            RonDrivenUI,
                        ))
                        .id();
                    info!(
                        "[UI Sprite] Spawned static sprite '{}' (Entity {:?}) with image: {:?}",
                        node_def.name, entity_id, sprite_def.path
                    );
                }
            }
            return;
        }

        // =====================================================================
        // Case 2: UIBox Node (has ui_shape_logic)
        // 情况 2: UIBox 节点（有 ui_shape_logic）
        // =====================================================================
        if has_ui_box {
            let ui_shape_logic = node_def.ui_shape_logic.as_ref().unwrap();
            info!(
                "[UI Box] Creating UIBox '{}' with dimensions: {}x{}, border: {}, offset: {:?}",
                node_def.name,
                ui_shape_logic.width,
                ui_shape_logic.height,
                ui_shape_logic.border_width,
                ui_shape_logic.offset
            );
            let visibility_rule = node_def
                .visibility_rule
                .as_ref()
                .map(parse_visibility_rule)
                .unwrap_or(UILayerVisibilityRule::Always);

            let texts = node_def
                .texts
                .iter()
                .map(|text_def| {
                    build_text_config(text_def, mortar_strings, player_data, item_registry)
                })
                .collect::<Vec<_>>();

            let offset = ui_shape_logic.offset.to_static_vec3();
            let dynamic_anchor = if ui_shape_logic.offset.x.as_expr().is_some()
                || ui_shape_logic.offset.y.as_expr().is_some()
                || ui_shape_logic.offset.z.as_expr().is_some()
            {
                Some(CameraAnchoredDynamic {
                    x_expression: ui_shape_logic.offset.x.as_expr().cloned(),
                    y_expression: ui_shape_logic.offset.y.as_expr().cloned(),
                    z_expression: ui_shape_logic.offset.z.as_expr().cloned(),
                })
            } else {
                None
            };

            // Convert fill color from RON definition
            // 从 RON 定义转换填充颜色
            let fill_color = ui_shape_logic
                .fill_color
                .as_ref()
                .map(|c| Color::srgba(c.r, c.g, c.b, c.a))
                .unwrap_or(Color::BLACK);

            let mut box_entity = if is_top_level {
                // Top-level nodes use CameraAnchored
                parent.spawn((
                    UIBox::new_full(
                        ui_shape_logic.width,
                        ui_shape_logic.height,
                        ui_shape_logic.border_width,
                        texts,
                        ui_shape_logic.fill_shader.clone(),
                        ui_shape_logic.structure_file.clone(),
                        fill_color,
                    ),
                    UIBoxVisibility::new(visibility_rule.clone()),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    CameraAnchoredBundle::from_camera_transform(camera_transform, offset),
                    Name::new(node_def.name.clone()),
                    RonDrivenUI,
                ))
            } else {
                // Child nodes use regular Transform relative to parent
                parent.spawn((
                    UIBox::new_full(
                        ui_shape_logic.width,
                        ui_shape_logic.height,
                        ui_shape_logic.border_width,
                        texts,
                        ui_shape_logic.fill_shader.clone(),
                        ui_shape_logic.structure_file.clone(),
                        fill_color,
                    ),
                    UIBoxVisibility::new(visibility_rule.clone()),
                    Transform::from_translation(offset),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    Name::new(node_def.name.clone()),
                    RonDrivenUI,
                ))
            };

            if node_def.tags.contains(&"BattleBox".to_string()) {
                box_entity.insert(crate::app_state::battle::collision::BattleBox);
                info!("[UI Box] Added BattleBox marker to '{}'", node_def.name);
            }

            info!(
                "[UI Box] Spawned UIBox '{}' at camera offset: {:?} with structure_file: {:?}",
                node_def.name, offset, ui_shape_logic.structure_file
            );

            if let Some(dynamic) = dynamic_anchor {
                box_entity.insert(dynamic);
                info!("[UI Box] Added dynamic anchor to '{}'", node_def.name);
            }

            if let Some(sprite_def) = &node_def.sprite {
                info!(
                    "[UI Box] Adding child sprite to UIBox '{}': {:?}",
                    node_def.name, sprite_def.path
                );
                spawn_ui_sprite(
                    &mut box_entity,
                    asset_server,
                    sprite_def,
                    sprite_params,
                    node_def.name.as_str(),
                    animation_assets,
                    player_data,
                );
            }

            if let Some(cursor_def) = &node_def.cursor {
                let mut sprite_context = sprite_params.create_sprite_context();
                let mut sprite = match sprite_context.get_sprite("common", "heartsmall") {
                    Ok(s) => s,
                    Err(e) => {
                        warn!(
                            "Failed to load cursor sprite 'common/heartsmall': {}. using default.",
                            e
                        );
                        sprite_context.get_missing_sprite()
                    }
                };
                sprite.color = Color::srgb(1.0, 0.0, 0.0);

                let cursor_position = if let Some(default_pos) = &cursor_def.default_translation {
                    match default_pos {
                        BoxCursorPositionDef::Static(vec) => {
                            BoxCursorPosition::Static(vec.to_static_vec3())
                        }
                        BoxCursorPositionDef::Linear { origin, step } => {
                            BoxCursorPosition::Linear {
                                origin: origin.to_static_vec3(),
                                step: step.to_static_vec3(),
                            }
                        }
                        BoxCursorPositionDef::Custom { positions } => BoxCursorPosition::Custom(
                            positions.iter().map(|v| v.to_static_vec3()).collect(),
                        ),
                    }
                } else if let Some(transform) = &cursor_def.transform {
                    if let Some(translation) = &transform.translation {
                        BoxCursorPosition::Static(translation.to_static_vec3())
                    } else {
                        BoxCursorPosition::Static(Vec3::ZERO)
                    }
                } else {
                    BoxCursorPosition::Static(Vec3::ZERO)
                };

                let cursor_visibility = if let Some(vis_rule) = &cursor_def.visibility_rule {
                    parse_visibility_rule(vis_rule)
                } else if let UILayerVisibilityRule::OnlyIn(ref layers) = visibility_rule {
                    BoxCursorVisibility::OnlyIn(layers.clone())
                } else {
                    BoxCursorVisibility::OnlyIn(vec![UILayer::BACKPACK_MENU])
                };

                let mut placement = BoxCursorPlacement::new(cursor_position);

                for (layer_name, position_def) in &cursor_def.overrides {
                    let layer = UILayer::new(layer_name.clone());
                    let position = match position_def {
                        BoxCursorPositionDef::Static(vec) => {
                            BoxCursorPosition::Static(vec.to_static_vec3())
                        }
                        BoxCursorPositionDef::Linear { origin, step } => {
                            BoxCursorPosition::Linear {
                                origin: origin.to_static_vec3(),
                                step: step.to_static_vec3(),
                            }
                        }
                        BoxCursorPositionDef::Custom { positions } => BoxCursorPosition::Custom(
                            positions.iter().map(|v| v.to_static_vec3()).collect(),
                        ),
                    };
                    placement = placement.with_override(layer, position);
                }

                let cursor_transform = if let Some(transform_def) = &cursor_def.transform {
                    let mut transform = Transform::default();
                    if let Some(scale) = &transform_def.scale {
                        transform.scale = scale.to_static_vec3();
                    } else {
                        transform.scale = Vec3::splat(1.0);
                    }
                    if let Some(rotation) = transform_def.rotation {
                        transform.rotation = Quat::from_rotation_z(rotation.to_radians());
                    }
                    transform
                } else {
                    Transform::from_scale(Vec3::splat(1.0))
                };

                box_entity.insert(BoxCursor::new(
                    sprite,
                    cursor_visibility,
                    placement,
                    cursor_transform,
                ));
            }

            // Store entity ID for recursive child processing after closure ends
            // 存储实体 ID 以便在闭包结束后进行递归子节点处理
            spawned_entity_id = Some(box_entity.id());
            return;
        }

        // =====================================================================
        // Case 3: Pure Container Node (no UIBox, no sprite, but has texts/children)
        // 情况 3: 纯容器节点（无 UIBox，无 sprite，但有 texts 或 children）
        // =====================================================================
        if is_pure_container {
            info!(
                "[UI Container] Creating pure container '{}' with {} texts and {} children",
                node_def.name,
                node_def.texts.len(),
                node_def.children.len()
            );

            let visibility_rule = node_def
                .visibility_rule
                .as_ref()
                .map(parse_visibility_rule)
                .unwrap_or(UILayerVisibilityRule::Always);

            // Spawn container entity with UIContainer marker
            // 使用 UIContainer 标记生成容器实体
            let mut container_entity = parent.spawn((
                UIContainer,
                UIContainerVisibility::new(visibility_rule),
                Transform::default(),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                CameraAnchored::new(Vec3::ZERO),
                Name::new(node_def.name.clone()),
                RonDrivenUI,
            ));

            // Spawn texts directly as children of the container
            // 将文本直接作为容器的子节点生成
            container_entity.with_children(|container_parent| {
                spawn_container_texts(
                    container_parent,
                    &node_def.texts,
                    mortar_strings,
                    player_data,
                    item_registry,
                    camera_transform,
                );
            });

            // Store entity ID for recursive child processing after closure ends
            // 存储实体 ID 以便在闭包结束后进行递归子节点处理
            spawned_entity_id = Some(container_entity.id());
        }
    });

    // Process children recursively AFTER the closure ends to avoid borrowing conflicts
    // 在闭包结束后递归处理子节点，以避免借用冲突
    if let Some(entity_id) = spawned_entity_id {
        // Add DynamicUIElement component if needed
        if is_standalone_sprite {
            let sprite_def = node_def.sprite.as_ref().unwrap();
            let has_dynamic = sprite_def.transform.as_ref().is_some_and(|t| {
                t.translation.is_dynamic() || t.scale.as_ref().is_some_and(|s| s.is_dynamic())
            }) || sprite_def
                .shader_params
                .as_ref()
                .is_some_and(|p| p.is_dynamic());

            if has_dynamic {
                info!("Adding DynamicUIElement to entity {:?}", entity_id);
                commands
                    .entity(entity_id)
                    .insert(super::super::components::DynamicUIElement {
                        sprite_def: Some(sprite_def.clone()),
                        text_def: None,
                    });
            }
        }

        for child_def in &node_def.children {
            spawn_ui_node(
                commands,
                asset_server,
                entity_id,
                child_def,
                camera_transform,
                sprite_params,
                animation_assets,
                mortar_strings,
                player_data,
                item_registry,
                false, // Child nodes are not top-level
            );
        }
    }
}

pub(crate) fn spawn_container_texts(
    parent: &mut ChildSpawnerCommands,
    texts: &[TextDef],
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &crate::core::data::PlayerData,
    item_registry: &crate::core::item::ItemRegistry,
    camera_transform: &Transform,
) {
    use bevy_rich_text3d::Text3dStyling;

    for text_def in texts {
        let text_config = build_text_config(text_def, mortar_strings, player_data, item_registry);

        info!(
            "[UI Container] Spawning text '{}' for container",
            text_config.name
        );

        let text3d = parse_text_preserving_whitespace(&text_config.content);

        // Calculate text position relative to camera
        // 计算相对于相机的文本位置
        let text_world_transform = Transform::from_translation(
            camera_transform.translation + text_config.transform.translation,
        )
        .with_rotation(text_config.transform.rotation)
        .with_scale(text_config.transform.scale);

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
            CameraAnchored::new(text_config.transform.translation),
            super::super::text::NeedsGlyphRefresh,
            RonDrivenUI,
        ));

        if let Some(template) = &text_config.template {
            cmd.insert(UITextTemplate(template.clone()));
        }

        // Add DynamicUIElement if transform has dynamic expressions
        let has_dynamic = text_def.transform.translation.is_dynamic()
            || text_def
                .transform
                .scale
                .as_ref()
                .is_some_and(|s| s.is_dynamic());

        if has_dynamic {
            cmd.insert(super::super::components::DynamicUIElement {
                sprite_def: None,
                text_def: Some(text_def.clone()),
            });
        }
    }
}

fn spawn_ui_sprite(
    parent: &mut EntityCommands,
    asset_server: &AssetServer,
    sprite_def: &SpriteDef,
    _sprite_params: &mut SpriteParams,
    node_name: &str,
    _animation_assets: &Assets<crate::core::character_asset::AnimationConfigAsset>,
    player_data: &crate::core::data::PlayerData,
) {
    let mut transform = Transform::default();
    if let Some(t_def) = &sprite_def.transform {
        transform.translation = Vec3::new(
            evaluate_float_expr(&t_def.translation.x, player_data, None),
            evaluate_float_expr(&t_def.translation.y, player_data, None),
            evaluate_float_expr(&t_def.translation.z, player_data, None),
        );
        if let Some(scale) = &t_def.scale {
            transform.scale = Vec3::new(
                evaluate_float_expr(&scale.x, player_data, None),
                evaluate_float_expr(&scale.y, player_data, None),
                evaluate_float_expr(&scale.z, player_data, None),
            );
        }
        if let Some(rot) = t_def.rotation {
            transform.rotation = Quat::from_rotation_z(rot.to_radians());
        }
    }

    if sprite_def.is_animation {
        let config_handle = asset_server
            .load::<crate::core::character_asset::AnimationConfigAsset>(&sprite_def.path);

        parent.with_children(|p| {
            p.spawn((
                crate::core::character_asset::CharacterAnimator {
                    config: config_handle,
                },
                UIAnimationState {
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
    } else {
        // Static sprite
        let texture_handle = asset_server.load(&sprite_def.path);

        let anchor_component = if let Some(pivot) = &sprite_def.pivot {
            bevy::sprite::Anchor(Vec2::new(pivot.x - 0.5, pivot.y - 0.5))
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
                        .clone()
                        .map(Color::from)
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
}

fn parse_visibility_rule(rule_def: &UIVisibilityRuleDef) -> UILayerVisibilityRule {
    match rule_def.rule_type.as_str() {
        "Always" => UILayerVisibilityRule::Always,
        "AlwaysHidden" => UILayerVisibilityRule::AlwaysHidden,
        "OnlyIn" => {
            if let Some(layers) = &rule_def.layers {
                let ui_layers = layers
                    .iter()
                    .map(|name| UILayer::new(name.clone()))
                    .collect();
                UILayerVisibilityRule::OnlyIn(ui_layers)
            } else {
                UILayerVisibilityRule::Always
            }
        }
        "Except" => {
            if let Some(layers) = &rule_def.layers {
                let ui_layers = layers
                    .iter()
                    .map(|name| UILayer::new(name.clone()))
                    .collect();
                UILayerVisibilityRule::Except(ui_layers)
            } else {
                UILayerVisibilityRule::Always
            }
        }
        _ => UILayerVisibilityRule::Always,
    }
}

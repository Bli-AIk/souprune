use super::super::components::*;
use super::super::layout::*;
use super::super::lifecycle::BackpackViewRoot;
use super::super::sdf_view_shape::parse_text_preserving_whitespace;
use super::parsing::{
    PlayerDataView, evaluate_condition, evaluate_float_expr, resolve_text_content,
};
use super::resources::{RonDrivenView, ViewGenerated, ViewLayoutHandle, ViewLayoutWatcher};
use crate::app_state::battle::BattleViewRoot;
use crate::app_state::overworld::chase::ChaseHUDRoot;
use crate::core::sprite::params::SpriteParams;
use crate::extra::debug::DebugCamera;
use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;

/// System to spawn view elements from RON layout.
///
/// 从 RON 布局生成视图元素的系统。
///
/// This system handles all UI root types:
/// - BackpackViewRoot: OW Backpack
/// - BattleViewRoot: Battle UI
/// - ChaseHUDRoot: Chase HUD
///
/// 该系统处理所有 UI 根类型：
/// - BackpackViewRoot：OW 背包
/// - BattleViewRoot：Battle UI
/// - ChaseHUDRoot：Chase HUD
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn spawn_ron_view_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    view_layout_handle: Option<Res<ViewLayoutHandle>>,
    view_layouts: Res<Assets<ViewLayoutAsset>>,
    animation_assets: Res<Assets<crate::core::character_asset::AnimationConfigAsset>>,
    backpack_root_query: Query<
        Entity,
        (
            With<BackpackViewRoot>,
            Without<ViewGenerated>,
            Without<ViewBox>,
        ),
    >,
    battle_root_query: Query<
        Entity,
        (
            With<BattleViewRoot>,
            Without<ViewGenerated>,
            Without<ViewBox>,
        ),
    >,
    chase_root_query: Query<Entity, (With<ChaseHUDRoot>, Without<ViewGenerated>, Without<ViewBox>)>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<DebugCamera>)>,
    mut sprite_params: SpriteParams,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    layered_db: Res<LayeredFactDatabase>,
    item_registry: Res<crate::core::item::ItemRegistry>,
    mut watcher: Option<ResMut<ViewLayoutWatcher>>,
) {
    let player_data = PlayerDataView::new(&layered_db);

    let Some(view_layout_handle) = view_layout_handle else {
        return;
    };

    let Some(view_layout) = view_layouts.get(&view_layout_handle.handle) else {
        return;
    };

    let mut spawned_any = false;

    // Helper closure to spawn view for an entity
    let mut spawn_for_entity = |view_entity: Entity, label: &str| {
        info!("Spawning view from RON layout ({})", label);

        let camera_transform = match camera_query.single() {
            Ok(transform) => transform,
            Err(_) => {
                warn!("No Camera2d found for view spawning!");
                return false;
            }
        };

        spawn_ron_view_for_entity(
            &mut commands,
            &asset_server,
            view_entity,
            view_layout,
            camera_transform,
            &mut sprite_params,
            &animation_assets,
            &mortar_strings,
            &player_data,
            &item_registry,
            &view_layout_handle.path,
        );
        commands.entity(view_entity).insert(ViewGenerated);
        true
    };

    // Handle BackpackViewRoot entities (OW Backpack)
    // 处理 BackpackViewRoot 实体（OW 背包）
    for view_entity in backpack_root_query.iter() {
        if spawn_for_entity(view_entity, "BackpackViewRoot") {
            spawned_any = true;
        }
    }

    // Handle BattleViewRoot entities (Battle UI)
    // 处理 BattleViewRoot 实体（Battle UI）
    for view_entity in battle_root_query.iter() {
        if spawn_for_entity(view_entity, "BattleViewRoot") {
            spawned_any = true;
        }
    }

    // Handle ChaseHUDRoot entities (Chase HUD)
    // 处理 ChaseHUDRoot 实体（Chase HUD）
    for view_entity in chase_root_query.iter() {
        if spawn_for_entity(view_entity, "ChaseHUDRoot") {
            spawned_any = true;
        }
    }

    if spawned_any && let Some(ref mut w) = watcher {
        w.pending_reload = false;
    }
}

/// Spawn view elements for a specific entity.
///
/// 为特定实体生成视图元素。
#[allow(clippy::too_many_arguments)]
pub fn spawn_ron_view_for_entity(
    commands: &mut Commands,
    asset_server: &AssetServer,
    view_entity: Entity,
    view_layout: &ViewLayoutAsset,
    camera_transform: &Transform,
    sprite_params: &mut SpriteParams,
    animation_assets: &Assets<crate::core::character_asset::AnimationConfigAsset>,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &PlayerDataView<'_>,
    item_registry: &crate::core::item::ItemRegistry,
    layout_path: &str,
) {
    // Generate namespace from layout path
    // 从布局路径生成命名空间
    let namespace = crate::core::view::components::ViewRoot::namespace_from_path(layout_path);

    // Attach ViewRoot to the view entity
    // 为视图实体附加 ViewRoot 组件
    commands
        .entity(view_entity)
        .insert(crate::core::view::components::ViewRoot::new(
            layout_path.to_string(),
        ));

    // Spawn InteractiveLayers if defined
    // 如果定义了交互层则生成
    if let Some(interactive_layers) = &view_layout.interactive_layers {
        for (layer_id, layer_def) in interactive_layers {
            let interactive_layer = layer_def.build(layer_id, player_data);
            info!(
                "Creating InteractiveLayer '{}' with navigator: {:?}",
                layer_id, interactive_layer.navigator
            );
            // Add Name component so state_sprite system can identify this layer
            // 添加 Name 组件以便 state_sprite 系统能识别此层
            commands.spawn((
                interactive_layer,
                Name::new(format!("InteractiveLayer:{}", layer_id)),
            ));
        }
    }

    for root in &view_layout.roots {
        spawn_view_node(
            commands,
            asset_server,
            view_entity,
            root,
            camera_transform,
            sprite_params,
            animation_assets,
            mortar_strings,
            player_data,
            item_registry,
            true,       // Top-level nodes
            &namespace, // Pass namespace to children
        );
    }
}

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
        ..Default::default()
    }
}

/// Spawn a single view node and its children.
///
/// 生成单个视图节点及其子节点。
#[allow(clippy::too_many_arguments)]
pub fn spawn_view_node(
    commands: &mut Commands,
    asset_server: &AssetServer,
    parent_entity: Entity,
    node_def: &ViewNodeDef,
    camera_transform: &Transform,
    sprite_params: &mut SpriteParams,
    animation_assets: &Assets<crate::core::character_asset::AnimationConfigAsset>,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &PlayerDataView<'_>,
    item_registry: &crate::core::item::ItemRegistry,
    is_top_level: bool,
    namespace: &str, // New parameter: namespace for ViewElement
) {
    // Determine if this node has a ViewBox (ui_shape_logic)
    let has_ui_box = node_def.ui_shape_logic.is_some();
    // Determine if this is a standalone sprite node (sprite without ViewBox)
    let is_standalone_sprite = !has_ui_box && node_def.sprite.is_some();
    // Determine if this is a state sprite node
    let is_state_sprite = !has_ui_box && node_def.state_sprite.is_some();
    // Determine if this is a pure container (no ViewBox, no standalone sprite, but may have texts/children)
    let is_pure_container = !has_ui_box
        && !is_standalone_sprite
        && !is_state_sprite
        && (!node_def.texts.is_empty() || !node_def.children.is_empty());

    // Create ViewElement for named nodes
    // 为具名节点创建 ViewElement
    let view_element = if !node_def.name.is_empty() {
        Some(crate::core::view::components::ViewElement::new(
            namespace.to_string(),
            node_def.name.clone(),
            node_def.tags.clone(),
        ))
    } else {
        None
    };

    // Variable to track the spawned entity ID for recursive child processing
    // 用于追踪生成的实体 ID 以进行递归子节点处理的变量
    let mut spawned_entity_id: Option<Entity> = None;

    commands.entity(parent_entity).with_children(|parent| {
        // =====================================================================
        // Case 0: State Sprite Node (data-driven state-based sprite)
        // 情况 0: 状态精灵节点（数据驱动的状态切换精灵）
        // =====================================================================
        if is_state_sprite {
            let state_sprite_config = node_def.state_sprite.as_ref().unwrap();
            let mut transform = Transform::default();
            if let Some(t_def) = &state_sprite_config.transform {
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

            info!(
                "[State Sprite] Spawning state sprite '{}' at position: {:?}",
                node_def.name, transform.translation
            );

            // Create StateSpriteState from config
            let state_sprite_state = StateSpriteState::from_config(state_sprite_config);

            // Load default texture
            let texture_handle: Handle<Image> = asset_server.load(&state_sprite_config.default);

            let mut entity_cmd = parent.spawn((
                Sprite {
                    image: texture_handle,
                    ..Default::default()
                },
                transform,
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                Name::new(node_def.name.clone()),
                RonDrivenView,
                state_sprite_state,
            ));

            // Attach ViewElement if the node has a name
            if let Some(ref ve) = view_element {
                entity_cmd.insert(ve.clone());
            }

            let entity_id = entity_cmd.id();
            spawned_entity_id = Some(entity_id);

            info!(
                "[State Sprite] Spawned state sprite '{}' (Entity {:?})",
                node_def.name, entity_id
            );
            return;
        }

        // =====================================================================
        // Case 1: Standalone Sprite Node (no ViewBox, has sprite)
        // 情况 1: 独立精灵节点（无 ViewBox，有 sprite）
        // =====================================================================
        if is_standalone_sprite {
            let sprite_def = node_def.sprite.as_ref().unwrap();
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


            info!(
                "[UI Sprite] Spawning standalone sprite '{}' at position: {:?}, scale: {:?}",
                node_def.name, transform.translation, transform.scale
            );

            if sprite_def.is_animation {
                let config_handle = asset_server
                    .load::<crate::core::character_asset::AnimationConfigAsset>(&sprite_def.path);

                let mut entity_cmd = parent.spawn((
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
                    Name::new(node_def.name.clone()),
                    RonDrivenView,
                ));
                // Attach ViewElement if the node has a name
                // 如果节点有名称，则附加 ViewElement
                if let Some(ref ve) = view_element {
                    entity_cmd.insert(ve.clone());
                }
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
                        let (pivot_x, pivot_y) = vec2_tuple_to_static(pivot);
                        let shift_x = (0.5 - pivot_x) * transform.scale.x;
                        let shift_y = (0.5 - pivot_y) * transform.scale.y;
                        let shift = transform.rotation * Vec3::new(shift_x, shift_y, 0.0);
                        final_transform.translation += shift;
                    }

                    let mut entity_cmd = parent.spawn((
                        final_transform,
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Name::new(node_def.name.clone()),
                        RonDrivenView,
                        HPBarSprite {
                            shader_params: sprite_def
                                .shader_params
                                .as_ref()
                                .map(dynamic_color_to_static)
                                .unwrap_or(Color::WHITE),
                        },
                    ));
                    // Attach ViewElement if the node has a name
                    // 如果节点有名称，则附加 ViewElement
                    if let Some(ref ve) = view_element {
                        entity_cmd.insert(ve.clone());
                    }

                    let entity_id = entity_cmd.id();

                    // Store entity ID to add DynamicViewElement later outside closure
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
                        let (pivot_x, pivot_y) = vec2_tuple_to_static(pivot);
                        bevy::sprite::Anchor(Vec2::new(pivot_x - 0.5, pivot_y - 0.5))
                    } else {
                        bevy::sprite::Anchor(Vec2::ZERO)
                    };

                    let mut entity_cmd = parent.spawn((
                        Sprite {
                            image: texture_handle.clone(),
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
                        Name::new(node_def.name.clone()),
                        RonDrivenView,
                    ));
                    // Attach ViewElement if the node has a name
                    // 如果节点有名称，则附加 ViewElement
                    if let Some(ref ve) = view_element {
                        entity_cmd.insert(ve.clone());
                    }

                    let entity_id = entity_cmd.id();

                    spawned_entity_id = Some(entity_id);

                    info!(
                        "[UI Sprite] Spawned static sprite '{}' (Entity {:?}) with image: {:?}",
                        node_def.name, entity_id, sprite_def.path
                    );
                }
            }
            return;
        }

        // =====================================================================
        // Case 2: ViewBox Node (has ui_shape_logic)
        // 情况 2: ViewBox 节点（有 ui_shape_logic）
        // =====================================================================
        if has_ui_box {
            let ui_shape_logic = node_def.ui_shape_logic.as_ref().unwrap();
            info!(
                "[UI Box] Creating ViewBox '{}' with dimensions: {}x{}, border: {}, offset: {:?}",
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
                .unwrap_or(ViewLayerVisibilityRule::Always);

            let texts = node_def
                .texts
                .iter()
                .map(|text_def| {
                    build_text_config(text_def, mortar_strings, player_data, item_registry)
                })
                .collect::<Vec<_>>();

            let offset = serializable_vec3_to_static(&ui_shape_logic.offset);
            let dynamic_anchor = if ui_shape_logic.offset.0.as_expr().is_some()
                || ui_shape_logic.offset.1.as_expr().is_some()
                || ui_shape_logic.offset.2.as_expr().is_some()
            {
                Some(CameraAnchoredDynamic {
                    x_expression: ui_shape_logic.offset.0.as_expr().map(|s| s.to_string()),
                    y_expression: ui_shape_logic.offset.1.as_expr().map(|s| s.to_string()),
                    z_expression: ui_shape_logic.offset.2.as_expr().map(|s| s.to_string()),
                })
            } else {
                None
            };

            // Convert fill color from RON definition
            // 从 RON 定义转换填充颜色
            let fill_color = ui_shape_logic
                .fill_color
                .as_ref()
                .map(|c| {
                    let (r, g, b, a) = color_tuple_to_static(c);
                    Color::srgba(r, g, b, a)
                })
                .unwrap_or(Color::BLACK);

            let mut box_entity = if is_top_level {
                // Top-level nodes use CameraAnchored
                let mut entity_cmd = parent.spawn((
                    ViewBox::new_full(
                        ui_shape_logic.width,
                        ui_shape_logic.height,
                        ui_shape_logic.border_width,
                        texts,
                        ui_shape_logic.fill_shader.clone(),
                        ui_shape_logic.structure_file.clone(),
                        fill_color,
                    ),
                    ViewBoxVisibility::new(visibility_rule.clone()),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    CameraAnchoredBundle::from_camera_transform(camera_transform, offset),
                    Name::new(node_def.name.clone()),
                    RonDrivenView,
                ));
                // Attach ViewElement if the node has a name
                // 如果节点有名称，则附加 ViewElement
                if let Some(ref ve) = view_element {
                    entity_cmd.insert(ve.clone());
                }
                entity_cmd
            } else {
                // Child nodes use regular Transform relative to parent
                let mut entity_cmd = parent.spawn((
                    ViewBox::new_full(
                        ui_shape_logic.width,
                        ui_shape_logic.height,
                        ui_shape_logic.border_width,
                        texts,
                        ui_shape_logic.fill_shader.clone(),
                        ui_shape_logic.structure_file.clone(),
                        fill_color,
                    ),
                    ViewBoxVisibility::new(visibility_rule.clone()),
                    Transform::from_translation(offset),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    Name::new(node_def.name.clone()),
                    RonDrivenView,
                ));
                // Attach ViewElement if the node has a name
                // 如果节点有名称，则附加 ViewElement
                if let Some(ref ve) = view_element {
                    entity_cmd.insert(ve.clone());
                }
                entity_cmd
            };

            if node_def.tags.contains(&"BattleBox".to_string()) {
                box_entity.insert(crate::app_state::battle::collision::BattleBox);
                info!("[UI Box] Added BattleBox marker to '{}'", node_def.name);
            }

            info!(
                "[UI Box] Spawned ViewBox '{}' at camera offset: {:?} with structure_file: {:?}",
                node_def.name, offset, ui_shape_logic.structure_file
            );

            if let Some(dynamic) = dynamic_anchor {
                box_entity.insert(dynamic);
                info!("[UI Box] Added dynamic anchor to '{}'", node_def.name);
            }

            if let Some(sprite_def) = &node_def.sprite {
                info!(
                    "[UI Box] Adding child sprite to ViewBox '{}': {:?}",
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

            // Process reactive indicator definition (selection indicator, etc.)
            // 处理响应式指示器定义（选择指示器等）
            if let Some(indicator_def) = &node_def.reactive_indicator {
                let mut sprite_context = sprite_params.create_sprite_context();
                let mut sprite = match sprite_context.get_sprite("common", "heartsmall") {
                    Ok(s) => s,
                    Err(e) => {
                        warn!(
                            "Failed to load indicator sprite 'common/heartsmall': {}. using default.",
                            e
                        );
                        sprite_context.get_missing_sprite()
                    }
                };
                sprite.color = Color::srgb(1.0, 0.0, 0.0);

                let indicator_position = if let Some(default_pos) = &indicator_def.default_translation {
                    match default_pos {
                        ReactivePositionDef::Static(vec) => {
                            ReactivePosition::Static(serializable_vec3_to_static(vec))
                        }
                        ReactivePositionDef::Linear { origin, step } => {
                            ReactivePosition::Linear {
                                origin: serializable_vec3_to_static(origin),
                                step: serializable_vec3_to_static(step),
                            }
                        }
                        ReactivePositionDef::Custom { positions } => ReactivePosition::Custom(
                            positions.iter().map(serializable_vec3_to_static).collect(),
                        ),
                    }
                } else if let Some(transform) = &indicator_def.transform {
                    if let Some(translation) = &transform.translation {
                        ReactivePosition::Static(serializable_vec3_to_static(translation))
                    } else {
                        ReactivePosition::Static(Vec3::ZERO)
                    }
                } else {
                    ReactivePosition::Static(Vec3::ZERO)
                };

                let indicator_visibility = if let Some(vis_rule) = &indicator_def.visibility_rule {
                    parse_visibility_rule(vis_rule)
                } else if let ViewLayerVisibilityRule::OnlyIn(ref layers) = visibility_rule {
                    ReactiveIndicatorVisibility::OnlyIn(layers.clone())
                } else {
                    // Default to always visible if no visibility rule is specified
                    // 如果未指定可见性规则，默认始终可见
                    ReactiveIndicatorVisibility::Always
                };

                let mut placement = ReactivePlacement::new(indicator_position);

                for (layer_name, position_def) in &indicator_def.overrides {
                    let layer = ViewLayer::new(layer_name.clone());
                    let position = match position_def {
                        ReactivePositionDef::Static(vec) => {
                            ReactivePosition::Static(serializable_vec3_to_static(vec))
                        }
                        ReactivePositionDef::Linear { origin, step} => {
                            ReactivePosition::Linear {
                                origin: serializable_vec3_to_static(origin),
                                step: serializable_vec3_to_static(step),
                            }
                        }
                        ReactivePositionDef::Custom { positions } => ReactivePosition::Custom(
                            positions.iter().map(serializable_vec3_to_static).collect(),
                        ),
                    };
                    placement = placement.with_override(layer, position);
                }

                let indicator_transform = if let Some(transform_def) = &indicator_def.transform {
                    let mut transform = Transform::default();
                    if let Some(scale) = &transform_def.scale {
                        transform.scale = serializable_vec3_to_static(scale);
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

                box_entity.insert(ReactiveIndicator::new(
                    sprite,
                    indicator_visibility,
                    placement,
                    indicator_transform,
                ));
            }

            // Store entity ID for recursive child processing after closure ends
            // 存储实体 ID 以便在闭包结束后进行递归子节点处理
            spawned_entity_id = Some(box_entity.id());
            return;
        }

        // =====================================================================
        // Case 3: Pure Container Node (no ViewBox, no sprite, but has texts/children)
        // 情况 3: 纯容器节点（无 ViewBox，无 sprite，但有 texts 或 children）
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
                .unwrap_or(ViewLayerVisibilityRule::Always);

            // Spawn container entity with ViewContainer marker
            // 使用 ViewContainer 标记生成容器实体
            let mut container_entity = parent.spawn((
                ViewContainer,
                ViewContainerVisibility::new(visibility_rule),
                Transform::default(),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                CameraAnchored::new(Vec3::ZERO),
                Name::new(node_def.name.clone()),
                RonDrivenView,
            ));
            // Attach ViewElement if the node has a name
            // 如果节点有名称，则附加 ViewElement
            if let Some(ref ve) = view_element {
                container_entity.insert(ve.clone());
            }

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
    info!("After closure, spawned_entity_id: {:?}", spawned_entity_id);
    if let Some(entity_id) = spawned_entity_id {
        // Add DynamicViewElement component if needed
        if is_standalone_sprite {
            let sprite_def = node_def.sprite.as_ref().unwrap();

            let mut has_dynamic = false;
            if let Some(t) = &sprite_def.transform {
                if let Some(trans) = &t.translation {
                    let tx = trans.0.is_dynamic();
                    let ty = trans.1.is_dynamic();
                    let tz = trans.2.is_dynamic();
                    info!(
                        "Checking dynamics for {}: x={}, y={}, z={}",
                        node_def.name, tx, ty, tz
                    );

                    if tx || ty || tz {
                        has_dynamic = true;
                    }
                }
                if let Some(s) = &t.scale
                    && (s.0.is_dynamic() || s.1.is_dynamic() || s.2.is_dynamic())
                {
                    has_dynamic = true;
                }
            }
            if sprite_def
                .shader_params
                .as_ref()
                .is_some_and(is_dynamic_color)
            {
                has_dynamic = true;
            }

            if has_dynamic {
                info!(
                    "Adding DynamicViewElement to entity {:?} ({})",
                    entity_id, node_def.name
                );
                commands
                    .entity(entity_id)
                    .insert(super::super::components::DynamicViewElement {
                        sprite_def: Some(sprite_def.clone()),
                        text_def: None,
                    });
            } else {
                info!("No dynamic properties found for {}", node_def.name);
            }
        }

        for child_def in &node_def.children {
            spawn_view_node(
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
                false,
                namespace,
            );
        }
    }
}

pub(crate) fn spawn_container_texts(
    parent: &mut ChildSpawnerCommands,
    texts: &[TextDef],
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &PlayerDataView<'_>,
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
            RonDrivenView,
        ));

        if let Some(template) = &text_config.template {
            cmd.insert(ViewTextTemplate(template.clone()));
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

        if has_dynamic {
            cmd.insert(super::super::components::DynamicViewElement {
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
    player_data: &PlayerDataView<'_>,
) {
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

    if sprite_def.is_animation {
        let config_handle = asset_server
            .load::<crate::core::character_asset::AnimationConfigAsset>(&sprite_def.path);

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
    } else {
        // Static sprite
        let texture_handle = asset_server.load(&sprite_def.path);

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
}

fn parse_visibility_rule(rule_def: &UIVisibilityRuleDef) -> ViewLayerVisibilityRule {
    match rule_def.rule_type.as_str() {
        "Always" => ViewLayerVisibilityRule::Always,
        "AlwaysHidden" => ViewLayerVisibilityRule::AlwaysHidden,
        "OnlyIn" => {
            if let Some(layers) = &rule_def.layers {
                let ui_layers = layers
                    .iter()
                    .map(|name| ViewLayer::new(name.clone()))
                    .collect();
                ViewLayerVisibilityRule::OnlyIn(ui_layers)
            } else {
                ViewLayerVisibilityRule::Always
            }
        }
        "Except" => {
            if let Some(layers) = &rule_def.layers {
                let ui_layers = layers
                    .iter()
                    .map(|name| ViewLayer::new(name.clone()))
                    .collect();
                ViewLayerVisibilityRule::Except(ui_layers)
            } else {
                ViewLayerVisibilityRule::Always
            }
        }
        _ => ViewLayerVisibilityRule::Always,
    }
}

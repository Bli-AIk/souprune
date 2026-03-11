use super::super::components::*;
use super::super::layout::*;
use super::parsing::{
    PlayerDataView, evaluate_float_expr_with_repeat, evaluate_visible_when,
    preprocess_sprite_def_for_repeat, vec3_tuple_depends_on_time,
};
use super::resources::RonDrivenView;
use super::spawn_helpers::{
    build_text_config, spawn_container_texts, spawn_standalone_static_sprite, spawn_ui_sprite,
};
use crate::core::sprite::params::SpriteParams;
use bevy::prelude::*;

/// Spawn a single view node and its children.
///
/// 生成单个视图节点及其子节点。
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
    // Handle repeat configuration - spawn multiple instances from array
    if let Some(repeat) = &node_def.repeat {
        // Get array length from source
        let array_len = if let Some(list) = player_data.get_fact_string_list(&repeat.source) {
            list.len()
        } else if let Some(list) = player_data.get_fact_int_list(&repeat.source) {
            list.len()
        } else {
            warn!(
                "[spawn_view_node] Repeat source '{}' not found for node '{}'",
                repeat.source, node_def.name
            );
            0
        };

        let limit = repeat.limit.unwrap_or(usize::MAX);
        let count = array_len.min(limit);

        info!(
            "[spawn_view_node] Repeating node '{}' {} times (source: '{}', len: {}, limit: {:?})",
            node_def.name, count, repeat.source, array_len, repeat.limit
        );

        for i in 0..count {
            // Create repeat context for this iteration
            let mut ctx = super::parsing::RepeatContext::new(i);

            // Get item value from array if item_var is specified
            if let Some(item_var) = &repeat.item_var
                && let Some(value) = resolve_repeat_item(player_data, &repeat.source, i)
            {
                ctx = ctx.with_item(item_var, value);
            }

            // Spawn with context
            spawn_view_node_with_repeat_context(
                commands,
                asset_server,
                parent_entity,
                node_def,
                camera_transform,
                sprite_params,
                animation_assets,
                mortar_strings,
                player_data,
                item_registry,
                is_top_level,
                namespace,
                Some(&ctx),
            );
        }
        return;
    }

    // No repeat - spawn normally without context
    spawn_view_node_with_repeat_context(
        commands,
        asset_server,
        parent_entity,
        node_def,
        camera_transform,
        sprite_params,
        animation_assets,
        mortar_strings,
        player_data,
        item_registry,
        is_top_level,
        namespace,
        None,
    );
}

/// Internal function to spawn a single view node with optional repeat context.
fn spawn_view_node_with_repeat_context(
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
    namespace: &str,
    repeat_ctx: Option<&super::parsing::RepeatContext>,
) {
    // Determine if this node has a ViewBox (view_box)
    let has_ui_box = node_def.view_box.is_some();
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
    // If repeat context exists, append index to name for uniqueness
    let node_name = if let Some(ctx) = repeat_ctx {
        if !node_def.name.is_empty() {
            format!("{}_{}", node_def.name, ctx.index)
        } else {
            String::new()
        }
    } else {
        node_def.name.clone()
    };

    let view_element = if !node_name.is_empty() {
        Some(crate::core::view::components::ViewElement::new(
            namespace.to_string(),
            node_name.clone(),
            node_def.tags.clone(),
        ))
    } else {
        None
    };

    // Variable to track the spawned entity ID for recursive child processing
    let mut spawned_entity_id: Option<Entity> = None;

    commands.entity(parent_entity).with_children(|parent| {
        // =====================================================================
        // Case 0: State Sprite Node (data-driven state-based sprite)
        // =====================================================================
        if is_state_sprite {
            let state_sprite_config = node_def
                .state_sprite
                .as_ref()
                .expect("state_sprite must exist when is_state_sprite is true");
            let transform = state_sprite_config
                .transform
                .as_ref()
                .map(|t| build_transform(t, player_data, None))
                .unwrap_or_default();

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
        // =====================================================================
        if is_standalone_sprite {
            let sprite_def = node_def
                .sprite
                .as_ref()
                .expect("sprite must exist when is_standalone_sprite is true");
            let transform = sprite_def
                .transform
                .as_ref()
                .map(|t| build_transform(t, player_data, repeat_ctx))
                .unwrap_or_default();

            info!(
                "[UI Sprite] Spawning standalone sprite '{}' at position: {:?}, scale: {:?}",
                node_name, transform.translation, transform.scale
            );

            let visual_path = sprite_def.visual.path().to_owned();

            spawned_entity_id = Some(spawn_standalone_sprite_node(
                parent,
                asset_server,
                sprite_def,
                &view_element,
                &visual_path,
                transform,
                &node_def.name,
                repeat_ctx,
            ));
            return;
        }

        // =====================================================================
        // Case 2: ViewBox Node (has view_box)
        // =====================================================================
        if has_ui_box {
            let view_box = node_def
                .view_box
                .as_ref()
                .expect("view_box must exist when has_ui_box is true");
            info!(
                "[UI Box] Creating ViewBox '{}' with dimensions: {}x{}, border: {}, offset: {:?}",
                node_def.name,
                view_box.width,
                view_box.height,
                view_box.border_width,
                view_box.offset
            );

            let texts = node_def
                .texts
                .iter()
                .map(|text_def| {
                    build_text_config(text_def, mortar_strings, player_data, item_registry)
                })
                .collect::<Vec<_>>();

            let offset = serializable_vec3_to_static(&view_box.offset);
            let dynamic_anchor = if view_box.offset.0.as_expr().is_some()
                || view_box.offset.1.as_expr().is_some()
                || view_box.offset.2.as_expr().is_some()
            {
                Some(CameraAnchoredDynamic {
                    x_expression: view_box.offset.0.as_expr().map(|s| s.to_string()),
                    y_expression: view_box.offset.1.as_expr().map(|s| s.to_string()),
                    z_expression: view_box.offset.2.as_expr().map(|s| s.to_string()),
                })
            } else {
                None
            };

            // Convert fill color from RON definition
            let fill_color = view_box
                .fill_color
                .as_ref()
                .map(|c| {
                    let (r, g, b, a) = color_tuple_to_static(c);
                    Color::srgba(r, g, b, a)
                })
                .unwrap_or(Color::BLACK);

            let mut box_entity = if is_top_level {
                // Top-level nodes use CameraAnchored
                parent.spawn((
                    ViewBox::new_full(
                        view_box.width,
                        view_box.height,
                        view_box.border_width,
                        texts,
                        view_box.fill_shader.clone(),
                        view_box.structure_file.clone(),
                        fill_color,
                    ),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    CameraAnchoredBundle::from_camera_transform(camera_transform, offset),
                    Name::new(node_def.name.clone()),
                    RonDrivenView,
                ))
            } else {
                // Child nodes use regular Transform relative to parent
                parent.spawn((
                    ViewBox::new_full(
                        view_box.width,
                        view_box.height,
                        view_box.border_width,
                        texts,
                        view_box.fill_shader.clone(),
                        view_box.structure_file.clone(),
                        fill_color,
                    ),
                    Transform::from_translation(offset),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    Name::new(node_def.name.clone()),
                    RonDrivenView,
                ))
            };

            // Attach ViewElement if the node has a name
            if let Some(ref ve) = view_element {
                box_entity.insert(ve.clone());
            }

            if node_def.tags.contains(&"BattleBox".to_string()) {
                box_entity.insert(crate::app_state::battle::collision::BattleBox);
                info!("[UI Box] Added BattleBox marker to '{}'", node_def.name);
            }

            info!(
                "[UI Box] Spawned ViewBox '{}' at camera offset: {:?} with structure_file: {:?}",
                node_def.name, offset, view_box.structure_file
            );

            if let Some(dynamic) = dynamic_anchor {
                box_entity.insert(dynamic);
                info!("[UI Box] Added dynamic anchor to '{}'", node_def.name);
            }

            if let Some(sprite_def) = &node_def.sprite {
                info!(
                    "[UI Box] Adding child sprite to ViewBox '{}': {:?}",
                    node_def.name,
                    sprite_def.visual.path()
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

            // Store entity ID for recursive child processing after closure ends
            spawned_entity_id = Some(box_entity.id());
            return;
        }

        // =====================================================================
        // Case 3: Pure Container Node (no ViewBox, no sprite, but has texts/children)
        // =====================================================================
        if is_pure_container {
            info!(
                "[UI Container] Creating pure container '{}' with {} texts and {} children",
                node_def.name,
                node_def.texts.len(),
                node_def.children.len()
            );

            // Spawn container entity with ViewContainer marker
            let mut container_entity = parent.spawn((
                ViewContainer,
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
            if let Some(ref ve) = view_element {
                container_entity.insert(ve.clone());
            }

            // Spawn texts directly as children of the container
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
            spawned_entity_id = Some(container_entity.id());
        }
    });

    // Process children recursively AFTER the closure ends to avoid borrowing conflicts
    info!("After closure, spawned_entity_id: {:?}", spawned_entity_id);
    let Some(entity_id) = spawned_entity_id else {
        return;
    };

    // Add VisibleWhen component if node has visible_when expression
    if let Some(visible_when_expr) = &node_def.visible_when {
        apply_visible_when(
            commands,
            entity_id,
            visible_when_expr,
            &node_def.name,
            player_data,
            repeat_ctx,
        );
    }

    // Add DynamicViewElement component if needed
    if is_standalone_sprite {
        apply_dynamic_element(commands, entity_id, node_def, repeat_ctx);
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

// =============================================================================
// Helper functions extracted to reduce nesting depth
// =============================================================================

/// Resolve a single item value from a repeat source array at the given index.
fn resolve_repeat_item(
    player_data: &PlayerDataView<'_>,
    source: &str,
    index: usize,
) -> Option<String> {
    if let Some(list) = player_data.get_fact_string_list(source) {
        list.get(index).cloned()
    } else if let Some(list) = player_data.get_fact_int_list(source) {
        list.get(index).map(|v| v.to_string())
    } else {
        None
    }
}

/// Build a `Transform` from a serializable transform definition.
fn build_transform(
    t_def: &super::super::layout::serde_types::SerializableTransform,
    player_data: &PlayerDataView<'_>,
    repeat_ctx: Option<&super::parsing::RepeatContext>,
) -> Transform {
    let mut transform = Transform::default();
    if let Some(trans) = &t_def.translation {
        transform.translation = Vec3::new(
            evaluate_float_expr_with_repeat(&trans.0, player_data, None, repeat_ctx),
            evaluate_float_expr_with_repeat(&trans.1, player_data, None, repeat_ctx),
            evaluate_float_expr_with_repeat(&trans.2, player_data, None, repeat_ctx),
        );
    }
    if let Some(scale) = &t_def.scale {
        transform.scale = Vec3::new(
            evaluate_float_expr_with_repeat(&scale.0, player_data, None, repeat_ctx),
            evaluate_float_expr_with_repeat(&scale.1, player_data, None, repeat_ctx),
            evaluate_float_expr_with_repeat(&scale.2, player_data, None, repeat_ctx),
        );
    }
    if let Some(rot) = t_def.rotation {
        transform.rotation = Quat::from_rotation_z(rot.to_radians());
    }
    transform
}

/// Spawn a standalone sprite node, dispatching to material, protocol, resolved visual, or fallback paths.
fn spawn_standalone_sprite_node(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    sprite_def: &super::super::layout::view_schema::SpriteDef,
    view_element: &Option<crate::core::view::components::ViewElement>,
    visual_path: &str,
    transform: Transform,
    node_name: &str,
    repeat_ctx: Option<&super::parsing::RepeatContext>,
) -> Entity {
    if sprite_def.material.is_some() {
        return spawn_material_sprite(
            parent,
            asset_server,
            sprite_def,
            view_element,
            visual_path,
            transform,
            node_name,
            repeat_ctx,
        );
    }

    if visual_path.contains("://") {
        return spawn_protocol_sprite(
            parent,
            asset_server,
            sprite_def,
            view_element,
            visual_path,
            transform,
            node_name,
        );
    }

    use crate::config::load_config;
    use crate::core::visual::{get_asset_path, resolve_visual_path};
    let config = load_config();
    if let Some(resolved) = resolve_visual_path(visual_path, &config.project.mod_name) {
        let asset_path = get_asset_path(&resolved, &config.project.mod_name);
        return spawn_resolved_sprite(
            parent,
            asset_server,
            sprite_def,
            view_element,
            &resolved,
            &asset_path,
            transform,
            node_name,
        );
    }

    // Fallback: try direct load
    let texture_handle = asset_server.load(visual_path.to_owned());
    let mut entity_id = None;
    spawn_standalone_static_sprite(
        parent,
        sprite_def,
        view_element,
        texture_handle,
        transform,
        node_name,
        &mut entity_id,
        visual_path,
    );
    entity_id.expect("spawn_standalone_static_sprite must set entity_id")
}

/// Spawn a sprite with a protocol path (e.g., "procedural://white_pixel").
fn spawn_protocol_sprite(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    sprite_def: &super::super::layout::view_schema::SpriteDef,
    view_element: &Option<crate::core::view::components::ViewElement>,
    visual_path: &str,
    transform: Transform,
    node_name: &str,
) -> Entity {
    let texture_handle: Handle<Image> = if visual_path.starts_with("procedural://") {
        Handle::default()
    } else {
        asset_server.load(visual_path.to_owned())
    };
    let mut entity_id = None;
    spawn_standalone_static_sprite(
        parent,
        sprite_def,
        view_element,
        texture_handle,
        transform,
        node_name,
        &mut entity_id,
        visual_path,
    );
    entity_id.expect("spawn_standalone_static_sprite must set entity_id")
}

/// Spawn a sprite from a resolved visual path (character animation, static sprite, or frame animation).
fn spawn_resolved_sprite(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    sprite_def: &super::super::layout::view_schema::SpriteDef,
    view_element: &Option<crate::core::view::components::ViewElement>,
    resolved: &crate::core::visual::ResolvedVisual,
    asset_path: &str,
    transform: Transform,
    node_name: &str,
) -> Entity {
    match resolved {
        crate::core::visual::ResolvedVisual::CharacterAnimation(_) => {
            let config_handle = asset_server
                .load::<crate::core::character_asset::AnimationConfigAsset>(asset_path.to_owned());
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
                Name::new(node_name.to_owned()),
                RonDrivenView,
            ));
            if let Some(ve) = view_element {
                entity_cmd.insert(ve.clone());
            }
            info!("[UI Sprite] Spawned animated sprite '{}'", node_name);
            entity_cmd.id()
        }
        crate::core::visual::ResolvedVisual::Sprite(_)
        | crate::core::visual::ResolvedVisual::FrameAnimation(_) => {
            let texture_handle = asset_server.load(asset_path.to_owned());
            let mut entity_id = None;
            spawn_standalone_static_sprite(
                parent,
                sprite_def,
                view_element,
                texture_handle,
                transform,
                node_name,
                &mut entity_id,
                asset_path,
            );
            entity_id.expect("spawn_standalone_static_sprite must set entity_id")
        }
    }
}

/// Spawn a sprite using the DynamicMaterial2d / ShaderMaterial system.
fn spawn_material_sprite(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    sprite_def: &super::super::layout::view_schema::SpriteDef,
    view_element: &Option<crate::core::view::components::ViewElement>,
    visual_path: &str,
    transform: Transform,
    node_name: &str,
    repeat_ctx: Option<&super::parsing::RepeatContext>,
) -> Entity {
    use crate::core::view::components::ShaderMaterial;
    use crate::core::view::reconcile::ShaderMaterialPendingSetup;

    let processed_sprite_def = if let Some(ctx) = repeat_ctx {
        preprocess_sprite_def_for_repeat(sprite_def, ctx)
    } else {
        sprite_def.clone()
    };
    let material_def = processed_sprite_def
        .material
        .as_ref()
        .expect("material must exist in spawn_material_sprite");

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

    // Shader path: mod:// paths are expanded relative to the project root
    let shader_path = if material_def.shader.starts_with("mod://") {
        material_def.shader.replacen("mod://", "", 1)
    } else {
        material_def.shader.clone()
    };
    let shader_handle = asset_server.load(&shader_path);

    // For procedural:// paths, use default handle - setup system will replace with real texture
    let texture_handle: Handle<Image> = if visual_path.starts_with("procedural://") {
        Handle::default()
    } else {
        asset_server.load(visual_path.to_owned())
    };

    let shader_material = ShaderMaterial::from_def(shader_handle.clone(), material_def);

    let mut entity_cmd = parent.spawn((
        final_transform,
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Name::new(node_name.to_owned()),
        RonDrivenView,
        shader_material,
        ShaderMaterialPendingSetup {
            texture: texture_handle,
        },
    ));
    if let Some(ve) = view_element {
        entity_cmd.insert(ve.clone());
    }

    let entity_id = entity_cmd.id();
    info!(
        "[UI Sprite] Spawned shader material sprite '{}' (Entity {:?})",
        node_name, entity_id
    );
    entity_id
}

/// Apply VisibleWhen component to an entity, substituting repeat context variables.
fn apply_visible_when(
    commands: &mut Commands,
    entity_id: Entity,
    visible_when_expr: &str,
    node_name: &str,
    player_data: &PlayerDataView<'_>,
    repeat_ctx: Option<&super::parsing::RepeatContext>,
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
    commands.entity(entity_id).insert(VisibleWhen {
        expression: processed_expr.clone(),
    });

    let is_visible = evaluate_visible_when(&processed_expr, player_data);
    if !is_visible {
        commands.entity(entity_id).insert(Visibility::Hidden);
    }
}

/// Replace @i, @index, and other repeat context variables in an expression string.
fn substitute_repeat_vars(
    expr: &str,
    repeat_ctx: Option<&super::parsing::RepeatContext>,
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

/// Check for dynamic properties on a standalone sprite and insert DynamicViewElement if needed.
fn apply_dynamic_element(
    commands: &mut Commands,
    entity_id: Entity,
    node_def: &super::super::layout::view_schema::ViewNodeDef,
    repeat_ctx: Option<&super::parsing::RepeatContext>,
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
        .insert(super::super::components::DynamicViewElement {
            sprite_def: Some(processed_sprite_def),
            text_def: None,
        });

    if has_time_dependency {
        commands
            .entity(entity_id)
            .insert(super::super::components::TimeDependentTransform);
    }
}

/// Check whether a sprite def has dynamic transform properties or time dependencies.
fn check_sprite_dynamics(
    sprite_def: &super::super::layout::view_schema::SpriteDef,
    node_name: &str,
) -> (bool, bool) {
    let mut has_dynamic = false;
    let mut has_time_dependency = false;

    if let Some(t) = &sprite_def.transform {
        if let Some(trans) = &t.translation {
            let tx = trans.0.is_dynamic();
            let ty = trans.1.is_dynamic();
            let tz = trans.2.is_dynamic();
            info!(
                "Checking dynamics for {}: x={}, y={}, z={}",
                node_name, tx, ty, tz
            );
            if tx || ty || tz {
                has_dynamic = true;
            }
            if vec3_tuple_depends_on_time(trans) {
                has_time_dependency = true;
            }
        }
        if let Some(s) = &t.scale {
            if s.0.is_dynamic() || s.1.is_dynamic() || s.2.is_dynamic() {
                has_dynamic = true;
            }
            if vec3_tuple_depends_on_time(s) {
                has_time_dependency = true;
            }
        }
    }

    if sprite_def
        .shader_params
        .as_ref()
        .is_some_and(is_dynamic_color)
    {
        has_dynamic = true;
    }

    (has_dynamic, has_time_dependency)
}

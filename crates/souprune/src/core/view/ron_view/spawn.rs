use super::super::components::*;
use super::super::layout::*;
use super::super::lifecycle::BackpackViewRoot;
use super::super::sdf_view_shape::parse_text_preserving_whitespace;
use super::parsing::{
    PlayerDataView, evaluate_condition, evaluate_float_expr, evaluate_float_expr_with_repeat,
    evaluate_visible_when, preprocess_sprite_def_for_repeat, resolve_text_content,
    vec3_tuple_depends_on_time,
};
use super::resources::{HotReloadableViewRoot, RonDrivenView, ViewGenerated, ViewLayoutHandle};
use crate::app_state::battle::BattleViewRoot;
use crate::app_state::overworld::chase::ChaseHUDRoot;
use crate::core::sprite::params::SpriteParams;
use crate::extra::debug::DebugCamera;
use bevy::prelude::*;
use bevy_fact_rule_event::{FreAsset, LayeredFactDatabase};

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
    fre_assets: Res<Assets<FreAsset>>,
    pending_bindings: Option<
        Res<crate::app_state::battle::sequencer::view_action::PendingViewBindings>,
    >,
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
) {
    let player_data = PlayerDataView::new(&layered_db);

    let Some(view_layout_handle) = view_layout_handle else {
        return;
    };

    let Some(view_layout) = view_layouts.get(&view_layout_handle.handle) else {
        return;
    };

    // Check if all FRE assets in pending bindings are loaded
    // 检查所有待处理绑定中的 FRE 资产是否已加载
    if let Some(ref bindings_res) = pending_bindings {
        for handle in &bindings_res.fre_handles {
            if fre_assets.get(handle).is_none() {
                // FRE asset not yet loaded, wait for next frame
                // FRE 资产尚未加载，等待下一帧
                trace!("[spawn_ron_view] Waiting for FRE assets to load...");
                return;
            }
        }
    }

    // Log query counts for debugging
    let backpack_count = backpack_root_query.iter().count();
    let battle_count = battle_root_query.iter().count();
    let chase_count = chase_root_query.iter().count();
    trace!(
        "[spawn_ron_view] backpack_roots={}, battle_roots={}, chase_roots={}, layout_path='{}'",
        backpack_count, battle_count, chase_count, view_layout_handle.path
    );

    // Helper closure to spawn view for an entity
    let mut spawn_for_entity = |view_entity: Entity, label: &str| {
        info!(
            "[spawn_ron_view] Spawning view from RON layout ({}), entity={:?}",
            label, view_entity
        );

        let camera_transform = match camera_query.single() {
            Ok(transform) => transform,
            Err(_) => {
                warn!("[spawn_ron_view] No Camera2d found for view spawning!");
                return false;
            }
        };

        // Get bindings if available
        let bindings = pending_bindings.as_ref().map(|b| &b.bindings);

        spawn_ron_view_for_entity(
            &mut commands,
            &asset_server,
            view_entity,
            view_layout,
            camera_transform,
            &mut sprite_params,
            &animation_assets,
            &fre_assets,
            &mortar_strings,
            &player_data,
            &item_registry,
            &view_layout_handle.path,
            bindings,
            &layered_db,
        );

        // Add ViewGenerated and HotReloadableViewRoot for hot reload support
        // 添加 ViewGenerated 和 HotReloadableViewRoot 以支持热重载
        commands.entity(view_entity).insert((
            ViewGenerated,
            HotReloadableViewRoot {
                layout_path: view_layout_handle.path.clone(),
                layout_handle: view_layout_handle.handle.clone(),
            },
        ));

        info!(
            "[spawn_ron_view] Added ViewGenerated and HotReloadableViewRoot to entity {:?}",
            view_entity
        );
        true
    };

    // Handle BackpackViewRoot entities (OW Backpack)
    // 处理 BackpackViewRoot 实体（OW 背包）
    for view_entity in backpack_root_query.iter() {
        spawn_for_entity(view_entity, "BackpackViewRoot");
    }

    // Handle BattleViewRoot entities (Battle UI)
    // 处理 BattleViewRoot 实体（Battle UI）
    for view_entity in battle_root_query.iter() {
        spawn_for_entity(view_entity, "BattleViewRoot");
    }

    // Handle ChaseHUDRoot entities (Chase HUD)
    // 处理 ChaseHUDRoot 实体（Chase HUD）
    for view_entity in chase_root_query.iter() {
        spawn_for_entity(view_entity, "ChaseHUDRoot");
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
    fre_assets: &Assets<FreAsset>,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &PlayerDataView<'_>,
    item_registry: &crate::core::item::ItemRegistry,
    layout_path: &str,
    bindings: Option<
        &std::collections::HashMap<String, crate::app_state::battle::chapter_schema::DataBinding>,
    >,
    layered_db: &LayeredFactDatabase,
) {
    // Generate namespace from layout path
    // 从布局路径生成命名空间
    let namespace = crate::core::view::components::ViewRoot::namespace_from_path(layout_path);

    // Create ViewRoot with local facts initialized from layout
    // 创建带有从布局初始化的局部事实的 ViewRoot
    let mut view_root = crate::core::view::components::ViewRoot::new(layout_path.to_string());

    // Process requires declarations
    // 处理 requires 声明
    for requirement in &view_layout.requires {
        match requirement {
            DataRequirement::File(path) => {
                // Load FRE file if already loaded
                // 如果已加载则加载 FRE 文件
                let handle: Handle<FreAsset> = asset_server.load(path.clone());
                if let Some(fre_asset) = fre_assets.get(&handle) {
                    load_fre_into_view_root(&mut view_root, fre_asset, mortar_strings);
                    info!("[ViewRoot] Loaded FRE file '{}' via requires", path);
                } else {
                    warn!("[ViewRoot] FRE file '{}' not yet loaded, skipping", path);
                }
            }
            DataRequirement::Interface {
                interface,
                expects: _,
            } => {
                // Look up binding for this interface
                // 查找此接口的绑定
                if let Some(bindings) = bindings {
                    if let Some(binding) = bindings.get(interface) {
                        match binding {
                            crate::app_state::battle::chapter_schema::DataBinding::File(path) => {
                                let handle: Handle<FreAsset> = asset_server.load(path.clone());
                                if let Some(fre_asset) = fre_assets.get(&handle) {
                                    load_fre_into_view_root(
                                        &mut view_root,
                                        fre_asset,
                                        mortar_strings,
                                    );
                                    info!(
                                        "[ViewRoot] Bound interface '{}' to file '{}'",
                                        interface, path
                                    );
                                }
                            }
                            crate::app_state::battle::chapter_schema::DataBinding::Files(paths) => {
                                for path in paths {
                                    let handle: Handle<FreAsset> = asset_server.load(path.clone());
                                    if let Some(fre_asset) = fre_assets.get(&handle) {
                                        load_fre_into_view_root(
                                            &mut view_root,
                                            fre_asset,
                                            mortar_strings,
                                        );
                                    }
                                }
                                info!(
                                    "[ViewRoot] Bound interface '{}' to {} files",
                                    interface,
                                    paths.len()
                                );
                            }
                            crate::app_state::battle::chapter_schema::DataBinding::LocalLayer => {
                                // Copy facts from LOCAL layer to view's local_facts
                                // 从 LOCAL 层复制 facts 到 view 的 local_facts
                                for (key, value) in layered_db.iter_local() {
                                    // Resolve localization for string values
                                    // 解析字符串值的本地化
                                    match value {
                                        bevy_fact_rule_event::FactValue::String(s) => {
                                            let resolved =
                                                resolve_simple_localization(s, mortar_strings);
                                            view_root.local_facts.set(key.0.clone(), resolved);
                                        }
                                        bevy_fact_rule_event::FactValue::StringList(list) => {
                                            let resolved_list: Vec<String> = list
                                                .iter()
                                                .map(|s| {
                                                    resolve_simple_localization(s, mortar_strings)
                                                })
                                                .collect();
                                            view_root.local_facts.set(key.0.clone(), resolved_list);
                                        }
                                        _ => {
                                            view_root.local_facts.set(key.0.clone(), value.clone());
                                        }
                                    }
                                }
                                info!("[ViewRoot] Bound interface '{}' to LocalLayer", interface);
                            }
                            crate::app_state::battle::chapter_schema::DataBinding::Expr(_expr) => {
                                warn!(
                                    "[ViewRoot] Expr binding not yet implemented for interface '{}'",
                                    interface
                                );
                            }
                        }
                    } else {
                        warn!(
                            "[ViewRoot] No binding provided for interface '{}'",
                            interface
                        );
                    }
                } else {
                    warn!(
                        "[ViewRoot] Interface '{}' requires binding but none provided",
                        interface
                    );
                }
            }
        }
    }

    // Initialize local_facts from inline facts in layout
    // 从布局中的内联 facts 初始化 local_facts
    if let Some(facts) = &view_layout.facts {
        for (key, value) in facts {
            use crate::core::view::layout::InitialFactValue;
            match value {
                InitialFactValue::Int(i) => view_root.local_facts.set(key.clone(), *i),
                InitialFactValue::Float(f) => view_root.local_facts.set(key.clone(), *f),
                InitialFactValue::Bool(b) => view_root.local_facts.set(key.clone(), *b),
                InitialFactValue::String(s) => view_root.local_facts.set(key.clone(), s.clone()),
                InitialFactValue::StringList(list) => {
                    // Resolve localization references in string list items
                    // 解析字符串列表项中的本地化引用
                    let resolved_list: Vec<String> = list
                        .iter()
                        .map(|s| resolve_simple_localization(s, mortar_strings))
                        .collect();
                    view_root.local_facts.set(key.clone(), resolved_list)
                }
                InitialFactValue::IntList(list) => {
                    view_root.local_facts.set(key.clone(), list.clone())
                }
            }
        }
        info!(
            "[ViewRoot] Initialized {} local facts for '{}'",
            facts.len(),
            layout_path
        );
    }

    // Spawn view nodes BEFORE attaching ViewRoot, using a player_data with local_facts
    // 在附加 ViewRoot 之前生成视图节点，使用带有 local_facts 的 player_data
    {
        // Create player_data with local_facts for spawning children
        // 使用 local_facts 创建 player_data 以生成子节点
        let player_data_with_locals =
            PlayerDataView::with_local_facts(player_data.db(), &view_root.local_facts);

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
                &player_data_with_locals,
                item_registry,
                true,       // Top-level nodes
                &namespace, // Pass namespace to children
            );
        }
    }

    // Attach ViewRoot to the view entity AFTER spawning children
    // 在生成子节点之后将 ViewRoot 附加到视图实体
    commands.entity(view_entity).insert(view_root);
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
        visible_when: text_def.visible_when.clone(),
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
    // Handle repeat configuration - spawn multiple instances from array
    // 处理重复配置 - 从数组生成多个实例
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
            if let Some(item_var) = &repeat.item_var {
                let item_value =
                    if let Some(list) = player_data.get_fact_string_list(&repeat.source) {
                        list.get(i).cloned()
                    } else if let Some(list) = player_data.get_fact_int_list(&repeat.source) {
                        list.get(i).map(|v| v.to_string())
                    } else {
                        None
                    };
                if let Some(value) = item_value {
                    ctx = ctx.with_item(item_var, value);
                }
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
///
/// 带可选重复上下文生成单个视图节点的内部函数。
#[allow(clippy::too_many_arguments)]
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
            use crate::config::load_config;
            use crate::core::visual::{ResolvedVisual, get_asset_path, resolve_visual_path};

            let sprite_def = node_def.sprite.as_ref().unwrap();
            let mut transform = Transform::default();
            if let Some(t_def) = &sprite_def.transform {
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
            }

            info!(
                "[UI Sprite] Spawning standalone sprite '{}' at position: {:?}, scale: {:?}",
                node_name, transform.translation, transform.scale
            );

            let config = load_config();
            let visual_path = sprite_def.visual.path().to_owned();

            // Check if using custom shader (HP bar)
            let use_custom_material = sprite_def.custom_shader.is_some();

            if use_custom_material {
                // Use Material2d with custom shader (HP bar)
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
                        shader_params_expr: sprite_def.shader_params.clone(),
                    },
                ));
                if let Some(ref ve) = view_element {
                    entity_cmd.insert(ve.clone());
                }

                let entity_id = entity_cmd.id();
                spawned_entity_id = Some(entity_id);

                info!(
                    "[UI Sprite] Spawned HP bar sprite '{}' (Entity {:?}) - will apply material in setup system",
                    node_def.name, entity_id
                );
            } else if visual_path.contains("://") {
                // Handle special protocol paths (e.g., "procedural://white_pixel")
                let texture_handle = if visual_path.starts_with("procedural://") {
                    Handle::default() // Will be replaced by setup system
                } else {
                    asset_server.load(&visual_path)
                };
                spawn_standalone_static_sprite(
                    parent, sprite_def, &view_element, texture_handle, transform,
                    &node_def.name, &mut spawned_entity_id, &visual_path,
                );
            } else if let Some(resolved) = resolve_visual_path(&visual_path, &config.project.mod_name) {
                let asset_path = get_asset_path(&resolved, &config.project.mod_name);

                match resolved {
                    ResolvedVisual::CharacterAnimation(_) => {
                        // Character animation (.character.ron)
                        let config_handle = asset_server
                            .load::<crate::core::character_asset::AnimationConfigAsset>(&asset_path);

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
                        if let Some(ref ve) = view_element {
                            entity_cmd.insert(ve.clone());
                        }
                        info!("[UI Sprite] Spawned animated sprite '{}'", node_def.name);
                    }
                    ResolvedVisual::Sprite(_) | ResolvedVisual::FrameAnimation(_) => {
                        let texture_handle = asset_server.load(&asset_path);
                        spawn_standalone_static_sprite(
                            parent, sprite_def, &view_element, texture_handle, transform,
                            &node_def.name, &mut spawned_entity_id, &asset_path,
                        );
                    }
                }
            } else {
                // Fallback: try direct load
                let texture_handle = asset_server.load(&visual_path);
                spawn_standalone_static_sprite(
                    parent, sprite_def, &view_element, texture_handle, transform,
                    &node_def.name, &mut spawned_entity_id, &visual_path,
                );
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
                    node_def.name, sprite_def.visual.path()
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

            // Spawn container entity with ViewContainer marker
            // 使用 ViewContainer 标记生成容器实体
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
        // Add VisibleWhen component if node has visible_when expression
        // 如果节点有 visible_when 表达式则添加 VisibleWhen 组件
        if let Some(visible_when_expr) = &node_def.visible_when {
            let expr = visible_when_expr.trim();
            if !expr.is_empty() {
                info!(
                    "Adding VisibleWhen to entity {:?} ({}): '{}'",
                    entity_id, node_def.name, expr
                );
                commands.entity(entity_id).insert(VisibleWhen {
                    expression: expr.to_string(),
                });

                // Evaluate initial visibility
                let is_visible = evaluate_visible_when(expr, player_data);
                if !is_visible {
                    commands.entity(entity_id).insert(Visibility::Hidden);
                }
            }
        }

        // Add DynamicViewElement component if needed
        if is_standalone_sprite {
            let sprite_def = node_def.sprite.as_ref().unwrap();

            let mut has_dynamic = false;
            let mut has_time_dependency = false;
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

                    // Check for time dependency
                    // 检查时间依赖
                    if vec3_tuple_depends_on_time(trans) {
                        has_time_dependency = true;
                    }
                }
                if let Some(s) = &t.scale {
                    if s.0.is_dynamic() || s.1.is_dynamic() || s.2.is_dynamic() {
                        has_dynamic = true;
                    }
                    // Check scale for time dependency
                    // 检查 scale 的时间依赖
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

            if has_dynamic {
                info!(
                    "Adding DynamicViewElement to entity {:?} ({}) [time_dependent={}]",
                    entity_id, node_def.name, has_time_dependency
                );

                // Preprocess sprite_def to resolve repeat variables if repeat context exists
                // 如果存在 repeat 上下文，预处理 sprite_def 以解析 repeat 变量
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

                // Add TimeDependentTransform marker if expression uses @time
                // 如果表达式使用 @time 则添加 TimeDependentTransform 标记
                if has_time_dependency {
                    commands
                        .entity(entity_id)
                        .insert(super::super::components::TimeDependentTransform);
                }
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

        // Add VisibleWhen component if text has visible_when expression
        // 如果文本有 visible_when 表达式则添加 VisibleWhen 组件
        if let Some(visible_when_expr) = &text_def.visible_when {
            let expr = visible_when_expr.trim();
            if !expr.is_empty() {
                // Evaluate initial visibility
                let is_visible = evaluate_visible_when(expr, player_data);

                // Debug: check if we can access local facts
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
                // Set initial visibility
                if is_visible {
                    cmd.insert(Visibility::Inherited);
                } else {
                    cmd.insert(Visibility::Hidden);
                }
            }
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

fn spawn_ui_sprite(
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
#[allow(clippy::too_many_arguments)]
fn spawn_standalone_static_sprite(
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
fn resolve_simple_localization(
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
fn load_fre_into_view_root(
    view_root: &mut crate::core::view::components::ViewRoot,
    fre_asset: &FreAsset,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
) {
    use bevy_fact_rule_event::FactValue;

    for (key, value_def) in fre_asset.get_facts() {
        let fact_value: FactValue = value_def.clone().into();
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
        }
    }
}

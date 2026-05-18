//! Spawns a concrete entity tree from a loaded RON view layout and its bound data.
//!
//! 根据已加载的 RON View 布局及其绑定数据，生成具体的实体树。
//!
//! Acts as the main construction step of the RON view runtime. It loads
//! required FRE assets, initializes per-view fact state, applies interface
//! bindings, and walks layout nodes to create the actual Bevy entities that the
//! rest of the view system will later reconcile and update.
//!
//! RON View 运行时的主要构建步骤。它负责加载所需的 FRE 资源、
//! 初始化每个 View 自己的事实状态、应用接口绑定，并遍历布局节点生成真正的
//! Bevy 实体树，供后续的 View 对账与更新系统继续处理。

mod pre_spawn_events;

use super::super::components::*;
use super::super::layout::*;
use super::parsing::{
    DataPathResolvers, ExprFunctionResolvers, PlayerDataView, resolve_text_content,
};
use super::resources::{HotReloadableViewRoot, RonDrivenView, ViewGenerated};
use super::spawn_helpers::resolve_simple_localization;
use crate::core::sprite::params::SpriteParams;
use crate::extra::debug::DebugCamera;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_fact_rule_event::{LayeredFactDatabase, RuleScope};
use pre_spawn_events::apply_pre_spawn_events;

use super::spawn_helpers::load_fre_into_view_root;
use super::spawn_nodes::spawn_view_node;
use crate::core::game_action::{GameFreAsset, GameRuleRegistry};

/// System parameter bundle for FRE-related resources.
/// Reduces system parameter count to stay within Bevy's 16-parameter limit.
///
/// FRE 相关资源的系统参数包。
/// 减少系统参数数量以保持在 Bevy 的 16 参数限制内。
#[derive(SystemParam)]
pub struct FreSystemParams<'w> {
    pub rule_registry: ResMut<'w, GameRuleRegistry>,
    pub enum_registry: ResMut<'w, bevy_fact_rule_event::EnumRegistry>,
}

fn layout_requests_focus_scope(view_layout: &ViewLayoutAsset) -> bool {
    view_layout.roots.iter().any(node_requests_focus_scope)
}

fn node_requests_focus_scope(node: &ViewNodeDef) -> bool {
    matches!(
        node.focus_policy,
        Some(ViewFocusPolicyDef::Focusable | ViewFocusPolicyDef::Scope)
    ) || node.children.iter().any(node_requests_focus_scope)
}

/// Register view-scoped FRE rules from a loaded FreAsset.
/// Returns the number of rules registered.
fn register_fre_rules_from_asset(
    fre_asset: &GameFreAsset,
    view_entity: Entity,
    rule_registry: &mut GameRuleRegistry,
) -> usize {
    let rule_defs = fre_asset.get_rule_defs();
    let scope = fre_asset.scope();
    for (idx, rule_def) in rule_defs.iter().enumerate() {
        // Use View scope for rules loaded via requires (override Local → View)
        let effective_scope = if scope == RuleScope::Local {
            RuleScope::View
        } else {
            scope
        };

        let rule = rule_def.to_rule_with_index(idx, effective_scope);

        if effective_scope == RuleScope::View {
            rule_registry.register_view_rule(view_entity, rule);
        } else {
            rule_registry.register(rule);
        }
    }
    rule_defs.len()
}

/// Process a single interface requirement by resolving its binding.
fn process_interface_requirement(
    interface: &str,
    bindings: Option<
        &std::collections::HashMap<String, crate::core::sequencer::chapter_schema::DataBinding>,
    >,
    asset_server: &AssetServer,
    fre_assets: &Assets<GameFreAsset>,
    view_root: &mut crate::core::view::components::ViewRoot,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    layered_db: &LayeredFactDatabase,
    view_entity: Entity,
    rule_registry: &mut GameRuleRegistry,
    enum_registry: &mut bevy_fact_rule_event::EnumRegistry,
) {
    let Some(bindings) = bindings else {
        warn!(
            "[ViewRoot] Interface '{}' requires binding but none provided",
            interface
        );
        return;
    };
    let Some(binding) = bindings.get(interface) else {
        warn!(
            "[ViewRoot] No binding provided for interface '{}'",
            interface
        );
        return;
    };
    match binding {
        crate::core::sequencer::chapter_schema::DataBinding::File(path) => {
            let handle: Handle<GameFreAsset> = asset_server.load(path.clone());
            let Some(fre_asset) = fre_assets.get(&handle) else {
                return;
            };
            enum_registry.register_from_asset(fre_asset);
            load_fre_into_view_root(view_root, fre_asset, mortar_strings, enum_registry);
            let num_rules = register_fre_rules_from_asset(fre_asset, view_entity, rule_registry);
            info!(
                "[ViewRoot] Bound interface '{}' to file '{}' ({} rules)",
                interface, path, num_rules
            );
        }
        crate::core::sequencer::chapter_schema::DataBinding::Files(paths) => {
            let mut total_rules = 0;
            for path in paths {
                let handle: Handle<GameFreAsset> = asset_server.load(path.clone());
                let Some(fre_asset) = fre_assets.get(&handle) else {
                    continue;
                };
                enum_registry.register_from_asset(fre_asset);
                load_fre_into_view_root(view_root, fre_asset, mortar_strings, enum_registry);
                total_rules += register_fre_rules_from_asset(fre_asset, view_entity, rule_registry);
            }
            info!(
                "[ViewRoot] Bound interface '{}' to {} files ({} rules)",
                interface,
                paths.len(),
                total_rules
            );
        }
        crate::core::sequencer::chapter_schema::DataBinding::LocalLayer => {
            // Copy facts from LOCAL layer to the View's LocalState.
            // Skip dialogue:* facts — they are system-managed and updated
            // every frame in LayeredFactDatabase. Copying them here would
            // create stale snapshots that shadow the live values during
            // condition evaluation (which checks LocalState first).
            //
            // 从 LOCAL 层复制 facts 到 View 的 LocalState。
            // 跳过 dialogue:* facts —— 它们由系统管理并每帧更新。
            for (key, value) in layered_db.iter_local() {
                if key.starts_with("dialogue:") {
                    continue;
                }
                // Resolve localization for string values
                // 解析字符串值的本地化
                match value {
                    bevy_fact_rule_event::FactValue::String(s) => {
                        let resolved = resolve_simple_localization(s, mortar_strings);
                        view_root.set_local_value(key.clone(), resolved);
                    }
                    bevy_fact_rule_event::FactValue::StringList(list) => {
                        let resolved_list: Vec<String> = list
                            .iter()
                            .map(|s| resolve_simple_localization(s, mortar_strings))
                            .collect();
                        view_root.set_local_value(key.clone(), resolved_list);
                    }
                    _ => {
                        view_root.set_local_value(key.clone(), value.clone());
                    }
                }
            }
            info!("[ViewRoot] Bound interface '{}' to LocalLayer", interface);
        }
        crate::core::sequencer::chapter_schema::DataBinding::Expr(_expr) => {
            warn!(
                "[ViewRoot] Expr binding not yet implemented for interface '{}'",
                interface
            );
        }
    }
}

/// Spawn view elements for a specific entity.
///
/// 为特定实体生成视图元素。
pub fn spawn_ron_view_for_entity(
    commands: &mut Commands,
    asset_server: &AssetServer,
    view_entity: Entity,
    view_layout: &ViewLayoutAsset,
    sprite_params: &mut SpriteParams,
    animation_assets: &Assets<crate::core::character_asset::AnimationConfigAsset>,
    fre_assets: &Assets<GameFreAsset>,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &PlayerDataView<'_>,
    layout_path: &str,
    pre_spawn_events: &[String],
    bindings: Option<
        &std::collections::HashMap<String, crate::core::sequencer::chapter_schema::DataBinding>,
    >,
    layered_db: &LayeredFactDatabase,
    rule_registry: &mut GameRuleRegistry,
    enum_registry: &mut bevy_fact_rule_event::EnumRegistry,
    layout_viewport_size: Vec2,
) {
    // Generate namespace from layout path
    // 从布局路径生成命名空间
    let namespace = crate::core::view::components::ViewRoot::namespace_from_path(layout_path);

    // Create ViewRoot with LocalState initialized from layout.
    // 创建带有从布局初始化的局部事实的 ViewRoot
    let mut view_root = crate::core::view::components::ViewRoot::new(layout_path.to_string());

    // Track pending FRE files that need delayed registration (store handles to keep loading alive)
    // 跟踪需要延迟注册的待处理 FRE 文件（存储句柄以保持加载请求）
    let mut pending_fre_handles: Vec<(String, Handle<GameFreAsset>)> = Vec::new();
    let mut loaded_rule_defs = Vec::new();

    // Process requires declarations
    // 处理 requires 声明
    for requirement in &view_layout.requires {
        match requirement {
            DataRequirement::File(path) => {
                // Load FRE file if already loaded
                // 如果已加载则加载 FRE 文件
                let handle: Handle<GameFreAsset> = asset_server.load(path.clone());
                let Some(fre_asset) = fre_assets.get(&handle) else {
                    // FRE file not yet loaded - add to pending for delayed registration
                    // Store the handle to keep the loading request alive
                    // FRE 文件尚未加载 - 添加到待处理列表以延迟注册
                    // 存储句柄以保持加载请求不被取消
                    info!(
                        "[ViewRoot] FRE file '{}' not yet loaded, adding to pending",
                        path
                    );
                    pending_fre_handles.push((path.clone(), handle));
                    continue;
                };
                enum_registry.register_from_asset(fre_asset);
                load_fre_into_view_root(&mut view_root, fre_asset, mortar_strings, enum_registry);
                loaded_rule_defs.extend(fre_asset.get_rule_defs().iter().cloned());

                // Register View-scoped rules from this FRE file
                // 从此 FRE 文件注册 View 作用域的规则
                let num_rules =
                    register_fre_rules_from_asset(fre_asset, view_entity, rule_registry);
                if num_rules > 0 {
                    info!(
                        "[ViewRoot] Registered {} rules from '{}' for View entity {:?}",
                        num_rules, path, view_entity
                    );
                }
                info!("[ViewRoot] Loaded FRE file '{}' via requires", path);
            }
            DataRequirement::Interface {
                interface,
                expects: _,
            } => {
                process_interface_requirement(
                    interface,
                    bindings,
                    asset_server,
                    fre_assets,
                    &mut view_root,
                    mortar_strings,
                    layered_db,
                    view_entity,
                    rule_registry,
                    enum_registry,
                );
            }
        }
    }

    // Initialize LocalState from inline facts in layout.
    // 从布局中的内联 facts 初始化 LocalState。
    if let Some(facts) = &view_layout.facts {
        for (key, value) in facts {
            use crate::core::view::layout::InitialFactValue;
            match value {
                InitialFactValue::Int(i) => view_root.set_local_value(key.clone(), *i),
                InitialFactValue::Float(f) => view_root.set_local_value(key.clone(), *f),
                InitialFactValue::Bool(b) => view_root.set_local_value(key.clone(), *b),
                InitialFactValue::String(s) => view_root.set_local_value(key.clone(), s.clone()),
                InitialFactValue::StringList(list) => {
                    // Resolve localization references in string list items
                    // 解析字符串列表项中的本地化引用
                    let resolved_list: Vec<String> = list
                        .iter()
                        .map(|s| resolve_simple_localization(s, mortar_strings))
                        .collect();
                    view_root.set_local_value(key.clone(), resolved_list)
                }
                InitialFactValue::IntList(list) => {
                    view_root.set_local_value(key.clone(), list.clone())
                }
            }
        }
        info!(
            "[ViewRoot] Initialized {} local facts for '{}'",
            facts.len(),
            layout_path
        );
    }

    apply_pre_spawn_events(
        &mut view_root,
        &loaded_rule_defs,
        pre_spawn_events,
        layered_db,
        enum_registry,
    );

    // Spawn view nodes BEFORE attaching ViewRoot, using player_data with LocalState.
    // 在附加 ViewRoot 之前生成视图节点，使用带有 LocalState 的 player_data。
    {
        // Create player_data with LocalState for spawning children.
        // 使用 LocalState 创建 player_data 以生成子节点。
        let player_data_with_locals =
            PlayerDataView::with_local_state(player_data.db(), view_root.local_state());
        let repeat_count = |repeat: &RepeatDef| {
            Some(resolve_layout_repeat_count(
                &player_data_with_locals,
                repeat,
            ))
        };
        let repeat_item = |repeat: &RepeatDef, index: usize| {
            resolve_layout_repeat_item(&player_data_with_locals, &repeat.source, index)
        };
        let text_content = |content: &str, repeat_ctx: Option<&ViewLayoutRepeatContext>| {
            let content = apply_layout_repeat_context_to_text(content, repeat_ctx);
            resolve_text_content(&content, mortar_strings, &player_data_with_locals)
        };
        let layout_slots = match compute_taffy_layout_with_context(
            view_layout,
            layout_viewport_size,
            ViewLayoutContext {
                repeat_count: &repeat_count,
                repeat_item: &repeat_item,
                text_content: &text_content,
            },
        ) {
            Ok(slots) => Some(slots),
            Err(error) => {
                warn!(
                    "[ViewRoot] Failed to compute Taffy layout for '{}': {}",
                    layout_path, error
                );
                None
            }
        };

        for (root_idx, root) in view_layout.roots.iter().enumerate() {
            let root_path = layout_root_path(root_idx, root);
            spawn_view_node(
                commands,
                asset_server,
                view_entity,
                root,
                sprite_params,
                animation_assets,
                mortar_strings,
                &player_data_with_locals,
                &namespace,
                layout_slots.as_ref(),
                &root_path,
            );
        }
    }

    // Attach ViewRoot to the view entity AFTER spawning children
    // 在生成子节点之后将 ViewRoot 附加到视图实体
    commands.entity(view_entity).insert(view_root);

    // If there are pending FRE files, add PendingViewRules component for delayed registration
    // 如果有待处理的 FRE 文件，添加 PendingViewRules 组件以延迟注册
    if !pending_fre_handles.is_empty() {
        info!(
            "[ViewRoot] Adding PendingViewRules with {} pending handles for entity {:?}",
            pending_fre_handles.len(),
            view_entity
        );
        commands.entity(view_entity).insert(PendingViewRules {
            pending_handles: pending_fre_handles,
        });
    }
}

fn resolve_layout_repeat_count(player_data: &PlayerDataView, repeat: &RepeatDef) -> usize {
    let array_len = if let Some(list) = player_data.get_fact_string_list(&repeat.source) {
        list.len()
    } else if let Some(list) = player_data.get_fact_int_list(&repeat.source) {
        list.len()
    } else {
        0
    };

    array_len.min(repeat.limit.unwrap_or(usize::MAX))
}

fn resolve_layout_repeat_item(
    player_data: &PlayerDataView,
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

fn required_pre_spawn_fre_files_loaded(
    view_layout: &ViewLayoutAsset,
    asset_server: &AssetServer,
    fre_assets: &Assets<GameFreAsset>,
    hot_reload_root: &mut HotReloadableViewRoot,
) -> bool {
    if hot_reload_root.pre_spawn_events.is_empty() {
        return true;
    }

    if hot_reload_root.pre_spawn_fre_handles.is_empty() {
        hot_reload_root.pre_spawn_fre_handles = view_layout
            .requires
            .iter()
            .filter_map(|requirement| {
                let DataRequirement::File(path) = requirement else {
                    return None;
                };
                info!(
                    "[spawn_dynamic_view] Pre-loading required FRE before initial spawn: {}",
                    path
                );
                Some(asset_server.load(path.clone()))
            })
            .collect();
    }

    hot_reload_root
        .pre_spawn_fre_handles
        .iter()
        .all(|handle| fre_assets.get(handle).is_some())
}

fn camera_visible_size(projection: &Projection) -> Option<Vec2> {
    let Projection::Orthographic(orthographic) = projection else {
        return None;
    };
    let size = Vec2::new(
        orthographic.area.width().abs(),
        orthographic.area.height().abs(),
    );
    if size.x > 0.0 && size.y > 0.0 {
        Some(size)
    } else {
        None
    }
}

fn layout_viewport_size(view_layout: &ViewLayoutAsset, camera_visible_size: Option<Vec2>) -> Vec2 {
    if let Some(CoordinateSpaceDef {
        extent: CoordinateExtentDef::Explicit((width, height)),
        ..
    }) = view_layout.coordinate_space.as_ref()
        && *width > 0.0
        && *height > 0.0
    {
        return Vec2::new(*width, *height);
    }

    camera_visible_size.unwrap_or(Vec2::new(640.0, 480.0))
}

fn camera_relative_view_offset(
    view_layout: &ViewLayoutAsset,
    camera_visible_size: Option<Vec2>,
) -> Vec2 {
    if view_layout.world_space {
        return Vec2::ZERO;
    }

    let Some(CoordinateSpaceDef {
        axis_origin,
        extent: CoordinateExtentDef::Explicit((source_width, source_height)),
        ..
    }) = view_layout.coordinate_space.as_ref()
    else {
        return Vec2::ZERO;
    };

    let Some(visible_size) = camera_visible_size else {
        return Vec2::ZERO;
    };

    if *source_width <= 0.0
        || *source_height <= 0.0
        || visible_size.x <= 0.0
        || visible_size.y <= 0.0
    {
        return Vec2::ZERO;
    }

    let origin_x = match &axis_origin.0 {
        crate::core::sequencer::chapter_schema::Value::Static(value) => *value,
        crate::core::sequencer::chapter_schema::Value::Expr(_) => return Vec2::ZERO,
    };
    let origin_y = match &axis_origin.1 {
        crate::core::sequencer::chapter_schema::Value::Static(value) => *value,
        crate::core::sequencer::chapter_schema::Value::Expr(_) => return Vec2::ZERO,
    };

    Vec2::new(
        (origin_x - 0.5) * (visible_size.x - *source_width),
        (0.5 - origin_y) * (visible_size.y - *source_height),
    )
}

/// Unified system to spawn all Views (backpack, battle, chase, dialogue).
/// All View spawning goes through SpawnViewRequest → this system.
///
/// 统一的 View 生成系统（背包、战斗、追逐、对话）。
/// 所有 View 生成都通过 SpawnViewRequest → 此系统。
pub fn spawn_dynamic_view_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    view_layouts: Res<Assets<ViewLayoutAsset>>,
    animation_assets: Res<Assets<crate::core::character_asset::AnimationConfigAsset>>,
    fre_assets: Res<Assets<GameFreAsset>>,
    // Query for views with HotReloadableViewRoot + RonDrivenView but not yet generated
    // 查询有 HotReloadableViewRoot + RonDrivenView 但尚未生成的 View
    mut dynamic_view_query: Query<
        (
            Entity,
            &mut HotReloadableViewRoot,
            &ViewRoot,
            Option<&PendingViewData>,
        ),
        (
            With<RonDrivenView>,
            Without<ViewGenerated>,
            Without<ViewBox>,
        ),
    >,
    camera_query: Query<
        (Entity, &Transform, &Camera, &Projection),
        (
            With<Camera2d>,
            With<crate::core::camera::MainGameCamera>,
            Without<DebugCamera>,
        ),
    >,
    mut sprite_params: SpriteParams,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    layered_db: Res<LayeredFactDatabase>,
    mut fre_params: FreSystemParams,
    data_resolvers: Option<Res<DataPathResolvers>>,
    expr_func_resolvers: Option<Res<ExprFunctionResolvers>>,
) {
    let player_data = PlayerDataView::new(&layered_db)
        .with_resolvers(data_resolvers.as_deref(), None)
        .with_expr_functions(expr_func_resolvers.as_deref());

    for (view_entity, mut hot_reload_root, _view_root, pending_view_data) in
        dynamic_view_query.iter_mut()
    {
        // Check if asset is loaded
        let Some(view_layout) = view_layouts.get(&hot_reload_root.layout_handle) else {
            trace!(
                "[spawn_dynamic_view] Waiting for asset to load: {}",
                hot_reload_root.layout_path
            );
            continue;
        };

        // If there are pending bindings, wait for all FRE assets to load
        if let Some(pvd) = pending_view_data {
            let all_loaded = pvd.fre_handles.iter().all(|h| fre_assets.get(h).is_some());
            if !all_loaded {
                trace!(
                    "[spawn_dynamic_view] Waiting for FRE binding assets: {}",
                    hot_reload_root.layout_path
                );
                continue;
            }
        }

        if !required_pre_spawn_fre_files_loaded(
            view_layout,
            &asset_server,
            &fre_assets,
            &mut hot_reload_root,
        ) {
            trace!(
                "[spawn_dynamic_view] Waiting for required FRE assets: {}",
                hot_reload_root.layout_path
            );
            continue;
        }

        let Some((camera_entity, _, _, projection)) =
            camera_query.iter().find(|(_, _, c, _)| c.is_active)
        else {
            warn!("[spawn_dynamic_view] No Camera2d found for view spawning!");
            continue;
        };

        info!(
            "[spawn_dynamic_view] Spawning view: {}, entity={:?}",
            hot_reload_root.layout_path, view_entity
        );

        // Get bindings from PendingViewData if present
        let bindings = pending_view_data.map(|pvd| &pvd.bindings);
        let visible_size = camera_visible_size(projection);

        spawn_ron_view_for_entity(
            &mut commands,
            &asset_server,
            view_entity,
            view_layout,
            &mut sprite_params,
            &animation_assets,
            &fre_assets,
            &mortar_strings,
            &player_data,
            &hot_reload_root.layout_path,
            &hot_reload_root.pre_spawn_events,
            bindings,
            &layered_db,
            &mut fre_params.rule_registry,
            &mut fre_params.enum_registry,
            layout_viewport_size(view_layout, visible_size),
        );

        // Camera-relative views: parent the view entity to the camera so child
        // transforms are automatically relative to the camera position.
        // World-space views (battle): keep the view entity as a standalone world entity.
        if !view_layout.world_space {
            commands.entity(view_entity).insert((
                Transform::from_translation(
                    camera_relative_view_offset(view_layout, visible_size).extend(0.0),
                ),
                ChildOf(camera_entity),
            ));
        }

        // 自动推断 ViewFocusScope：有 requires（FRE 规则声明）或 bindings（外部数据绑定）
        // → 进入焦点栈，由生命周期系统派生唯一 ActiveView
        if !view_layout.requires.is_empty()
            || pending_view_data.is_some()
            || layout_requests_focus_scope(view_layout)
        {
            commands.entity(view_entity).insert(ViewFocusScope);
        }

        // Add ViewGenerated and ReconciliationEnabled; remove PendingViewData
        commands.entity(view_entity).insert((
            ViewGenerated,
            crate::core::view::reconcile::ReconciliationEnabled,
        ));
        commands.entity(view_entity).remove::<PendingViewData>();

        info!(
            "[spawn_dynamic_view] Added ViewGenerated to entity {:?}",
            view_entity
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn explicit_screen_layout(world_space: bool) -> ViewLayoutAsset {
        ViewLayoutAsset {
            roots: Vec::new(),
            requires: Vec::new(),
            facts: None,
            world_space,
            coordinate_system: CoordinateSystem::Standard,
            coordinate_space: Some(CoordinateSpaceDef {
                axis_origin: (
                    crate::core::sequencer::chapter_schema::Value::Static(0.0),
                    crate::core::sequencer::chapter_schema::Value::Static(0.0),
                ),
                y_axis: YAxisDirectionDef::Down,
                rotation: RotationDirectionDef::CounterClockwise,
                extent: CoordinateExtentDef::Explicit((640.0, 480.0)),
            }),
        }
    }

    #[test]
    fn camera_relative_view_offsets_explicit_coordinate_space_to_camera_viewport() {
        let layout = explicit_screen_layout(false);

        let offset = camera_relative_view_offset(&layout, Some(Vec2::new(320.0, 240.0)));

        assert_eq!(offset, Vec2::new(160.0, -120.0));
    }

    #[test]
    fn world_space_view_does_not_offset_explicit_coordinate_space() {
        let layout = explicit_screen_layout(true);

        let offset = camera_relative_view_offset(&layout, Some(Vec2::new(320.0, 240.0)));

        assert_eq!(offset, Vec2::ZERO);
    }

    #[test]
    fn layout_viewport_prefers_explicit_coordinate_space() {
        let layout = explicit_screen_layout(false);

        let viewport = layout_viewport_size(&layout, Some(Vec2::new(320.0, 240.0)));

        assert_eq!(viewport, Vec2::new(640.0, 480.0));
    }

    fn focus_node(policy: Option<ViewFocusPolicyDef>, children: Vec<ViewNodeDef>) -> ViewNodeDef {
        ViewNodeDef {
            name: "FocusNode".to_string(),
            tags: Vec::new(),
            style: StyleDef::default(),
            transform: None,
            focus_policy: policy,
            visible_when: None,
            background_color: None,
            border_color: None,
            image: None,
            sprite: None,
            state_sprite: None,
            texts: Vec::new(),
            view_box: None,
            children,
            repeat: None,
        }
    }

    #[test]
    fn focus_policy_marks_layout_as_focus_scope_candidate() {
        let mut layout = explicit_screen_layout(false);
        layout.roots = vec![focus_node(Some(ViewFocusPolicyDef::Scope), Vec::new())];

        assert!(layout_requests_focus_scope(&layout));
    }

    #[test]
    fn child_focus_policy_marks_layout_as_focus_scope_candidate() {
        let mut layout = explicit_screen_layout(false);
        layout.roots = vec![focus_node(
            None,
            vec![focus_node(Some(ViewFocusPolicyDef::Focusable), Vec::new())],
        )];

        assert!(layout_requests_focus_scope(&layout));
    }
}

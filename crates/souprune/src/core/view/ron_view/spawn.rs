use super::super::components::*;
use super::super::layout::*;
use super::super::lifecycle::BackpackViewRoot;
use super::parsing::PlayerDataView;
use super::resources::{HotReloadableViewRoot, RonDrivenView, ViewGenerated, ViewLayoutHandle};
use super::spawn_helpers::resolve_simple_localization;
use crate::app_state::battle::BattleViewRoot;
use crate::app_state::overworld::chase::ChaseHUDRoot;
use crate::app_state::overworld::trigger::RuleActionDefs;
use crate::core::sprite::params::SpriteParams;
use crate::extra::debug::DebugCamera;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_fact_rule_event::{FreAsset, LayeredFactDatabase, LayeredRuleRegistry, RuleScope};

// Re-export from sibling modules for backwards compatibility
pub use super::spawn_helpers::{load_fre_into_view_root, load_procedural_image_handle};
pub use super::spawn_nodes::spawn_view_node;

/// System parameter bundle for FRE-related resources.
/// Reduces system parameter count to stay within Bevy's 16-parameter limit.
///
/// FRE 相关资源的系统参数包。
/// 减少系统参数数量以保持在 Bevy 的 16 参数限制内。
#[derive(SystemParam)]
pub struct FreSystemParams<'w> {
    pub rule_registry: ResMut<'w, LayeredRuleRegistry>,
    pub action_defs: ResMut<'w, RuleActionDefs>,
}

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
    mut fre_params: FreSystemParams,
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
            &mut fre_params.rule_registry,
            &mut fre_params.action_defs,
        );

        // Add ViewGenerated and HotReloadableViewRoot for hot reload support
        // 添加 ViewGenerated 和 HotReloadableViewRoot 以支持热重载
        // Also add ReconciliationEnabled to let the reconciliation system handle updates
        // 同时添加 ReconciliationEnabled 让协调系统处理更新
        commands.entity(view_entity).insert((
            ViewGenerated,
            HotReloadableViewRoot {
                layout_path: view_layout_handle.path.clone(),
                layout_handle: view_layout_handle.handle.clone(),
            },
            crate::core::view::reconcile::ReconciliationEnabled,
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
    rule_registry: &mut LayeredRuleRegistry,
    action_defs: &mut RuleActionDefs,
) {
    // Generate namespace from layout path
    // 从布局路径生成命名空间
    let namespace = crate::core::view::components::ViewRoot::namespace_from_path(layout_path);

    // Create ViewRoot with local facts initialized from layout
    // 创建带有从布局初始化的局部事实的 ViewRoot
    let mut view_root = crate::core::view::components::ViewRoot::new(layout_path.to_string());

    // Track pending FRE files that need delayed registration (store handles to keep loading alive)
    // 跟踪需要延迟注册的待处理 FRE 文件（存储句柄以保持加载请求）
    let mut pending_fre_handles: Vec<(String, Handle<FreAsset>)> = Vec::new();

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

                    // Register View-scoped rules from this FRE file
                    // 从此 FRE 文件注册 View 作用域的规则
                    let rule_defs = fre_asset.get_rule_defs();
                    let scope = fre_asset.scope();
                    for (idx, rule_def) in rule_defs.iter().enumerate() {
                        // Use the FRE file's declared scope, or default to View for FRE files loaded via requires
                        // 使用 FRE 文件声明的作用域，或对于通过 requires 加载的文件默认为 View
                        let effective_scope = if scope == RuleScope::Local {
                            // If the file says Local but is loaded via View's requires, treat as View
                            // 如果文件声明为 Local 但通过 View 的 requires 加载，则视为 View
                            RuleScope::View
                        } else {
                            scope
                        };

                        let rule = rule_def.to_rule_with_index(idx, effective_scope);
                        let rule_id = rule_def.generate_id(idx);

                        // Store actions for this rule in action_defs
                        // 将此规则的 actions 存储到 action_defs 中
                        if !rule_def.actions.is_empty() {
                            action_defs
                                .actions_by_rule
                                .insert(rule_id.clone(), rule_def.actions.clone());
                        }

                        if effective_scope == RuleScope::View {
                            rule_registry.register_view_rule(view_entity, rule);
                            info!(
                                "[ViewRoot] Registered View rule '{}' for entity {:?} from '{}'",
                                rule_id, view_entity, path
                            );
                        } else {
                            rule_registry.register(rule);
                        }
                    }
                    if !rule_defs.is_empty() {
                        info!(
                            "[ViewRoot] Registered {} rules from '{}' for View entity {:?}",
                            rule_defs.len(),
                            path,
                            view_entity
                        );
                    }
                    info!("[ViewRoot] Loaded FRE file '{}' via requires", path);
                } else {
                    // FRE file not yet loaded - add to pending for delayed registration
                    // Store the handle to keep the loading request alive
                    // FRE 文件尚未加载 - 添加到待处理列表以延迟注册
                    // 存储句柄以保持加载请求不被取消
                    info!(
                        "[ViewRoot] FRE file '{}' not yet loaded, adding to pending",
                        path
                    );
                    pending_fre_handles.push((path.clone(), handle));
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

                                    // Register View-scoped rules from interface binding
                                    // 从接口绑定注册 View 作用域的规则
                                    let rule_defs = fre_asset.get_rule_defs();
                                    let scope = fre_asset.scope();
                                    for (idx, rule_def) in rule_defs.iter().enumerate() {
                                        let effective_scope = if scope == RuleScope::Local {
                                            RuleScope::View
                                        } else {
                                            scope
                                        };
                                        let rule =
                                            rule_def.to_rule_with_index(idx, effective_scope);
                                        let rule_id = rule_def.generate_id(idx);

                                        // Store actions for this rule
                                        if !rule_def.actions.is_empty() {
                                            action_defs
                                                .actions_by_rule
                                                .insert(rule_id, rule_def.actions.clone());
                                        }

                                        if effective_scope == RuleScope::View {
                                            rule_registry.register_view_rule(view_entity, rule);
                                        } else {
                                            rule_registry.register(rule);
                                        }
                                    }

                                    info!(
                                        "[ViewRoot] Bound interface '{}' to file '{}' ({} rules)",
                                        interface,
                                        path,
                                        rule_defs.len()
                                    );
                                }
                            }
                            crate::app_state::battle::chapter_schema::DataBinding::Files(paths) => {
                                let mut total_rules = 0;
                                for path in paths {
                                    let handle: Handle<FreAsset> = asset_server.load(path.clone());
                                    if let Some(fre_asset) = fre_assets.get(&handle) {
                                        load_fre_into_view_root(
                                            &mut view_root,
                                            fre_asset,
                                            mortar_strings,
                                        );

                                        // Register View-scoped rules from interface binding
                                        // 从接口绑定注册 View 作用域的规则
                                        let rule_defs = fre_asset.get_rule_defs();
                                        let scope = fre_asset.scope();
                                        for (idx, rule_def) in rule_defs.iter().enumerate() {
                                            let effective_scope = if scope == RuleScope::Local {
                                                RuleScope::View
                                            } else {
                                                scope
                                            };
                                            let rule =
                                                rule_def.to_rule_with_index(idx, effective_scope);
                                            let rule_id = rule_def.generate_id(idx);

                                            // Store actions for this rule
                                            if !rule_def.actions.is_empty() {
                                                action_defs
                                                    .actions_by_rule
                                                    .insert(rule_id, rule_def.actions.clone());
                                            }

                                            if effective_scope == RuleScope::View {
                                                rule_registry.register_view_rule(view_entity, rule);
                                            } else {
                                                rule_registry.register(rule);
                                            }
                                        }
                                        total_rules += rule_defs.len();
                                    }
                                }
                                info!(
                                    "[ViewRoot] Bound interface '{}' to {} files ({} rules)",
                                    interface,
                                    paths.len(),
                                    total_rules
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

/// System to spawn dynamically requested Views (via SpawnViewRequest).
/// This handles Views that are spawned at runtime without going through
/// the global ViewLayoutHandle resource.
///
/// 处理动态请求的 View（通过 SpawnViewRequest）的系统。
/// 处理在运行时 spawn 的 View，无需全局 ViewLayoutHandle 资源。
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn spawn_dynamic_view_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    view_layouts: Res<Assets<ViewLayoutAsset>>,
    animation_assets: Res<Assets<crate::core::character_asset::AnimationConfigAsset>>,
    fre_assets: Res<Assets<FreAsset>>,
    // Query for dynamic views: have HotReloadableViewRoot and RonDrivenView, but not ViewGenerated
    // 查询动态 View：有 HotReloadableViewRoot 和 RonDrivenView，但没有 ViewGenerated
    dynamic_view_query: Query<
        (Entity, &HotReloadableViewRoot, &ViewRoot),
        (
            With<RonDrivenView>,
            Without<ViewGenerated>,
            Without<ViewBox>,
            // Exclude specific root types handled by spawn_ron_view_system
            Without<BackpackViewRoot>,
            Without<BattleViewRoot>,
            Without<ChaseHUDRoot>,
        ),
    >,
    camera_query: Query<&Transform, (With<Camera2d>, Without<DebugCamera>)>,
    mut sprite_params: SpriteParams,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    layered_db: Res<LayeredFactDatabase>,
    item_registry: Res<crate::core::item::ItemRegistry>,
    mut fre_params: FreSystemParams,
) {
    let player_data = PlayerDataView::new(&layered_db);

    for (view_entity, hot_reload_root, _view_root) in dynamic_view_query.iter() {
        // Check if asset is loaded
        let Some(view_layout) = view_layouts.get(&hot_reload_root.layout_handle) else {
            // Asset not yet loaded, wait
            trace!(
                "[spawn_dynamic_view] Waiting for asset to load: {}",
                hot_reload_root.layout_path
            );
            continue;
        };

        let camera_transform = match camera_query.single() {
            Ok(transform) => transform,
            Err(_) => {
                warn!("[spawn_dynamic_view] No Camera2d found for view spawning!");
                continue;
            }
        };

        info!(
            "[spawn_dynamic_view] Spawning dynamic view: {}, entity={:?}",
            hot_reload_root.layout_path, view_entity
        );

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
            &hot_reload_root.layout_path,
            None, // No bindings for dynamic views
            &layered_db,
            &mut fre_params.rule_registry,
            &mut fre_params.action_defs,
        );

        // Add ViewGenerated and ReconciliationEnabled
        commands.entity(view_entity).insert((
            ViewGenerated,
            crate::core::view::reconcile::ReconciliationEnabled,
        ));

        info!(
            "[spawn_dynamic_view] Added ViewGenerated to entity {:?}",
            view_entity
        );
    }
}

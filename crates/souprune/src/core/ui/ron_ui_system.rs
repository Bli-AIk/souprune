//! # ron_ui_system.rs
//!
//! # ron_ui_system.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles loading UI layouts from RON files.
//!
//! 本模块处理从 RON 文件加载 UI 布局。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It spawns UI entities with proper components, navigation, and transitions configured.
//!
//! 生成配置了适当组件、导航和转换的 UI 实体。

use super::components::*;
use super::layout::*;
use crate::app_state::overworld::OverworldState;
use crate::core::input::Action;
use crate::core::sprite::params::SpriteParams;
use bevy::asset::AssetEvent;
use bevy::ecs::prelude::MessageReader;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TiledMapAsset, tiled};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Resource)]
pub struct UILayoutHandle {
    pub handle: Handle<UILayoutAsset>,
    pub last_modified: Option<SystemTime>,
}

#[derive(Component)]
pub struct RonDrivenUI;

#[derive(Resource, Default)]
pub struct UILayoutWatcher {
    timer: Timer,
    pending_reload: bool,
}

#[derive(Resource, Default)]

pub struct UIGlobalTriggerConfig {
    pub triggers: HashMap<Action, Vec<GlobalTriggerRule>>,
}

#[derive(Clone)]

pub struct GlobalTriggerRule {
    pub target_state: OverworldState,

    pub sound: Option<String>,

    pub allowed_states: Vec<OverworldState>,
}

impl UILayoutWatcher {
    fn new() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),

            pending_reload: false,
        }
    }
}

pub fn update_ui_from_map_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tiled_maps_query: Query<&TiledMap>,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    mut watcher: Option<ResMut<UILayoutWatcher>>,
    mut current_ui_path: Local<Option<String>>,
) {
    for tiled_map in tiled_maps_query.iter() {
        if let Some(map_asset) = tiled_map_assets.get(&tiled_map.0)
            && let Some(property_value) = map_asset.map.properties.get("backpack_ui")
            && let tiled::PropertyValue::StringValue(path) = property_value
            && current_ui_path.as_deref() != Some(path)
        {
            info!("Switching backpack UI to: {}", path);
            *current_ui_path = Some(path.clone());

            let handle = asset_server.load(path.clone());

            commands.insert_resource(UILayoutHandle {
                handle,
                last_modified: None,
            });

            if let Some(ref mut w) = watcher {
                w.pending_reload = true;
            } else {
                let mut w = UILayoutWatcher::new();
                w.pending_reload = true;
                commands.insert_resource(w);
            }
        }
    }
}

/// System to watch for asset changes and trigger hot reload.
///
/// 监听资产变更并触发热重载的系统。
pub fn watch_ui_layout_changes_system(
    mut events: MessageReader<AssetEvent<UILayoutAsset>>,
    ui_layout_handle: Option<Res<UILayoutHandle>>,
    mut watcher: Option<ResMut<UILayoutWatcher>>,
    mut commands: Commands,
) {
    let Some(ui_layout_handle) = ui_layout_handle else {
        return;
    };

    for event in events.read() {
        if let AssetEvent::Modified { id } = event
            && *id == ui_layout_handle.handle.id()
        {
            info!("[Hot Reload] RON UI asset modified, triggering reload...");
            if let Some(ref mut w) = watcher {
                w.pending_reload = true;
            } else {
                let mut w = UILayoutWatcher::new();
                w.pending_reload = true;
                commands.insert_resource(w);
            }
        }
    }
}

pub fn load_navigation_and_transitions_system(
    ui_layout_handle: Option<Res<UILayoutHandle>>,
    ui_layouts: Res<Assets<UILayoutAsset>>,
    mut navigation_config: ResMut<UILayerNavigationConfig>,
    mut transition_config: ResMut<UILayerTransitionConfig>,
    mut global_trigger_config: ResMut<UIGlobalTriggerConfig>,
    mut last_processed_handle: Local<Option<AssetId<UILayoutAsset>>>,
    mut events: MessageReader<AssetEvent<UILayoutAsset>>,
) {
    let Some(ui_layout_handle) = ui_layout_handle else {
        return;
    };

    // Check if asset was modified - reset last_processed_handle to force reload
    //
    // 检查资产是否被修改 - 重置 last_processed_handle 以强制重新加载
    for event in events.read() {
        if let AssetEvent::Modified { id } = event
            && *id == ui_layout_handle.handle.id()
        {
            info!("[Hot Reload] Reloading navigation and transitions config...");
            *last_processed_handle = None;
        }
    }

    if last_processed_handle.as_ref() == Some(&ui_layout_handle.handle.id()) {
        return;
    }

    let Some(ui_layout) = ui_layouts.get(&ui_layout_handle.handle) else {
        return;
    };

    *last_processed_handle = Some(ui_layout_handle.handle.id());

    if let Some(global_triggers) = &ui_layout.global_triggers {
        for (action_str, rules_def) in global_triggers {
            if let Some(action) = parse_action(action_str) {
                let mut rules = Vec::new();

                for rule_def in rules_def {
                    if let Some(target_state) = parse_overworld_state(&rule_def.target_state) {
                        let allowed_states = rule_def
                            .allowed_states
                            .as_ref()
                            .map(|states| {
                                states
                                    .iter()
                                    .filter_map(|s| parse_overworld_state(s))
                                    .collect()
                            })
                            .unwrap_or_default();

                        rules.push(GlobalTriggerRule {
                            target_state,

                            sound: rule_def.sound.clone(),

                            allowed_states,
                        });
                    } else {
                        warn!(
                            "Unknown target state '{}' in global triggers",
                            rule_def.target_state
                        );
                    }
                }

                global_trigger_config.triggers.insert(action, rules);
            } else {
                warn!("Unknown action '{}' in global triggers", action_str);
            }
        }

        info!(
            "Loaded global trigger config from RON with {} triggers",
            global_triggers.len()
        );
    }

    if let Some(navigation) = &ui_layout.navigation {
        for (layer_name, nav_rule_def) in navigation.iter() {
            let mut adjustments = HashMap::new();

            for (action_str, delta) in &nav_rule_def.mappings {
                if let Some(action) = parse_action(action_str) {
                    adjustments.insert(action, *delta);
                }
            }

            let min_index = nav_rule_def
                .min_index
                .as_ref()
                .map(|bound_def| match bound_def {
                    IndexBoundDef::Static(value) => IndexBound::Static(*value),
                    IndexBoundDef::Dynamic(expr) => IndexBound::Dynamic(expr.clone()),
                });

            let max_index = nav_rule_def
                .max_index
                .as_ref()
                .map(|bound_def| match bound_def {
                    IndexBoundDef::Static(value) => IndexBound::Static(*value),
                    IndexBoundDef::Dynamic(expr) => IndexBound::Dynamic(expr.clone()),
                });

            let layer = UILayer::new(layer_name.clone());
            let rule = UILayerNavigationRule::new_with_bounds(
                adjustments.into_iter(),
                nav_rule_def.looping,
                min_index,
                max_index,
                nav_rule_def.sound_on_navigate.clone(),
            );
            navigation_config.set_rule(layer, rule);
        }
        info!(
            "Loaded navigation config from RON with {} layers",
            navigation.len()
        );
    }

    if let Some(transitions) = &ui_layout.transitions {
        for (layer_name, transitions_def) in transitions.iter() {
            let on_confirm = transitions_def
                .on_confirm
                .as_ref()
                .map(|rules| {
                    rules
                        .iter()
                        .map(|rule_def| {
                            use super::components::{TransitionAction, TransitionRule};
                            TransitionRule {
                                condition: rule_def.condition.clone(),
                                action: match &rule_def.action {
                                    TransitionActionDef::GotoLayer(layer) => {
                                        TransitionAction::GotoLayer(UILayer::new(layer.clone()))
                                    }
                                    TransitionActionDef::PopState => TransitionAction::PopState,
                                    TransitionActionDef::PushState(state) => {
                                        TransitionAction::PushState(state.clone())
                                    }
                                },
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            let on_cancel = transitions_def.on_cancel.as_ref().map(|action_def| {
                use super::components::TransitionAction;
                match action_def {
                    TransitionActionDef::GotoLayer(layer) => {
                        TransitionAction::GotoLayer(UILayer::new(layer.clone()))
                    }
                    TransitionActionDef::PopState => TransitionAction::PopState,
                    TransitionActionDef::PushState(state) => {
                        TransitionAction::PushState(state.clone())
                    }
                }
            });

            let layer = UILayer::new(layer_name.clone());
            transition_config.set_transitions(
                layer,
                LayerTransitions {
                    on_confirm,
                    on_cancel,
                    sound_on_confirm: transitions_def.sound_on_confirm.clone(),
                    sound_on_cancel: transitions_def.sound_on_cancel.clone(),
                },
            );
        }
        info!(
            "Loaded transition config from RON with {} layers",
            transitions.len()
        );
    }
}

fn parse_action(action_str: &str) -> Option<Action> {
    match action_str {
        "Up" => Some(Action::Up),
        "Down" => Some(Action::Down),
        "Left" => Some(Action::Left),
        "Right" => Some(Action::Right),
        "Confirm" => Some(Action::Confirm),
        "Cancel" => Some(Action::Cancel),
        "Menu" => Some(Action::Menu),
        _ => None,
    }
}

fn parse_overworld_state(state_str: &str) -> Option<OverworldState> {
    match state_str {
        "Normal" => Some(OverworldState::Normal),
        "Backpack" => Some(OverworldState::Backpack),
        "Cutscene" => Some(OverworldState::Cutscene),
        _ => None,
    }
}

pub(crate) fn evaluate_index_bound(
    bound: &IndexBound,
    player_data: &crate::core::data::PlayerData,
) -> usize {
    match bound {
        IndexBound::Static(value) => *value,
        IndexBound::Dynamic(expr) => evaluate_index_expression(expr, player_data),
    }
}

fn evaluate_index_expression(expr: &str, player_data: &crate::core::data::PlayerData) -> usize {
    let expr = expr.trim();

    // 支持的表达式：
    // "inventory.len()" - 背包物品数量
    // "inventory_capacity" - 背包容量
    // "min(inventory.len(), inventory_capacity)" - 两者较小值
    // "max(inventory.len(), inventory_capacity)" - 两者较大值

    if expr == "inventory.len()" {
        return player_data.inventory.len();
    }

    if expr == "inventory_capacity" {
        return player_data.inventory_capacity;
    }

    if expr.starts_with("min(") && expr.ends_with(")") {
        let inner = &expr[4..expr.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let a = evaluate_index_expression(parts[0], player_data);
            let b = evaluate_index_expression(parts[1], player_data);
            return a.min(b);
        }
    }

    if expr.starts_with("max(") && expr.ends_with(")") {
        let inner = &expr[4..expr.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let a = evaluate_index_expression(parts[0], player_data);
            let b = evaluate_index_expression(parts[1], player_data);
            return a.max(b);
        }
    }

    // 尝试解析为数字
    if let Ok(value) = expr.parse::<usize>() {
        return value;
    }

    warn!(
        "Unable to evaluate index expression: {}, defaulting to 1",
        expr
    );
    1
}

#[derive(Component)]
pub(super) struct UIGenerated;

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

    // Clear pending_reload flag to prevent rebuild_reloaded_ui_system from running
    // on initial spawn, which would cause duplicate UI elements
    //
    // 清除 pending_reload 标志以防止 rebuild_reloaded_ui_system 在初次生成时运行，
    // 这会导致重复的 UI 元素
    if spawned_any && let Some(ref mut w) = watcher {
        w.pending_reload = false;
    }
}

fn despawn_entity_tree(commands: &mut Commands, root: Entity) {
    // Schedule recursive despawn to avoid borrowing the world inside the system.
    //
    // 调度递归 despawn 以避免在系统内借用 world。
    commands.queue(move |world: &mut World| {
        let mut stack = vec![root];
        while let Some(entity) = stack.pop() {
            if let Ok(entity_ref) = world.get_entity(entity)
                && let Some(children) = entity_ref.get::<Children>()
            {
                for child in children.iter() {
                    stack.push(child);
                }
            }

            let _ = world.despawn(entity);
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub fn rebuild_reloaded_ui_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_layout_handle: Option<Res<UILayoutHandle>>,
    mut watcher: Option<ResMut<UILayoutWatcher>>,
    ui_layouts: Res<Assets<UILayoutAsset>>,
    animation_assets: Res<Assets<crate::core::character_asset::AnimationConfigAsset>>,
    overworld_ui_query: Query<(Entity, &RonUI), Without<RonDrivenUI>>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut sprite_params: SpriteParams,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    player_data: Res<crate::core::data::PlayerData>,
    item_registry: Res<crate::core::item::ItemRegistry>,
    ron_ui_query: Query<Entity, With<RonDrivenUI>>,
) {
    let Some(ref mut watcher) = watcher else {
        return;
    };

    if !watcher.pending_reload {
        return;
    }

    let Some(handle) = ui_layout_handle else {
        return;
    };

    let Some(ui_layout) = ui_layouts.get(&handle.handle) else {
        return;
    };

    let has_target = overworld_ui_query.iter().any(|_| true);

    if !has_target {
        debug!("RON UI hot reload pending - no UI entity active, will retry rebuild");
        return;
    }

    info!("Asset loaded! Rebuilding UI...");

    let Ok(camera_transform) = camera_query.single() else {
        warn!("No Camera2d found for UI rebuild!");
        watcher.pending_reload = false;
        return;
    };

    // Despawn old UI first (only now that we know we're rebuilding)
    //
    // 首先 despawn 旧 UI（仅当我们知道正在重建时）
    let despawn_count = ron_ui_query.iter().count();
    if despawn_count > 0 {
        info!(
            "Despawning {} old UI entities before rebuild",
            despawn_count
        );
        for entity in ron_ui_query.iter() {
            despawn_entity_tree(&mut commands, entity);
        }
    }

    let mut rebuilt_count = 0;
    for (ui_entity, _ron_ui) in overworld_ui_query.iter() {
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
        rebuilt_count += 1;
    }

    watcher.pending_reload = rebuilt_count == 0;
    info!(
        "RON UI hot reload complete! Rebuilt {} UI entities",
        rebuilt_count
    );
}

#[allow(clippy::too_many_arguments)]
fn spawn_ron_ui_for_entity(
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

use crate::core::ui::smud_shape::parse_text_preserving_whitespace;
use bevy_rich_text3d::Text3d;

/// Helper function to build UITextConfig from TextDef.
///
/// 从 TextDef 构建 UITextConfig 的辅助函数。
fn build_text_config(
    text_def: &TextDef,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &crate::core::data::PlayerData,
    item_registry: &crate::core::item::ItemRegistry,
) -> UITextConfig {
    let raw_content = text_def.content.as_deref().unwrap_or("");
    let mut content = resolve_text_content(raw_content, mortar_strings, player_data, item_registry);

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
            let mut t =
                Transform::from_translation(text_def.transform.translation.to_static_vec3());
            if let Some(scale) = &text_def.transform.scale {
                t.scale = scale.to_static_vec3();
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
fn spawn_ui_node(
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
                transform.translation = t_def.translation.to_static_vec3();
                if let Some(scale) = &t_def.scale {
                    transform.scale = scale.to_static_vec3();
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
                    let entity_id = parent
                        .spawn((
                            Transform::from_translation(transform.translation)
                                .with_scale(transform.scale)
                                .with_rotation(transform.rotation),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                            Name::new(node_def.name.clone()),
                            RonDrivenUI,
                            HPBarSprite {
                                shader_params: sprite_def
                                    .shader_params
                                    .clone()
                                    .map(Color::from)
                                    .unwrap_or(Color::WHITE),
                            },
                        ))
                        .id();
                    
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

/// Spawn text entities directly as children of a container (without UIBox).
///
/// 将文本实体直接作为容器的子节点生成（无 UIBox）。
fn spawn_container_texts(
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
            super::text::NeedsTextMaterial,
            text_world_transform,
            Visibility::Hidden,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            CameraAnchored::new(text_config.transform.translation),
            super::text::NeedsGlyphRefresh,
            RonDrivenUI,
        ));

        if let Some(template) = &text_config.template {
            cmd.insert(UITextTemplate(template.clone()));
        }
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

pub(crate) fn resolve_text_content(
    template: &str,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &crate::core::data::PlayerData,
    item_registry: &crate::core::item::ItemRegistry,
) -> String {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{'
            && let Some(&next_ch) = chars.peek()
        {
            if next_ch == '{' {
                chars.next();
                let mut key = String::new();
                let mut found_closing = false;

                while let Some(ch) = chars.next() {
                    if ch == '}'
                        && let Some(&next_ch) = chars.peek()
                        && next_ch == '}'
                    {
                        chars.next();
                        found_closing = true;
                        break;
                    }
                    key.push(ch);
                }

                if found_closing {
                    let resolved = mortar_strings.resolve(&key);
                    result.push_str(resolved);
                } else {
                    result.push_str("{{");
                    result.push_str(&key);
                }
                continue;
            } else if next_ch == '@' {
                chars.next();
                let mut path = String::new();
                let mut found_closing = false;

                for ch in chars.by_ref() {
                    if ch == '}' {
                        found_closing = true;
                        break;
                    }
                    path.push(ch);
                }

                if found_closing {
                    let value =
                        resolve_data_path(&path, player_data, item_registry, mortar_strings);
                    result.push_str(&value);
                } else {
                    result.push_str("{@");
                    result.push_str(&path);
                }
                continue;
            }
        }
        result.push(ch);
    }

    result
}

fn evaluate_condition(condition: &str, player_data: &crate::core::data::PlayerData) -> bool {
    match condition {
        "player.inventory.is_empty" => player_data.inventory.is_empty(),
        "player.inventory.is_not_empty" => !player_data.inventory.is_empty(),
        _ => {
            if condition.starts_with("player.") {
                let parts: Vec<&str> = condition.split('.').collect();
                if parts.len() >= 3 {
                    match (parts[1], parts[2]) {
                        ("hp", "is_low") => player_data.hp < player_data.hp_max / 4,
                        ("hp", "is_critical") => player_data.hp <= 1,
                        ("gold", "is_zero") => player_data.gold == 0,
                        _ => false,
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}

pub(crate) fn resolve_data_path(
    path: &str,
    player_data: &crate::core::data::PlayerData,
    item_registry: &crate::core::item::ItemRegistry,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
) -> String {
    use crate::core::item::ItemType;

    match path {
        "player.name" => player_data.name.clone(),
        "player.lv" => player_data.lv.to_string(),
        "player.hp" => player_data.hp.to_string(),
        "player.hp_max" => player_data.hp_max.to_string(),
        "player.gold" => player_data.gold.to_string(),
        "player.exp" => player_data.exp.to_string(),
        "player.next_exp" => player_data.next_exp.to_string(),
        "player.attack" => player_data.attack.to_string(),
        "player.defense" => player_data.defense.to_string(),
        "player.inventory" => player_data
            .inventory
            .iter()
            .take(8)
            .map(|item_id| {
                if let Some(item) = item_registry.get(&item_id.0) {
                    let key = format!("{}:{}", item.locate_file, item.locate_name);
                    mortar_strings.resolve(&key).to_string()
                } else {
                    warn!("Item ID '{}' not found in registry!", item_id.0);
                    format!("UNDEFINED ({})", item_id.0)
                }
            })
            .collect::<Vec<String>>()
            .join("\n"),
        "player.weapon" => {
            if let Some(item) = item_registry.get(&player_data.weapon.0) {
                let key = format!("{}:{}", item.locate_file, item.locate_name);
                mortar_strings.resolve(&key).to_string()
            } else {
                player_data.weapon.0.clone()
            }
        }
        "player.weapon_atk" => {
            if let Some(item) = item_registry.get(&player_data.weapon.0)
                && let ItemType::Weapon { damage, .. } = item.item_type
            {
                return damage.to_string();
            }
            "0".to_string()
        }
        "player.total_attack" => {
            let weapon_atk = if let Some(item) = item_registry.get(&player_data.weapon.0) {
                if let ItemType::Weapon { damage, .. } = item.item_type {
                    damage as usize
                } else {
                    0
                }
            } else {
                0
            };
            (player_data.attack + weapon_atk).to_string()
        }
        "player.armor" => {
            if let Some(item) = item_registry.get(&player_data.armor.0) {
                let key = format!("{}:{}", item.locate_file, item.locate_name);
                mortar_strings.resolve(&key).to_string()
            } else {
                player_data.armor.0.clone()
            }
        }
        "player.armor_def" => {
            if let Some(item) = item_registry.get(&player_data.armor.0)
                && let ItemType::Armor { defense } = item.item_type
            {
                return defense.to_string();
            }
            "0".to_string()
        }
        "player.total_defense" => {
            let armor_def = if let Some(item) = item_registry.get(&player_data.armor.0) {
                if let ItemType::Armor { defense } = item.item_type {
                    defense as usize
                } else {
                    0
                }
            } else {
                0
            };
            (player_data.defense + armor_def).to_string()
        }
        _ => format!("<unknown:{}>", path),
    }
}

pub(crate) fn update_dynamic_text_system(
    mut text_query: Query<(&UITextTemplate, &mut Text3d)>,
    player_data: Res<crate::core::data::PlayerData>,
    item_registry: Res<crate::core::item::ItemRegistry>,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
) {
    if !player_data.is_changed() {
        return;
    }

    for (template, mut text3d) in text_query.iter_mut() {
        let new_content =
            resolve_text_content(&template.0, &mortar_strings, &player_data, &item_registry);

        // We also need to check if there is a conditional style embedded (not fully supported by simple re-resolve yet)
        // But the original spawn logic handled conditional color.
        // For now, let's just update the content. Re-implementing conditional color here would be ideal.
        //
        // 我们还需要检查是否嵌入了条件样式（目前的简单重新解析尚未完全支持）。
        // 但原始生成逻辑处理了条件颜色。
        // 目前，我们只更新内容。在此处理想情况下重新实现条件颜色。

        // Re-parsing the text3d
        *text3d = parse_text_preserving_whitespace(&new_content);

        // Note: This simple update doesn't handle the "conditional_style" color change logic present in `spawn_ui_node`.
        // To support that, we would need to store the `conditional_style` in a component too.
        // For HP update, it is usually just text change, so this might be enough for the bug report.
    }
}

pub(crate) fn ui_animation_init_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &crate::core::character_asset::CharacterAnimator,
            &UIAnimationState,
        ),
        Without<Sprite>,
    >,
    anim_configs: Res<Assets<crate::core::character_asset::AnimationConfigAsset>>,
    mut sprite_params: SpriteParams,
) {
    for (entity, animator, anim_state) in query.iter_mut() {
        let Some(config) = anim_configs.get(&animator.config) else {
            continue;
        };

        let clip_name = if let Some(mapping) = config.states.get(&anim_state.state_name) {
            mapping.get_clip_name(&crate::core::basic_components::Direction::Down)
        } else {
            warn!(
                "State {} not found in animation config for UI entity {:?}",
                anim_state.state_name, entity
            );
            continue;
        };

        let clip = match crate::core::animation::components::SpriteAnimationClip::new(
            &mut sprite_params.create_sprite_context(),
            &config.sprite_source,
            clip_name,
        ) {
            Ok(c) => c,
            Err(e) => {
                error!(
                    "Failed to load initial UI animation clip {}: {}. Using fallback.",
                    clip_name, e
                );
                crate::core::animation::components::SpriteAnimationClip::fallback(
                    &mut sprite_params.create_sprite_context(),
                    &config.sprite_source,
                    clip_name,
                )
            }
        };

        let sprite = clip.get_current_sprite().clone();
        let frame_duration = sprite_params
            .create_sprite_context()
            .get_animation_frame_duration(clip.clip_name());

        commands.entity(entity).insert((
            sprite,
            clip,
            crate::core::animation::components::SpriteAnimationCurrentFrame::default(),
            crate::core::animation::components::SpriteAnimationTimer::new(frame_duration),
        ));
    }
}

fn spawn_ui_sprite(
    parent: &mut EntityCommands,
    asset_server: &AssetServer,
    sprite_def: &SpriteDef,
    _sprite_params: &mut SpriteParams,
    node_name: &str,
    _animation_assets: &Assets<crate::core::character_asset::AnimationConfigAsset>,
) {
    let mut transform = Transform::default();
    if let Some(t_def) = &sprite_def.transform {
        transform.translation = t_def.translation.to_static_vec3();
        if let Some(scale) = &t_def.scale {
            transform.scale = scale.to_static_vec3();
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
                transform,
                Visibility::default(),
                Name::new(format!("{}_sprite", node_name)),
            ));
        });
    }
}

/// Setup custom shader sprites with Material2d.
/// 自定义着色器精灵的 Material2d 设置。
pub(crate) fn setup_hp_bar_sprites(
    mut commands: Commands,
    procedural_textures: Option<Res<super::procedural_textures::ProceduralTextures>>,
    player_data: Option<Res<crate::core::data::PlayerData>>,
    mut materials: ResMut<Assets<super::custom_sprite_material::CustomSpriteMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    // Add Without<Mesh2d> to prevent running every frame
    query: Query<(Entity, &HPBarSprite, &Transform), (Without<Sprite>, Without<Mesh2d>)>,
) {
    let Some(textures) = procedural_textures else {
        return;
    };

    // Use actual player HP if available, otherwise default to full
    let hp_ratio = if let Some(pd) = player_data {
        pd.hp as f32 / pd.hp_max as f32
    } else {
        1.0
    };

    let half_width = 40.0;

    // Create quad mesh (unit square, will be scaled by Transform)
    let mesh = meshes.add(Rectangle::new(1.0, 1.0));

    for (entity, _hp_bar, _transform) in query.iter() {
        let material = materials.add(super::custom_sprite_material::CustomSpriteMaterial {
            color_params: LinearRgba::new(hp_ratio, hp_ratio, half_width, 1.0),
            texture: textures.white_pixel.clone(),
        });

        commands.entity(entity).insert((
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material),
            HPBarLag::new(hp_ratio),
        ));

        info!(
            "[HP Bar Setup] Spawned HP bar for entity {:?}. Initial HP ratio: {:.2}",
            entity, hp_ratio
        );
    }
}

/// Update HP bar shader parameters based on player HP.
/// 根据玩家HP更新HP条着色器参数。
pub(crate) fn update_hp_bar_shader_params(
    time: Res<Time>,
    player_data: Res<crate::core::data::PlayerData>,
    mut materials: ResMut<Assets<super::custom_sprite_material::CustomSpriteMaterial>>,
    mut query: Query<(
        &MeshMaterial2d<super::custom_sprite_material::CustomSpriteMaterial>,
        &mut HPBarLag,
    )>,
) {
    let hp_ratio = player_data.hp as f32 / player_data.hp_max as f32;

        for (material_handle, mut lag) in query.iter_mut() {
            // Detect significant HP drop (Damage taken)
            if hp_ratio < lag.last_hp_ratio {
                // Start the sequence immediately
                lag.delay_timer = 0.0; 
                lag.start_lag_ratio = lag.lag_hp_ratio;
                lag.anim_progress = 0.0;
                info!("[HP Bar] Damage detected! Starting OutCirc animation immediately.");
            }
            
            lag.last_hp_ratio = hp_ratio;
    
            if hp_ratio > lag.lag_hp_ratio {
                // HEALED: Instant sync
                lag.lag_hp_ratio = hp_ratio;
                lag.anim_progress = 0.5;
                lag.delay_timer = 0.0;
            } else if hp_ratio < lag.lag_hp_ratio {
                if lag.anim_progress < 0.5 {
                    lag.anim_progress = (lag.anim_progress + time.delta_secs()).min(0.5);
                    
                    // OutCirc easing formula
                    // t: 0.0 -> 1.0
                    let t = lag.anim_progress / 0.5;
                    let eased_t = (1.0 - (t - 1.0).powi(2)).sqrt();
                    
                    // Interpolate between start and current actual HP
                    lag.lag_hp_ratio = lag.start_lag_ratio + (hp_ratio - lag.start_lag_ratio) * eased_t;
                }
            }
        // Final safety sync
        if (lag.lag_hp_ratio - hp_ratio).abs() < 0.001 {
            lag.lag_hp_ratio = hp_ratio;
        }

        let half_width = 40.0; // Match the value in RON config
        let target_params = LinearRgba::new(hp_ratio, lag.lag_hp_ratio, half_width, 1.0);

        if let Some(material) = materials.get_mut(material_handle) {
            let m_old_hp = material.color_params.red;
            let m_old_lag = material.color_params.green;

            // Always update to ensure Material is marked as changed
            material.color_params = target_params;

            // Log whenever values change significantly
            if (m_old_hp - hp_ratio).abs() > 0.001 || (m_old_lag - lag.lag_hp_ratio).abs() > 0.001 {
                info!(
                    "[HP Bar] Material Updated: Entity HP={:.3}, Lag={:.3}",
                    hp_ratio, lag.lag_hp_ratio
                );
            }
        }
    }
}

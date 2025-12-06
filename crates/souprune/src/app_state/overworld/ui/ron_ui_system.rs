use super::components::*;
use super::layout::*;
use crate::core::sprite::params::SpriteParams;
use bevy::prelude::*;
use std::time::SystemTime;

const UI_LAYOUT_ASSET_PATH: &str = "ui/backpack.ui.ron";
const UI_LAYOUT_FS_PATH: &str = "projects/example/ui/backpack.ui.ron";

#[derive(Resource)]
pub struct UILayoutHandle {
    pub handle: Handle<UILayoutAsset>,
    pub last_modified: Option<SystemTime>,
}

#[derive(Component)]
pub struct RonDrivenUI;

#[derive(Component)]
pub struct UITextId(pub String);

#[derive(Resource, Default)]
pub struct UILayoutWatcher {
    timer: Timer,
    pending_reload: bool,
}

impl UILayoutWatcher {
    fn new() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            pending_reload: false,
        }
    }
}

pub fn load_ui_layout_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<UILayoutAsset> = asset_server.load(UI_LAYOUT_ASSET_PATH);

    let last_modified = std::fs::metadata(UI_LAYOUT_FS_PATH)
        .ok()
        .and_then(|meta| meta.modified().ok());

    commands.insert_resource(UILayoutHandle {
        handle,
        last_modified,
    });
    commands.insert_resource(UILayoutWatcher::new());
    info!("Loading UI layout from RON file");
}

pub fn spawn_ron_ui_system(
    mut commands: Commands,
    ui_layout_handle: Option<Res<UILayoutHandle>>,
    ui_layouts: Res<Assets<UILayoutAsset>>,
    overworld_ui_query: Query<
        (Entity, &super::components::OverworldUI),
        (
            Added<super::components::OverworldUI>,
            Without<super::components::OverworldUIBox>,
        ),
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
        if *overworld_ui.layer() != super::components::UILayer::BACKPACK_MENU {
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
    time: Res<Time>,
    mut ui_layout_handle: Option<ResMut<UILayoutHandle>>,
    mut watcher: Option<ResMut<UILayoutWatcher>>,
    mut ui_layouts: ResMut<Assets<UILayoutAsset>>,
) {
    let Some(ref mut watcher) = watcher else {
        return;
    };

    if !watcher.timer.tick(time.delta()).just_finished() {
        return;
    }

    let Some(ref mut handle) = ui_layout_handle else {
        return;
    };

    let modified = std::fs::metadata(UI_LAYOUT_FS_PATH)
        .ok()
        .and_then(|meta| meta.modified().ok());

    if modified.is_none() {
        return;
    }

    if handle.last_modified == modified {
        return;
    }

    let bytes = match std::fs::read(UI_LAYOUT_FS_PATH) {
        Ok(bytes) => bytes,
        Err(err) => {
            warn!("Failed to read UI layout file: {err}");
            return;
        }
    };

    let parsed = match ron::de::from_bytes::<UILayoutAsset>(&bytes) {
        Ok(layout) => layout,
        Err(err) => {
            warn!("Failed to parse UI layout: {err}");
            return;
        }
    };

    if let Err(err) = ui_layouts.insert(handle.handle.id(), parsed) {
        warn!("Failed to update UI layout asset: {err}");
        return;
    }

    handle.last_modified = modified;

    watcher.pending_reload = true;

    info!("⏳ Marked for reload, will rebuild UI when asset is ready");
}

fn despawn_entity_tree(commands: &mut Commands, root: Entity) {
    // Schedule recursive despawn to avoid borrowing the world inside the system.
    commands.queue(move |world: &mut World| {
        let mut stack = vec![root];
        while let Some(entity) = stack.pop() {
            if let Ok(entity_ref) = world.get_entity(entity) {
                if let Some(children) = entity_ref.get::<Children>() {
                    for child in children.iter() {
                        stack.push(child);
                    }
                }
            }
            let _ = world.despawn(entity);
        }
    });
}

pub fn rebuild_reloaded_ui_system(
    mut commands: Commands,
    ui_layout_handle: Option<Res<UILayoutHandle>>,
    mut watcher: Option<ResMut<UILayoutWatcher>>,
    ui_layouts: Res<Assets<UILayoutAsset>>,
    overworld_ui_query: Query<(Entity, &super::components::OverworldUI), Without<RonDrivenUI>>,
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

    let has_target = overworld_ui_query.iter().any(|(_, overworld_ui)| {
        *overworld_ui.layer() == super::components::UILayer::BACKPACK_MENU
    });

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
        if *overworld_ui.layer() != super::components::UILayer::BACKPACK_MENU {
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
        "✅ RON UI hot reload complete! Rebuilt {} UI entities",
        rebuilt_count
    );
}

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
        if let Some(ui_box_logic) = &node_def.ui_box_logic {
            let visibility_rule = node_def
                .visibility_rule
                .as_ref()
                .map(parse_visibility_rule)
                .unwrap_or(UILayerVisibilityRule::Always);

            let texts = node_def
                .texts
                .iter()
                .map(|text_def| {
                    let content = resolve_text_content(
                        text_def.content.as_deref().unwrap_or(""),
                        mortar_strings,
                        player_data,
                        item_registry,
                    );

                    UITextConfig {
                        name: Name::new(text_def.id.clone()),
                        content,
                        font: text_def.font.clone().into(),
                        world_scale: text_def.world_scale.clone().into(),
                        color: bevy::color::Srgba::new(
                            text_def.color.r,
                            text_def.color.g,
                            text_def.color.b,
                            text_def.color.a,
                        ),
                        transform: Transform::from_translation(text_def.transform.clone().into()),
                        line_height: text_def.line_height.unwrap_or(1.0),
                        ..Default::default()
                    }
                })
                .collect::<Vec<_>>();

            let offset: Vec3 = ui_box_logic.offset.clone().into();

            let mut box_entity = parent.spawn((
                OverworldUIBox::new_with_texts(
                    ui_box_logic.width,
                    ui_box_logic.height,
                    ui_box_logic.border_width,
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
                let mut sprite = sprite_context.get_sprite("common", "heartsmall");
                sprite.color = Color::srgb(1.0, 0.0, 0.0);

                let cursor_position = match &cursor_def.default_position {
                    BoxCursorPositionDef::Static(vec) => {
                        BoxCursorPosition::Static(vec.clone().into())
                    }
                    BoxCursorPositionDef::Linear { origin, step } => BoxCursorPosition::Linear {
                        origin: origin.clone().into(),
                        step: step.clone().into(),
                    },
                    BoxCursorPositionDef::Custom { positions } => BoxCursorPosition::Custom(
                        positions.iter().map(|v| v.clone().into()).collect(),
                    ),
                };

                let cursor_visibility =
                    if let UILayerVisibilityRule::OnlyIn(ref layers) = visibility_rule {
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

                box_entity.insert(BoxCursor::new(
                    sprite,
                    cursor_visibility,
                    placement,
                    Transform::from_scale(Vec3::splat(1.0)),
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

fn resolve_text_content(
    template: &str,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &crate::core::data::PlayerData,
    item_registry: &crate::core::item::ItemRegistry,
) -> String {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if let Some(&next_ch) = chars.peek() {
                if next_ch == '{' {
                    chars.next();
                    let mut key = String::new();
                    let mut found_closing = false;

                    while let Some(ch) = chars.next() {
                        if ch == '}' {
                            if let Some(&next_ch) = chars.peek() {
                                if next_ch == '}' {
                                    chars.next();
                                    found_closing = true;
                                    break;
                                }
                            }
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

                    while let Some(ch) = chars.next() {
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
        }
        result.push(ch);
    }

    result
}

fn resolve_data_path(
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
            if let Some(item) = item_registry.get(&player_data.weapon) {
                let key = format!("{}:{}", item.locate_file, item.locate_name);
                mortar_strings.resolve(&key).to_string()
            } else {
                player_data.weapon.clone()
            }
        }
        "player.weapon_atk" => {
            if let Some(item) = item_registry.get(&player_data.weapon) {
                if let ItemType::Weapon { damage, .. } = item.item_type {
                    return damage.to_string();
                }
            }
            "0".to_string()
        }
        "player.total_attack" => {
            let weapon_atk = if let Some(item) = item_registry.get(&player_data.weapon) {
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
            if let Some(item) = item_registry.get(&player_data.armor) {
                let key = format!("{}:{}", item.locate_file, item.locate_name);
                mortar_strings.resolve(&key).to_string()
            } else {
                player_data.armor.clone()
            }
        }
        "player.armor_def" => {
            if let Some(item) = item_registry.get(&player_data.armor) {
                if let ItemType::Armor { defense } = item.item_type {
                    return defense.to_string();
                }
            }
            "0".to_string()
        }
        "player.total_defense" => {
            let armor_def = if let Some(item) = item_registry.get(&player_data.armor) {
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

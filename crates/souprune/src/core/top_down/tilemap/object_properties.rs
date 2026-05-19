//! # object_properties.rs
//!
//! ## Module Overview
//! This module handles custom property detection and processing for Tiled objects.
//! It provides a unified trigger system that supports multiple trigger modes
//! (confirm, enter, exit) and auto-generates FRE rules for dialogue.
//!
//! ## 模块概述
//! 该模块处理 Tiled 对象的自定义属性检测和处理。
//! 它提供了统一的触发系统，支持多种触发模式（confirm、enter、exit），
//! 并为对话自动生成 FRE 规则。

use crate::config::SoupruneConfig;
use crate::core::camera::{CameraBoundsZone, TiledCameraBounds};
use crate::core::collision::Rect2DCollider;
use crate::core::game_action::{GameActionDef, GameRule, GameRuleRegistry};
use crate::core::map_property_schema::{
    get_object_bool_property, get_object_string_property, object_keys,
};
use crate::core::mode::{ModeScoped, SequenceMode};
use crate::core::top_down::tilemap::systems::TilemapCollider;
use crate::core::top_down::trigger::{Interactable, TriggerZone};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TiledMapAsset, tiled};
use bevy_fact_rule_event::RuleScope;

/// Marker component for objects with collision property
/// 具有碰撞属性的对象的标记组件
#[derive(Component)]
pub struct ObjectCollider;

/// Marker component for trigger objects spawned from Tiled (unified).
/// 从 Tiled 生成的触发对象标记组件（统一）。
#[derive(Component)]
pub struct TiledTriggerObject;

/// System to process Tiled object layer properties and spawn corresponding entities.
/// Handles collision objects, trigger zones, interactables, and auto-generated rules.
///
/// 处理 Tiled 对象层属性并生成相应实体的系统。
/// 处理碰撞对象、触发区域、可交互物体和自动生成的规则。
#[allow(dead_code)]
pub fn process_map_object_properties_system(
    mut commands: Commands,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    tiled_maps_query: Query<&TiledMap>,
    existing_triggers: Query<&TiledTriggerObject>,
    loaded_rule_sets: Res<crate::core::top_down::trigger::LoadedRuleSets>,
    mut registry: ResMut<GameRuleRegistry>,
    config: Res<SoupruneConfig>,
    sequence_mode: Res<SequenceMode>,
) {
    if !existing_triggers.is_empty() {
        return;
    }
    if !loaded_rule_sets.initialized {
        return;
    }
    let Some(scope) = crate::core::top_down::top_down_scoped(&sequence_mode) else {
        return;
    };

    let map_count = tiled_maps_query.iter().count();
    if map_count == 0 {
        return;
    }

    let mut any_map_loaded = false;
    for tiled_map_handle in tiled_maps_query.iter() {
        if tiled_map_assets.get(&tiled_map_handle.0).is_some() {
            any_map_loaded = true;
            break;
        }
    }

    if !any_map_loaded {
        return;
    }

    info!("Object properties system processing: {} maps", map_count);

    for tiled_map_handle in tiled_maps_query.iter() {
        let Some(tiled_map_asset) = tiled_map_assets.get(&tiled_map_handle.0) else {
            continue;
        };

        info!("Processing tiled map for trigger objects");

        let tile_width = tiled_map_asset.map.tile_width as f32;
        let tile_height = tiled_map_asset.map.tile_height as f32;
        let map_width = tiled_map_asset.map.width as f32 * tile_width;
        let map_height = tiled_map_asset.map.height as f32 * tile_height;
        let center_offset_x = -map_width / 2.0;
        let center_offset_y = -map_height / 2.0;

        for layer in tiled_map_asset.map.layers() {
            let Some(object_layer) = layer.as_object_layer() else {
                continue;
            };

            trace!(
                "Processing object layer '{}' with {} objects",
                layer.name,
                object_layer.objects().count()
            );

            if layer.name == object_keys::CAMERA_BOUNDS {
                spawn_camera_bounds_from_layer(
                    &mut commands,
                    scope.clone(),
                    &object_layer,
                    center_offset_x,
                    center_offset_y,
                    map_height,
                );
                continue;
            }

            for object_data in object_layer.objects() {
                process_tiled_object(
                    &mut commands,
                    &object_data,
                    center_offset_x,
                    center_offset_y,
                    map_height,
                    &mut registry,
                    &config,
                    scope.clone(),
                );
            }
        }

        break;
    }

    info!("Object properties system completed processing");
}

/// Process a single Tiled object, dispatching to the appropriate handler.
///
/// 处理单个 Tiled 对象，分发到对应的处理器。
fn process_tiled_object(
    commands: &mut Commands,
    object_data: &tiled::ObjectData,
    center_offset_x: f32,
    center_offset_y: f32,
    map_height: f32,
    registry: &mut GameRuleRegistry,
    config: &SoupruneConfig,
    scope: ModeScoped,
) {
    trace!(
        "Checking object '{}' at ({}, {}) with shape {:?}",
        object_data.name, object_data.x, object_data.y, object_data.shape
    );

    // Camera bounds zones
    if get_object_bool_property(&object_data.properties, object_keys::CAMERA_BOUNDS) == Some(true) {
        spawn_camera_bounds_zone(
            commands,
            scope,
            object_data,
            center_offset_x,
            center_offset_y,
            map_height,
        );
        return;
    }

    // Unified object kinds (collision, trigger, etc.)
    if let Some(kind_str) = get_object_string_property(&object_data.properties, object_keys::KIND) {
        let kinds = ObjectKinds::parse(kind_str);

        // Collision is handled by generate_collision_tiles_system
        if kinds.has_trigger() {
            spawn_trigger_object(
                commands,
                object_data,
                center_offset_x,
                center_offset_y,
                map_height,
                &kinds,
                registry,
                config,
                scope,
            );
        }
    }
}

/// Parsed object kinds from comma-separated `kind` property.
///
/// 从逗号分隔的 `kind` 属性解析出的对象类型集合。
struct ObjectKinds {
    collision: bool,
    confirm: bool,
    enter: bool,
    exit: bool,
}

impl ObjectKinds {
    fn parse(kind_str: &str) -> Self {
        let mut kinds = Self {
            collision: false,
            confirm: false,
            enter: false,
            exit: false,
        };
        for part in kind_str.split(',') {
            match part.trim() {
                "collision" => kinds.collision = true,
                "confirm" => kinds.confirm = true,
                "enter" => kinds.enter = true,
                "exit" => kinds.exit = true,
                other => warn!(
                    "Unknown object kind '{}', expected collision/confirm/enter/exit",
                    other
                ),
            }
        }
        kinds
    }

    fn has_trigger(&self) -> bool {
        self.confirm || self.enter || self.exit
    }
}

/// Dialogue properties extracted from a Tiled object.
///
/// 从 Tiled 对象中提取的对话属性。
struct DialogueProps {
    mortar: String,
    node: String,
    view: Option<String>,
    typewriter: bool,
    focus: bool,
    voice: Option<String>,
}

impl DialogueProps {
    fn from_object(object_data: &tiled::ObjectData, default_view: &str) -> Option<Self> {
        let mortar = get_object_string_property(&object_data.properties, object_keys::MORTAR)?;

        let node = get_object_string_property(&object_data.properties, object_keys::MORTAR_NODE)
            .map(|s| s.to_string())
            .unwrap_or_else(|| object_data.name.clone());

        let view = get_object_string_property(&object_data.properties, object_keys::VIEW)
            .map(|s| s.to_string())
            .or_else(|| {
                if default_view.is_empty() {
                    None
                } else {
                    Some(default_view.to_string())
                }
            });

        let typewriter = get_object_bool_property(&object_data.properties, object_keys::TYPEWRITER)
            .unwrap_or(true);

        let focus =
            get_object_bool_property(&object_data.properties, object_keys::FOCUS).unwrap_or(true);

        let voice = get_object_string_property(&object_data.properties, object_keys::VOICE)
            .map(|s| s.to_string());

        Some(Self {
            mortar: mortar.to_string(),
            node,
            view,
            typewriter,
            focus,
            voice,
        })
    }

    fn to_action(&self) -> GameActionDef {
        GameActionDef::StartDialogue {
            channel: None,
            mortar: self.mortar.clone(),
            node: self.node.clone(),
            view: self.view.clone(),
            typewriter: self.typewriter,
            focus: self.focus,
            voice: self.voice.clone(),
        }
    }
}

/// Spawn a unified trigger object from Tiled properties.
///
/// Parses trigger modes (confirm, enter, exit), attaches the appropriate
/// components, and auto-generates StartDialogue FRE rules if dialogue
/// properties are present.
///
/// 从 Tiled 属性生成统一触发对象。
///
/// 解析触发模式（confirm、enter、exit），挂载相应组件，
/// 如果存在对话属性则自动生成 StartDialogue FRE 规则。
fn spawn_trigger_object(
    commands: &mut Commands,
    object_data: &tiled::ObjectData,
    center_offset_x: f32,
    center_offset_y: f32,
    map_height: f32,
    kinds: &ObjectKinds,
    registry: &mut GameRuleRegistry,
    config: &SoupruneConfig,
    scope: ModeScoped,
) {
    let tiled::ObjectShape::Rect { width, height } = object_data.shape else {
        trace!("Trigger object '{}' is not a rectangle", object_data.name);
        return;
    };

    let object_id = if !object_data.name.is_empty() {
        object_data.name.clone()
    } else {
        format!("trigger_{}", object_data.id())
    };

    let world_x = center_offset_x + object_data.x + width / 2.0;
    let world_y = center_offset_y + (map_height - object_data.y - height / 2.0);
    let size = Vec2::new(width, height);

    info!(
        "FRE: Creating trigger object '{}' at ({:.1}, {:.1}) size ({}, {})",
        object_id, world_x, world_y, width, height
    );

    let mut entity_cmd = commands.spawn((
        scope,
        TiledTriggerObject,
        Rect2DCollider::new(size, Vec2::ZERO),
        Transform::from_xyz(world_x, world_y, 0.0),
        Visibility::Hidden,
        Name::new(format!("TriggerObject_{}", object_id)),
    ));

    if kinds.enter || kinds.exit {
        entity_cmd.insert(TriggerZone::new(&object_id));
    }

    if kinds.confirm {
        entity_cmd.insert(Interactable::new(&object_id));
    }

    // Auto-generate dialogue rules from Tiled properties
    let dialogue_props =
        DialogueProps::from_object(object_data, &config.game.dialogue_view_default);
    if let Some(props) = &dialogue_props {
        let action = props.to_action();

        if kinds.confirm {
            let rule_id = format!("auto_interact_{}", object_id);
            let event_id = format!("interact_{}", object_id);
            let rule = GameRule::builder(rule_id, event_id)
                .scope(RuleScope::Local)
                .priority(-100)
                .action(action.clone())
                .build();
            registry.register(rule);
            info!("FRE: Auto-generated confirm rule for '{}'", object_id);
        }

        if kinds.enter {
            let rule_id = format!("auto_trigger_enter_{}", object_id);
            let event_id = format!("trigger_enter_{}", object_id);
            let rule = GameRule::builder(rule_id, event_id)
                .scope(RuleScope::Local)
                .priority(-100)
                .action(action.clone())
                .build();
            registry.register(rule);
            info!("FRE: Auto-generated enter rule for '{}'", object_id);
        }

        if kinds.exit {
            let rule_id = format!("auto_trigger_exit_{}", object_id);
            let event_id = format!("trigger_exit_{}", object_id);
            let rule = GameRule::builder(rule_id, event_id)
                .scope(RuleScope::Local)
                .priority(-100)
                .action(action)
                .build();
            registry.register(rule);
            info!("FRE: Auto-generated exit rule for '{}'", object_id);
        }
    }
}

/// Spawn a collision entity from a Tiled object.
///
/// 从 Tiled 对象生成碰撞实体。
fn spawn_collision_object(
    commands: &mut Commands,
    object_data: &tiled::ObjectData,
    center_offset_x: f32,
    center_offset_y: f32,
    map_height: f32,
) {
    let tiled::ObjectShape::Rect { width, height } = object_data.shape else {
        trace!("Collision object '{}' is not a rectangle", object_data.name);
        return;
    };

    let world_x = center_offset_x + object_data.x + width / 2.0;
    let world_y = center_offset_y + (map_height - object_data.y - height / 2.0);
    let size = Vec2::new(width, height);

    trace!(
        "Creating collision object '{}' at ({}, {}) size ({}, {})",
        object_data.name, world_x, world_y, width, height
    );

    commands.spawn((
        ObjectCollider,
        TilemapCollider,
        Rect2DCollider::new(size, Vec2::ZERO),
        Transform::from_xyz(world_x, world_y, 0.0),
        Visibility::Hidden,
        Name::new(format!("ObjectCollision_{}", object_data.name)),
    ));
}

/// Spawn camera bounds zones for all objects in a dedicated camera bounds layer.
///
/// 从专用摄像机边界图层中为所有对象生成摄像机边界区域。
fn spawn_camera_bounds_from_layer(
    commands: &mut Commands,
    scope: ModeScoped,
    object_layer: &tiled::ObjectLayer,
    center_offset_x: f32,
    center_offset_y: f32,
    map_height: f32,
) {
    for object_data in object_layer.objects() {
        spawn_camera_bounds_zone(
            commands,
            scope.clone(),
            &object_data,
            center_offset_x,
            center_offset_y,
            map_height,
        );
    }
}

/// Spawn a camera bounds zone entity from a Tiled object.
///
/// 从 Tiled 对象生成摄像机边界区域实体。
fn spawn_camera_bounds_zone(
    commands: &mut Commands,
    scope: ModeScoped,
    object_data: &tiled::ObjectData,
    center_offset_x: f32,
    center_offset_y: f32,
    map_height: f32,
) {
    let tiled::ObjectShape::Rect { width, height } = object_data.shape else {
        warn!(
            "Camera bounds object '{}' is not a rectangle, skipping",
            object_data.name
        );
        return;
    };

    let world_x = center_offset_x + object_data.x + width / 2.0;
    let world_y = center_offset_y + (map_height - object_data.y - height / 2.0);

    let half_w = width / 2.0;
    let half_h = height / 2.0;
    let rect = bevy::math::Rect::new(
        world_x - half_w,
        world_y - half_h,
        world_x + half_w,
        world_y + half_h,
    );

    let zone_name = if !object_data.name.is_empty() {
        object_data.name.clone()
    } else {
        format!("camera_zone_{}", object_data.id())
    };

    info!(
        "Creating camera bounds zone '{}' at ({:.1}, {:.1}) size ({}, {})",
        zone_name, world_x, world_y, width, height
    );

    commands.spawn((
        scope,
        TiledCameraBounds,
        CameraBoundsZone { rect },
        Name::new(format!("CameraBounds_{}", zone_name)),
    ));
}

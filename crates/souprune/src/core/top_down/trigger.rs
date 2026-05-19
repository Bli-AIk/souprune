//! # trigger.rs
//!
//! FRE-based trigger zones and interactable objects for top_down areas.

use crate::core::basic_components::{Direction, Facing};
use crate::core::collision::Rect2DCollider;
use crate::core::input::{InputCommand, InputEnvelopeEvent, InputTarget};
use crate::core::map_property_schema::{get_string_property, keys};
use crate::core::top_down::character::components::PlayerControlled;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::{TiledMap, TiledMapAsset};
use bevy_fact_rule_event::{FactEvent, FactEventId, LayeredFactDatabase};

use crate::core::game_action::{GameFreAsset, GameRuleRegistry};

mod action_handlers;

pub use action_handlers::{
    PendingDanmakuActions, PendingViewActions, apply_pending_view_actions_system,
    log_unhandled_top_down_custom_actions_system, play_danmaku_from_actions_system,
    setup_action_handlers_system,
};

/// Marker component for trigger zones.
#[derive(Component, Debug)]
pub struct TriggerZone {
    /// Unique identifier for this trigger.
    pub id: String,
    /// Event to emit when player enters this zone.
    pub enter_event: String,
    /// Whether the player is currently inside this zone.
    pub player_inside: bool,
}

impl TriggerZone {
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        let enter_event = format!("trigger_enter_{}", id);
        Self {
            id,
            enter_event,
            player_inside: false,
        }
    }
}

/// Marker component for interactable objects.
#[derive(Component, Debug)]
pub struct Interactable {
    /// Unique identifier for this interactable.
    pub id: String,
    /// Maximum interaction distance from player.
    pub max_distance: f32,
}

impl Default for Interactable {
    fn default() -> Self {
        Self {
            id: String::new(),
            max_distance: 20.0,
        }
    }
}

impl Interactable {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            max_distance: 20.0,
        }
    }

    pub fn with_distance(mut self, distance: f32) -> Self {
        self.max_distance = distance;
        self
    }
}

/// Resource to track the currently focused interactable entity.
#[derive(Resource, Default)]
pub struct FocusedInteractable {
    pub entity: Option<Entity>,
    pub id: Option<String>,
}

/// Resource to track loaded rule set handles.
#[derive(Resource, Default)]
pub struct LoadedRuleSets {
    pub handles: Vec<Handle<GameFreAsset>>,
    pub initialized: bool,
    pub registered: bool,
}

/// System to detect player entering/exiting trigger zones and emit FRE events.
pub fn trigger_zone_detection_system(
    mut triggers: Query<(&Transform, &Rect2DCollider, &mut TriggerZone)>,
    player: Query<(&Transform, &Rect2DCollider, Entity), With<PlayerControlled>>,
    mut event_writer: MessageWriter<FactEvent>,
) {
    let Ok((player_transform, player_collider, player_entity)) = player.single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate() + player_collider.offset;
    let player_half_size = player_collider.size * 0.5;

    for (trigger_transform, trigger_collider, mut trigger) in triggers.iter_mut() {
        let trigger_pos = trigger_transform.translation.truncate() + trigger_collider.offset;
        let trigger_half_size = trigger_collider.size * 0.5;

        let overlap = (player_pos.x - trigger_pos.x).abs()
            < (player_half_size.x + trigger_half_size.x)
            && (player_pos.y - trigger_pos.y).abs() < (player_half_size.y + trigger_half_size.y);

        if overlap && !trigger.player_inside {
            trigger.player_inside = true;
            info!("FRE: Player entered trigger zone '{}'", trigger.id);
            event_writer.write(
                FactEvent::with_entity(trigger.enter_event.clone(), player_entity)
                    .with_data("trigger_id", &trigger.id),
            );
        } else if !overlap && trigger.player_inside {
            trigger.player_inside = false;
            info!("FRE: Player exited trigger zone '{}'", trigger.id);
            let exit_event = format!("trigger_exit_{}", trigger.id);
            event_writer.write(
                FactEvent::with_entity(exit_event, player_entity)
                    .with_data("trigger_id", &trigger.id),
            );
        }
    }
}

/// System to load FRE rules from map's `rules_file` property and dependency chain.
pub fn load_fre_rules_system(
    asset_server: Res<AssetServer>,
    mut loaded_rule_sets: ResMut<LoadedRuleSets>,
    tiled_maps: Query<&TiledMap>,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    sequence_mode: Res<crate::core::mode::SequenceMode>,
    mode_registry: Res<crate::core::mode::ModeRegistry>,
) {
    if loaded_rule_sets.initialized {
        return;
    }

    let mode_rules = sequence_mode
        .0
        .as_deref()
        .and_then(|mode| mode_registry.mode(mode))
        .map(|mode_config| mode_config.rules.as_slice())
        .unwrap_or(&[]);

    // Load project-declared rules for the active top-down mode.
    for rules_path in mode_rules {
        let handle: Handle<GameFreAsset> = asset_server.load(rules_path);
        loaded_rule_sets.handles.push(handle);
        info!("FRE: Loading top-down mode rule: {}", rules_path);
    }

    // Load per-map rules from the map's rules_file property
    for tiled_map in tiled_maps.iter() {
        if let Some(map_asset) = tiled_map_assets.get(&tiled_map.0)
            && let Some(rules_path) =
                get_string_property(&map_asset.map.properties, keys::RULES_FILE)
        {
            let rules_path_owned = rules_path.to_string();
            let handle: Handle<GameFreAsset> = asset_server.load(&rules_path_owned);
            loaded_rule_sets.handles.push(handle);
            loaded_rule_sets.initialized = true;
            info!(
                "FRE: Loading rules from map property '{}': {}",
                keys::RULES_FILE,
                rules_path_owned
            );
            return;
        }
    }

    if !tiled_maps.is_empty() {
        for tiled_map in tiled_maps.iter() {
            if tiled_map_assets.get(&tiled_map.0).is_some() {
                loaded_rule_sets.initialized = true;
                info!("FRE: No rules_file property found in map");
                return;
            }
        }
    }
}

/// System to register rules from loaded assets.
pub fn register_loaded_rules_system(
    mut loaded_rule_sets: ResMut<LoadedRuleSets>,
    fre_assets: Res<Assets<GameFreAsset>>,
    mut registry: ResMut<GameRuleRegistry>,
    mut fact_db: ResMut<LayeredFactDatabase>,
    mut enum_registry: ResMut<bevy_fact_rule_event::EnumRegistry>,
) {
    if loaded_rule_sets.registered || !loaded_rule_sets.initialized {
        return;
    }

    let all_loaded = loaded_rule_sets
        .handles
        .iter()
        .all(|h| fre_assets.get(h).is_some());

    if !all_loaded {
        return;
    }

    for handle in &loaded_rule_sets.handles {
        let Some(fre_asset) = fre_assets.get(handle) else {
            continue;
        };

        enum_registry.register_from_asset(fre_asset);

        for (key, value) in fre_asset.resolve_facts(&enum_registry) {
            fact_db.set_local(key.as_str(), value);
            info!("FRE: Set fact '{}' to Local layer from FRE file", key);
        }

        fre_asset.register_rules_layered(&mut registry);
        info!("FRE: Rules registered from FRE asset");
    }

    loaded_rule_sets.registered = true;
    info!(
        "FRE: All {} rule sets registered",
        loaded_rule_sets.handles.len()
    );
}

/// System to log fact database changes (debug).
pub fn log_fact_changes_system(
    mut events: MessageReader<FactEvent>,
    fact_db: Res<LayeredFactDatabase>,
) {
    for event in events.read() {
        if event.id == FactEventId::new("demo_visit_updated") {
            let count = fact_db.get_int_or("demo_area_visit_count", 0);
            info!("FRE: demo_area_visit_count = {}", count);
        }
    }
}

fn ray_aabb_intersection(
    origin: Vec2,
    dir: Vec2,
    center: Vec2,
    half_size: Vec2,
    max_dist: f32,
) -> Option<f32> {
    let min = center - half_size;
    let max = center + half_size;
    let (mut t_min, mut t_max) = (0.0_f32, max_dist);

    if dir.x.abs() < 1e-6 {
        if origin.x < min.x || origin.x > max.x {
            return None;
        }
    } else {
        let inv_d = 1.0 / dir.x;
        let mut t1 = (min.x - origin.x) * inv_d;
        let mut t2 = (max.x - origin.x) * inv_d;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }
        t_min = t_min.max(t1);
        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    }

    if dir.y.abs() < 1e-6 {
        if origin.y < min.y || origin.y > max.y {
            return None;
        }
    } else {
        let inv_d = 1.0 / dir.y;
        let mut t1 = (min.y - origin.y) * inv_d;
        let mut t2 = (max.y - origin.y) * inv_d;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }
        t_min = t_min.max(t1);
        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    }

    if t_min >= 0.0 && t_min <= max_dist {
        Some(t_min)
    } else if t_max >= 0.0 && t_max <= max_dist {
        Some(t_max)
    } else {
        None
    }
}

/// System to detect interactable objects in front of the player.
pub fn interactable_detection_system(
    player_query: Query<(&Transform, &Facing, &Rect2DCollider), With<PlayerControlled>>,
    interactables: Query<(Entity, &Transform, &Interactable, Option<&Rect2DCollider>)>,
    mut focused: ResMut<FocusedInteractable>,
    mut logged_once: Local<bool>,
) {
    let Ok((player_transform, facing, player_collider)) = player_query.single() else {
        return;
    };

    if !*logged_once {
        let count = interactables.iter().count();
        info!(
            "Interactable detection: found {} interactable entities",
            count
        );
        *logged_once = true;
    }

    let player_pos = player_transform.translation.truncate() + player_collider.offset;
    let facing_dir = match facing.value {
        Direction::Up | Direction::UpLeft | Direction::UpRight => Vec2::Y,
        Direction::Down | Direction::DownLeft | Direction::DownRight => -Vec2::Y,
        Direction::Left => -Vec2::X,
        Direction::Right => Vec2::X,
    };

    let mut best_match: Option<(Entity, String, f32)> = None;

    for (entity, interactable_transform, interactable, opt_collider) in interactables.iter() {
        let (center, half_size) = match opt_collider {
            Some(collider) => (
                interactable_transform.translation.truncate() + collider.offset,
                collider.size / 2.0,
            ),
            None => (
                interactable_transform.translation.truncate(),
                Vec2::splat(8.0),
            ),
        };

        if let Some(hit_dist) = ray_aabb_intersection(
            player_pos,
            facing_dir,
            center,
            half_size,
            interactable.max_distance,
        ) && best_match
            .as_ref()
            .is_none_or(|(_, _, best_dist)| hit_dist < *best_dist)
        {
            best_match = Some((entity, interactable.id.clone(), hit_dist));
        }
    }

    match best_match {
        Some((entity, id, _)) => {
            if focused.entity != Some(entity) {
                focused.entity = Some(entity);
                focused.id = Some(id.clone());
                debug!("FRE: Player can interact with '{}'", id);
            }
        }
        None => {
            if focused.entity.is_some() {
                debug!("FRE: No interactable in range");
                focused.entity = None;
                focused.id = None;
            }
        }
    }
}

/// System to handle player interaction when confirm is pressed.
pub fn handle_interaction_input_system(
    mut input_events: MessageReader<InputEnvelopeEvent>,
    focused: Res<FocusedInteractable>,
    interactables: Query<&Interactable>,
    current_state: Res<State<crate::core::mode::SequenceSubState>>,
    state_config: Option<Res<crate::core::state_config::LoadedStateConfig>>,
    mut event_writer: MessageWriter<FactEvent>,
) {
    let can_interact = state_config
        .as_ref()
        .map(|config| config.can_interact(&current_state.0))
        .unwrap_or(true);

    if !can_interact {
        return;
    }

    let has_confirm = input_events.read().any(|event| {
        matches!(event.envelope.target, InputTarget::FreScope)
            && matches!(event.envelope.command, InputCommand::Confirm)
    });
    if !has_confirm {
        return;
    }

    let Some(entity) = focused.entity else {
        return;
    };

    let Some(ref interactable_id) = focused.id else {
        return;
    };

    if interactables.get(entity).is_err() {
        warn!(
            "FRE: Focused entity {:?} has no Interactable component",
            entity
        );
        return;
    }

    let event_id = format!("interact_{}", interactable_id);
    info!(
        "FRE: Player interacting with '{}', emitting '{}'",
        interactable_id, event_id
    );
    event_writer.write(FactEvent::new(event_id));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::game_action::GameActionDef;
    use crate::core::input::{
        InputCommand, InputContextId, InputEnvelope, InputEnvelopeEvent, InputTarget,
    };
    use crate::core::mode::SequenceSubState;
    use bevy::ecs::message::Messages;

    fn app_with_interaction_input_system() -> App {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.add_message::<InputEnvelopeEvent>()
            .add_message::<FactEvent>()
            .init_state::<SequenceSubState>()
            .init_resource::<FocusedInteractable>()
            .add_systems(Update, handle_interaction_input_system);
        app
    }

    fn write_confirm(app: &mut App) {
        app.world_mut()
            .write_message(InputEnvelopeEvent::new(InputEnvelope::new(
                InputContextId::Mode("field".to_string()),
                InputTarget::FreScope,
                InputCommand::Confirm,
                "Confirm",
            )));
    }

    fn fact_event_ids(app: &App) -> Vec<String> {
        let events = app.world().resource::<Messages<FactEvent>>();
        let mut cursor = events.get_cursor();
        cursor
            .read(events)
            .map(|event| event.id.0.clone())
            .collect()
    }

    #[test]
    fn confirm_envelope_triggers_focused_interactable() {
        let mut app = app_with_interaction_input_system();
        let entity = app.world_mut().spawn(Interactable::new("dummy")).id();
        *app.world_mut().resource_mut::<FocusedInteractable>() = FocusedInteractable {
            entity: Some(entity),
            id: Some("dummy".to_string()),
        };

        write_confirm(&mut app);
        app.update();

        assert_eq!(fact_event_ids(&app), vec!["interact_dummy"]);
    }

    #[test]
    fn confirm_envelope_without_focus_is_ignored() {
        let mut app = app_with_interaction_input_system();

        write_confirm(&mut app);
        app.update();

        assert!(fact_event_ids(&app).is_empty());
    }

    #[test]
    fn interaction_menu_rule_uses_semantic_menu_event_to_set_sub_state() {
        let rule = bevy_fact_rule_event::RuleDef {
            id: "open_menu_state".to_string(),
            event: bevy_fact_rule_event::RuleEventDef::Event("input:menu".to_string()),
            conditions: Vec::new(),
            actions: vec![GameActionDef::Custom {
                action_type: "SetSubState".to_string(),
                params: [("state".to_string(), "Backpack".to_string())]
                    .into_iter()
                    .collect(),
            }],
            modifications: Vec::new(),
            outputs: Vec::new(),
            enabled: true,
            priority: 0,
            consume_event: true,
        };

        assert_eq!(rule.event.to_event_id(), "input:menu");
        assert!(rule.actions.iter().any(|action| {
            matches!(
                action,
                GameActionDef::Custom { action_type, params }
                if action_type == "SetSubState"
                    && params.get("state").map(String::as_str) == Some("Backpack")
            )
        }));
    }
}

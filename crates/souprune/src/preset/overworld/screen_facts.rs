//! Synchronizes overworld screen-space facts for FRE rules.
//!
//! 为 FRE 规则同步 Overworld 屏幕空间 facts。

use crate::core::camera::MainGameCamera;
use crate::core::mode::SequenceSubState;
use crate::core::state_config::LoadedStateConfig;
use crate::preset::overworld::character::components::PlayerControlled;
use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};

pub(crate) fn sync_overworld_screen_facts_system(
    mut facts: ResMut<LayeredFactDatabase>,
    mut event_writer: MessageWriter<FactEvent>,
    state_config: Option<Res<LoadedStateConfig>>,
    sub_state: Res<State<SequenceSubState>>,
    player: Query<&Transform, (With<PlayerControlled>, Without<Camera2d>)>,
    camera: Query<(&Transform, &Projection, &Camera), (With<Camera2d>, With<MainGameCamera>)>,
) {
    let Some(projection_config) = state_config
        .as_ref()
        .and_then(|config| config.get_screen_fact_projection(sub_state.name()))
    else {
        return;
    };

    let Ok(player_transform) = player.single() else {
        return;
    };
    let Some((camera_transform, projection, _)) =
        camera.iter().find(|(_, _, camera)| camera.is_active)
    else {
        return;
    };
    let Some(view_size) = orthographic_visible_size(projection) else {
        return;
    };

    let screen_pos = player_screen_position_from_view_top_left(
        player_transform.translation.truncate(),
        camera_transform.translation.truncate(),
        view_size,
    );

    if let Some(fact_key) = projection_config.player_x_fact.as_deref() {
        facts.set_global_if_changed(fact_key, FactValue::Float(screen_pos.x as f64));
    }
    if let Some(fact_key) = projection_config.player_y_fact.as_deref() {
        facts.set_global_if_changed(fact_key, FactValue::Float(screen_pos.y as f64));
    }
    if let Some(event_id) = projection_config.updated_event.as_deref() {
        event_writer.write(FactEvent::new(event_id));
    }
}

fn orthographic_visible_size(projection: &Projection) -> Option<Vec2> {
    let Projection::Orthographic(orthographic) = projection else {
        return None;
    };
    let size = Vec2::new(
        orthographic.area.width().abs(),
        orthographic.area.height().abs(),
    );
    (size.x > 0.0 && size.y > 0.0).then_some(size)
}

fn player_screen_position_from_view_top_left(
    player_world: Vec2,
    camera_world: Vec2,
    view_size: Vec2,
) -> Vec2 {
    Vec2::new(
        player_world.x - camera_world.x + view_size.x * 0.5,
        camera_world.y + view_size.y * 0.5 - player_world.y,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fact_rule_event::FactValue;
    use souprune_schema::config::{
        ScreenFactProjectionDef, StateConfig as SchemaStateConfig, StateDefinition,
    };
    use std::collections::HashMap;

    #[test]
    fn bevy_world_position_is_converted_to_gms_screen_position() {
        let position = player_screen_position_from_view_top_left(
            Vec2::new(-320.0, 240.0),
            Vec2::ZERO,
            Vec2::new(640.0, 480.0),
        );
        assert_eq!(position, Vec2::ZERO);

        let position = player_screen_position_from_view_top_left(
            Vec2::new(0.0, 100.0),
            Vec2::ZERO,
            Vec2::new(640.0, 480.0),
        );
        assert_eq!(position, Vec2::new(320.0, 140.0));
    }

    #[test]
    fn screen_projection_is_owned_by_state_config() {
        let mut states = HashMap::new();
        states.insert(
            "Backpack".to_string(),
            StateDefinition {
                screen_fact_projection: Some(ScreenFactProjectionDef {
                    player_x_fact: Some("project:screen_x".to_string()),
                    player_y_fact: Some("project:screen_y".to_string()),
                    updated_event: Some("project:screen_updated".to_string()),
                }),
                ..Default::default()
            },
        );
        let config = LoadedStateConfig(SchemaStateConfig { states });
        let projection = config
            .get_screen_fact_projection("Backpack")
            .expect("Backpack should declare screen projection");

        let mut facts = LayeredFactDatabase::new();
        let position = player_screen_position_from_view_top_left(
            Vec2::new(0.0, 100.0),
            Vec2::ZERO,
            Vec2::new(640.0, 480.0),
        );

        facts.set_global(
            projection
                .player_x_fact
                .as_deref()
                .expect("x fact should be configured"),
            FactValue::Float(position.x as f64),
        );
        facts.set_global(
            projection
                .player_y_fact
                .as_deref()
                .expect("y fact should be configured"),
            FactValue::Float(position.y as f64),
        );

        assert_eq!(facts.get_float("project:screen_x"), Some(320.0));
        assert_eq!(facts.get_float("project:screen_y"), Some(140.0));
        assert_eq!(
            projection.updated_event.as_deref(),
            Some("project:screen_updated")
        );
        assert!(config.get_screen_fact_projection("Normal").is_none());
    }
}

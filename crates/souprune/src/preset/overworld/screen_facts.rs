//! Synchronizes overworld screen-space facts for FRE rules.
//!
//! 为 FRE 规则同步 Overworld 屏幕空间 facts。

use crate::core::camera::MainGameCamera;
use crate::preset::overworld::character::components::PlayerControlled;
use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};

const SCREEN_FACTS_UPDATED_EVENT: &str = "overworld:screen_facts_updated";
const PLAYER_SCREEN_X_FACT: &str = "overworld:player_screen_x";
const PLAYER_SCREEN_Y_FACT: &str = "overworld:player_screen_y";

pub(crate) fn sync_overworld_screen_facts_system(
    mut facts: ResMut<LayeredFactDatabase>,
    mut event_writer: MessageWriter<FactEvent>,
    player: Query<&Transform, (With<PlayerControlled>, Without<Camera2d>)>,
    camera: Query<(&Transform, &Projection, &Camera), (With<Camera2d>, With<MainGameCamera>)>,
) {
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

    facts.set_global_if_changed(PLAYER_SCREEN_X_FACT, FactValue::Float(screen_pos.x as f64));
    facts.set_global_if_changed(PLAYER_SCREEN_Y_FACT, FactValue::Float(screen_pos.y as f64));
    event_writer.write(FactEvent::new(SCREEN_FACTS_UPDATED_EVENT));
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
}

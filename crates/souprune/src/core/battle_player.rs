//! Shared battle player runtime types.
//!
//! 共享的战斗内玩家运行时类型。

use crate::core::collision::{PhysicsCollider, TriggerCollider};
use bevy::color::LinearRgba;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use souprune_schema::battle::{
    BattleColliderShape, BattleInvincibilityConfig as SchemaInvincibilityConfig,
    BattlePlayerConfig as SchemaBattlePlayerConfig, ColliderConfig as SchemaColliderConfig,
};
use souprune_schema::bevy_types::BevyColor;

#[derive(Resource, Debug, Clone)]
pub struct BattleInvincibilityConfig {
    pub duration: f32,
    pub flash_interval: f32,
    pub normal_color: Color,
    pub flash_color: Color,
    pub damage_sound: Option<String>,
}

impl Default for BattleInvincibilityConfig {
    fn default() -> Self {
        Self {
            duration: 1.0,
            flash_interval: 0.25,
            normal_color: Color::srgb(1.0, 0.0, 0.0),
            flash_color: Color::srgb(0.5, 0.0, 0.0),
            damage_sound: None,
        }
    }
}

/// Runtime Bevy asset wrapper for `.battle_player.ron`.
#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct BattlePlayerConfig(pub SchemaBattlePlayerConfig);

impl BattlePlayerConfig {
    pub fn sprite_path(&self) -> &str {
        &self.0.sprite_path
    }

    pub fn sprite_color(&self) -> Color {
        bevy_color_to_color(&self.0.color)
    }

    pub fn physics_collider(&self) -> PhysicsCollider {
        collider_to_physics(&self.0.physics_collider)
    }

    pub fn damage_trigger(&self) -> TriggerCollider {
        collider_to_trigger(&self.0.damage_trigger)
    }

    pub fn z_position(&self) -> f32 {
        self.0.z_position
    }

    pub fn default_mode_id(&self) -> &str {
        &self.0.default_mode_id
    }

    pub fn default_box(&self) -> &str {
        &self.0.default_box
    }

    pub fn invincibility(&self) -> BattleInvincibilityConfig {
        runtime_invincibility_config(&self.0.invincibility)
    }
}

fn collider_to_physics(collider: &SchemaColliderConfig) -> PhysicsCollider {
    match collider.shape {
        BattleColliderShape::Circle { radius } => PhysicsCollider::Circle { radius },
        BattleColliderShape::Box { half_size } => PhysicsCollider::Box {
            half_size: Vec2::new(half_size.0, half_size.1),
        },
    }
}

fn collider_to_trigger(collider: &SchemaColliderConfig) -> TriggerCollider {
    match collider.shape {
        BattleColliderShape::Circle { radius } => TriggerCollider::Circle { radius },
        BattleColliderShape::Box { half_size } => TriggerCollider::Box {
            half_size: Vec2::new(half_size.0, half_size.1),
        },
    }
}

fn runtime_invincibility_config(config: &SchemaInvincibilityConfig) -> BattleInvincibilityConfig {
    BattleInvincibilityConfig {
        duration: config.duration,
        flash_interval: config.flash_interval,
        normal_color: bevy_color_to_color(&config.normal_color),
        flash_color: bevy_color_to_color(&config.flash_color),
        damage_sound: config.damage_sound.clone(),
    }
}

fn bevy_color_to_color(color: &BevyColor) -> Color {
    match color {
        BevyColor::Srgba(srgba) => Color::srgba(srgba.red, srgba.green, srgba.blue, srgba.alpha),
        BevyColor::LinearRgba(linear) => Color::LinearRgba(LinearRgba::new(
            linear.red,
            linear.green,
            linear.blue,
            linear.alpha,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_runtime_wrapper_and_converts_battle_player_fields() {
        let ron = r#"(
            sprite_path: "assets/textures/common/view/heart.png",
            color: Srgba((red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0)),
            physics_collider: (
                shape: Circle(radius: 8.0),
                debug_z_offset: 10.0,
            ),
            damage_trigger: (
                shape: Box(half_size: (2.0, 2.0)),
                debug_z_offset: 12.0,
            ),
            z_position: 10.0,
            default_mode_id: "soul_red",
            speed: 150.0,
            focus_speed_ratio: 0.5,
            invincibility: (
                damage_sound: Some("audios/sfx/hurtsound.wav"),
            ),
        )"#;

        let config: BattlePlayerConfig = ron::from_str(ron).expect("battle player config");
        let invincibility = config.invincibility();

        assert_eq!(config.default_box(), "main");
        assert_eq!(config.default_mode_id(), "soul_red");
        assert!(matches!(
            config.physics_collider(),
            PhysicsCollider::Circle { radius } if (radius - 8.0).abs() < f32::EPSILON
        ));
        assert!(matches!(
            config.damage_trigger(),
            TriggerCollider::Box { half_size }
                if (half_size.x - 2.0).abs() < f32::EPSILON
                    && (half_size.y - 2.0).abs() < f32::EPSILON
        ));
        let srgba = invincibility.normal_color.to_srgba();
        assert_eq!(
            invincibility.damage_sound.as_deref(),
            Some("audios/sfx/hurtsound.wav")
        );
        assert!((srgba.red - 1.0).abs() < f32::EPSILON);
        assert!((srgba.alpha - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn converts_linear_rgba_color() {
        let color = bevy_color_to_color(&BevyColor::LinearRgba(
            souprune_schema::bevy_types::SrgbaColor {
                red: 0.1,
                green: 0.2,
                blue: 0.3,
                alpha: 0.4,
            },
        ));

        assert!(matches!(color, Color::LinearRgba(_)));
    }
}

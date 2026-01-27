//! Spiral Homing danmaku behavior - spirals outward while tracking the player
//! 螺旋追踪弹幕行为 - 向外螺旋运动同时追踪玩家

use souprune_sdk::{BulletContext, BulletOutput, DanmakuBehavior, Vec2};

/// Spiral Homing Danmaku - spirals outward while slowly homing toward the player.
/// 螺旋追踪弹幕 - 向外螺旋运动同时缓慢追踪玩家。
///
/// Properties (from RON config):
/// - "spiral_speed": outward spiral speed (default: 80.0)
/// - "angular_velocity": rotation speed in rad/s (default: 3.0)
/// - "homing_strength": how strongly it tracks player 0-1 (default: 0.5)
/// - "homing_delay": delay before homing starts in seconds (default: 0.5)
pub struct SpiralHomingDanmaku {
    // Configuration
    spiral_speed: f32,
    angular_velocity: f32,
    homing_strength: f32,
    homing_delay: f32,

    // Cached state from OnEnter
    captured_direction: Vec2,

    // Runtime state
    current_angle: f32,
    accumulated_radius: f32,
}

impl SpiralHomingDanmaku {
    pub fn new() -> Self {
        Self {
            spiral_speed: 80.0,
            angular_velocity: 3.0,
            homing_strength: 0.5,
            homing_delay: 0.5,
            captured_direction: Vec2::new(0.0, -1.0),
            current_angle: 0.0,
            accumulated_radius: 0.0,
        }
    }

    fn lerp(a: Vec2, b: Vec2, t: f32) -> Vec2 {
        Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
    }
}

impl Default for SpiralHomingDanmaku {
    fn default() -> Self {
        Self::new()
    }
}

impl DanmakuBehavior for SpiralHomingDanmaku {
    fn on_enter(&mut self, ctx: &BulletContext) {
        // Read config from props
        self.spiral_speed = ctx.get_float("spiral_speed").unwrap_or(80.0);
        self.angular_velocity = ctx.get_float("angular_velocity").unwrap_or(3.0);
        self.homing_strength = ctx.get_float("homing_strength").unwrap_or(0.5);
        self.homing_delay = ctx.get_float("homing_delay").unwrap_or(0.5);

        // Calculate initial direction towards player
        let to_player = ctx.player_pos.sub(&ctx.spawn_position());
        self.captured_direction = if to_player.length() > 0.001 {
            to_player.normalize()
        } else {
            Vec2::new(0.0, -1.0)
        };

        // Initialize angle from spawn angle
        self.current_angle = ctx.initial_angle;
        self.accumulated_radius = ctx.initial_radius;
    }

    fn on_update(&mut self, ctx: &BulletContext) -> BulletOutput {
        // Update spiral angle
        self.current_angle += self.angular_velocity * ctx.delta_time;

        // Increase radius over time (spiral outward)
        self.accumulated_radius += self.spiral_speed * ctx.delta_time;

        // Calculate base spiral position
        let spiral_x = self.current_angle.cos() * self.accumulated_radius;
        let spiral_y = self.current_angle.sin() * self.accumulated_radius;
        let mut spiral_offset = Vec2::new(spiral_x, spiral_y);

        // After delay, blend in homing towards current player position
        if ctx.elapsed > self.homing_delay {
            let homing_t = ((ctx.elapsed - self.homing_delay) * self.homing_strength).min(1.0);

            // Calculate direction to current player
            let current_to_player = ctx.player_pos.sub(&ctx.spawn_position());
            let target_direction = if current_to_player.length() > 0.001 {
                current_to_player.normalize()
            } else {
                self.captured_direction
            };

            // Blend captured direction towards current player
            let blended_direction =
                Self::lerp(self.captured_direction, target_direction, homing_t * 0.3);

            // Add a drift component in the homing direction
            let drift_magnitude = ctx.elapsed * 30.0 * homing_t;
            spiral_offset = Vec2::new(
                spiral_offset.x + blended_direction.x * drift_magnitude,
                spiral_offset.y + blended_direction.y * drift_magnitude,
            );
        }

        // Apply visual rotation based on movement
        let rotation = self.current_angle * 0.5;

        BulletOutput::new(spiral_offset.x, spiral_offset.y).with_rotation(rotation)
    }
}

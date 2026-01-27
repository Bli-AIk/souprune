//! Gravity Drop danmaku behavior - falls with gravity and bounces
//! 重力下落弹幕行为 - 受重力影响下落并反弹

use souprune_sdk::{BulletContext, BulletOutput, DanmakuBehavior};

/// Gravity Drop Danmaku - falls with gravity and bounces off the bottom.
/// 重力下落弹幕 - 受重力影响下落并在底部反弹。
///
/// Properties (from RON config):
/// - "gravity": gravity acceleration (default: 200.0)
/// - "bounce_damping": velocity retained after bounce 0-1 (default: 0.7)
pub struct GravityDropDanmaku {
    // Configuration
    initial_velocity_x: f32,
    gravity: f32,
    bounce_y: f32,
    bounce_damping: f32,

    // State
    velocity_y: f32,
    pos_x: f32,
    pos_y: f32,
    bounce_count: i32,
}

impl GravityDropDanmaku {
    pub fn new() -> Self {
        Self {
            initial_velocity_x: 50.0,
            gravity: 200.0,
            bounce_y: -80.0,
            bounce_damping: 0.7,
            velocity_y: -100.0,
            pos_x: 0.0,
            pos_y: 0.0,
            bounce_count: 0,
        }
    }
}

impl Default for GravityDropDanmaku {
    fn default() -> Self {
        Self::new()
    }
}

impl DanmakuBehavior for GravityDropDanmaku {
    fn on_enter(&mut self, ctx: &BulletContext) {
        // Read config from props
        self.gravity = ctx.get_float("gravity").unwrap_or(200.0);
        self.bounce_damping = ctx.get_float("bounce_damping").unwrap_or(0.7);

        self.velocity_y = -100.0; // Initial upward velocity
        self.pos_x = 0.0;
        self.pos_y = 0.0;
        self.bounce_count = 0;

        // Randomize initial X velocity based on angle
        self.initial_velocity_x = ctx.initial_angle.cos() * 80.0;
    }

    fn on_update(&mut self, ctx: &BulletContext) -> BulletOutput {
        // Apply gravity to Y velocity
        self.velocity_y += self.gravity * ctx.delta_time;

        // Update positions
        self.pos_x += self.initial_velocity_x * ctx.delta_time;
        self.pos_y += self.velocity_y * ctx.delta_time;

        // Check for bounce (max 3 bounces)
        if self.pos_y > self.bounce_y && self.bounce_count < 3 && self.velocity_y > 0.0 {
            self.pos_y = self.bounce_y;
            self.velocity_y = -self.velocity_y * self.bounce_damping;
            self.bounce_count += 1;
        }

        // Rotation based on velocity direction
        let rotation = self.velocity_y.atan2(self.initial_velocity_x);

        BulletOutput::new(self.pos_x, self.pos_y).with_rotation(rotation)
    }
}

//! Wave Burst danmaku behavior - moves in sine wave pattern with burst acceleration
//! 波动爆发弹幕行为 - 以正弦波模式移动并带有爆发加速

use souprune_sdk::{BulletContext, BulletOutput, DanmakuBehavior, Vec2};

/// Wave Burst Danmaku - moves in sine wave pattern with initial burst speed.
/// 波动爆发弹幕 - 以正弦波模式移动，初始有爆发速度。
///
/// Properties (from RON config):
/// - "base_speed": base movement speed (default: 120.0)
/// - "wave_amplitude": wave height in pixels (default: 30.0)
/// - "wave_frequency": waves per second (default: 4.0)
/// - "burst_time": burst duration in seconds (default: 0.8)
/// - "burst_multiplier": initial speed multiplier (default: 2.5)
pub struct WaveBurstDanmaku {
    // Configuration
    base_speed: f32,
    wave_amplitude: f32,
    wave_frequency: f32,
    burst_time: f32,
    burst_multiplier: f32,

    // State
    direction: Vec2,
    perpendicular: Vec2,
}

impl WaveBurstDanmaku {
    pub fn new() -> Self {
        Self {
            base_speed: 120.0,
            wave_amplitude: 30.0,
            wave_frequency: 4.0,
            burst_time: 0.8,
            burst_multiplier: 2.5,
            direction: Vec2::new(0.0, -1.0),
            perpendicular: Vec2::new(1.0, 0.0),
        }
    }
}

impl Default for WaveBurstDanmaku {
    fn default() -> Self {
        Self::new()
    }
}

impl DanmakuBehavior for WaveBurstDanmaku {
    fn on_enter(&mut self, ctx: &BulletContext) {
        // Read config from props
        self.base_speed = ctx.get_float("base_speed").unwrap_or(120.0);
        self.wave_amplitude = ctx.get_float("wave_amplitude").unwrap_or(30.0);
        self.wave_frequency = ctx.get_float("wave_frequency").unwrap_or(4.0);
        self.burst_time = ctx.get_float("burst_time").unwrap_or(0.8);
        self.burst_multiplier = ctx.get_float("burst_multiplier").unwrap_or(2.5);

        // Use initial angle to determine direction
        self.direction = Vec2::new(ctx.initial_angle.cos(), ctx.initial_angle.sin());
        self.perpendicular = Vec2::new(-self.direction.y, self.direction.x);
    }

    fn on_update(&mut self, ctx: &BulletContext) -> BulletOutput {
        // Calculate burst multiplier (ease out)
        let speed_multiplier = if ctx.elapsed < self.burst_time {
            let t = ctx.elapsed / self.burst_time;
            1.0 + (self.burst_multiplier - 1.0) * (1.0 - t * t)
        } else {
            1.0
        };

        // Calculate linear movement
        let linear_dist = self.base_speed * ctx.elapsed * speed_multiplier;

        // Calculate wave offset
        let wave = (ctx.elapsed * self.wave_frequency * std::f32::consts::TAU).sin()
            * self.wave_amplitude;

        // Combine movements
        let offset_x = self.direction.x * linear_dist + self.perpendicular.x * wave;
        let offset_y = self.direction.y * linear_dist + self.perpendicular.y * wave;

        // Rotation follows wave
        let rotation = (ctx.elapsed * self.wave_frequency * std::f32::consts::TAU).sin() * 0.3;

        BulletOutput::new(offset_x, offset_y).with_rotation(rotation)
    }
}

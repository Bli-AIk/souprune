//! Aimed Spear danmaku behavior - aims at player position at spawn time
//! 自机狙长矛弹幕行为 - 在生成时瞄准玩家位置

use souprune_sdk::{BulletContext, BulletOutput, DanmakuBehavior, Vec2};

/// Aimed spear that captures player position on spawn and moves toward it.
/// The bullet moves in a straight line toward where the player WAS at spawn time.
///
/// 自机狙长矛，在生成时捕获玩家位置并朝向移动。
/// 弹幕沿直线向玩家生成时的位置移动。
///
/// Properties (from RON config):
/// - "speed": pixels per second (default: 200.0)
/// - "smoothness": turn rate toward player (future support)
pub struct AimedSpear {
    /// Target position captured at spawn time
    target_pos: Vec2,
    /// Movement direction (normalized)
    direction: Vec2,
    /// Speed in pixels per second
    speed: f32,
}

impl AimedSpear {
    pub fn new() -> Self {
        Self {
            target_pos: Vec2::ZERO,
            direction: Vec2::ZERO,
            speed: 200.0,
        }
    }
}

impl Default for AimedSpear {
    fn default() -> Self {
        Self::new()
    }
}

impl DanmakuBehavior for AimedSpear {
    fn on_enter(&mut self, ctx: &BulletContext) {
        // Capture player position at spawn time - this is the key!
        // 在生成时捕获玩家位置 - 这是关键！
        self.target_pos = ctx.player_pos;

        // Get speed from named properties
        self.speed = ctx.get_float("speed").unwrap_or(200.0);

        // Calculate direction from spawn position to player position
        let spawn_pos = ctx.spawn_position();
        let to_target = self.target_pos - spawn_pos;
        self.direction = to_target.normalize();
    }

    fn on_update(&mut self, ctx: &BulletContext) -> BulletOutput {
        // Move in the direction calculated at spawn time
        // 沿生成时计算的方向移动
        let offset = self.direction * self.speed * ctx.elapsed;

        // Calculate target angle from direction vector
        // 计算方向向量的目标角度
        let target_angle = self.direction.y.atan2(self.direction.x);

        let mut output = BulletOutput::new(offset.x, offset.y);

        // Apply rotation on the first frame to face the player
        // 在第一帧应用旋转以朝向玩家
        // Correct for sprite pointing UP (PI/2) by default
        // 修正图片默认朝上 (PI/2) 的偏移
        if ctx.elapsed <= ctx.delta_time {
            let rotation_diff = target_angle - std::f32::consts::FRAC_PI_2;
            output = output.with_rotation(rotation_diff);
        }

        output
    }
}

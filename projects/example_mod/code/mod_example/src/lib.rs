use souprune_sdk::{Action, Behavior, BulletStateC, Context, Vec2C, declare_algorithms, declare_behaviors};

struct RedSoul {
    speed: f32,
    focus_ratio: f32,
}

impl RedSoul {
    fn new() -> Self {
        // Default values, ideally these should be passed from config via on_enter
        Self {
            speed: 150.0,
            focus_ratio: 0.5,
        }
    }
}

impl Behavior for RedSoul {
    fn on_enter(&mut self, context: &mut Context) {
        context.log("Red Soul Mode Activated!");
        // TODO: Read parameters from context if supported in future
    }

    fn on_update(&mut self, context: &mut Context, _dt: f32) {
        let mut velocity_x: f32 = 0.0;
        let mut velocity_y: f32 = 0.0;

        let input = context.input();

        if input.pressed(Action::Left) {
            velocity_x -= 1.0;
        }
        if input.pressed(Action::Right) {
            velocity_x += 1.0;
        }
        if input.pressed(Action::Up) {
            velocity_y += 1.0;
        }
        if input.pressed(Action::Down) {
            velocity_y -= 1.0;
        }

        // Normalize vector if moving diagonally
        if velocity_x != 0.0 || velocity_y != 0.0 {
            let length = (velocity_x * velocity_x + velocity_y * velocity_y).sqrt();
            velocity_x /= length;
            velocity_y /= length;

            let mut current_speed = self.speed;

            // Check for Focus (Cancel button usually maps to Shift/X)
            if input.pressed(Action::Cancel) {
                current_speed *= self.focus_ratio;
            }

            velocity_x *= current_speed;
            velocity_y *= current_speed;
        }

        context.kinematics().set_velocity(velocity_x, velocity_y);
    }

    fn on_exit(&mut self, context: &mut Context) {
        context.log("Red Soul Mode Deactivated.");
        context.kinematics().set_velocity(0.0, 0.0);
    }
}

// === Blue Soul (Gravity Mode) ===
struct BlueSoul;

impl Behavior for BlueSoul {
    fn on_enter(&mut self, context: &mut Context) {
        context.log("Blue Soul Mode (Gravity) Activated!");
    }

    fn on_update(&mut self, context: &mut Context, _dt: f32) {
        // Simple gravity simulation for demo
        context.kinematics().set_velocity(0.0, -200.0);

        if context.input().pressed(Action::Confirm) {
            context.log("Blue Soul Jump!");
            context.kinematics().set_velocity(0.0, 300.0);
        }
    }

    fn on_exit(&mut self, context: &mut Context) {
        context.log("Blue Soul Mode Deactivated.");
    }
}

declare_behaviors!(
    ("soul_red", RedSoul, || RedSoul::new()),
    ("soul_blue", BlueSoul, || BlueSoul)
);

// ============================================================================
// Danmaku Algorithms (弹幕算法)
// ============================================================================

/// Homing spear algorithm - spears that track toward player position
/// The algorithm calculates position based on initial direction modified by homing behavior
///
/// 自机狙长矛算法 - 追踪玩家位置的长矛
/// 算法根据初始方向并结合追踪行为计算位置
///
/// Parameters (from RON config):
/// - params[0]: speed (pixels per second)
/// - params[1]: homing_strength (0.0 = no homing, 1.0 = instant tracking)
extern "C" fn homing_spear_algorithm(state: *const BulletStateC) -> Vec2C {
    let s = unsafe { &*state };

    // Get parameters with defaults
    let speed = if s.params_len > 0 {
        unsafe { *s.params.add(0) }
    } else {
        200.0
    };
    let homing_strength = if s.params_len > 1 {
        unsafe { *s.params.add(1) }
    } else {
        0.5
    };

    // Current position (spawn + offset + accumulated movement)
    let current_x = s.spawn_x + s.offset_x;
    let current_y = s.spawn_y + s.offset_y;

    // For simplicity, assume player is at origin (0, 0) as we don't have access to player pos
    // In a real implementation, player position would need to be passed through params
    let player_x = 0.0;
    let player_y = -80.0; // Player typically spawns here in demo

    // Calculate direction to player
    let to_player_x = player_x - current_x;
    let to_player_y = player_y - current_y;
    let dist = (to_player_x * to_player_x + to_player_y * to_player_y).sqrt();

    if dist < 0.01 {
        return Vec2C { x: 0.0, y: 0.0 };
    }

    let target_dir_x = to_player_x / dist;
    let target_dir_y = to_player_y / dist;

    // Blend between initial direction and target direction based on homing strength
    let blend = (homing_strength * s.elapsed).min(1.0);
    let dir_x = s.dir_x * (1.0 - blend) + target_dir_x * blend;
    let dir_y = s.dir_y * (1.0 - blend) + target_dir_y * blend;

    // Normalize blended direction
    let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
    let (norm_x, norm_y) = if len > 0.01 {
        (dir_x / len, dir_y / len)
    } else {
        (0.0, -1.0)
    };

    // Return position offset based on elapsed time and speed
    Vec2C {
        x: norm_x * speed * s.elapsed,
        y: norm_y * speed * s.elapsed,
    }
}

declare_algorithms!(
    ("homing_spear", homing_spear_algorithm),
);

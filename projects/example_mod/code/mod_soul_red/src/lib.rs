use souprune_sdk::{declare_soul_mode, Action, Context, SoulMode};

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

impl SoulMode for RedSoul {
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

declare_soul_mode!("soul_red", RedSoul, || RedSoul::new());
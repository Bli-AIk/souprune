//! Blue Soul behavior - gravity mode
//! 蓝魂行为 - 重力模式

use souprune_sdk::{Action, Behavior, Context};

pub struct BlueSoul;

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

use souprune_sdk::{Context, SoulMode, declare_soul_mode};

struct MyTestSoul {
    counter: u32,
}

impl SoulMode for MyTestSoul {
    fn on_enter(&mut self, context: &mut Context) {
        context.log("TestMod: I have entered the stage!");
    }

    fn on_update(&mut self, context: &mut Context, dt: f32) {
        context.log(&format!("Update: dt={}, counter={}", dt, self.counter));
        self.counter += 1;

        // 测试输入
        if context.input().pressed(souprune_sdk::Action::Right) {
            context.log("Right action pressed!");
            // 测试运动学
            context.kinematics().set_velocity(100.0, 0.0);
        } else {
            context.kinematics().set_velocity(0.0, 0.0);
        }
    }

    fn on_exit(&mut self, context: &mut Context) {
        context.log("TestMod: Bye bye!");
    }
}

// 注册 Mod
declare_soul_mode!(MyTestSoul, || MyTestSoul { counter: 0 });

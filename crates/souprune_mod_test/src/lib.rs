use souprune_sdk::{Context, SoulMode, declare_soul_mode};

struct MyTestSoul;

impl SoulMode for MyTestSoul {
    fn on_enter(&mut self, context: &mut Context) {
        context.log("TestMod: I have entered the stage!");
    }

    fn on_update(&mut self, context: &mut Context, _dt: f32) {
        // 测试传感器
        let val = context.input().axis("Horizontal");

        // 测试逻辑
        if val != 0.0 {
            let msg = format!("TestMod: Moving with speed {}", val * 10.0);
            context.log(&msg);

            // 测试执行器
            context.physics().set_velocity(val * 10.0, 0.0);
        }
    }

    fn on_exit(&mut self, context: &mut Context) {
        context.log("TestMod: Bye bye!");
    }
}

// 注册 Mod
declare_soul_mode!(MyTestSoul, || MyTestSoul);

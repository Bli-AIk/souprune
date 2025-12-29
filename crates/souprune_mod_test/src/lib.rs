//! A reference implementation of a Mod using the SDK.
//! It serves as an example and a test case to verify that the mod compilation and loading pipeline works correctly.
//!
//! 使用 SDK 的模组参考实现。
//! 作为示例和测试用例，用于验证模组编译和加载流程是否正常工作。

use souprune_sdk::{Behavior, Context, declare_behaviors};

struct MyTestSoul {
    counter: u32,
}

impl Behavior for MyTestSoul {
    fn on_enter(&mut self, context: &mut Context) {
        context.log("TestMod: I have entered the stage!");
    }

    fn on_update(&mut self, context: &mut Context, delta_time: f32) {
        context.log(&format!(
            "Update: delta_time={}, counter={}",
            delta_time, self.counter
        ));
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

struct MySecondSoul;

impl Behavior for MySecondSoul {
    fn on_enter(&mut self, context: &mut Context) {
        context.log("SecondSoul: Hello from the second soul!");
    }

    fn on_update(&mut self, context: &mut Context, _delta_time: f32) {
        if context.input().pressed(souprune_sdk::Action::Confirm) {
            context.log("SecondSoul: Confirm pressed!");
        }
    }

    fn on_exit(&mut self, context: &mut Context) {
        context.log("SecondSoul: Goodbye!");
    }
}

// 注册 Mod
declare_behaviors!(
    ("test_soul", MyTestSoul, || MyTestSoul { counter: 0 }),
    ("second_soul", MySecondSoul, || MySecondSoul)
);

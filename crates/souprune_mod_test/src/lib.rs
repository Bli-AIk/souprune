//! A reference implementation of a Mod using the SDK.
//! Compiled to `wasm32-wasip2` and loaded by the engine at runtime.
//!
//! 使用 SDK 的模组参考实现。
//! 编译为 `wasm32-wasip2`，运行时由引擎加载。

use souprune_sdk::prelude::*;

struct MyTestSoul {
    counter: u32,
}

impl Behavior for MyTestSoul {
    fn on_enter(&mut self, context: &mut Context) {
        context.log("TestMod: I have entered the stage!");
    }

    fn on_update(&mut self, context: &mut Context, _delta_time: f32) {
        context.log(&format!("Update: counter={}", self.counter));
        self.counter += 1;

        if context.input().pressed(Action::Right) {
            context.log("Right action pressed!");
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
        if context.input().pressed(Action::Confirm) {
            context.log("SecondSoul: Confirm pressed!");
        }
    }

    fn on_exit(&mut self, context: &mut Context) {
        context.log("SecondSoul: Goodbye!");
    }
}

export_mod! {
    behaviors: [
        ("test_soul", MyTestSoul, || MyTestSoul { counter: 0 }),
        ("second_soul", MySecondSoul, || MySecondSoul),
    ],
    danmaku: [],
}

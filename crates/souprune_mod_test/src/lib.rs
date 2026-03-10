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

/// Test danmaku that logs player_pos to verify WASM data flow.
struct DebugDanmaku {
    captured_player_pos: Vec2,
    captured_spawn_pos: Vec2,
}

impl DanmakuBehavior for DebugDanmaku {
    fn on_enter(&mut self, ctx: &BulletContext) {
        self.captured_player_pos = ctx.player_pos;
        self.captured_spawn_pos = ctx.spawn_pos;
    }

    fn on_update(&mut self, ctx: &BulletContext) -> BulletOutput {
        let dir = self.captured_player_pos - self.captured_spawn_pos;
        let norm = if dir.length() > 0.001 {
            dir.normalize()
        } else {
            Vec2::new(0.0, -1.0)
        };
        let offset = norm * 100.0 * ctx.elapsed;
        BulletOutput::new(offset.x, offset.y)
    }
}

export_mod! {
    behaviors: [
        ("test_soul", MyTestSoul, || MyTestSoul { counter: 0 }),
        ("second_soul", MySecondSoul, || MySecondSoul),
    ],
    danmaku: [
        ("debug_danmaku", DebugDanmaku, || DebugDanmaku {
            captured_player_pos: Vec2::ZERO,
            captured_spawn_pos: Vec2::ZERO,
        }),
    ],
}

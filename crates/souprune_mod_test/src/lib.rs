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

        if context.input().just_pressed(Action::Confirm) {
            context.log("Confirm just pressed — emitting test_confirm event");
            context.emit_event("test_confirm");
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

/// Test spawn pattern: a simple ring of N points around the center.
struct TestRingPattern;

impl SpawnPatternBehavior for TestRingPattern {
    fn generate(&self, ctx: &SpawnContext, params: &[SpawnParam]) -> Vec<SpawnOutput> {
        let count = params
            .iter()
            .find(|p| p.name == "count")
            .map_or(8, |p| p.as_usize().max(1));
        let radius = params
            .iter()
            .find(|p| p.name == "radius")
            .map_or(50.0, |p| p.as_f32());

        let step = std::f32::consts::TAU / count as f32;
        (0..count)
            .map(|i| {
                let angle = step * i as f32;
                SpawnOutput {
                    x: ctx.center.x + angle.cos() * radius,
                    y: ctx.center.y + angle.sin() * radius,
                    angle,
                    radius,
                }
            })
            .collect()
    }
}

/// Demo custom action handler — handles "test_action" and "debug_print".
#[derive(Default)]
struct TestActionHandler;

impl CustomActionHandler for TestActionHandler {
    fn handled_actions() -> Vec<String> {
        vec!["test_action".to_string(), "debug_print".to_string()]
    }

    fn handle_action(&self, ctx: &Context, action_type: &str, params: &[ActionParam]) -> bool {
        match action_type {
            "test_action" => {
                let message = params
                    .iter()
                    .find(|p| p.name == "message")
                    .map(|p| p.value.as_str())
                    .unwrap_or("(no message)");
                ctx.log(&format!("[TestActionHandler] test_action: {}", message));
                ctx.set_fact_string("test:last_action_message", message);
                true
            }
            "debug_print" => {
                let pos_x = ctx
                    .get_fact_float("player:pos_x")
                    .map(|v| format!("{v}"))
                    .unwrap_or_default();
                let pos_y = ctx
                    .get_fact_float("player:pos_y")
                    .map(|v| format!("{v}"))
                    .unwrap_or_default();
                ctx.log(&format!(
                    "[TestActionHandler] debug_print — player pos: ({}, {})",
                    pos_x, pos_y
                ));
                true
            }
            _ => false,
        }
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
    patterns: [
        ("test_ring", TestRingPattern, || TestRingPattern),
    ],
    custom_actions: TestActionHandler,
}

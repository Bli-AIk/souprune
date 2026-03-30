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

/// Fight bar behavior — attack bar sweep and Z-key interaction.
/// Replaces the engine-side `fight_bar.rs` systems with a data-driven WASM behavior.
///
/// Configuration facts (read):
///   `fight:bar_speed`     — sweep speed in px/s (default: 330.0, matching UT 11px/frame@30fps)
///   `fight:bar_right_edge`— X coordinate where the bar stops (default: 272.0)
///   `fight:bar_start_x`   — starting X offset (default: -274.0)
///   `fight:flash_interval` — seconds between flash color swaps (default: 0.083 ≈ 6Hz)
///
/// State facts (written):
///   `fight:bar_active`  — set externally to start sweep; cleared when done
///   `fight:bar_x`       — current bar X position (drives View offset)
///   `fight:bar_done`    — true when sweep finishes (hit or miss)
///   `fight:confirmed`   — true if player pressed Confirm (hit), false if missed
///   `fight:bar_flash_on`— toggles for SDF FactToggle color swap after hit
///
/// Events emitted:
///   `fight:hit` — when player presses Confirm during sweep (for sound FRE rules)
struct FightBarBehavior {
    sweep_x: f32,
    flash_elapsed: f32,
    flash_active: bool,
    sweep_done: bool,
}

impl Behavior for FightBarBehavior {
    fn on_enter(&mut self, ctx: &mut Context) {
        let start_x: f32 = ctx
            .get_fact("fight:bar_start_x")
            .and_then(|s| s.parse().ok())
            .unwrap_or(-274.0);
        self.sweep_x = start_x;
        self.flash_elapsed = 0.0;
        self.flash_active = false;
        self.sweep_done = false;
    }

    fn on_update(&mut self, ctx: &mut Context, dt: f32) {
        let active = ctx
            .get_fact("fight:bar_active")
            .map(|v| v == "true")
            .unwrap_or(false);

        if !active && !self.flash_active {
            return;
        }

        // Flash phase (after hit): toggle SDF colors until target is hidden
        if self.flash_active {
            let visible = ctx
                .get_fact("fight_target_visible")
                .map(|v| v == "true")
                .unwrap_or(false);
            if !visible {
                self.flash_active = false;
                ctx.set_fact("fight:bar_flash_on", "false");
                return;
            }

            let flash_interval: f32 = ctx
                .get_fact("fight:flash_interval")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.083);
            self.flash_elapsed += dt;
            let cycle = (self.flash_elapsed / flash_interval) as u32;
            let on = cycle % 2 != 0;
            ctx.set_fact("fight:bar_flash_on", if on { "true" } else { "false" });
            return;
        }

        if self.sweep_done {
            return;
        }

        // Read configurable parameters from facts
        let speed: f32 = ctx
            .get_fact("fight:bar_speed")
            .and_then(|s| s.parse().ok())
            .unwrap_or(330.0);
        let right_edge: f32 = ctx
            .get_fact("fight:bar_right_edge")
            .and_then(|s| s.parse().ok())
            .unwrap_or(272.0);
        let start_x: f32 = ctx
            .get_fact("fight:bar_start_x")
            .and_then(|s| s.parse().ok())
            .unwrap_or(-274.0);

        // Initialize sweep position on first active frame
        if self.sweep_x < start_x + 0.01 {
            self.sweep_x = start_x;
        }

        self.sweep_x += speed * dt;
        ctx.set_fact("fight:bar_x", &format!("{}", self.sweep_x));

        // Miss: reached right edge without input
        if self.sweep_x >= right_edge {
            self.sweep_x = right_edge;
            self.sweep_done = true;
            ctx.set_fact("fight:bar_x", &format!("{}", right_edge));
            ctx.set_fact("fight:bar_done", "true");
            ctx.set_fact("fight:confirmed", "false");
            return;
        }

        // Hit: player pressed Confirm
        if ctx.input().just_pressed(Action::Confirm) {
            self.sweep_done = true;
            self.flash_active = true;
            self.flash_elapsed = 0.0;
            ctx.set_fact("fight:bar_done", "true");
            ctx.set_fact("fight:confirmed", "true");
            ctx.emit_event("fight:hit");
        }
    }

    fn on_exit(&mut self, ctx: &mut Context) {
        ctx.set_fact("fight:bar_flash_on", "false");
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
                ctx.set_fact("test:last_action_message", message);
                true
            }
            "debug_print" => {
                let pos_x = ctx.get_fact("player:pos_x").unwrap_or_default();
                let pos_y = ctx.get_fact("player:pos_y").unwrap_or_default();
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
        ("fight_bar", FightBarBehavior, || FightBarBehavior {
            sweep_x: 0.0,
            flash_elapsed: 0.0,
            flash_active: false,
            sweep_done: false,
        }),
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

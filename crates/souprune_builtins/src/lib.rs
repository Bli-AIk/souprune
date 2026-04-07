//! Provides the built-in spawn patterns and danmaku behaviors bundled with Souprune.
//!
//! 提供随 Souprune 一起分发的内置生成模式与弹幕行为实现。
//!
//! This crate is the default library of script-facing primitives used when a
//! project wants standard bullet spawning and motion without shipping a custom
//! WASM component. The file registers those built-ins through `souprune_sdk`
//! exports, so it is both the module root of the crate and the concrete list of
//! built-in behaviors that the framework can load.
//!
//! 这个 crate 是面向脚本层的默认内置能力库。当项目不提供自定义 WASM 组件时，
//! 它负责提供标准的子弹生成与运动行为。这个文件既是 crate 根入口，也是通过
//! `souprune_sdk` 导出这些内置行为的具体注册点。

use souprune_sdk::prelude::*;
use souprune_sdk::{BulletContext, BulletOutput, SpawnContext, SpawnOutput, SpawnParam, export_mod};

// ============================================================================
// Spawn Patterns
// ============================================================================

struct SinglePattern;

impl SpawnPatternBehavior for SinglePattern {
    fn generate(&self, ctx: &SpawnContext, _params: &[SpawnParam]) -> Vec<SpawnOutput> {
        vec![SpawnOutput {
            x: ctx.center.x,
            y: ctx.center.y,
            angle: 0.0,
            radius: 0.0,
        }]
    }
}

struct RingPattern;

impl SpawnPatternBehavior for RingPattern {
    fn generate(&self, ctx: &SpawnContext, params: &[SpawnParam]) -> Vec<SpawnOutput> {
        let count = get_param(params, "count", 1.0) as usize;
        let radius = get_param(params, "radius", 0.0) as f32;
        let start_angle = get_param(params, "start_angle", 0.0) as f32;

        if count == 0 {
            return vec![];
        }

        let angle_step = core::f32::consts::TAU / count as f32;
        (0..count)
            .map(|i| {
                let angle = start_angle + angle_step * i as f32;
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

struct LinePattern;

impl SpawnPatternBehavior for LinePattern {
    fn generate(&self, ctx: &SpawnContext, params: &[SpawnParam]) -> Vec<SpawnOutput> {
        let count = get_param(params, "count", 1.0) as usize;
        let spacing = get_param(params, "spacing", 20.0) as f32;
        let dir_x = get_param(params, "dir_x", 0.0) as f32;
        let dir_y = get_param(params, "dir_y", -1.0) as f32;

        if count == 0 {
            return vec![];
        }

        let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
        let (nx, ny) = if len > 0.001 {
            (dir_x / len, dir_y / len)
        } else {
            (0.0, -1.0)
        };
        // perpendicular direction
        let (px, py) = (-ny, nx);

        let total_width = spacing * (count - 1) as f32;
        let start_offset = -total_width / 2.0;
        let angle = ny.atan2(nx);

        (0..count)
            .map(|i| {
                let offset = start_offset + spacing * i as f32;
                SpawnOutput {
                    x: ctx.center.x + px * offset,
                    y: ctx.center.y + py * offset,
                    angle,
                    radius: 0.0,
                }
            })
            .collect()
    }
}

struct EdgePattern;

impl SpawnPatternBehavior for EdgePattern {
    fn generate(&self, ctx: &SpawnContext, params: &[SpawnParam]) -> Vec<SpawnOutput> {
        let count = get_param(params, "count", 1.0) as usize;
        let side = get_param(params, "side", 0.0) as u32;
        let spacing = get_param(params, "spacing", 30.0) as f32;
        let margin = get_param(params, "margin", 200.0) as f32;

        if count == 0 {
            return vec![];
        }

        // side: 0=Left, 1=Right, 2=Top, 3=Bottom
        let (move_x, move_y): (f32, f32) = match side {
            0 => (1.0, 0.0),   // Left → fires right
            1 => (-1.0, 0.0),  // Right → fires left
            2 => (0.0, -1.0),  // Top → fires down
            3 => (0.0, 1.0),   // Bottom → fires up
            _ => (1.0, 0.0),
        };
        let (offset_x, offset_y): (f32, f32) = match side {
            0 => (-margin, 0.0),
            1 => (margin, 0.0),
            2 => (0.0, margin),
            3 => (0.0, -margin),
            _ => (-margin, 0.0),
        };
        // perpendicular
        let (px, py) = (-move_y, move_x);

        let total_width = spacing * (count - 1) as f32;
        let start = -total_width / 2.0;
        let angle = move_y.atan2(move_x);

        (0..count)
            .map(|i| {
                let perp_offset = start + spacing * i as f32;
                SpawnOutput {
                    x: ctx.center.x + offset_x + px * perp_offset,
                    y: ctx.center.y + offset_y + py * perp_offset,
                    angle,
                    radius: 0.0,
                }
            })
            .collect()
    }
}

// ============================================================================
// Danmaku Behaviors
// ============================================================================

struct LinearBehavior {
    dir_x: f32,
    dir_y: f32,
    speed: f32,
}

impl DanmakuBehavior for LinearBehavior {
    fn on_enter(&mut self, ctx: &BulletContext) {
        self.dir_x = ctx.get_float("dir_x").unwrap_or(0.0);
        self.dir_y = ctx.get_float("dir_y").unwrap_or(-1.0);
        self.speed = ctx.get_float("speed").unwrap_or(100.0);
    }

    fn on_update(&mut self, ctx: &BulletContext) -> BulletOutput {
        let len = (self.dir_x * self.dir_x + self.dir_y * self.dir_y).sqrt();
        let (nx, ny) = if len > 0.001 {
            (self.dir_x / len, self.dir_y / len)
        } else {
            (0.0, 0.0)
        };
        BulletOutput::new(nx * self.speed * ctx.elapsed, ny * self.speed * ctx.elapsed)
    }
}

struct OrbitalBehavior {
    angular_velocity: f32,
    radial_velocity: f32,
}

impl DanmakuBehavior for OrbitalBehavior {
    fn on_enter(&mut self, ctx: &BulletContext) {
        self.angular_velocity = ctx.get_float("angular_velocity").unwrap_or(1.0);
        self.radial_velocity = ctx.get_float("radial_velocity").unwrap_or(0.0);
    }

    fn on_update(&mut self, ctx: &BulletContext) -> BulletOutput {
        let current_angle = ctx.initial_angle + self.angular_velocity * ctx.elapsed;
        let current_radius =
            f32::max(ctx.initial_radius + self.radial_velocity * ctx.elapsed, 0.0);

        // Orbital position relative to spawn center
        let orbital_x = current_angle.cos() * current_radius;
        let orbital_y = current_angle.sin() * current_radius;

        BulletOutput::new(orbital_x - ctx.offset.x, orbital_y - ctx.offset.y)
            .with_rotation(self.angular_velocity * ctx.delta_time)
    }
}

struct SineBehavior {
    axis_x: f32,
    axis_y: f32,
    amplitude: f32,
    frequency: f32,
    phase: f32,
}

impl DanmakuBehavior for SineBehavior {
    fn on_enter(&mut self, ctx: &BulletContext) {
        self.axis_x = ctx.get_float("axis_x").unwrap_or(1.0);
        self.axis_y = ctx.get_float("axis_y").unwrap_or(0.0);
        self.amplitude = ctx.get_float("amplitude").unwrap_or(20.0);
        self.frequency = ctx.get_float("frequency").unwrap_or(2.0);
        self.phase = ctx.get_float("phase").unwrap_or(0.0);
    }

    fn on_update(&mut self, ctx: &BulletContext) -> BulletOutput {
        let len = (self.axis_x * self.axis_x + self.axis_y * self.axis_y).sqrt();
        let (nx, ny) = if len > 0.001 {
            (self.axis_x / len, self.axis_y / len)
        } else {
            (1.0, 0.0)
        };
        let wave = (ctx.elapsed * self.frequency * core::f32::consts::TAU + self.phase).sin();
        BulletOutput::new(nx * wave * self.amplitude, ny * wave * self.amplitude)
    }
}

struct StationaryBehavior;

impl DanmakuBehavior for StationaryBehavior {
    fn on_update(&mut self, _ctx: &BulletContext) -> BulletOutput {
        BulletOutput::ZERO
    }
}

struct AimedBehavior {
    dir_x: f32,
    dir_y: f32,
    speed: f32,
}

impl DanmakuBehavior for AimedBehavior {
    fn on_enter(&mut self, ctx: &BulletContext) {
        self.speed = ctx.get_float("speed").unwrap_or(120.0);
        let angle_offset = ctx.get_float("angle_offset").unwrap_or(0.0);

        let spawn_x = ctx.spawn_pos.x + ctx.offset.x;
        let spawn_y = ctx.spawn_pos.y + ctx.offset.y;
        let dx = ctx.player_pos.x - spawn_x;
        let dy = ctx.player_pos.y - spawn_y;
        let base_angle = dy.atan2(dx);
        let final_angle = base_angle + angle_offset;

        self.dir_x = final_angle.cos();
        self.dir_y = final_angle.sin();
    }

    fn on_update(&mut self, ctx: &BulletContext) -> BulletOutput {
        BulletOutput::new(
            self.dir_x * self.speed * ctx.elapsed,
            self.dir_y * self.speed * ctx.elapsed,
        )
    }
}

/// TweenTarget mapping: 0=Opacity, 1=Scale, 2=ScaleX, 3=ScaleY,
/// 4=PositionX, 5=PositionY, 6=Rotation
struct TweenBehavior {
    target: u32,
    duration: f32,
    ease: u32,
    range_start: f32,
    range_end: f32,
    delay: f32,
    timer: f32,
}

impl DanmakuBehavior for TweenBehavior {
    fn on_enter(&mut self, ctx: &BulletContext) {
        self.target = ctx.get_float("target").unwrap_or(0.0) as u32;
        self.duration = ctx.get_float("duration").unwrap_or(1.0);
        self.ease = ctx.get_float("ease").unwrap_or(0.0) as u32;
        self.range_start = ctx.get_float("range_start").unwrap_or(0.0);
        self.range_end = ctx.get_float("range_end").unwrap_or(1.0);
        self.delay = ctx.get_float("delay").unwrap_or(0.0);
        self.timer = 0.0;
    }

    fn on_update(&mut self, ctx: &BulletContext) -> BulletOutput {
        self.timer += ctx.delta_time;
        let t = self.timer - self.delay;

        let value = if t < 0.0 {
            self.range_start
        } else if t >= self.duration {
            self.range_end
        } else {
            let progress = (t / self.duration).clamp(0.0, 1.0);
            let eased = apply_easing(self.ease, progress);
            self.range_start + (self.range_end - self.range_start) * eased
        };

        apply_tween_output(self.target, value)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_param(params: &[SpawnParam], name: &str, default: f64) -> f64 {
    params
        .iter()
        .find(|p| p.name == name)
        .map(|p| p.value)
        .unwrap_or(default)
}

fn apply_easing(ease: u32, t: f32) -> f32 {
    match ease {
        0 => t,
        1 => t * t,
        2 => 1.0 - (1.0 - t) * (1.0 - t),
        3 => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0) * (-2.0 * t + 2.0) / 2.0
            }
        }
        4 => t * t * t,
        5 => 1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t),
        6 => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0) * (-2.0 * t + 2.0) * (-2.0 * t + 2.0) / 2.0
            }
        }
        7 => 1.0 - (t * core::f32::consts::FRAC_PI_2).cos(),
        8 => (t * core::f32::consts::FRAC_PI_2).sin(),
        9 => -(t * core::f32::consts::PI).cos() / 2.0 + 0.5,
        _ => t,
    }
}

fn apply_tween_output(target: u32, value: f32) -> BulletOutput {
    match target {
        0 => BulletOutput::ZERO.with_opacity(value),
        1 => BulletOutput {
            scale_x: value - 1.0,
            scale_y: value - 1.0,
            ..BulletOutput::ZERO
        },
        2 => BulletOutput {
            scale_x: value - 1.0,
            ..BulletOutput::ZERO
        },
        3 => BulletOutput {
            scale_y: value - 1.0,
            ..BulletOutput::ZERO
        },
        4 => BulletOutput::new(value, 0.0),
        5 => BulletOutput::new(0.0, value),
        6 => BulletOutput::ZERO.with_rotation(value),
        _ => BulletOutput::ZERO,
    }
}

// ============================================================================
// Registration
// ============================================================================

export_mod! {
    behaviors: [],
    danmaku: [
        ("builtin.linear", LinearBehavior, || LinearBehavior { dir_x: 0.0, dir_y: -1.0, speed: 100.0 }),
        ("builtin.orbital", OrbitalBehavior, || OrbitalBehavior { angular_velocity: 1.0, radial_velocity: 0.0 }),
        ("builtin.sine", SineBehavior, || SineBehavior { axis_x: 1.0, axis_y: 0.0, amplitude: 20.0, frequency: 2.0, phase: 0.0 }),
        ("builtin.stationary", StationaryBehavior, || StationaryBehavior),
        ("builtin.aimed", AimedBehavior, || AimedBehavior { dir_x: 0.0, dir_y: -1.0, speed: 120.0 }),
        ("builtin.tween", TweenBehavior, || TweenBehavior { target: 0, duration: 1.0, ease: 0, range_start: 0.0, range_end: 1.0, delay: 0.0, timer: 0.0 }),
    ],
    patterns: [
        ("builtin.single", SinglePattern, || SinglePattern),
        ("builtin.ring", RingPattern, || RingPattern),
        ("builtin.line", LinePattern, || LinePattern),
        ("builtin.edge", EdgePattern, || EdgePattern),
    ],
}

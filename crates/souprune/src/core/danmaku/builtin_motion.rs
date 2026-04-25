//! Native Rust implementations of builtin danmaku motion behaviors.
//!
//! 内置弹幕运动行为的原生 Rust 实现。
//!
//! Replaces WASM dispatch for the six standard behaviors (Linear, Orbital,
//! Sine, Tween, Stationary, Aimed) with direct Rust math. Only
//! `BulletBehavior::Custom` still goes through the WASM runtime.

use bevy::prelude::*;
use souprune_schema::danmaku::{DanmakuTweenTarget, Easing};

/// Per-bullet runtime state for a single builtin behavior.
#[derive(Debug, Clone)]
pub enum BuiltinBehaviorState {
    Linear {
        dir: Vec2,
        speed: f32,
    },
    Orbital {
        angular_velocity: f32,
        radial_velocity: f32,
    },
    Sine {
        axis: Vec2,
        amplitude: f32,
        frequency: f32,
        phase: f32,
    },
    Tween {
        target: DanmakuTweenTarget,
        duration: f32,
        ease: Easing,
        from: f32,
        to: f32,
        delay: f32,
        mode: souprune_schema::danmaku::TweenMode,
        timer: f32,
    },
    Stationary,
    Aimed {
        dir: Vec2,
        speed: f32,
    },
}

/// Stack of active builtin behavior states for a bullet.
#[derive(Component, Default, Clone)]
pub struct BuiltinMotionStack {
    pub states: Vec<BuiltinBehaviorState>,
}

/// Compact output mirroring `souprune_api::BulletOutput`.
#[derive(Debug, Clone, Copy, Default)]
pub struct MotionOutput {
    pub offset: Vec2,
    pub rotation: f32,
    /// Negative means no change.
    pub opacity: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    /// When set, overrides the X position absolutely (not as an offset).
    pub absolute_x: Option<f32>,
    /// When set, overrides the Y position absolutely (not as an offset).
    pub absolute_y: Option<f32>,
}

impl MotionOutput {
    pub const ZERO: Self = Self {
        offset: Vec2::ZERO,
        rotation: 0.0,
        opacity: -1.0,
        scale_x: 0.0,
        scale_y: 0.0,
        absolute_x: None,
        absolute_y: None,
    };
}

/// Initialize a builtin behavior state from schema config + spawn context.
pub fn init_builtin_behavior(
    behavior: &souprune_schema::danmaku::BulletBehavior,
    spawn_pos: Vec2,
    offset: Vec2,
    player_pos: Vec2,
) -> Option<BuiltinBehaviorState> {
    use souprune_schema::danmaku::BulletBehavior;
    match behavior {
        BulletBehavior::Linear(cfg) => {
            let raw = Vec2::new(cfg.dir.0, cfg.dir.1);
            let dir = if raw.length_squared() > 0.001 * 0.001 {
                raw.normalize()
            } else {
                Vec2::ZERO
            };
            Some(BuiltinBehaviorState::Linear {
                dir,
                speed: cfg.speed,
            })
        }
        BulletBehavior::Orbital(cfg) => Some(BuiltinBehaviorState::Orbital {
            angular_velocity: cfg.angular_velocity,
            radial_velocity: cfg.radial_velocity,
        }),
        BulletBehavior::Sine(cfg) => {
            let raw = Vec2::new(cfg.axis.0, cfg.axis.1);
            let axis = if raw.length_squared() > 0.001 * 0.001 {
                raw.normalize()
            } else {
                Vec2::X
            };
            Some(BuiltinBehaviorState::Sine {
                axis,
                amplitude: cfg.amplitude,
                frequency: cfg.frequency,
                phase: cfg.phase,
            })
        }
        BulletBehavior::Tween(cfg) => Some(BuiltinBehaviorState::Tween {
            target: cfg.target,
            duration: cfg.duration,
            ease: cfg.ease,
            from: cfg.from,
            to: cfg.to,
            delay: cfg.delay,
            mode: cfg.mode,
            timer: 0.0,
        }),
        BulletBehavior::Stationary() => Some(BuiltinBehaviorState::Stationary),
        BulletBehavior::Aimed(cfg) => {
            let bullet_pos = spawn_pos + offset;
            let dx = player_pos.x - bullet_pos.x;
            let dy = player_pos.y - bullet_pos.y;
            let base_angle = dy.atan2(dx);
            let final_angle = base_angle + cfg.angle_offset;
            Some(BuiltinBehaviorState::Aimed {
                dir: Vec2::new(final_angle.cos(), final_angle.sin()),
                speed: cfg.speed,
            })
        }
        BulletBehavior::Custom { .. } => None,
    }
}

/// Compute one frame of a builtin behavior. Mutates `state` for Tween timer.
pub fn compute_builtin_output(
    state: &mut BuiltinBehaviorState,
    elapsed: f32,
    delta_time: f32,
    initial_offset: Vec2,
    initial_angle: f32,
    initial_radius: f32,
) -> MotionOutput {
    match state {
        BuiltinBehaviorState::Linear { dir, speed } => MotionOutput {
            offset: *dir * *speed * elapsed,
            ..MotionOutput::ZERO
        },
        BuiltinBehaviorState::Orbital {
            angular_velocity,
            radial_velocity,
        } => {
            let current_angle = initial_angle + *angular_velocity * elapsed;
            let current_radius = (initial_radius + *radial_velocity * elapsed).max(0.0);
            let orbital = Vec2::new(
                current_angle.cos() * current_radius,
                current_angle.sin() * current_radius,
            );
            MotionOutput {
                offset: orbital - initial_offset,
                rotation: *angular_velocity * delta_time,
                ..MotionOutput::ZERO
            }
        }
        BuiltinBehaviorState::Sine {
            axis,
            amplitude,
            frequency,
            phase,
        } => {
            let wave = (elapsed * *frequency * std::f32::consts::TAU + *phase).sin();
            MotionOutput {
                offset: *axis * wave * *amplitude,
                ..MotionOutput::ZERO
            }
        }
        BuiltinBehaviorState::Tween {
            target,
            duration,
            ease,
            from,
            to,
            delay,
            mode,
            timer,
        } => {
            *timer += delta_time;
            let t = *timer - *delay;
            let value = if t < 0.0 {
                *from
            } else if t >= *duration {
                *to
            } else {
                let progress = (t / *duration).clamp(0.0, 1.0);
                let eased = apply_easing(*ease, progress);
                *from + (*to - *from) * eased
            };
            apply_tween(*target, value, *mode)
        }
        BuiltinBehaviorState::Stationary => MotionOutput::ZERO,
        BuiltinBehaviorState::Aimed { dir, speed } => MotionOutput {
            offset: *dir * *speed * elapsed,
            ..MotionOutput::ZERO
        },
    }
}

fn apply_easing(ease: Easing, t: f32) -> f32 {
    match ease {
        Easing::Linear => t,
        Easing::InQuad => t * t,
        Easing::OutQuad => 1.0 - (1.0 - t) * (1.0 - t),
        Easing::InOutQuad => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0) * (-2.0 * t + 2.0) / 2.0
            }
        }
        Easing::InCubic => t * t * t,
        Easing::OutCubic => 1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t),
        Easing::InOutCubic => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0) * (-2.0 * t + 2.0) * (-2.0 * t + 2.0) / 2.0
            }
        }
        Easing::InSine => 1.0 - (t * std::f32::consts::FRAC_PI_2).cos(),
        Easing::OutSine => (t * std::f32::consts::FRAC_PI_2).sin(),
        Easing::InOutSine => -(t * std::f32::consts::PI).cos() / 2.0 + 0.5,
        Easing::InQuart => t * t * t * t,
        Easing::OutQuart => 1.0 - (1.0 - t).powi(4),
        Easing::InOutQuart => {
            if t < 0.5 {
                8.0 * t * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
            }
        }
        Easing::InQuint => t * t * t * t * t,
        Easing::OutQuint => 1.0 - (1.0 - t).powi(5),
        Easing::InOutQuint => {
            if t < 0.5 {
                16.0 * t * t * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
            }
        }
        Easing::InExpo => {
            if t <= 0.0 {
                0.0
            } else {
                (10.0_f32).powf(10.0 * t - 10.0)
            }
        }
        Easing::OutExpo => {
            if t >= 1.0 {
                1.0
            } else {
                1.0 - (10.0_f32).powf(-10.0 * t)
            }
        }
        Easing::InOutExpo => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else if t < 0.5 {
                (10.0_f32).powf(20.0 * t - 10.0) / 2.0
            } else {
                (2.0 - (10.0_f32).powf(-20.0 * t + 10.0)) / 2.0
            }
        }
        Easing::InCirc => 1.0 - (1.0 - t * t).sqrt(),
        Easing::OutCirc => (1.0 - (t - 1.0).powi(2)).sqrt(),
        Easing::InOutCirc => {
            if t < 0.5 {
                (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
            } else {
                ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
            }
        }
        Easing::InBack => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            c3 * t.powi(3) - c1 * t * t
        }
        Easing::OutBack => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
        }
        Easing::InOutBack => {
            let c1 = 1.70158;
            let c2 = c1 * 1.525;
            if t < 0.5 {
                let t2 = 2.0 * t;
                (t2 * t2 * ((c2 + 1.0) * t2 - c2)) / 2.0
            } else {
                let t2 = 2.0 * t - 2.0;
                ((t2 * t2 * ((c2 + 1.0) * t2 + c2)) + 2.0) / 2.0
            }
        }
        Easing::InElastic => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else {
                -(10.0_f32).powf(10.0 * t - 10.0) * ((10.0 * t - 10.75) * std::f32::consts::TAU / 3.0).sin()
            }
        }
        Easing::OutElastic => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else {
                (10.0_f32).powf(-10.0 * t) * ((10.0 * t - 0.75) * std::f32::consts::TAU / 3.0).sin() + 1.0
            }
        }
        Easing::InOutElastic => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else if t < 0.5 {
                let t2 = 2.0 * t;
                -(10.0_f32).powf(10.0 * t2 - 10.0)
                    * ((10.0 * t2 - 10.75) * std::f32::consts::TAU / 3.0).sin()
                    / 2.0
            } else {
                let t2 = 2.0 * t - 1.0;
                (10.0_f32).powf(-10.0 * t2)
                    * ((10.0 * t2 - 0.75) * std::f32::consts::TAU / 3.0).sin()
                    / 2.0
                    + 1.0
            }
        }
        Easing::InBounce => 1.0 - ease_out_bounce(1.0 - t),
        Easing::OutBounce => ease_out_bounce(t),
        Easing::InOutBounce => {
            if t < 0.5 {
                (1.0 - ease_out_bounce(1.0 - 2.0 * t)) / 2.0
            } else {
                (1.0 + ease_out_bounce(2.0 * t - 1.0)) / 2.0
            }
        }
    }
}

fn ease_out_bounce(x: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;

    if x < 1.0 / d1 {
        n1 * x * x
    } else if x < 2.0 / d1 {
        let x = x - 1.5 / d1;
        n1 * x * x + 0.75
    } else if x < 2.5 / d1 {
        let x = x - 2.25 / d1;
        n1 * x * x + 0.9375
    } else {
        let x = x - 2.625 / d1;
        n1 * x * x + 0.984375
    }
}

fn apply_tween(
    target: DanmakuTweenTarget,
    value: f32,
    mode: souprune_schema::danmaku::TweenMode,
) -> MotionOutput {
    use souprune_schema::danmaku::TweenMode;

    match target {
        DanmakuTweenTarget::Opacity => MotionOutput {
            opacity: value,
            ..MotionOutput::ZERO
        },
        DanmakuTweenTarget::Scale => MotionOutput {
            scale_x: value - 1.0,
            scale_y: value - 1.0,
            ..MotionOutput::ZERO
        },
        DanmakuTweenTarget::ScaleX => MotionOutput {
            scale_x: value - 1.0,
            ..MotionOutput::ZERO
        },
        DanmakuTweenTarget::ScaleY => MotionOutput {
            scale_y: value - 1.0,
            ..MotionOutput::ZERO
        },
        DanmakuTweenTarget::PositionX => match mode {
            TweenMode::Relative => MotionOutput {
                offset: Vec2::new(value, 0.0),
                ..MotionOutput::ZERO
            },
            TweenMode::Absolute => MotionOutput {
                absolute_x: Some(value),
                ..MotionOutput::ZERO
            },
        },
        DanmakuTweenTarget::PositionY => match mode {
            TweenMode::Relative => MotionOutput {
                offset: Vec2::new(0.0, value),
                ..MotionOutput::ZERO
            },
            TweenMode::Absolute => MotionOutput {
                absolute_y: Some(value),
                ..MotionOutput::ZERO
            },
        },
        DanmakuTweenTarget::Rotation => MotionOutput {
            rotation: value,
            ..MotionOutput::ZERO
        },
    }
}

/// Returns true if the behavior is a builtin variant (not Custom).
pub fn is_builtin(behavior: &souprune_schema::danmaku::BulletBehavior) -> bool {
    !matches!(
        behavior,
        souprune_schema::danmaku::BulletBehavior::Custom { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use souprune_schema::danmaku::LinearConfig;

    #[test]
    fn linear_motion_correct() {
        let mut state = BuiltinBehaviorState::Linear {
            dir: Vec2::new(0.0, -1.0),
            speed: 100.0,
        };
        let out = compute_builtin_output(&mut state, 1.0, 0.016, Vec2::ZERO, 0.0, 0.0);
        assert!((out.offset.y - (-100.0)).abs() < 0.01);
        assert!(out.offset.x.abs() < 0.01);
    }

    #[test]
    fn orbital_returns_to_start() {
        let mut state = BuiltinBehaviorState::Orbital {
            angular_velocity: std::f32::consts::TAU,
            radial_velocity: 0.0,
        };
        let offset = Vec2::new(50.0, 0.0);
        let out = compute_builtin_output(&mut state, 1.0, 0.016, offset, 0.0, 50.0);
        // After full revolution, orbital pos ≈ initial offset → delta ≈ 0
        assert!(out.offset.length() < 1.0);
    }

    #[test]
    fn sine_oscillates() {
        let mut state = BuiltinBehaviorState::Sine {
            axis: Vec2::X,
            amplitude: 20.0,
            frequency: 1.0,
            phase: 0.0,
        };
        let out_quarter = compute_builtin_output(&mut state, 0.25, 0.016, Vec2::ZERO, 0.0, 0.0);
        // At t=0.25 with freq=1, wave = sin(π/2) = 1.0
        assert!((out_quarter.offset.x - 20.0).abs() < 0.01);
    }

    #[test]
    fn tween_linear_midpoint() {
        let mut state = BuiltinBehaviorState::Tween {
            target: DanmakuTweenTarget::Opacity,
            duration: 1.0,
            ease: Easing::Linear,
            from: 0.0,
            to: 1.0,
            delay: 0.0,
            mode: souprune_schema::danmaku::TweenMode::Relative,
            timer: 0.0,
        };
        // Simulate 0.5 seconds of updates
        let out = compute_builtin_output(&mut state, 0.5, 0.5, Vec2::ZERO, 0.0, 0.0);
        assert!((out.opacity - 0.5).abs() < 0.01);
    }

    #[test]
    fn aimed_fires_toward_player() {
        let behavior = souprune_schema::danmaku::BulletBehavior::Aimed(
            souprune_schema::danmaku::AimedConfig {
                speed: 100.0,
                angle_offset: 0.0,
            },
        );
        let state = init_builtin_behavior(&behavior, Vec2::ZERO, Vec2::ZERO, Vec2::new(100.0, 0.0))
            .unwrap();
        let BuiltinBehaviorState::Aimed { dir, speed } = state else {
            panic!("Expected Aimed");
        };
        assert!((dir.x - 1.0).abs() < 0.01);
        assert!(dir.y.abs() < 0.01);
        assert!((speed - 100.0).abs() < 0.01);
    }

    #[test]
    fn stationary_returns_zero() {
        let mut state = BuiltinBehaviorState::Stationary;
        let out = compute_builtin_output(&mut state, 5.0, 0.016, Vec2::ZERO, 0.0, 0.0);
        assert_eq!(out.offset, Vec2::ZERO);
        assert_eq!(out.rotation, 0.0);
    }

    #[test]
    fn easing_boundaries() {
        assert!((apply_easing(Easing::InQuad, 0.0)).abs() < 0.001);
        assert!((apply_easing(Easing::InQuad, 1.0) - 1.0).abs() < 0.001);
        assert!((apply_easing(Easing::InOutCubic, 0.0)).abs() < 0.001);
        assert!((apply_easing(Easing::InOutCubic, 1.0) - 1.0).abs() < 0.001);
        assert!((apply_easing(Easing::InOutSine, 0.5) - 0.5).abs() < 0.001);
    }

    #[test]
    fn easing_new_categories_boundaries() {
        // All easings should map 0->0 and 1->1
        let easings = [
            Easing::InQuart,
            Easing::OutQuart,
            Easing::InOutQuart,
            Easing::InQuint,
            Easing::OutQuint,
            Easing::InOutQuint,
            Easing::InExpo,
            Easing::OutExpo,
            Easing::InOutExpo,
            Easing::InCirc,
            Easing::OutCirc,
            Easing::InOutCirc,
            Easing::InBack,
            Easing::OutBack,
            Easing::InOutBack,
            Easing::InElastic,
            Easing::OutElastic,
            Easing::InOutElastic,
            Easing::InBounce,
            Easing::OutBounce,
            Easing::InOutBounce,
        ];
        for e in easings {
            let v0 = apply_easing(e, 0.0);
            let v1 = apply_easing(e, 1.0);
            assert!(v0.abs() < 0.001, "{e:?} at t=0 expected 0, got {v0}");
            assert!((v1 - 1.0).abs() < 0.001, "{e:?} at t=1 expected 1, got {v1}");
        }
    }

    #[test]
    fn is_builtin_correctly_identifies() {
        assert!(is_builtin(
            &souprune_schema::danmaku::BulletBehavior::Linear(LinearConfig::default())
        ));
        assert!(is_builtin(
            &souprune_schema::danmaku::BulletBehavior::Stationary()
        ));
        assert!(!is_builtin(
            &souprune_schema::danmaku::BulletBehavior::Custom {
                id: "test".into(),
                props: Default::default(),
            }
        ));
    }
}

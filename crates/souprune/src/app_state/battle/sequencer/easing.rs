//! # sequencer/easing.rs
//!
//! ## Module Overview
//!
//! Easing functions for tween animations.
//!
//! 补间动画的缓动函数。

use super::super::chapter_schema::EasingFunction;

/// Apply easing function to a linear progress value (0.0 to 1.0).
///
/// 对线性进度值（0.0 到 1.0）应用缓动函数。
pub fn apply_easing(t: f32, easing: EasingFunction) -> f32 {
    match easing {
        EasingFunction::Linear => t,
        EasingFunction::QuadIn => t * t,
        EasingFunction::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
        EasingFunction::QuadInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
        EasingFunction::CubicIn => t * t * t,
        EasingFunction::CubicOut => 1.0 - (1.0 - t).powi(3),
        EasingFunction::CubicInOut => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
            }
        }
        EasingFunction::SineIn => 1.0 - (t * std::f32::consts::FRAC_PI_2).cos(),
        EasingFunction::SineOut => (t * std::f32::consts::FRAC_PI_2).sin(),
        EasingFunction::SineInOut => -(((t * std::f32::consts::PI).cos() - 1.0) / 2.0),
        EasingFunction::ExpoIn => {
            if t == 0.0 {
                0.0
            } else {
                2.0_f32.powf(10.0 * t - 10.0)
            }
        }
        EasingFunction::ExpoOut => {
            if t == 1.0 {
                1.0
            } else {
                1.0 - 2.0_f32.powf(-10.0 * t)
            }
        }
        EasingFunction::ExpoInOut => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else if t < 0.5 {
                2.0_f32.powf(20.0 * t - 10.0) / 2.0
            } else {
                (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
            }
        }
        EasingFunction::ElasticIn => {
            let c4 = (2.0 * std::f32::consts::PI) / 3.0;
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                -2.0_f32.powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * c4).sin()
            }
        }
        EasingFunction::ElasticOut => {
            let c4 = (2.0 * std::f32::consts::PI) / 3.0;
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
            }
        }
        EasingFunction::ElasticInOut => {
            let c5 = (2.0 * std::f32::consts::PI) / 4.5;
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else if t < 0.5 {
                -(2.0_f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
            } else {
                (2.0_f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0 + 1.0
            }
        }
        EasingFunction::BounceIn => 1.0 - apply_easing(1.0 - t, EasingFunction::BounceOut),
        EasingFunction::BounceOut => {
            let n1 = 7.5625;
            let d1 = 2.75;
            if t < 1.0 / d1 {
                n1 * t * t
            } else if t < 2.0 / d1 {
                let t = t - 1.5 / d1;
                n1 * t * t + 0.75
            } else if t < 2.5 / d1 {
                let t = t - 2.25 / d1;
                n1 * t * t + 0.9375
            } else {
                let t = t - 2.625 / d1;
                n1 * t * t + 0.984375
            }
        }
        EasingFunction::BounceInOut => {
            if t < 0.5 {
                (1.0 - apply_easing(1.0 - 2.0 * t, EasingFunction::BounceOut)) / 2.0
            } else {
                (1.0 + apply_easing(2.0 * t - 1.0, EasingFunction::BounceOut)) / 2.0
            }
        }
        EasingFunction::BackIn => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            c3 * t * t * t - c1 * t * t
        }
        EasingFunction::BackOut => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
        }
        EasingFunction::BackInOut => {
            let c1 = 1.70158;
            let c2 = c1 * 1.525;
            if t < 0.5 {
                ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
            } else {
                ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
            }
        }
    }
}

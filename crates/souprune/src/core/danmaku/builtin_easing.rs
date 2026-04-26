//! Easing functions for builtin danmaku motion.
//!
//! Extracted from `builtin_motion.rs` to keep that file under the 500-line limit.

use souprune_schema::danmaku::Easing;

pub fn apply_easing(ease: Easing, t: f32) -> f32 {
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
                -(10.0_f32).powf(10.0 * t - 10.0)
                    * ((10.0 * t - 10.75) * std::f32::consts::TAU / 3.0).sin()
            }
        }
        Easing::OutElastic => {
            if t <= 0.0 {
                0.0
            } else if t >= 1.0 {
                1.0
            } else {
                (10.0_f32).powf(-10.0 * t) * ((10.0 * t - 0.75) * std::f32::consts::TAU / 3.0).sin()
                    + 1.0
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

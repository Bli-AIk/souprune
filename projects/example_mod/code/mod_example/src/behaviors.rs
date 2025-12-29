//! Behavior module - exports all behaviors
//! 行为模块 - 导出所有行为

mod red_soul;
mod blue_soul;
mod homing_spear;

pub use red_soul::RedSoul;
pub use blue_soul::BlueSoul;
pub use homing_spear::HomingSpear;

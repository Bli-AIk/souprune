//! Behavior module - exports all behaviors
//! 行为模块 - 导出所有行为

mod aimed_spear;
mod gravity_drop;
mod red_soul;
mod spiral_homing;
mod wave_burst;

pub use aimed_spear::AimedSpear;
pub use gravity_drop::GravityDropDanmaku;
pub use red_soul::RedSoul;
pub use spiral_homing::SpiralHomingDanmaku;
pub use wave_burst::WaveBurstDanmaku;

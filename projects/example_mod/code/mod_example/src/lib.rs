//! Example mod demonstrating the Souprune modding API.
//! Contains player behaviors (RedSoul) and danmaku behaviors (AimedSpear, SpiralHoming, etc.).
//!
//! 示例模组，演示 Souprune 模组 API。
//! 包含玩家行为（红魂）和弹幕行为（自机狙长矛、螺旋追踪等）。

mod behaviors;

use behaviors::{AimedSpear, GravityDropDanmaku, RedSoul, SpiralHomingDanmaku, WaveBurstDanmaku};
use souprune_sdk::{declare_behaviors, declare_danmaku};

// Register player behaviors
// 注册玩家行为
declare_behaviors!(
    ("soul_red", RedSoul, || RedSoul::new()),
);

// Register danmaku behaviors
// 注册弹幕行为
declare_danmaku!(
    ("aimed_spear", AimedSpear, || AimedSpear::new()),
    ("spiral_homing", SpiralHomingDanmaku, || SpiralHomingDanmaku::new()),
    ("wave_burst", WaveBurstDanmaku, || WaveBurstDanmaku::new()),
    ("gravity_drop", GravityDropDanmaku, || GravityDropDanmaku::new()),
);

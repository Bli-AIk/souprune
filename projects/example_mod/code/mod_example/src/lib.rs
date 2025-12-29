//! Example mod demonstrating the Souprune modding API.
//! Contains player behaviors (RedSoul, BlueSoul) and danmaku behaviors (AimedSpear).
//!
//! 示例模组，演示 Souprune 模组 API。
//! 包含玩家行为（红魂、蓝魂）和弹幕行为（自机狙长矛）。

mod behaviors;

use behaviors::{AimedSpear, RedSoul};
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
);

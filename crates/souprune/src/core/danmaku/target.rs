//! # danmaku/target.rs
//!
//! ## Module Overview
//!
//! Defines the `BulletTarget` component for unified bullet targeting.
//! Any entity with this component can be tracked by bullets.
//!
//! ## 模块概述
//!
//! 定义 `BulletTarget` 组件，用于统一的弹幕追踪目标。
//! 任何带有此组件的实体都可以被弹幕追踪。

use bevy::prelude::*;

/// Marker component for entities that can be targeted by bullets.
/// Add this to any player-controlled or scripted entity that
/// should be tracked by aimed/homing behaviors.
///
/// 可被弹幕追踪的实体标记组件。
/// 将此组件添加到任何需要被自机狙/追踪行为追踪的实体
/// （例如玩家控制实体或脚本控制实体）。
#[derive(Component, Debug, Default, Clone)]
pub struct BulletTarget {
    /// Optional priority for targeting (higher = more likely to be targeted)
    /// 可选的目标优先级（越高越可能被追踪）
    pub priority: u8,

    /// Whether this target is currently active
    /// 此目标当前是否激活
    pub active: bool,
}

impl BulletTarget {
    pub fn new() -> Self {
        Self {
            priority: 0,
            active: true,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn inactive(mut self) -> Self {
        self.active = false;
        self
    }
}

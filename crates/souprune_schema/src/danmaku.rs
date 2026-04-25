//! # Danmaku Schema
//!
//! DanmakuPerformance schema types for `.performance.ron` files.
//! Mirrors `souprune::core::danmaku::danmaku_schema` without Bevy dependency.
//!
//! # 弹幕 Schema
//!
//! `.performance.ron` 文件的弹幕演出 Schema 类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod behavior;
mod prototype;
mod timeline;

pub use behavior::*;
pub use prototype::*;
pub use timeline::*;

/// Danmaku Performance asset — top-level `.performance.ron` schema.
///
/// 弹幕演出资源 — `.performance.ron` 的顶层 Schema。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DanmakuPerformance {
    #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
    pub prototypes: HashMap<String, BulletPrototype>,
    #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
    pub behaviors: HashMap<String, BulletBehavior>,
    pub timeline: Vec<TimelineEvent>,
    /// How long this performance lasts.
    /// - Literal: `duration: 5.0`
    /// - Omitted / 0 / negative: fire-and-forget (won't block sequencer).
    ///
    /// 演出持续时间。
    /// - 字面量：`duration: 5.0`
    /// - 省略 / 0 / 负数：不阻塞序列器。
    #[serde(default)]
    pub duration: Option<f32>,
}

impl DanmakuPerformance {
    /// Resolve `duration` to a blocking duration value.
    /// Returns `None` if duration is omitted. Callers treat zero or negative values as
    /// fire-and-forget.
    ///
    /// 将 `duration` 解析为阻塞时长。
    /// 如果省略则返回 `None`。调用方将零或负数视为不阻塞。
    pub fn resolved_duration(&self) -> Option<f32> {
        self.duration
    }
}

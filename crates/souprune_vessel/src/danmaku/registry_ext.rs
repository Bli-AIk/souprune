//! Extension trait for registering `DanmakuPerformance` entries.
//!
//! 注册 `DanmakuPerformance` 条目的扩展 trait。

use crate::guest;
use anyhow::Result;
use souprune_schema::danmaku::DanmakuPerformance;

/// Extension methods for [`Registry`] to register danmaku performances.
///
/// 为 [`Registry`] 添加弹幕演出注册方法。
pub trait PerformanceRegistry {
    /// Register a `DanmakuPerformance` under `states/battle/danmaku/{name}.performance.ron`.
    ///
    /// 注册一个弹幕演出，输出为 `states/battle/danmaku/{name}.performance.ron`。
    fn performance(&mut self, name: &str, perf: DanmakuPerformance) -> Result<()>;

    /// Register a `DanmakuPerformance` with a custom subdirectory path prefix.
    ///
    /// 注册一个弹幕演出，使用自定义子目录前缀。
    fn performance_at(&mut self, dir: &str, name: &str, perf: DanmakuPerformance) -> Result<()>;
}

impl PerformanceRegistry for guest::Registry {
    fn performance(&mut self, name: &str, perf: DanmakuPerformance) -> Result<()> {
        let path = format!("states/battle/danmaku/{name}.performance.ron");
        self.emit_ron(path, &perf)
    }

    fn performance_at(&mut self, dir: &str, name: &str, perf: DanmakuPerformance) -> Result<()> {
        let path = format!("{dir}/{name}.performance.ron");
        self.emit_ron(path, &perf)
    }
}

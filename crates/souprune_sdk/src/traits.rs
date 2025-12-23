use crate::context::Context;

/// All Soul Modes must implement this trait.
/// Corresponds to the lifecycle in api_design.md.
///
/// 所有的 Soul Mode 都要实现这个特征
/// 对应 api_design.md 里的生命周期
pub trait SoulMode {
    /// Initialization
    /// 初始化
    fn on_enter(&mut self, _context: &mut Context) {}

    /// Per-frame update
    /// 每帧更新
    fn on_update(&mut self, context: &mut Context, dt: f32);

    /// Exit cleanup
    /// 退出清理
    fn on_exit(&mut self, _context: &mut Context) {}
}

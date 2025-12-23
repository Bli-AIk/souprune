use crate::context::Context;

/// 所有的 Soul Mode 都要实现这个特征
/// 对应 api_design.md 里的生命周期
pub trait SoulMode {
    /// 初始化
    fn on_enter(&mut self, _context: &mut Context) {}

    /// 每帧更新
    fn on_update(&mut self, context: &mut Context, dt: f32);

    /// 退出清理
    fn on_exit(&mut self, _context: &mut Context) {}
}

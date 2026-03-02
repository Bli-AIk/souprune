//! 平台适配模块。

#[cfg(target_os = "android")]
mod android;
mod desktop;

pub use desktop::DesktopPlatformPlugin;

use bevy::prelude::*;
use bevy_workbench::layout::ActiveLayout;

/// 根据当前平台选择合适的插件组合。
pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_os = "android")]
        app.add_plugins(android::AndroidPlatformPlugin);

        #[cfg(not(target_os = "android"))]
        app.add_plugins(DesktopPlatformPlugin);

        // 响应式字体大小（所有平台通用）
        app.init_resource::<EditorFontScale>()
            .add_systems(Update, update_editor_font_scale);
    }
}

/// 检查当前是否处于竖屏（移动端）布局。
#[allow(dead_code)]
pub(crate) fn is_portrait(layout: &bevy_workbench::layout::LayoutState) -> bool {
    layout.active == ActiveLayout::Portrait
}

// ─── P7.6 响应式字体大小 ─────────────────────────────────

/// 编辑器特有控件的字体缩放因子。
///
/// bevy_workbench 已提供全局 `ui_scale`，这里补充编辑器面板中
/// 卡片标题、属性标签等自定义控件的缩放。
#[derive(Resource)]
pub(crate) struct EditorFontScale {
    /// 当前字体缩放因子（1.0 = 默认）。
    pub factor: f32,
}

impl Default for EditorFontScale {
    fn default() -> Self {
        Self { factor: 1.0 }
    }
}

/// 根据窗口 DPI 和平台自动更新编辑器字体缩放。
fn update_editor_font_scale(windows: Query<&Window>, mut font_scale: ResMut<EditorFontScale>) {
    let Ok(window) = windows.single() else {
        return;
    };
    let scale = window.scale_factor();

    // Android 使用更大的基础字体
    #[cfg(target_os = "android")]
    let base = 1.2;
    #[cfg(not(target_os = "android"))]
    let base = 1.0;

    let new_factor = base * (scale / 1.0_f32).clamp(0.8, 2.5);

    if (font_scale.factor - new_factor).abs() > 0.01 {
        font_scale.factor = new_factor;
    }
}

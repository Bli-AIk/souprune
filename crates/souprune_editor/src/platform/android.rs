//! Android 平台适配 — 手势识别、虚拟键盘处理、性能优化。

use bevy::prelude::*;
use bevy_workbench::theme::ThemeState;

use crate::panels::sequence_timeline::EditorSequenceState;

// ─── 手势常量 ─────────────────────────────────────────────

const LONG_PRESS_DURATION: f32 = 0.5;
const SWIPE_THRESHOLD: f32 = 50.0;
const TAP_THRESHOLD: f32 = 10.0;

// ─── 插件 ─────────────────────────────────────────────────

/// Android 平台插件 — 触摸模式、手势识别、键盘适配、性能配置。
#[allow(dead_code)]
pub struct AndroidPlatformPlugin;

impl Plugin for AndroidPlatformPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ThemeState::touch())
            .init_resource::<TouchGestureState>()
            .init_resource::<AndroidPerfConfig>()
            .add_systems(Update, (gesture_recognition_system, virtual_keyboard_system));
    }
}

// ─── P7.1 手势状态 ────────────────────────────────────────

/// 手势识别器内部状态。
#[derive(Resource, Default)]
struct TouchGestureState {
    /// 主触摸点起始位置。
    primary_start: Option<Vec2>,
    /// 主触摸点按下时长。
    primary_held: f32,
    /// 是否已触发 long press（防止重复）。
    long_pressed: bool,
    /// 双指缩放起始距离。
    pinch_start_dist: Option<f32>,
}

/// 手势识别系统 — 将 touch 事件转为编辑操作。
fn gesture_recognition_system(
    touches: Res<Touches>,
    time: Res<Time>,
    mut gesture: ResMut<TouchGestureState>,
    mut state: ResMut<EditorSequenceState>,
) {
    let pressed: Vec<_> = touches.iter().collect();

    // 双指 pinch 检测
    if pressed.len() >= 2 {
        let dist = pressed[0].position().distance(pressed[1].position());
        if gesture.pinch_start_dist.is_none() {
            gesture.pinch_start_dist = Some(dist);
        }
        // pinch 结束后在 release 阶段处理
    } else {
        gesture.pinch_start_dist = None;
    }

    // 单指手势
    if let Some(touch) = touches.iter_just_pressed().next() {
        gesture.primary_start = Some(touch.position());
        gesture.primary_held = 0.0;
        gesture.long_pressed = false;
    }

    if gesture.primary_start.is_some() && pressed.len() == 1 {
        gesture.primary_held += time.delta_secs();

        // Long press → 打开章节上下文菜单
        if !gesture.long_pressed && gesture.primary_held >= LONG_PRESS_DURATION {
            gesture.long_pressed = true;
            // long press 行为由 egui 层处理，这里仅标记
        }
    }

    // 手指抬起 → 判断 tap 或 swipe
    for released in touches.iter_just_released() {
        if let Some(start) = gesture.primary_start.take() {
            let delta = released.position() - start;
            let dist = delta.length();

            if dist < TAP_THRESHOLD && !gesture.long_pressed {
                // Tap — 选择章节（具体索引由 egui 处理）
            } else if dist >= SWIPE_THRESHOLD && !gesture.long_pressed {
                // Swipe 导航
                if delta.y.abs() > delta.x.abs() {
                    // 垂直滑动 → 上下导航
                    let direction = if delta.y < 0.0 { -1i32 } else { 1 };
                    if let Some(idx) = state.selected_chapter {
                        let new_idx = (idx as i32 + direction).max(0) as usize;
                        let max = state
                            .current
                            .as_ref()
                            .map(|s| s.chapters.len().saturating_sub(1))
                            .unwrap_or(0);
                        state.selected_chapter = Some(new_idx.min(max));
                    }
                }
            }
        }
    }
}

// ─── P7.2 虚拟键盘处理 ───────────────────────────────────

/// 虚拟键盘适配系统 — 检测键盘状态并调整 UI 布局偏移。
fn virtual_keyboard_system(
    windows: Query<&Window>,
    mut _egui_ctx: bevy_egui::EguiContexts,
) {
    // Bevy 0.18 没有直接提供虚拟键盘高度 API。
    // 当 egui 文本输入获得焦点时，Android 系统会自动弹出键盘。
    // bevy_egui 在 Android 上会自动处理视口调整。
    // 这里预留钩子：如果未来需要手动偏移，可通过窗口尺寸变化检测。
    let Ok(window) = windows.single() else {
        return;
    };
    let _height = window.resolution.height();
    // 未来可在此检测分辨率突变（键盘弹出/收起）并调整 egui area offset。
}

// ─── P7.3 性能优化配置 ───────────────────────────────────

/// Android 性能优化配置。
#[derive(Resource)]
struct AndroidPerfConfig {
    /// 虚拟滚动时可见的最大章节卡片数量。
    pub max_visible_cards: usize,
    /// Game View 分辨率缩放因子（0.5 = 半分辨率）。
    pub game_view_scale: f32,
}

impl Default for AndroidPerfConfig {
    fn default() -> Self {
        Self {
            max_visible_cards: 30,
            game_view_scale: 0.5,
        }
    }
}

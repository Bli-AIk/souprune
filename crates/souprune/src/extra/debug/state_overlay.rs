//! F3 debug state overlay — shows sequence, view, and performance paths.
//!
//! F3 调试状态覆盖层 — 显示序列、视图和弹幕演出路径。
//!
//! When the F3 performance UI is toggled on, an additional panel appears
//! in the bottom-left corner showing the currently active runtime state:
//! which sequence file is executing, which view layout is loaded, and
//! which danmaku performance is playing.
//!
//! 当 F3 性能 UI 开启时，在左下角显示额外面板，展示当前活跃的
//! 运行时状态：正在执行的序列文件、已加载的视图布局、以及正在
//! 播放的弹幕演出。

#[cfg(feature = "debug")]
pub(super) mod debug_state_overlay {
    use crate::core::danmaku::PerformanceHandle;
    use crate::core::danmaku::PerformancePlayer;
    use crate::core::sequencer::SequenceDebugInfo;
    use crate::core::view::components::{ActiveView, ViewRoot};
    use bevy::prelude::*;

    /// Marker component for the state overlay root entity.
    ///
    /// 状态覆盖层根实体的标记组件。
    #[derive(Component)]
    struct StateOverlayRoot;

    /// Marker component for each text line in the overlay.
    ///
    /// 覆盖层中每行文本的标记组件。
    #[derive(Component)]
    enum StateOverlayLine {
        CurrentSequence,
        PreviousSequence,
        CurrentView,
        CurrentPerformance,
    }

    pub fn setup_state_overlay(app: &mut App) {
        app.add_systems(
            Update,
            (toggle_state_overlay_system, update_state_overlay_system).chain(),
        );
    }

    fn toggle_state_overlay_system(
        mut commands: Commands,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        overlay_query: Query<Entity, With<StateOverlayRoot>>,
    ) {
        if !keyboard_input.just_pressed(KeyCode::F3) {
            return;
        }

        // F3 toggles perf UI. If perf UI is being turned ON (it didn't exist
        // before the toggle_perf_ui_system despawned/spawned it), spawn overlay.
        // We check AFTER the perf UI toggle, so if PerfUiRoot now exists → ON.
        // But since system ordering isn't guaranteed, we use a simpler heuristic:
        // if overlay exists, despawn it; otherwise spawn it.
        if let Ok(entity) = overlay_query.single() {
            commands.entity(entity).despawn();
        } else {
            spawn_overlay(&mut commands);
        }
    }

    fn spawn_overlay(commands: &mut Commands) {
        commands
            .spawn((
                StateOverlayRoot,
                Node {
                    display: Display::Flex,
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.0),
                    left: Val::Px(10.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(4.0)),
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                GlobalZIndex(i32::MAX - 2),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            ))
            .with_children(|parent| {
                let font = TextFont::from_font_size(11.0);
                let color = TextColor(Color::srgba(0.7, 1.0, 0.7, 0.9));

                let lines = [
                    (StateOverlayLine::CurrentSequence, "Seq: —"),
                    (StateOverlayLine::PreviousSequence, "Prev: —"),
                    (StateOverlayLine::CurrentView, "View: —"),
                    (StateOverlayLine::CurrentPerformance, "Perf: —"),
                ];

                for (marker, initial_text) in lines {
                    parent.spawn((marker, Text::new(initial_text), font.clone(), color));
                }
            });
    }

    fn update_state_overlay_system(
        debug_info: Res<SequenceDebugInfo>,
        view_query: Query<&ViewRoot, With<ActiveView>>,
        perf_query: Query<&PerformanceHandle, With<PerformancePlayer>>,
        asset_server: Res<AssetServer>,
        mut text_query: Query<(&StateOverlayLine, &mut Text)>,
    ) {
        for (line, mut text) in text_query.iter_mut() {
            match line {
                StateOverlayLine::CurrentSequence => {
                    let path = debug_info.current_path.as_deref().unwrap_or("—");
                    **text = format!("Seq: {path}");
                }
                StateOverlayLine::PreviousSequence => {
                    let path = debug_info.previous_path.as_deref().unwrap_or("—");
                    **text = format!("Prev: {path}");
                }
                StateOverlayLine::CurrentView => {
                    let path = view_query
                        .iter()
                        .next()
                        .map(|v| v.layout_path.as_str())
                        .unwrap_or("—");
                    **text = format!("View: {path}");
                }
                StateOverlayLine::CurrentPerformance => {
                    let path = perf_query
                        .iter()
                        .next()
                        .and_then(|h| asset_server.get_path(h.0.id()).map(|p| p.to_string()))
                        .unwrap_or_else(|| "—".to_string());
                    **text = format!("Perf: {path}");
                }
            }
        }
    }
}

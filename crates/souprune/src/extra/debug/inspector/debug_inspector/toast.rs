//! Shows short-lived debug toast messages inside the standalone inspector tooling.
//!
//! 在独立 Inspector 调试工具中显示短暂的提示气泡消息。
//!
//! This file owns the tiny notification surface used when debug features are
//! toggled. It creates the toast entity tree, updates the displayed message from
//! incoming events, and hides the toast again after its timer expires.
//!
//! 这个文件负责调试功能切换时显示的小型提示层。它创建 toast 实体树，从事件里
//! 更新提示文本，并在计时结束后把提示隐藏回去。

use super::set_text_entities_color;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use std::time::Duration;

/// Toast notification shown when debug features are toggled.
#[derive(Component)]
struct DebugToast {
    timer: Timer,
    fade_out_started: bool,
}

pub(super) fn setup_debug_toast_system(mut commands: Commands) {
    commands
        .spawn((
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: Val::Px(10.),
                left: Val::Px(10.),
                ..default()
            },
            GlobalZIndex(i32::MAX - 1),
            DebugToast {
                timer: Timer::new(Duration::from_secs(2), TimerMode::Once),
                fade_out_started: false,
            },
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new(""),
                TextColor(Color::WHITE),
                TextFont::from_font_size(14.0),
            ));
        });
}

pub(super) fn handle_debug_toast_event_system(
    mut events: MessageReader<super::super::DebugToastEvent>,
    mut q_toast: Query<(&mut DebugToast, &mut Node, &Children)>,
    mut q_text: Query<(&mut Text, &mut TextColor)>,
) {
    let Some(event) = events.read().last() else {
        return;
    };

    let Ok((mut toast, mut style, children)) = q_toast.single_mut() else {
        return;
    };

    for child in children.iter() {
        if let Ok((mut text, mut tc)) = q_text.get_mut(child) {
            **text = event.message.clone();
            tc.0 = Color::WHITE;
        }
    }

    style.display = Display::Flex;
    toast.timer.reset();
    toast.fade_out_started = false;
}

pub(super) fn fade_debug_toast_system(
    time: Res<Time>,
    mut q_toast: Query<(&mut DebugToast, &mut Node, &Children)>,
    mut q_text_colors: Query<&mut TextColor>,
) {
    let Ok((mut toast, mut style, children)) = q_toast.single_mut() else {
        return;
    };

    if toast.fade_out_started {
        return;
    }

    toast.timer.tick(time.delta());

    if toast.timer.is_finished() {
        toast.fade_out_started = false;
        let entities: Vec<Entity> = children.iter().collect();
        set_text_entities_color(
            &entities,
            Color::srgba(1.0, 1.0, 1.0, 0.0),
            &mut q_text_colors,
        );
        style.display = Display::None;
    }
}

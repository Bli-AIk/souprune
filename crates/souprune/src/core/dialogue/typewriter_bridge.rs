//! Bridge between Mortar events and Typewriter control.
//!
//! Mortar 事件与 Typewriter 控制的桥接。
//!
//! Handles `MortarGameEvent` for typewriter-related functions:
//! - `set_typewriter_speed(speed: Number)` - Change typing speed
//! - `pause_typewriter()` - Pause typewriter
//! - `resume_typewriter()` - Resume typewriter
//!
//! 处理打字机相关的 `MortarGameEvent`：
//! - `set_typewriter_speed(speed: Number)` - 更改打字速度
//! - `pause_typewriter()` - 暂停打字机
//! - `resume_typewriter()` - 恢复打字机

use bevy::prelude::*;
use bevy_ecs_typewriter::Typewriter;
use bevy_mortar_bond::MortarGameEvent;
use std::time::Duration;

use super::components::MortarController;

/// Handles Mortar game events that control typewriter behavior.
///
/// 处理控制打字机行为的 Mortar 游戏事件。
///
/// Mortar scripts can call these functions within text events:
/// ```mortar
/// text: "This text starts slow..."
/// with events: [
///     0, set_typewriter_speed(0.1)    // 0.1 seconds per character
///     10, set_typewriter_speed(0.02)  // Speed up!
/// ]
/// ```
///
/// Mortar 脚本可在文本事件中调用这些函数：
/// ```mortar
/// text: "这段文字开始很慢……"
/// with events: [
///     0, set_typewriter_speed(0.1)    // 每字符0.1秒
///     10, set_typewriter_speed(0.02)  // 加速！
/// ]
/// ```
pub fn handle_typewriter_mortar_events(
    mut events: MessageReader<MortarGameEvent>,
    mut typewriters: Query<&mut Typewriter, With<MortarController>>,
) {
    for event in events.read() {
        match event.name.as_str() {
            "set_typewriter_speed" => {
                handle_set_speed(event, &mut typewriters);
            }
            "pause_typewriter" => {
                handle_pause(event, &mut typewriters);
            }
            "resume_typewriter" => {
                handle_resume(event, &mut typewriters);
            }
            _ => {
                // Other events handled by other systems
            }
        }
    }
}

fn handle_set_speed(
    event: &MortarGameEvent,
    typewriters: &mut Query<&mut Typewriter, With<MortarController>>,
) {
    let Some(speed_str) = event.args.first() else {
        warn!("set_typewriter_speed: missing speed argument");
        return;
    };

    let Ok(speed) = speed_str.parse::<f32>() else {
        warn!("set_typewriter_speed: invalid speed value '{}'", speed_str);
        return;
    };

    if speed <= 0.0 {
        warn!(
            "set_typewriter_speed: speed must be positive, got {}",
            speed
        );
        return;
    }

    // If event has source entity, only update that entity
    // Otherwise update all typewriters with MortarController
    if let Some(entity) = event.source {
        if let Ok(mut tw) = typewriters.get_mut(entity) {
            tw.timer.set_duration(Duration::from_secs_f32(speed));
            debug!("Typewriter speed set to {} for entity {:?}", speed, entity);
        }
    } else {
        for mut tw in typewriters.iter_mut() {
            tw.timer.set_duration(Duration::from_secs_f32(speed));
        }
        debug!("Typewriter speed set to {} for all typewriters", speed);
    }
}

fn handle_pause(
    event: &MortarGameEvent,
    typewriters: &mut Query<&mut Typewriter, With<MortarController>>,
) {
    if let Some(entity) = event.source {
        if let Ok(mut tw) = typewriters.get_mut(entity) {
            tw.pause();
            debug!("Typewriter paused for entity {:?}", entity);
        }
    } else {
        for mut tw in typewriters.iter_mut() {
            tw.pause();
        }
        debug!("All typewriters paused");
    }
}

fn handle_resume(
    event: &MortarGameEvent,
    typewriters: &mut Query<&mut Typewriter, With<MortarController>>,
) {
    if let Some(entity) = event.source {
        if let Ok(mut tw) = typewriters.get_mut(entity) {
            tw.resume();
            debug!("Typewriter resumed for entity {:?}", entity);
        }
    } else {
        for mut tw in typewriters.iter_mut() {
            tw.resume();
        }
        debug!("All typewriters resumed");
    }
}

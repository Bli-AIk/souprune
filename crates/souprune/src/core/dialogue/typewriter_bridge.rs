//! Bridge between Mortar events and Typewriter control.
//!
//! Mortar 事件与 Typewriter 控制的桥接。
//!
//! Handles `MortarGameEvent` for typewriter-related functions:
//! - `set_typewriter_speed(speed: Number)` - Change typing speed
//! - `pause_typewriter()` - Pause typewriter
//! - `resume_typewriter()` - Resume typewriter
//! - `apply_shake(intensity?)` - Add shake effect to the current glyph
//! - `apply_wave(amplitude?, frequency?)` - Add wave effect to the current glyph
//!
//! 处理打字机相关的 `MortarGameEvent`：
//! - `set_typewriter_speed(speed: Number)` - 更改打字速度
//! - `pause_typewriter()` - 暂停打字机
//! - `resume_typewriter()` - 恢复打字机
//! - `apply_shake(intensity?)` - 为当前字形添加抖动效果
//! - `apply_wave(amplitude?, frequency?)` - 为当前字形添加波浪效果

use bevy::prelude::*;
use bevy_bitmap_text::{GlyphEntity, ShakeEffect, WaveEffect};
use bevy_ecs_typewriter::Typewriter;
use bevy_mortar_bond::MortarGameEvent;
use std::time::Duration;

use super::auto_pause::AutoPauseTimer;
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
    events: Option<MessageReader<MortarGameEvent>>,
    mut typewriters: Query<&mut Typewriter, With<MortarController>>,
    mut commands: Commands,
    glyph_query: Query<(Entity, &GlyphEntity, &ChildOf)>,
) {
    let Some(mut events) = events else {
        return;
    };

    for event in events.read() {
        match event.name.as_str() {
            "set_typewriter_speed" => {
                handle_set_speed(event, &mut typewriters);
            }
            "pause_typewriter" => {
                handle_pause(event, &mut typewriters, &mut commands);
            }
            "resume_typewriter" => {
                handle_resume(event, &mut typewriters);
            }
            "apply_shake" => {
                handle_apply_shake(event, &mut commands, &typewriters, &glyph_query);
            }
            "apply_wave" => {
                handle_apply_wave(event, &mut commands, &typewriters, &glyph_query);
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
    commands: &mut Commands,
) {
    let duration = event.args.first().and_then(|s| s.parse::<f64>().ok());

    if let Some(entity) = event.source {
        if let Ok(mut tw) = typewriters.get_mut(entity) {
            tw.pause();
            if let Some(secs) = duration {
                commands.entity(entity).insert(AutoPauseTimer::new(secs));
            }
            debug!(
                "Typewriter paused for entity {:?} (duration: {:?})",
                entity, duration
            );
        }
    } else {
        for mut tw in typewriters.iter_mut() {
            tw.pause();
        }
        debug!("All typewriters paused (duration arg ignored for broadcast)");
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

/// Find the glyph entity at a given char_index relative to the typewriter's current position.
///
/// Mortar events fire at a specific char index. We look up the GlyphEntity
/// child whose `char_index` matches the event's index (derived from the
/// typewriter's current_char_index at the time the event fired).
fn find_glyph_at_index(
    char_index: usize,
    typewriters: &Query<&mut Typewriter, With<MortarController>>,
    glyph_query: &Query<(Entity, &GlyphEntity, &ChildOf)>,
) -> Option<Entity> {
    // Get current char index from the typewriter (the index that triggered this event).
    let _tw = typewriters.iter().next()?;

    // Search all glyph entities for one matching the char_index.
    // In a dialogue, there's typically one active text block.
    glyph_query
        .iter()
        .find(|(_, ge, _)| ge.char_index == char_index)
        .map(|(entity, _, _)| entity)
}

fn handle_apply_shake(
    event: &MortarGameEvent,
    commands: &mut Commands,
    typewriters: &Query<&mut Typewriter, With<MortarController>>,
    glyph_query: &Query<(Entity, &GlyphEntity, &ChildOf)>,
) {
    let intensity = event
        .args
        .first()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(2.0);

    // The event index in Mortar corresponds to a char_index in the text.
    let char_index = typewriters
        .iter()
        .next()
        .map(|tw| tw.current_char_index.saturating_sub(1))
        .unwrap_or(0);

    if let Some(entity) = find_glyph_at_index(char_index, typewriters, glyph_query) {
        commands.entity(entity).insert(ShakeEffect {
            intensity,
            elapsed: 0.0,
        });
        debug!(
            "Applied shake effect (intensity={}) to glyph at index {}",
            intensity, char_index
        );
    }
}

fn handle_apply_wave(
    event: &MortarGameEvent,
    commands: &mut Commands,
    typewriters: &Query<&mut Typewriter, With<MortarController>>,
    glyph_query: &Query<(Entity, &GlyphEntity, &ChildOf)>,
) {
    let amplitude = event
        .args
        .first()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(3.0);

    let frequency = event
        .args
        .get(1)
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(4.0);

    let char_index = typewriters
        .iter()
        .next()
        .map(|tw| tw.current_char_index.saturating_sub(1))
        .unwrap_or(0);

    if let Some(entity) = find_glyph_at_index(char_index, typewriters, glyph_query) {
        commands.entity(entity).insert(WaveEffect {
            amplitude,
            frequency,
            elapsed: 0.0,
        });
        debug!(
            "Applied wave effect (amp={}, freq={}) to glyph at index {}",
            amplitude, frequency, char_index
        );
    }
}

//! Plays per-character dialogue voice sounds while a typewriter is revealing text.
//!
//! 在打字机逐字显示文本时播放对应的逐字语音音效。
//!
//! Keeps a deliberately narrow scope: it watches typewriter
//! progress, compares the newly revealed character index with the previous one,
//! and triggers the configured voice sound whenever another character appears.
//!
//! 刻意把职责收得很窄：它监视打字机的推进进度，对比这次新显示的字符
//! 索引与上一次的差值，并在出现新的字符时触发配置好的语音音效。

use bevy::prelude::*;
use bevy_ecs_typewriter::{Typewriter, TypewriterState};

use crate::core::dialogue::TypewriterVoice;

pub fn typewriter_voice_system(
    mut query: Query<(&Typewriter, &mut TypewriterVoice)>,
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
) {
    use bevy_kira_audio::AudioControl;

    for (typewriter, mut voice) in query.iter_mut() {
        if typewriter.state != TypewriterState::Playing {
            continue;
        }

        if typewriter.current_char_index > voice.last_char_index {
            let handle: Handle<bevy_kira_audio::AudioSource> = asset_server.load(&voice.sound_path);
            audio.play(handle);
            voice.last_char_index = typewriter.current_char_index;
        } else if typewriter.current_char_index < voice.last_char_index {
            voice.last_char_index = typewriter.current_char_index;
        }
    }
}

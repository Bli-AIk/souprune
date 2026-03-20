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

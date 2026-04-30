//! Mirrors dialogue presentation state back into facts and active view-local data.
//!
//! 把对话表现层状态反向同步回 facts 与当前活动 View 的局部数据。
//!
//! Keeps the rest of the game aware of dialogue progress. It writes
//! typewriter completion flags into the layered fact database, mirrors visible
//! dialogue text into active views, and handles pause/resume or replay behavior
//! when menu depth changes interrupt dialogue flow.
//!
//! 让游戏其他部分能够感知对话进度。它把打字机完成状态写回分层 fact
//! 数据库，把当前可见文本镜像到活动 View 中，并在菜单深度变化打断对话流程时
//! 处理暂停、恢复或重播行为。

use bevy::prelude::*;
use bevy_ecs_typewriter::{Typewriter, TypewriterState};
use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};
use bevy_mortar_bond::MortarRuntime;

use super::lifecycle::DialogueControllerEntity;
use crate::core::fre_facts;
use crate::core::view::components::{ActiveView, ViewRoot};

pub fn sync_typewriter_state_to_facts_system(
    query: Query<
        (&crate::core::dialogue::DialogueChannel, Option<&Typewriter>),
        With<DialogueControllerEntity>,
    >,
    mut facts: ResMut<LayeredFactDatabase>,
) {
    let mut any_channel_playing = false;
    let mut all_channels_finished = true;
    let mut any_channel_finished = false;
    let mut has_controllers = false;
    let mut changed = false;

    for (channel, typewriter) in query.iter() {
        has_controllers = true;

        let (any_playing, all_finished, any_finished) = match typewriter {
            Some(tw) => match tw.state {
                TypewriterState::Playing | TypewriterState::Paused => (true, false, false),
                TypewriterState::Finished => (false, true, true),
                TypewriterState::Idle => (false, true, false),
            },
            None => (false, true, true),
        };

        any_channel_playing |= any_playing;
        all_channels_finished &= all_finished;
        any_channel_finished |= any_finished;

        let db = facts.bypass_change_detection();
        if db.set_if_changed(
            fre_facts::dialogue_channel_key(&channel.name, "typewriter_playing"),
            any_playing,
        ) {
            changed = true;
        }
        if db.set_if_changed(
            fre_facts::dialogue_channel_key(&channel.name, "all_typewriters_finished"),
            all_finished,
        ) {
            changed = true;
        }
        if db.set_if_changed(
            fre_facts::dialogue_channel_key(&channel.name, "any_typewriter_finished"),
            any_finished,
        ) {
            changed = true;
        }
    }

    if !has_controllers {
        all_channels_finished = true;
        any_channel_finished = true;
    }

    let db = facts.bypass_change_detection();
    if db.set_if_changed(fre_facts::DIALOGUE_TYPEWRITER_PLAYING, any_channel_playing) {
        changed = true;
    }
    if db.set_if_changed(
        fre_facts::DIALOGUE_ALL_TYPEWRITERS_FINISHED,
        all_channels_finished,
    ) {
        changed = true;
    }
    if db.set_if_changed(
        fre_facts::DIALOGUE_ANY_TYPEWRITER_FINISHED,
        any_channel_finished,
    ) {
        changed = true;
    }

    if changed {
        facts.set_changed();
    }
}

pub fn sync_typewriter_text_to_facts_system(
    runtime: Res<MortarRuntime>,
    query: Query<
        (
            Entity,
            &crate::core::dialogue::DialogueChannel,
            Option<&Typewriter>,
        ),
        With<DialogueControllerEntity>,
    >,
    mut facts: ResMut<LayeredFactDatabase>,
) {
    let mut changed = false;

    for (entity, channel, typewriter) in query.iter() {
        let new_text = if let Some(typewriter) = typewriter {
            typewriter.current_text.clone()
        } else if let Some(state) = runtime.get_dialogue(entity) {
            state.current_text().unwrap_or("").to_string()
        } else {
            String::new()
        };

        trace!(
            "sync_typewriter_text_to_facts: channel='{}', text='{}'",
            channel.name, new_text
        );

        let dialogue_visible = !new_text.is_empty();
        let db = facts.bypass_change_detection();
        if db.set_if_changed(
            fre_facts::dialogue_channel_key(&channel.name, "text"),
            new_text,
        ) {
            changed = true;
        }
        if db.set_if_changed(
            fre_facts::dialogue_channel_key(&channel.name, "visible"),
            dialogue_visible,
        ) {
            changed = true;
        }
    }

    if changed {
        facts.set_changed();
    }
}

pub fn replay_typewriter_on_depth_resume_system(
    mut active_view_query: Query<&mut ViewRoot, With<ActiveView>>,
    mut typewriter_query: Query<&mut Typewriter, With<DialogueControllerEntity>>,
    mut prev_depth: Local<Option<i64>>,
) {
    let current_depth = active_view_query
        .iter()
        .next()
        .and_then(|view| view.local_facts.get_int("depth"));

    if *prev_depth != current_depth {
        debug!(
            "replay_typewriter_on_depth_resume: depth changed {:?} -> {:?}",
            *prev_depth, current_depth
        );
    }

    let left_zero = match (*prev_depth, current_depth) {
        (Some(prev), Some(curr)) if prev == 0 && curr != 0 => true,
        (None, Some(curr)) if curr != 0 => true,
        _ => false,
    };
    let resumed_to_zero = matches!((*prev_depth, current_depth),
        (Some(prev), Some(curr)) if prev != 0 && curr == 0
    );

    *prev_depth = current_depth;

    if left_zero {
        info!("replay_typewriter_on_depth_resume: depth left 0, pausing typewriters");
        for mut typewriter in typewriter_query.iter_mut() {
            typewriter.pause();
        }
        return;
    }

    if !resumed_to_zero {
        return;
    }

    info!("replay_typewriter_on_depth_resume: depth returned to 0");

    let replay_enabled = active_view_query
        .iter()
        .next()
        .map(|view| {
            view.local_facts
                .get_bool(fre_facts::DIALOGUE_REPLAY_ON_RESUME)
                .unwrap_or(false)
        })
        .unwrap_or(false);

    info!(
        "replay_typewriter_on_depth_resume: replay_enabled={}",
        replay_enabled
    );

    let typewriter_count = typewriter_query.iter().count();
    info!(
        "replay_typewriter_on_depth_resume: found {} typewriters",
        typewriter_count
    );

    if replay_enabled {
        for mut typewriter in typewriter_query.iter_mut() {
            info!(
                "replay_typewriter_on_depth_resume: restarting typewriter, source_text='{}'",
                typewriter.source_text
            );
            typewriter.restart();
        }

        for mut view_root in active_view_query.iter_mut() {
            view_root
                .local_facts
                .set("dialogue_text", FactValue::String(String::new()));
        }
    } else {
        for mut typewriter in typewriter_query.iter_mut() {
            info!(
                "replay_typewriter_on_depth_resume: resuming typewriter, source_text='{}'",
                typewriter.source_text
            );
            typewriter.resume();
        }
    }
}
